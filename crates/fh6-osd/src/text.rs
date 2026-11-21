use std::collections::HashMap;
use std::sync::{OnceLock, Mutex};

use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Stroke, Transform};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

pub struct TextStyle {
    pub size_px: f32,
    pub bold: bool,
    pub letter_spacing_px: f32,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub enum Align {
    Left,
    Center,
    Right,
}

struct FontFace {
    name: &'static str,
    face: Face<'static>,
    glyph_cache: HashMap<u16, Option<tiny_skia::Path>>,
    units_per_em: f32,
    _is_variable: bool,
}

impl FontFace {
    fn new(name: &'static str, data: &'static [u8]) -> Self {
        let face = Face::parse(data, 0).expect("parse font");
        let units_per_em = face.units_per_em() as f32;
        let is_variable = face.is_variable();
        Self { name, face, glyph_cache: HashMap::new(), units_per_em, _is_variable: is_variable }
    }

    fn scale_for_size(&self, size_px: f32) -> f32 {
        size_px / self.units_per_em
    }

    fn glyph_id(&self, ch: char) -> Option<u16> {
        self.face.glyph_index(ch).map(|g| g.0)
    }

    fn h_metrics(&self, gid: u16) -> (f32, f32) {
        let adv = self.face.glyph_hor_advance(GlyphId(gid)).unwrap_or(0) as f32;
        let bear = self.face.glyph_hor_side_bearing(GlyphId(gid)).unwrap_or(0) as f32;
        (adv, bear)
    }

    fn glyph_path(&mut self, gid: u16) -> Option<tiny_skia::Path> {
        if let Some(cached) = self.glyph_cache.get(&gid) {
            return cached.clone();
        }
        let mut pb = PathBuilder::new();
        struct Builder<'a> { pb: &'a mut PathBuilder }
        impl OutlineBuilder for Builder<'_> {
            fn move_to(&mut self, x: f32, y: f32) { self.pb.move_to(x, y); }
            fn line_to(&mut self, x: f32, y: f32) { self.pb.line_to(x, y); }
            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) { self.pb.quad_to(x1, y1, x, y); }
            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) { self.pb.cubic_to(x1, y1, x2, y2, x, y); }
            fn close(&mut self) { self.pb.close(); }
        }
        let mut b = Builder { pb: &mut pb };
        let _ = self.face.outline_glyph(GlyphId(gid), &mut b);
        let path_opt = pb.finish();
        self.glyph_cache.insert(gid, path_opt.clone());
        path_opt
    }
}

struct FontSet {
    faces: Vec<FontFace>,
}

impl FontSet {
    fn new() -> Self {
        static JP_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf");
        static LATIN_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
        let faces = vec![
            FontFace::new("NotoSansJP", JP_BYTES),
            FontFace::new("NotoSans", LATIN_BYTES),
        ];
        Self { faces }
    }

    fn find_glyph(&self, ch: char) -> Option<(usize, u16)> {
        for (i, f) in self.faces.iter().enumerate() {
            if let Some(gid) = f.glyph_id(ch) {
                if gid != 0 {
                    return Some((i, gid));
                }
            }
        }
        None
    }
}

static CACHE: OnceLock<Mutex<FontSet>> = OnceLock::new();

fn cache_lock() -> &'static Mutex<FontSet> {
    CACHE.get_or_init(|| Mutex::new(FontSet::new()))
}

pub struct GlyphResolution {
    pub font_name: &'static str,
    pub glyph_id: u32,
    pub has_outline: bool,
}

pub fn resolve_glyph(c: char) -> GlyphResolution {
    let guard = cache_lock().lock().unwrap();
    if let Some((i, gid)) = guard.find_glyph(c) {
        let name = guard.faces[i].name;
        GlyphResolution { font_name: name, glyph_id: gid as u32, has_outline: gid != 0 }
    } else {
        GlyphResolution { font_name: "fallback", glyph_id: 0, has_outline: false }
    }
}

pub fn measure(text: &str, style: &TextStyle) -> (f32, f32) {
    let guard = cache_lock().lock().unwrap();
    let mut width = 0.0;
    let mut max_height = 0.0f32;
    for ch in text.chars() {
        if let Some((i, gid)) = guard.find_glyph(ch) {
            let face = &guard.faces[i];
            let scale = face.scale_for_size(style.size_px);
            let (adv, _) = face.h_metrics(gid);
            width += adv * scale + style.letter_spacing_px;
            max_height = max_height.max(style.size_px);
        } else {
            // fallback width
            width += style.size_px * 0.6 + style.letter_spacing_px;
        }
    }
    (width, max_height)
}

fn draw_glyph(pixmap: &mut PixmapMut, x: f32, y: f32, face_idx: usize, gid: u16, style: &TextStyle, paint: &Paint) {
    let mut guard = cache_lock().lock().unwrap();
    let face = &mut guard.faces[face_idx];
    if let Some(path) = face.glyph_path(gid) {
        let scale = face.scale_for_size(style.size_px);
        let transform = Transform::from_row(scale, 0.0, 0.0, -scale, x, y);
        pixmap.fill_path(&path, paint, FillRule::Winding, transform, None);
        if style.bold {
            let stroke_width = 0.035 * face.units_per_em * scale;
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(style.color);
            stroke_paint.anti_alias = true;
            let stroke = Stroke { width: stroke_width, ..Default::default() };
            pixmap.stroke_path(&path, &stroke_paint, &stroke, transform, None);
        }
    }
}

pub fn draw(pixmap: &mut PixmapMut, text: &str, x: f32, y: f32, style: &TextStyle, align: Align) {
    let (text_width, _) = measure(text, style);
    let start_x = match align {
        Align::Left => x,
        Align::Center => x - text_width / 2.0,
        Align::Right => x - text_width,
    };
    let mut paint = Paint::default();
    paint.set_color(style.color);
    paint.anti_alias = true;

    let mut cur_x = start_x;
    for ch in text.chars() {
        let maybe = {
            let g = cache_lock().lock().unwrap();
            g.find_glyph(ch)
        };
        if let Some((face_idx, gid)) = maybe {
            draw_glyph(pixmap, cur_x, y, face_idx, gid, style, &paint);
            let (adv, _) = {
                let g = cache_lock().lock().unwrap();
                let face = &g.faces[face_idx];
                face.h_metrics(gid)
            };
            let scale = {
                let g = cache_lock().lock().unwrap();
                let face = &g.faces[face_idx];
                face.scale_for_size(style.size_px)
            };
            cur_x += adv * scale + style.letter_spacing_px;
        } else {
            cur_x += style.size_px * 0.6 + style.letter_spacing_px;
        }
    }
}

pub fn draw_glow(pixmap: &mut PixmapMut, text: &str, x: f32, y: f32, style: &TextStyle, align: Align) {
    let r = (style.color.red() * 255.0) as u8;
    let g = (style.color.green() * 255.0) as u8;
    let b = (style.color.blue() * 255.0) as u8;
    let a = 40u8;
    let glow_style = TextStyle {
        size_px: style.size_px * 1.05,
        bold: style.bold,
        letter_spacing_px: style.letter_spacing_px,
        color: Color::from_rgba8(r, g, b, a),
    };
    draw(pixmap, text, x, y, &glow_style, align);
    draw(pixmap, text, x, y, style, align);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_diagnostics() {
        static FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf");
        println!("len {}", FONT_BYTES.len());
        println!("first16 {:02X?}", &FONT_BYTES[..16]);
        let face = ttf_parser::Face::parse(FONT_BYTES, 0).expect("parse");
        println!("parse ok");
        println!("number_of_glyphs {}", face.number_of_glyphs());
        println!("is_variable {}", face.is_variable());
        println!("units_per_em {}", face.units_per_em());
        let chars = ['『','視','』','儀','光','調','Ø','G','J','死','Ŧ','乂','°','×','·','%','→'];
        for c in chars {
            let gid = face.glyph_index(c);
            println!("char {} U+{:04X} glyph_index {:?}", c, c as u32, gid);
        }
    }

    fn assert_covered(c: char) -> Result<(), String> {
        let res = resolve_glyph(c);
        if res.glyph_id == 0 { Err(format!("glyph id zero for '{}'", c)) } else if !res.has_outline { Err(format!("no outline for '{}'", c)) } else { Ok(()) }
    }

    #[test]
    fn test_glyph_coverage() {
        let chars = ['『','視','』','儀','光','調','Ø','G','死','Ŧ','J','乂'];
        for c in chars { assert!(assert_covered(c).is_ok(), "must be covered: {}", c); }
        for c in 0x20u8..=0x7E { let ch = c as char; assert!(assert_covered(ch).is_ok(), "ASCII must be covered: {}", c); }
        for c in ['°','×','·','%','→'] { assert!(assert_covered(c).is_ok(), "symbol must be covered: {}", c); }
    }

    #[test]
    fn test_glyph_coverage_negative_control() {
        let res = resolve_glyph('\u{E000}');
        assert_eq!(res.glyph_id, 0);
        assert!(!res.has_outline);
        assert!(assert_covered('\u{E000}').is_err());
    }

    #[test]
    fn test_deck_chars_from_bundled_font() {
        let chars = ['『','視','』','儀','光','調','Ø','G','J','死','Ŧ','乂'];
        for c in chars {
            let res = resolve_glyph(c);
            assert!(res.font_name == "NotoSansJP" || res.font_name == "NotoSans", "char {} must be from bundled font, got {}", c, res.font_name);
            assert!(res.glyph_id != 0);
            assert!(res.has_outline);
        }
    }

    #[test]
    fn test_glyph_coverage_direct() {
        static JP: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf");
        static LATIN: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
        let faces = [ttf_parser::Face::parse(JP,0).unwrap(), ttf_parser::Face::parse(LATIN,0).unwrap()];
        let chars = ['『','視','』','儀','光','調','Ø','G','J','死','Ŧ','乂','°','×','·','%'];
        for c in chars {
            let mut found = false;
            for f in &faces {
                if let Some(gid) = f.glyph_index(c) {
                    if gid.0 != 0 {
                        let mut pb = tiny_skia::PathBuilder::new();
                        struct B<'a>{pb:&'a mut tiny_skia::PathBuilder}
                        impl ttf_parser::OutlineBuilder for B<'_>{fn move_to(&mut self,x:f32,y:f32){self.pb.move_to(x,y);}fn line_to(&mut self,x:f32,y:f32){self.pb.line_to(x,y);}fn quad_to(&mut self,x1:f32,y1:f32,x:f32,y:f32){self.pb.quad_to(x1,y1,x,y);}fn curve_to(&mut self,x1:f32,y1:f32,x2:f32,y2:f32,x:f32,y:f32){self.pb.cubic_to(x1,y1,x2,y2,x,y);}fn close(&mut self){self.pb.close();}}
                        let mut b = B{pb:&mut pb};
                        let _ = f.outline_glyph(gid,&mut b);
                        if pb.finish().is_some() { found = true; break; }
                    }
                }
            }
            assert!(found, "char {} not found", c);
        }
    }

    #[test]
    fn test_glyph_renders_ink() {
        use tiny_skia::{Pixmap, Color};
        let style = TextStyle { size_px: 64.0, bold: false, letter_spacing_px: 0.0, color: Color::from_rgba8(255,255,255,255) };
        let chars = ['I','視','Ŧ'];
        for c in chars {
            let mut p = Pixmap::new(256,256).unwrap();
            let mut pm = p.as_mut();
            draw(&mut pm, &c.to_string(), 10.0, 200.0, &style, Align::Left);
            let lit = pm.data_mut().iter().step_by(4).filter(|&&v| v > 0).count();
            assert!(lit >= 100, "char {} lit pixels {} < 100", c, lit);
        }
    }

    #[test]
    fn test_ink_bbox_matches_pen() {
        use tiny_skia::{Pixmap, Color};
        let style = TextStyle { size_px: 64.0, bold: false, letter_spacing_px: 0.0, color: Color::from_rgba8(255,255,255,255) };
        for (ch, expected_face) in [('I',"NotoSansJP"), ('視',"NotoSansJP"), ('Ŧ',"NotoSans")] {
            let mut pix = Pixmap::new(256,256).unwrap();
            {
                let mut pm = pix.as_mut();
                draw(&mut pm, &ch.to_string(), 100.0, 200.0, &style, Align::Left);
            }
            // get xMin from font
            let (_face_idx, _gid, scale, x_min) = {
                let mut g = cache_lock().lock().unwrap();
                let (face_idx, gid) = g.find_glyph(ch).expect("glyph");
                let scale = g.faces[face_idx].scale_for_size(style.size_px);
                // outline to get bbox (requires mutable access for cache)
                let path_opt = g.faces[face_idx].glyph_path(gid);
                let x_min = path_opt.as_ref().map(|p| p.bounds().left()).unwrap_or(0.0);
                (face_idx, gid, scale, x_min)
            };
            let expected = 100.0 + x_min * scale;
            let width = pix.width() as usize;
            let height = pix.height() as usize;
            let stride = width * 4;
            let data = pix.data();
            let mut leftmost = 256;
            for y in 0..height {
                let row_start = y * stride;
                for x in 0..width {
                    let idx = row_start + x * 4 + 3;
                    if data[idx] > 0 { leftmost = leftmost.min(x); break; }
                }
            }
            let measured = leftmost as f32;
            println!("char {} measured {} expected {}", ch, measured, expected);
            assert!((measured - expected).abs() <= 1.0, "char {} leftmost {} vs expected {}", ch, measured, expected);
            let res = resolve_glyph(ch);
            if ch != 'I' { assert_eq!(res.font_name, expected_face); }
        }
    }

    #[test]
    fn test_bold_increases_ink() {
        use tiny_skia::{Pixmap, Color};
        let style_reg = TextStyle { size_px: 64.0, bold: false, letter_spacing_px: 0.0, color: Color::from_rgba8(255,255,255,255) };
        let style_bold = TextStyle { size_px: 64.0, bold: true, letter_spacing_px: 0.0, color: Color::from_rgba8(255,255,255,255) };
        for c in ['I','視','Ŧ'] {
            let mut p1 = Pixmap::new(256,256).unwrap();
            draw(&mut p1.as_mut(), &c.to_string(), 10.0, 200.0, &style_reg, Align::Left);
            let lit1 = p1.data().iter().step_by(4).filter(|&&v| v > 0).count();
            let mut p2 = Pixmap::new(256,256).unwrap();
            draw(&mut p2.as_mut(), &c.to_string(), 10.0, 200.0, &style_bold, Align::Left);
            let lit2 = p2.data().iter().step_by(4).filter(|&&v| v > 0).count();
            assert!(lit2 >= (lit1 as f32 * 1.08) as usize, "bold not larger for {}: {} vs {}", c, lit1, lit2);
        }
    }
}
