//! SVG rasterising (resvg, pure Rust) into straight-alpha sRGB images, and outlining text so a
//! generated SVG carries no font dependency.
use crate::error::{image_err, Result};
use crate::img::Img;
use resvg::{tiny_skia, usvg};
use std::path::Path;
use std::sync::Arc;

/// Parse options with the repo's fonts (and system fonts as a fallback) loaded.
pub fn options(font_dirs: &[&Path]) -> usvg::Options<'static> {
    let mut opt = usvg::Options::default();
    let db = Arc::make_mut(&mut opt.fontdb);
    for d in font_dirs {
        db.load_fonts_dir(d);
    }
    db.load_system_fonts();
    opt
}

/// Parses SVG bytes.
pub fn parse(bytes: &[u8], opt: &usvg::Options<'_>, label: &str) -> Result<usvg::Tree> {
    usvg::Tree::from_data(bytes, opt).map_err(|e| image_err(Path::new(label), e))
}

/// Rasterises so the longest side is `longest` px (aspect kept), straight alpha.
pub fn render_longest(tree: &usvg::Tree, longest: u32, label: &str) -> Result<Img> {
    let size = tree.size();
    let (sw, sh) = (size.width().max(1e-3), size.height().max(1e-3));
    let k = longest as f32 / sw.max(sh);
    let w = (sw * k).round().max(1.0) as u32;
    let h = (sh * k).round().max(1.0) as u32;
    render_into(tree, w, h, k, label)
}

/// Rasterises at exactly `w`x`h` with scale `k` from SVG units.
pub fn render_into(tree: &usvg::Tree, w: u32, h: u32, k: f32, label: &str) -> Result<Img> {
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or_else(|| image_err(Path::new(label), "zero-size pixmap"))?;
    resvg::render(tree, tiny_skia::Transform::from_scale(k, k), &mut pixmap.as_mut());
    Ok(from_premultiplied(pixmap.data(), w as usize, h as usize))
}

/// Converts premultiplied RGBA8 to a straight-alpha float image.
pub fn from_premultiplied(px: &[u8], w: usize, h: usize) -> Img {
    let mut img = Img::new(w, h, 4);
    for (i, p) in px.chunks_exact(4).enumerate() {
        let a = f32::from(p[3]) / 255.0;
        let un = |c: u8| if a > 0.0 { (f32::from(c) / 255.0 / a).min(1.0) } else { 0.0 };
        img.data[i * 4] = un(p[0]);
        img.data[i * 4 + 1] = un(p[1]);
        img.data[i * 4 + 2] = un(p[2]);
        img.data[i * 4 + 3] = a;
    }
    img
}

/// Re-serialises a parsed tree; text becomes outlined paths, so the file needs no fonts.
pub fn outlined(tree: &usvg::Tree) -> String {
    tree.to_string(&usvg::WriteOptions::default())
}
