//! [`Kit`]: the one object a host holds. It owns the fonts, images, meshes, tokens, palette, the
//! frame arena and the renderer.
use crate::cluster::{FontChoice, Layout, Setup};
use crate::images::{ImageError, Images};
use crate::math::Rect;
use crate::mesh::{MeshData, MeshId, Meshes};
use crate::paint::ImageId;
use crate::painter::Painter;
use crate::palette::Palette;
use crate::renderer::{FrameStats, GpuPassTime, Renderer};
use crate::scene::Scene;
use crate::text::{FontError, FontRole, Fonts};
use crate::tokens::Tokens;
use std::time::Duration;

/// Glyphs generated per frame at most when text uses characters not yet in the atlas.
const GLYPHS_PER_FRAME: usize = 16;

/// Errors creating the kit.
#[derive(Debug, thiserror::Error)]
pub enum KitError {
    /// The embedded font could not be loaded.
    #[error(transparent)]
    Font(#[from] FontError),
    /// An image failed to load.
    #[error(transparent)]
    Image(#[from] ImageError),
}

/// The render kit.
#[derive(Debug)]
pub struct Kit {
    renderer: Renderer,
    fonts: Fonts,
    images: Images,
    tokens: Tokens,
    palette: Palette,
    scene: Scene,
    frame_scale: f32,
}

impl Kit {
    /// Creates the kit on an existing device (from `fh6-gpu` natively, or WebGPU in a browser).
    ///
    /// # Errors
    /// [`KitError::Font`] if the embedded fallback font fails to load.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, tokens: Tokens) -> Result<Self, KitError> {
        let fonts = Fonts::new()?;
        let images = Images::new(device, queue);
        let renderer = Renderer::new(device, queue, &images);
        let palette = Palette::family(&tokens, "modernDigital");
        Ok(Self {
            renderer,
            fonts,
            images,
            tokens,
            palette,
            scene: Scene::new(),
            frame_scale: 1.0,
        })
    }

    /// The tokens.
    #[must_use]
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    /// Replaces the tokens (hot reload).
    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.tokens = tokens;
    }

    /// The fonts.
    #[must_use]
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// The fonts, for loading faces and assigning roles.
    pub fn fonts_mut(&mut self) -> &mut Fonts {
        &mut self.fonts
    }

    /// Binds the token font roles to loaded faces (call after loading `assets/fonts`).
    /// Returns how many roles found their family.
    pub fn bind_font_roles(&mut self) -> usize {
        self.fonts.bind_tokens(&self.tokens.fonts)
    }

    /// The font choice for a frame: the role faces, with `display_override` (a user font)
    /// replacing the display role when given.
    #[must_use]
    pub fn font_choice(&self, display_override: Option<crate::text::FontId>) -> FontChoice {
        FontChoice {
            display: display_override.unwrap_or_else(|| self.fonts.role(FontRole::Display)),
            text: self.fonts.role(FontRole::Text),
            wide: self.fonts.role(FontRole::Wide),
        }
    }

    /// The images.
    #[must_use]
    pub fn images(&self) -> &Images {
        &self.images
    }

    /// The images, for loading.
    pub fn images_mut(&mut self) -> &mut Images {
        &mut self.images
    }

    /// The mesh store.
    pub fn meshes_mut(&mut self) -> &mut Meshes {
        &mut self.renderer.meshes
    }

    /// Uploads a mesh.
    pub fn upload_mesh(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &MeshData) -> MeshId {
        self.renderer.meshes.upload(device, queue, data)
    }

    /// Loads an image file (`.png`, `.ktx2`, `.svg`).
    ///
    /// # Errors
    /// [`ImageError`] when it cannot be read or decoded.
    pub fn load_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<ImageId, ImageError> {
        self.images.load_file(device, queue, path, None)
    }

    /// The palette used by the next frames.
    #[must_use]
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Sets the palette.
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Resources for creating clusters.
    pub fn setup<'a>(
        &'a mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        assets: Option<&'a std::path::Path>,
    ) -> Setup<'a> {
        Setup {
            device,
            queue,
            images: &mut self.images,
            meshes: &mut self.renderer.meshes,
            fonts: &mut self.fonts,
            tokens: &self.tokens,
            assets,
        }
    }

    /// Starts a frame: clears the arena and returns a painter whose local space is `layout`.
    pub fn begin_frame(&mut self, layout: &Layout) -> Painter<'_> {
        self.scene.clear();
        self.frame_scale = layout.scale;
        Painter::new(
            &mut self.scene,
            &self.fonts,
            &self.images,
            &self.palette,
            &self.tokens,
            layout.transform(),
        )
    }

    /// The scene recorded since [`Kit::begin_frame`] (counts, bounds).
    #[must_use]
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Renders the recorded frame into `output` (a `target_size` texture of `format`), at
    /// `opacity` (the visibility fade). Records into `encoder`; the caller submits it.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        target_size: (u32, u32),
        opacity: f32,
    ) -> FrameStats {
        if !self.scene.missing_glyphs.is_empty() {
            let n = self.scene.missing_glyphs.len().min(GLYPHS_PER_FRAME);
            self.fonts.generate_missing(&self.scene.missing_glyphs[..n]);
        }
        let margin = self.tokens.glow.max_bloom_radius * self.frame_scale;
        self.renderer.render(
            device,
            queue,
            encoder,
            &self.scene,
            &self.images,
            &mut self.fonts,
            output,
            format,
            target_size,
            opacity,
            margin,
        )
    }

    /// Call once every command buffer that carries a [`Kit::render`] since the previous call has
    /// been submitted (or dropped): the frame loop calls it right after its submit. Every staging
    /// slot written since then is mapped again, so a command buffer recorded before this call but
    /// submitted after it would fail its submit. The kit then reuses its upload staging buffers
    /// instead of letting
    /// `Queue::write_buffer` create one per frame; without the call it falls back to
    /// `write_buffer` after a few frames, which is correct but costs more CPU.
    pub fn frame_submitted(&mut self) {
        self.renderer.frame_submitted();
    }

    /// Attaches (or detaches) the refraction source: the captured game frame, as a texture
    /// view covering `rect` of the target in pixels. Glass with `Refraction::World` samples it.
    pub fn set_refraction_source(&mut self, source: Option<(wgpu::TextureView, Rect)>) {
        self.renderer.set_world(source);
    }

    /// True when a refraction source is attached.
    #[must_use]
    pub fn has_refraction_source(&self) -> bool {
        self.renderer.has_world()
    }

    /// Turns bloom on or off for every layer (quality setting; default on).
    pub fn set_bloom(&mut self, on: bool) {
        self.renderer.set_bloom(on);
    }

    /// Sets a premultiplied black underlay of `alpha` (0..1) under the whole target, applied in
    /// the clear and the resolve (so a full-screen dim, such as behind the OSD, costs no shape
    /// pass). It scales with the frame opacity. 0 (the default) turns it off.
    pub fn set_underlay(&mut self, alpha: f32) {
        self.renderer.set_underlay(alpha);
    }

    /// Enables GPU timestamp queries; false when the device lacks `TIMESTAMP_QUERY`.
    pub fn set_gpu_timing(&mut self, on: bool) -> bool {
        self.renderer.set_timing(on)
    }

    /// GPU time of the last submitted frame (blocks until it is available; for benches).
    #[must_use]
    pub fn gpu_time(&self, device: &wgpu::Device) -> Option<Duration> {
        self.renderer.read_gpu_time(device)
    }

    /// Times every pass of each frame, not only the whole frame (turns GPU timing on with it;
    /// turning the timeline off leaves frame timing as it is); false when the device lacks
    /// `TIMESTAMP_QUERY`. A diagnostic for benches: the extra timestamps cost a little GPU time.
    pub fn set_gpu_timeline(&mut self, on: bool) -> bool {
        self.renderer.set_timeline(on)
    }

    /// Every pass of the last frame, in order, when the pass timeline is on (blocks until it is
    /// available; for benches). Read it once after each `render` and its submit: the pass names
    /// belong to the last frame rendered.
    #[must_use]
    pub fn gpu_timeline(&self, device: &wgpu::Device) -> Option<Vec<GpuPassTime>> {
        self.renderer.read_gpu_timeline(device)
    }
}
