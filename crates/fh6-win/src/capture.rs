//! Desktop screenshots through GDI, for the manual run checklist and docs.
//!
//! `BitBlt` from the screen DC with `CAPTUREBLT` reads the composed desktop, including layered
//! and DirectComposition windows such as the overlay. Call
//! [`crate::monitor::enable_per_monitor_dpi_awareness`] first to get physical pixels.
use crate::WinError;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HGDIOBJ, SRCCOPY,
};

/// A captured region as opaque RGBA8 rows (top to bottom).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screenshot {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes, RGBA.
    pub rgba: Vec<u8>,
}

/// Captures the desktop rectangle at (`x`, `y`) of size `width` x `height` (physical pixels).
///
/// # Errors
/// [`WinError`] if any GDI call fails.
pub fn capture_screen(x: i32, y: i32, width: u32, height: u32) -> Result<Screenshot, WinError> {
    let (w, h) = (
        i32::try_from(width.max(1)).unwrap_or(i32::MAX),
        i32::try_from(height.max(1)).unwrap_or(i32::MAX),
    );
    // SAFETY: every GDI object created here is released below on all paths; the DIB buffer is
    // sized exactly `w * h * 4` for a 32-bit top-down DIB, matching the BITMAPINFO passed in.
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return Err(WinError::last("GetDC"));
        }
        let mem = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, HGDIOBJ(bitmap.0));
        let blit = BitBlt(mem, 0, 0, w, h, Some(screen), x, y, SRCCOPY | CAPTUREBLT);

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>()).unwrap_or(40),
                biWidth: w,
                // Negative height: top-down rows.
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..BITMAPINFOHEADER::default()
            },
            ..BITMAPINFO::default()
        };
        let mut bgra = vec![0u8; (w as usize) * (h as usize) * 4];
        SelectObject(mem, old);
        let lines = if blit.is_ok() {
            GetDIBits(mem, bitmap, 0, h as u32, Some(bgra.as_mut_ptr().cast()), &mut info, DIB_RGB_COLORS)
        } else {
            0
        };
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);

        blit.map_err(|e| WinError::call("BitBlt", e))?;
        if lines != h {
            return Err(WinError::last("GetDIBits"));
        }
        for px in bgra.chunks_exact_mut(4) {
            px.swap(0, 2);
            px[3] = 255;
        }
        Ok(Screenshot {
            width: w as u32,
            height: h as u32,
            rgba: bgra,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_a_small_region() {
        let shot = capture_screen(0, 0, 32, 16).expect("desktop capture works in a user session");
        assert_eq!((shot.width, shot.height), (32, 16));
        assert_eq!(shot.rgba.len(), 32 * 16 * 4);
        assert!(shot.rgba.chunks_exact(4).all(|p| p[3] == 255));
    }
}
