//! Optional Windows.Graphics.Capture (WGC) capture of one window, delivered as CPU RGBA8.
//!
//! The GPU kit's Liquid Glass can refract the game frame behind the overlay
//! (`refraction_source`, off by default). This module captures the *game* window by `HWND`, so
//! our own overlay (topmost, `WS_EX_NOREDIRECTIONBITMAP`) is never in the picture. The app
//! uploads [`CaptureFrame::pixels`] with `queue.write_texture`.
//!
//! Pipeline per new frame: `TryGetNextFrame` (draining to the newest) → `CopySubresourceRegion`
//! of the requested region into a reusable staging texture → `Map` → BGRA→RGBA with an optional
//! power-of-two box downscale into the caller's buffer. Nothing is heap-allocated per frame on
//! the success path; the staging texture is recreated only when the region size changes.
//!
//! Coordinates are physical pixels: call [`crate::monitor::enable_per_monitor_dpi_awareness`]
//! first. Requires Windows 10 1903 or later ([`is_supported`]).
use std::ffi::c_void;
use std::time::{Duration, Instant};
use windows::core::Interface;
use windows::Graphics::Capture::{
    Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::SizeInt32;
use windows::Win32::Foundation::{HMODULE, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Multithread, ID3D11Texture2D, D3D11_BOX,
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_FLAG_DO_NOT_WAIT,
    D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{IDXGIDevice, DXGI_ERROR_WAS_STILL_DRAWING};
use windows::Win32::Graphics::Gdi::{ClientToScreen, InvalidateRect};
use windows::Win32::System::WinRT::Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClientRect, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
};

/// Number of buffers in the frame pool.
const POOL_BUFFERS: i32 = 2;

/// A failed capture setup or frame read.
#[derive(Debug, thiserror::Error)]
pub enum WgcError {
    /// Windows.Graphics.Capture is missing (older than Windows 10 1903, or disabled).
    #[error("Windows.Graphics.Capture is not supported on this system")]
    Unsupported,
    /// The handle does not name an existing window.
    #[error("{0:#x} is not a window handle")]
    InvalidWindow(isize),
    /// A zero-sized capture region or downscale target.
    #[error("invalid argument: {0}")]
    InvalidArgument(&'static str),
    /// The region lies entirely outside the captured frame.
    #[error("the capture region lies outside the {frame_width}x{frame_height} window frame")]
    RegionOutsideFrame {
        /// Captured frame width.
        frame_width: u32,
        /// Captured frame height.
        frame_height: u32,
    },
    /// The caller's [`CaptureFrame`] cannot hold the converted frame without growing.
    #[error("the frame buffer holds {capacity} bytes but {needed} are needed")]
    BufferTooSmall {
        /// Bytes the converted frame needs.
        needed: usize,
        /// Bytes the buffer can hold without reallocating.
        capacity: usize,
    },
    /// A Direct3D or WinRT call failed.
    #[error("{call} failed: {message} (HRESULT {code:#010x})")]
    Call {
        /// The API that failed.
        call: &'static str,
        /// The HRESULT.
        code: u32,
        /// The system message for it.
        message: String,
    },
}

impl WgcError {
    fn call(call: &'static str, err: &windows::core::Error) -> Self {
        Self::Call {
            call,
            // Bit-for-bit reinterpretation of the HRESULT for hex display.
            code: err.code().0 as u32,
            message: err.message(),
        }
    }
}

/// A rectangle in the target window's client-area pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureRegion {
    /// Left edge, client pixels.
    pub x: i32,
    /// Top edge, client pixels.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// A rectangle in captured-frame pixels, already inside the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRect {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width, at least 1.
    pub width: u32,
    /// Height, at least 1.
    pub height: u32,
}

/// Maps a client-area `region` into a `frame_width` x `frame_height` frame whose origin is
/// `client_offset` pixels up-left of the client origin, and clips it to the frame. `None` when
/// nothing of the region is inside the frame.
pub fn clamp_region(
    region: CaptureRegion,
    client_offset: (i32, i32),
    frame_width: u32,
    frame_height: u32,
) -> Option<FrameRect> {
    let left = i64::from(region.x) + i64::from(client_offset.0);
    let top = i64::from(region.y) + i64::from(client_offset.1);
    let right = left + i64::from(region.width);
    let bottom = top + i64::from(region.height);
    let (l, t) = (left.max(0), top.max(0));
    let (r, b) = (right.min(i64::from(frame_width)), bottom.min(i64::from(frame_height)));
    if r <= l || b <= t {
        return None;
    }
    Some(FrameRect {
        x: u32::try_from(l).ok()?,
        y: u32::try_from(t).ok()?,
        width: u32::try_from(r - l).ok()?,
        height: u32::try_from(b - t).ok()?,
    })
}

/// The power-of-two box factor (1, 2, 4, ...) that makes `width` x `height` fit inside
/// `max_width` x `max_height`: repeated 2x2 steps until it fits. `max_*` of 0 counts as 1.
pub fn downscale_factor(width: u32, height: u32, max_width: u32, max_height: u32) -> u32 {
    let (max_w, max_h) = (max_width.max(1), max_height.max(1));
    let mut factor = 1u32;
    while factor < (1 << 30) {
        let (w, h) = scaled_size(width, height, factor);
        if w <= max_w && h <= max_h {
            break;
        }
        factor *= 2;
    }
    factor
}

/// Output size for a box `factor`: each axis divided and floored, at least 1.
pub fn scaled_size(width: u32, height: u32, factor: u32) -> (u32, u32) {
    let f = factor.max(1);
    ((width / f).max(1), (height / f).max(1))
}

/// Converts `width` x `height` BGRA8 pixels (rows `src_pitch` bytes apart) to tightly packed
/// opaque RGBA8 in `dst`, averaging `factor` x `factor` boxes. Alpha is forced to 255 (a game
/// swap chain's alpha is not meaningful). Returns the output size, or `None` if `src` or `dst`
/// is too short.
pub fn convert_bgra_to_rgba(
    src: &[u8],
    src_pitch: usize,
    width: u32,
    height: u32,
    factor: u32,
    dst: &mut [u8],
) -> Option<(u32, u32)> {
    let (w, h) = (width as usize, height as usize);
    if w == 0 || h == 0 || src_pitch < w * 4 || src.len() < src_pitch * (h - 1) + w * 4 {
        return None;
    }
    let factor = factor.max(1);
    let (ow, oh) = scaled_size(width, height, factor);
    let (ow_us, oh_us) = (ow as usize, oh as usize);
    let dst = dst.get_mut(..ow_us * oh_us * 4)?;
    match factor {
        1 => swizzle_rows(src, src_pitch, w, dst)?,
        2 if w >= 2 && h >= 2 => box2_rows(src, src_pitch, ow_us, dst)?,
        _ => box_generic(src, src_pitch, w, h, factor as usize, ow_us, dst)?,
    }
    Some((ow, oh))
}

/// BGRA → opaque RGBA, one pixel per pixel, as 32-bit words (vectorises well).
fn swizzle_rows(src: &[u8], src_pitch: usize, w: usize, dst: &mut [u8]) -> Option<()> {
    for (y, out_row) in dst.chunks_exact_mut(w * 4).enumerate() {
        let row = src.get(y * src_pitch..y * src_pitch + w * 4)?;
        for (o, i) in out_row.chunks_exact_mut(4).zip(row.chunks_exact(4)) {
            let v = u32::from_le_bytes([i[0], i[1], i[2], i[3]]);
            let rgba = ((v >> 16) & 0xFF) | (v & 0xFF00) | ((v & 0xFF) << 16) | 0xFF00_0000;
            o.copy_from_slice(&rgba.to_le_bytes());
        }
    }
    Some(())
}

/// The 2x2 box, two source rows per output row.
fn box2_rows(src: &[u8], src_pitch: usize, ow: usize, dst: &mut [u8]) -> Option<()> {
    for (oy, out_row) in dst.chunks_exact_mut(ow * 4).enumerate() {
        let top = src.get(2 * oy * src_pitch..)?.get(..ow * 8)?;
        let bottom = src.get((2 * oy + 1) * src_pitch..)?.get(..ow * 8)?;
        for ((o, a), b) in out_row
            .chunks_exact_mut(4)
            .zip(top.chunks_exact(8))
            .zip(bottom.chunks_exact(8))
        {
            let avg = |c: usize| {
                let sum = u16::from(a[c]) + u16::from(a[c + 4]) + u16::from(b[c]) + u16::from(b[c + 4]);
                ((sum + 2) >> 2) as u8
            };
            o[0] = avg(2);
            o[1] = avg(1);
            o[2] = avg(0);
            o[3] = 255;
        }
    }
    Some(())
}

/// Any `f` x `f` box (clipped at the edges when an axis is shorter than `f`).
fn box_generic(src: &[u8], src_pitch: usize, w: usize, h: usize, f: usize, ow: usize, dst: &mut [u8]) -> Option<()> {
    for (oy, out_row) in dst.chunks_exact_mut(ow * 4).enumerate() {
        let y0 = oy * f;
        let y1 = (y0 + f).min(h);
        for (ox, o) in out_row.chunks_exact_mut(4).enumerate() {
            let x0 = ox * f;
            let x1 = (x0 + f).min(w);
            let (mut b, mut g, mut r) = (0u64, 0u64, 0u64);
            for y in y0..y1 {
                let row = src.get(y * src_pitch + x0 * 4..y * src_pitch + x1 * 4)?;
                for px in row.chunks_exact(4) {
                    b += u64::from(px[0]);
                    g += u64::from(px[1]);
                    r += u64::from(px[2]);
                }
            }
            let count = ((x1 - x0) * (y1 - y0)).max(1) as u64;
            let half = count / 2;
            // An average of u8 values always fits u8.
            o[0] = ((r + half) / count) as u8;
            o[1] = ((g + half) / count) as u8;
            o[2] = ((b + half) / count) as u8;
            o[3] = 255;
        }
    }
    Some(())
}

/// One converted capture, reused across frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureFrame {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// The frame's `SystemRelativeTime` (QPC time base).
    pub timestamp: Duration,
    /// CPU time of the last copy + map + convert.
    pub cpu: Duration,
    /// Tightly packed RGBA8, `width * height * 4` bytes after a successful
    /// [`WindowCapture::latest`]. Its capacity is fixed by [`CaptureFrame::with_capacity`].
    pub pixels: Vec<u8>,
}

impl CaptureFrame {
    /// Allocates a buffer for frames up to `max_width` x `max_height` (pass the same `max_size`
    /// given to [`WindowCapture::start`]). This is the only allocation; frames never grow it.
    pub fn with_capacity(max_width: u32, max_height: u32) -> Self {
        let bytes = (max_width as usize)
            .saturating_mul(max_height as usize)
            .saturating_mul(4);
        Self {
            width: 0,
            height: 0,
            timestamp: Duration::ZERO,
            cpu: Duration::ZERO,
            pixels: Vec::with_capacity(bytes),
        }
    }

    /// Sets `pixels.len()` to `bytes` without reallocating.
    ///
    /// # Errors
    /// [`WgcError::BufferTooSmall`] if `bytes` exceeds the fixed capacity.
    pub fn reserve_exact_len(&mut self, bytes: usize) -> Result<(), WgcError> {
        let capacity = self.pixels.capacity();
        if bytes > capacity {
            return Err(WgcError::BufferTooSmall {
                needed: bytes,
                capacity,
            });
        }
        // Within capacity: `resize` never reallocates; it only zero-fills newly exposed bytes.
        self.pixels.resize(bytes, 0);
        Ok(())
    }
}

/// True if Windows.Graphics.Capture is available.
pub fn is_supported() -> bool {
    GraphicsCaptureSession::IsSupported().unwrap_or(false)
}

/// Offset of the client origin inside the captured frame. WGC window frames cover the DWM
/// extended frame bounds (the visible window, without the invisible resize borders).
fn client_offset(hwnd: HWND) -> (i32, i32) {
    let mut origin = POINT::default();
    let mut bounds = RECT::default();
    // SAFETY: `origin` and `bounds` are valid locals of the sizes passed; a stale HWND only
    // makes the calls fail, which falls back to no offset.
    let ok = unsafe {
        ClientToScreen(hwnd, &mut origin).as_bool()
            && DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                (&mut bounds as *mut RECT).cast::<c_void>(),
                u32::try_from(std::mem::size_of::<RECT>()).unwrap_or(16),
            )
            .is_ok()
    };
    if ok {
        (origin.x - bounds.left, origin.y - bounds.top)
    } else {
        (0, 0)
    }
}

fn to_hwnd(hwnd: isize) -> HWND {
    HWND(hwnd as *mut c_void)
}

/// How [`WindowCapture::latest`] reads the GPU copy back to the CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Readback {
    /// Copy the new frame on the GPU now and read it on the *next* call, once the copy is done
    /// (`Map` with `DO_NOT_WAIT`). Never stalls on the GPU; the image lags one call behind.
    #[default]
    Pipelined,
    /// Copy and read in the same call; `Map` waits for the GPU copy (about 1 ms measured).
    Blocking,
}

/// A GPU copy into the staging texture that has not been read yet.
#[derive(Debug, Clone, Copy)]
struct PendingCopy {
    width: u32,
    height: u32,
    factor: u32,
    timestamp: Duration,
    issue_cost: Duration,
}

/// A running capture of one window.
#[derive(Debug)]
pub struct WindowCapture {
    hwnd: HWND,
    region: CaptureRegion,
    max_size: (u32, u32),
    readback: Readback,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    winrt_device: IDirect3DDevice,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    pool_size: SizeInt32,
    client_offset: (i32, i32),
    staging: Option<(ID3D11Texture2D, u32, u32)>,
    pending: Option<PendingCopy>,
}

impl WindowCapture {
    /// Starts capturing `hwnd` (a raw window handle). `region` is in the window's client pixels;
    /// each frame is box-downscaled by powers of two until it fits `max_size`.
    ///
    /// # Errors
    /// [`WgcError`] if WGC is unsupported, the window is invalid, or a D3D11 / WinRT call fails.
    pub fn start(hwnd: isize, region: CaptureRegion, max_size: (u32, u32)) -> Result<Self, WgcError> {
        if region.width == 0 || region.height == 0 {
            return Err(WgcError::InvalidArgument("capture region must be at least 1x1"));
        }
        if max_size.0 == 0 || max_size.1 == 0 {
            return Err(WgcError::InvalidArgument("max size must be at least 1x1"));
        }
        if !is_supported() {
            return Err(WgcError::Unsupported);
        }
        let window = to_hwnd(hwnd);
        // SAFETY: IsWindow accepts any value and only reports whether it names a window.
        if !unsafe { IsWindow(Some(window)) }.as_bool() {
            return Err(WgcError::InvalidWindow(hwnd));
        }

        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        // SAFETY: the out pointers are valid locals; a hardware device with BGRA support (WGC
        // surfaces are BGRA) and no software module.
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
        }
        .map_err(|e| WgcError::call("D3D11CreateDevice", &e))?;
        let (Some(device), Some(context)) = (device, context) else {
            return Err(WgcError::InvalidArgument("D3D11CreateDevice returned no device"));
        };
        // The free-threaded pool touches the device from system threads; serialise access.
        if let Ok(mt) = device.cast::<ID3D11Multithread>() {
            // SAFETY: a plain flag on a live device.
            unsafe {
                let _ = mt.SetMultithreadProtected(true);
            }
        }
        let dxgi: IDXGIDevice = device
            .cast()
            .map_err(|e| WgcError::call("ID3D11Device as IDXGIDevice", &e))?;
        // SAFETY: `dxgi` is a live DXGI device; the call returns a new WinRT wrapper for it.
        let winrt_device: IDirect3DDevice = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
            .and_then(|inspectable| inspectable.cast())
            .map_err(|e| WgcError::call("CreateDirect3D11DeviceFromDXGIDevice", &e))?;

        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
            .map_err(|e| WgcError::call("GraphicsCaptureItem factory", &e))?;
        // SAFETY: `window` was just checked with IsWindow; the interop returns a new item.
        let item: GraphicsCaptureItem = unsafe { interop.CreateForWindow(window) }
            .map_err(|e| WgcError::call("IGraphicsCaptureItemInterop::CreateForWindow", &e))?;
        let pool_size = item
            .Size()
            .map_err(|e| WgcError::call("GraphicsCaptureItem::Size", &e))?;

        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            POOL_BUFFERS,
            pool_size,
        )
        .map_err(|e| WgcError::call("Direct3D11CaptureFramePool::CreateFreeThreaded", &e))?;
        let session = match pool.CreateCaptureSession(&item) {
            Ok(session) => session,
            Err(e) => {
                let _ = pool.Close();
                return Err(WgcError::call("CreateCaptureSession", &e));
            }
        };
        // Windows 11 only (IGraphicsCaptureSession3); older systems keep the yellow border.
        if let Err(e) = session.SetIsBorderRequired(false) {
            tracing::debug!(error = %e.message(), "capture border cannot be disabled");
        }
        if let Err(e) = session.SetIsCursorCaptureEnabled(false) {
            tracing::debug!(error = %e.message(), "cursor capture cannot be disabled");
        }
        if let Err(e) = session.StartCapture() {
            let _ = session.Close();
            let _ = pool.Close();
            return Err(WgcError::call("GraphicsCaptureSession::StartCapture", &e));
        }
        Ok(Self {
            hwnd: window,
            region,
            max_size,
            readback: Readback::default(),
            device,
            context,
            winrt_device,
            pool,
            session,
            pool_size,
            client_offset: client_offset(window),
            staging: None,
            pending: None,
        })
    }

    /// Chooses how frames are read back (default [`Readback::Pipelined`]).
    pub fn set_readback(&mut self, readback: Readback) {
        self.readback = readback;
    }

    /// The captured window.
    pub fn hwnd(&self) -> isize {
        self.hwnd.0 as isize
    }

    /// Delivers the newest captured frame into `out`. Never waits for a frame to arrive: returns
    /// `Ok(false)` when there is nothing new. Older queued frames are dropped.
    ///
    /// With [`Readback::Pipelined`] (the default) a new frame is copied on the GPU in one call
    /// and converted in a later call once the copy has finished, so the result lags one call
    /// behind but `Map` never stalls. With [`Readback::Blocking`] it is copied and read at once.
    ///
    /// # Errors
    /// [`WgcError::BufferTooSmall`] if `out` was allocated for a smaller size,
    /// [`WgcError::RegionOutsideFrame`] if the window shrank below the region, or a D3D11 /
    /// WinRT failure.
    pub fn latest(&mut self, out: &mut CaptureFrame) -> Result<bool, WgcError> {
        let mut delivered = false;
        if let Some(pending) = self.pending {
            if !self.read_pending(pending, false, out)? {
                // The GPU is still copying; leave new frames queued until it is done.
                return Ok(false);
            }
            delivered = true;
        }
        let Some(frame) = self.newest_frame()? else {
            return Ok(delivered);
        };
        let issued = self.issue_copy(&frame);
        let _ = frame.Close();
        let pending = issued?;
        if self.readback == Readback::Blocking {
            return self.read_pending(pending, true, out);
        }
        Ok(delivered)
    }

    /// Drains the frame pool, keeping only the newest frame.
    fn newest_frame(&self) -> Result<Option<Direct3D11CaptureFrame>, WgcError> {
        let mut newest: Option<Direct3D11CaptureFrame> = None;
        loop {
            match self.pool.TryGetNextFrame() {
                Ok(frame) => {
                    if let Some(older) = newest.replace(frame) {
                        let _ = older.Close();
                    }
                }
                // A null frame (empty error) means the queue is drained.
                Err(e) if e.code().is_ok() => return Ok(newest),
                Err(e) => {
                    if let Some(frame) = newest {
                        let _ = frame.Close();
                    }
                    return Err(WgcError::call("TryGetNextFrame", &e));
                }
            }
        }
    }

    /// Copies the region of `frame` into the staging texture and records it as pending.
    fn issue_copy(&mut self, frame: &Direct3D11CaptureFrame) -> Result<PendingCopy, WgcError> {
        let content = frame
            .ContentSize()
            .map_err(|e| WgcError::call("Direct3D11CaptureFrame::ContentSize", &e))?;
        if content.Width != self.pool_size.Width || content.Height != self.pool_size.Height {
            // The window was resized: later frames come at the new size; the client origin may
            // also have moved inside the frame.
            self.pool
                .Recreate(
                    &self.winrt_device,
                    DirectXPixelFormat::B8G8R8A8UIntNormalized,
                    POOL_BUFFERS,
                    content,
                )
                .map_err(|e| WgcError::call("Direct3D11CaptureFramePool::Recreate", &e))?;
            self.pool_size = content;
            self.client_offset = client_offset(self.hwnd);
        }

        let started = Instant::now();
        let texture: ID3D11Texture2D = frame
            .Surface()
            .and_then(|surface| surface.cast::<IDirect3DDxgiInterfaceAccess>())
            // SAFETY: the access interface belongs to the live frame surface.
            .and_then(|access| unsafe { access.GetInterface::<ID3D11Texture2D>() })
            .map_err(|e| WgcError::call("frame surface as ID3D11Texture2D", &e))?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: `desc` is a valid out pointer for a live texture.
        unsafe { texture.GetDesc(&mut desc) };
        // The surface may be larger than the content (pool not yet recreated); use both bounds.
        let frame_w = desc.Width.min(u32::try_from(content.Width).unwrap_or(0));
        let frame_h = desc.Height.min(u32::try_from(content.Height).unwrap_or(0));
        let rect =
            clamp_region(self.region, self.client_offset, frame_w, frame_h).ok_or(WgcError::RegionOutsideFrame {
                frame_width: frame_w,
                frame_height: frame_h,
            })?;
        let factor = downscale_factor(rect.width, rect.height, self.max_size.0, self.max_size.1);

        let staging = self.staging_texture(rect.width, rect.height)?;
        let src_box = D3D11_BOX {
            left: rect.x,
            top: rect.y,
            front: 0,
            right: rect.x + rect.width,
            bottom: rect.y + rect.height,
            back: 1,
        };
        // SAFETY: both textures are live BGRA8 textures on `self.device`; the box lies inside
        // the source (clamped above) and matches the staging size exactly. The staging texture
        // is not mapped (every Map is paired with an Unmap before returning).
        unsafe {
            self.context
                .CopySubresourceRegion(&staging, 0, 0, 0, 0, &texture, 0, Some(&src_box));
            if self.readback == Readback::Pipelined {
                // Submit now so the copy is done by the next call's non-waiting Map.
                self.context.Flush();
            }
        }
        let ticks = frame.SystemRelativeTime().map(|t| t.Duration).unwrap_or(0);
        let pending = PendingCopy {
            width: rect.width,
            height: rect.height,
            factor,
            timestamp: Duration::from_nanos(u64::try_from(ticks).unwrap_or(0).saturating_mul(100)),
            issue_cost: started.elapsed(),
        };
        self.pending = Some(pending);
        Ok(pending)
    }

    /// Maps the staging texture and converts it into `out`. Without `wait`, returns `Ok(false)`
    /// while the GPU copy is still running.
    fn read_pending(&mut self, pending: PendingCopy, wait: bool, out: &mut CaptureFrame) -> Result<bool, WgcError> {
        let Some((staging, w, h)) = &self.staging else {
            self.pending = None;
            return Ok(false);
        };
        if (*w, *h) != (pending.width, pending.height) {
            self.pending = None;
            return Ok(false);
        }
        let started = Instant::now();
        let (ow, oh) = scaled_size(pending.width, pending.height, pending.factor);
        if let Err(e) = out.reserve_exact_len((ow as usize) * (oh as usize) * 4) {
            self.pending = None;
            return Err(e);
        }
        let flags = if wait { 0 } else { D3D11_MAP_FLAG_DO_NOT_WAIT.0 as u32 };
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: `staging` is a live CPU-readable staging texture on this context and is not
        // currently mapped; `mapped` is a valid out pointer.
        let map = unsafe { self.context.Map(staging, 0, D3D11_MAP_READ, flags, Some(&mut mapped)) };
        match map {
            Ok(()) => {}
            Err(e) if !wait && e.code() == DXGI_ERROR_WAS_STILL_DRAWING => return Ok(false),
            Err(e) => {
                self.pending = None;
                return Err(WgcError::call("ID3D11DeviceContext::Map", &e));
            }
        }
        let pitch = mapped.RowPitch as usize;
        let len = pitch * (pending.height as usize - 1) + pending.width as usize * 4;
        // SAFETY: a mapped staging texture of `pending.height` rows is readable for `RowPitch`
        // bytes per row (the last row for at least `width * 4`) until Unmap below; `pData` is
        // non-null because Map succeeded.
        let src = unsafe { std::slice::from_raw_parts(mapped.pData.cast::<u8>(), len) };
        let converted = convert_bgra_to_rgba(
            src,
            pitch,
            pending.width,
            pending.height,
            pending.factor,
            &mut out.pixels,
        );
        // SAFETY: unmaps the subresource mapped above; `src` is not used after this point.
        unsafe { self.context.Unmap(staging, 0) };
        self.pending = None;
        let Some((w, h)) = converted else {
            return Err(WgcError::InvalidArgument(
                "mapped staging texture is smaller than its region",
            ));
        };
        out.width = w;
        out.height = h;
        out.timestamp = pending.timestamp;
        out.cpu = pending.issue_cost + started.elapsed();
        Ok(true)
    }

    /// The staging texture for a `width` x `height` region, created on first use or size change.
    fn staging_texture(&mut self, width: u32, height: u32) -> Result<ID3D11Texture2D, WgcError> {
        if let Some((texture, w, h)) = &self.staging {
            if *w == width && *h == height {
                return Ok(texture.clone());
            }
        }
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut texture: Option<ID3D11Texture2D> = None;
        // SAFETY: `desc` describes a valid CPU-readable staging texture; `texture` is a valid
        // out pointer.
        unsafe { self.device.CreateTexture2D(&desc, None, Some(&mut texture)) }
            .map_err(|e| WgcError::call("ID3D11Device::CreateTexture2D", &e))?;
        let texture = texture.ok_or(WgcError::InvalidArgument("CreateTexture2D returned no texture"))?;
        self.staging = Some((texture.clone(), width, height));
        Ok(texture)
    }
}

impl Drop for WindowCapture {
    fn drop(&mut self) {
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}

struct FindState<'a> {
    exe: &'a str,
    best: isize,
    best_area: i64,
}

unsafe extern "system" fn find_window_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    // SAFETY: `lparam` is the `FindState` pointer passed by `find_process_window`, which
    // outlives the synchronous EnumWindows call.
    let state = unsafe { &mut *(lparam.0 as *mut FindState<'_>) };
    // SAFETY: `hwnd` comes from EnumWindows; the out pointers are valid locals.
    let (visible, pid, rect) = unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);
        (IsWindowVisible(hwnd).as_bool(), pid, rect)
    };
    if visible && pid != 0 {
        let area = i64::from(rect.right - rect.left) * i64::from(rect.bottom - rect.top);
        if area > state.best_area && crate::foreground::process_image_name(pid).is_some_and(|n| n == state.exe) {
            state.best = hwnd.0 as isize;
            state.best_area = area;
        }
    }
    true.into()
}

/// The largest visible top-level window of the process named `exe_name` (case-insensitive),
/// as a raw `HWND`. Not for the frame path (it enumerates every window).
pub fn find_process_window(exe_name: &str) -> Option<isize> {
    let exe = exe_name.to_ascii_lowercase();
    let mut state = FindState {
        exe: &exe,
        best: 0,
        best_area: 0,
    };
    // SAFETY: the callback only dereferences `state`, which lives across this synchronous call.
    let _ = unsafe {
        EnumWindows(
            Some(find_window_proc),
            LPARAM((&mut state as *mut FindState<'_>) as isize),
        )
    };
    (state.best != 0).then_some(state.best)
}

/// Asks `hwnd` to repaint its whole client area. WGC only delivers frames when the window's
/// content is recomposed, so a static test window (the stand-in) needs this to produce a
/// frame stream; the real game presents every frame on its own.
pub fn request_repaint(hwnd: isize) -> bool {
    // SAFETY: InvalidateRect tolerates any handle; a stale one just returns FALSE.
    unsafe { InvalidateRect(Some(to_hwnd(hwnd)), None, true) }.as_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: i32, y: i32, width: u32, height: u32) -> CaptureRegion {
        CaptureRegion { x, y, width, height }
    }

    #[test]
    fn bgra_becomes_opaque_rgba_and_skips_row_padding() {
        // 2x2 pixels, pitch 12 (4 bytes of padding per row).
        let src = [
            1, 2, 3, 9, 4, 5, 6, 9, 0xEE, 0xEE, 0xEE, 0xEE, //
            7, 8, 9, 0, 10, 11, 12, 0, 0xEE, 0xEE, 0xEE, 0xEE,
        ];
        let mut dst = [0u8; 16];
        assert_eq!(convert_bgra_to_rgba(&src, 12, 2, 2, 1, &mut dst), Some((2, 2)));
        assert_eq!(dst, [3, 2, 1, 255, 6, 5, 4, 255, 9, 8, 7, 255, 12, 11, 10, 255]);
    }

    #[test]
    fn two_by_two_box_averages_with_rounding() {
        // 4x2 BGRA → 2x1: left box blue 0,0,2,2 (avg 1), right box red 10,11,10,11 (avg 10.5).
        let px = |b: u8, g: u8, r: u8| [b, g, r, 255];
        let mut src = Vec::new();
        for p in [px(0, 0, 10), px(2, 0, 10), px(0, 0, 11), px(0, 0, 11)] {
            src.extend_from_slice(&p);
        }
        for p in [px(0, 4, 10), px(2, 4, 10), px(0, 0, 10), px(0, 0, 11)] {
            src.extend_from_slice(&p);
        }
        let mut dst = [0u8; 8];
        assert_eq!(convert_bgra_to_rgba(&src, 16, 4, 2, 2, &mut dst), Some((2, 1)));
        // Left: r 10, g (0+0+4+4)/4 = 2, b (0+2+0+2)/4 = 1. Right: r (11+11+10+11)/4 = 10.75 → 11.
        assert_eq!(dst, [10, 2, 1, 255, 11, 0, 0, 255]);
    }

    #[test]
    fn odd_sizes_floor_and_drop_the_last_column() {
        // 3x3 → 1x1 with factor 2: only the top-left 2x2 box is used.
        let mut src = vec![0u8; 3 * 3 * 4];
        for (i, p) in src.chunks_exact_mut(4).enumerate() {
            p[2] = if i % 3 == 2 || i >= 6 { 200 } else { 40 };
        }
        let mut dst = [0u8; 4];
        assert_eq!(convert_bgra_to_rgba(&src, 12, 3, 3, 2, &mut dst), Some((1, 1)));
        assert_eq!(dst, [40, 0, 0, 255]);
    }

    #[test]
    fn fast_2x2_path_matches_the_generic_box() {
        let (w, h, pitch) = (7usize, 5usize, 32usize);
        let mut seed = 0x1234_5678u32;
        let src: Vec<u8> = (0..pitch * h)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (seed >> 24) as u8
            })
            .collect();
        let (ow, oh) = scaled_size(w as u32, h as u32, 2);
        let mut fast = vec![0u8; (ow * oh * 4) as usize];
        let mut generic = fast.clone();
        assert_eq!(
            convert_bgra_to_rgba(&src, pitch, w as u32, h as u32, 2, &mut fast),
            Some((3, 2))
        );
        box_generic(&src, pitch, w, h, 2, ow as usize, &mut generic).expect("in bounds");
        assert_eq!(fast, generic);
    }

    #[test]
    fn short_buffers_are_rejected() {
        let src = [0u8; 15];
        let mut dst = [0u8; 16];
        assert_eq!(convert_bgra_to_rgba(&src, 8, 2, 2, 1, &mut dst), None);
        let src = [0u8; 16];
        let mut small = [0u8; 12];
        assert_eq!(convert_bgra_to_rgba(&src, 8, 2, 2, 1, &mut small), None);
        assert_eq!(
            convert_bgra_to_rgba(&src, 4, 2, 2, 1, &mut dst),
            None,
            "pitch below width"
        );
    }

    #[test]
    fn downscale_steps_until_it_fits() {
        assert_eq!(downscale_factor(1240, 560, 1240, 560), 1);
        assert_eq!(downscale_factor(1240, 560, 620, 280), 2);
        assert_eq!(downscale_factor(1240, 560, 619, 280), 4);
        assert_eq!(downscale_factor(3840, 2160, 960, 540), 4);
        assert_eq!(scaled_size(1240, 560, 4), (310, 140));
        // A 1-pixel-tall strip still reaches the limit; the short axis stays 1.
        let f = downscale_factor(1000, 1, 10, 10);
        assert_eq!(scaled_size(1000, 1, f), (7, 1));
        assert_eq!(downscale_factor(10, 10, 0, 0), 8);
    }

    #[test]
    fn regions_clamp_to_the_frame() {
        assert_eq!(
            clamp_region(region(10, 20, 100, 50), (0, 0), 1920, 1080),
            Some(FrameRect {
                x: 10,
                y: 20,
                width: 100,
                height: 50
            })
        );
        // The client origin sits 8,31 inside a framed window.
        assert_eq!(
            clamp_region(region(0, 0, 100, 50), (8, 31), 1920, 1080),
            Some(FrameRect {
                x: 8,
                y: 31,
                width: 100,
                height: 50
            })
        );
        // Partly off the top-left and bottom-right.
        assert_eq!(
            clamp_region(region(-10, -5, 30, 20), (0, 0), 1920, 1080),
            Some(FrameRect {
                x: 0,
                y: 0,
                width: 20,
                height: 15
            })
        );
        assert_eq!(
            clamp_region(region(1900, 1070, 100, 100), (0, 0), 1920, 1080),
            Some(FrameRect {
                x: 1900,
                y: 1070,
                width: 20,
                height: 10
            })
        );
        assert_eq!(clamp_region(region(2000, 0, 10, 10), (0, 0), 1920, 1080), None);
        assert_eq!(clamp_region(region(-20, 0, 10, 10), (0, 0), 1920, 1080), None);
        assert_eq!(clamp_region(region(0, 0, 10, 10), (0, 0), 0, 0), None);
        // Extreme values do not overflow: the offset region ends one pixel inside the frame.
        assert_eq!(
            clamp_region(region(i32::MAX, 0, u32::MAX, 1), (i32::MAX, 0), u32::MAX, 1),
            Some(FrameRect {
                x: u32::MAX - 1,
                y: 0,
                width: 1,
                height: 1
            })
        );
    }

    #[test]
    fn capture_frame_never_grows() {
        let mut frame = CaptureFrame::with_capacity(620, 280);
        let capacity = frame.pixels.capacity();
        assert!(capacity >= 620 * 280 * 4);
        let ptr = frame.pixels.as_ptr();
        frame.reserve_exact_len(620 * 280 * 4).expect("fits");
        assert_eq!(frame.pixels.len(), 620 * 280 * 4);
        frame.reserve_exact_len(310 * 140 * 4).expect("fits");
        assert_eq!(frame.pixels.len(), 310 * 140 * 4);
        frame.reserve_exact_len(620 * 280 * 4).expect("fits");
        assert_eq!(frame.pixels.capacity(), capacity);
        assert_eq!(frame.pixels.as_ptr(), ptr, "no reallocation");
        let err = frame.reserve_exact_len(capacity + 1);
        assert!(matches!(err, Err(WgcError::BufferTooSmall { .. })));
        assert_eq!(frame.pixels.capacity(), capacity);
    }

    #[test]
    fn zero_sized_requests_are_rejected_before_any_capture() {
        assert!(matches!(
            WindowCapture::start(0, region(0, 0, 0, 10), (8, 8)),
            Err(WgcError::InvalidArgument(_))
        ));
        assert!(matches!(
            WindowCapture::start(0, region(0, 0, 10, 10), (0, 8)),
            Err(WgcError::InvalidArgument(_))
        ));
    }
}
