//! Unified OSD 4K/1080p Renderer (60% live dyno preview on left, 40% controls on right).
#![allow(clippy::field_reassign_with_default)]
use crate::dyno::dyno_telemetry;
use crate::model::{get_row_label, get_row_value, UnifiedOsdState, TAB_NAMES};
use fh6_core::config::AppConfig;
use fh6_core::telemetry::FH6TelemetryData;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

#[derive(Debug)]
pub(crate) struct DeckLayout {
    pub _stage_x: f32,
    pub _stage_w: f32,
    pub deck_x: f32,
    pub deck_w: f32,
    pub card_x: f32,
    pub card_y: f32,
    pub card_w: f32,
    pub card_h: f32,
    pub radius: f32,
    pub header_h: f32,
    pub pill_w: f32,
    pub pill_h: f32,
    pub watermark: Rect,
    pub _body_rows: Rect,
    pub footer: Rect,
    #[cfg_attr(not(test), expect(dead_code, reason = "OSD renderer rebuilt in Phase 9"))]
    pub gutter: Rect,
}

pub(crate) fn deck_layout(canvas_w: u32, canvas_h: u32) -> DeckLayout {
    let w = canvas_w as f32;
    let h = canvas_h as f32;
    let base_w = 3840.0;
    // TODO(phase-7): base_h was likely meant to make `s` height-aware (min of w/base_w and h/base_h) for non-16:9 canvases.
    let _base_h = 2160.0;
    let s = (w / base_w).max(0.25);
    let deck_x = (0.60 * w).round();
    let deck_w = (w - deck_x).max(0.0);
    let inset = 48.0 * s;
    let card_x = deck_x + inset;
    let card_y = 48.0 * s;
    let card_w = (deck_w - 2.0 * inset).max(0.0);
    let card_h = (h - 2.0 * inset).max(0.0);
    let radius = 56.0 * s;
    let header_h = 72.0 * s;
    let pill_h = 44.0 * s;
    let pill_w = ((card_w - 32.0 * s - 24.0 * s).max(0.0)) / 4.0;
    let wm_w = 360.0 * s;
    let wm_h = 32.0 * s;
    let watermark = Rect::from_xywh((card_x + card_w - wm_w).max(card_x), (card_y + header_h - 8.0 * s).max(card_y), wm_w.max(0.0), wm_h.max(0.0)).unwrap_or_else(|| Rect::from_xywh(card_x, card_y, 0.0, 0.0).unwrap());
    let body_w = (card_w - 32.0 * s).max(0.0);
    let body_h = (card_h - header_h - 80.0 * s).max(0.0);
    let body_rows = Rect::from_xywh((card_x + 16.0 * s).max(card_x), (card_y + header_h + 16.0 * s).max(card_y), body_w, body_h).unwrap_or_else(|| Rect::from_xywh(card_x, card_y, 0.0, 0.0).unwrap());
    let footer_h = 48.0 * s;
    let footer = Rect::from_xywh(deck_x, (h - footer_h).max(0.0), deck_w, footer_h).unwrap_or_else(|| Rect::from_xywh(deck_x, 0.0, 0.0, 0.0).unwrap());
    let gutter_w = 24.0 * s;
    let gutter_h = body_h;
    let gutter = Rect::from_xywh((card_x + 16.0 * s).max(card_x), (card_y + header_h + 16.0 * s).max(card_y), gutter_w, gutter_h).unwrap_or_else(|| Rect::from_xywh(card_x, card_y, 0.0, 0.0).unwrap());
    DeckLayout {
        _stage_x: 0.0,
        _stage_w: 2304.0,
        deck_x,
        deck_w,
        card_x,
        card_y,
        card_w,
        card_h,
        radius,
        header_h,
        pill_w,
        pill_h,
        watermark,
        _body_rows: body_rows,
        footer,
        gutter,
    }
}

pub struct OsdRenderer {
    pub width: u32,
    pub height: u32,
}

impl Default for OsdRenderer {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

impl OsdRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Renders the complete Unified OSD frame to a tiny-skia Pixmap.
    pub fn render_frame(
        &self,
        pixmap: &mut PixmapMut,
        state: &UnifiedOsdState,
        config: &AppConfig,
        telemetry: Option<&FH6TelemetryData>,
    ) {
        let w = self.width as f32;
        let h = self.height as f32;

        // 1. OLED True-Black backdrop
        let mut bg_paint = Paint::default();
        bg_paint.set_color(Color::from_rgba8(6, 10, 16, 238));
        bg_paint.anti_alias = true;
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, w, h) {
            pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
        }

        // Left stage left untouched – clip to 2304
        let left_w = 2304.0_f32.min(w);
        let panel_h = h - 80.0;
        self.render_panel_box(pixmap, 40.0, 40.0, left_w - 80.0, panel_h, true);
        self.render_dyno_cluster(pixmap, 40.0, 130.0, left_w - 80.0, panel_h - 100.0, config, telemetry);

        // Right deck – macOS 27 Liquid Glass
        let layout = deck_layout(self.width, self.height);
        let deck_x = layout.deck_x;
        let deck_w = layout.deck_w;
        let card_x = layout.card_x;
        let card_y = layout.card_y;
        let card_w = layout.card_w;
        let card_h = layout.card_h;
        let radius = layout.radius;

        // Behind card vertical gradient
        self.fill_rect_gradient(pixmap, deck_x + 48.0, card_y, deck_w - 96.0, card_h, Color::from_rgba8(8,14,26,255), Color::from_rgba8(2,4,8,255));
        // Low-alpha azure bloom top-left
        let mut bloom_paint = Paint::default();
        bloom_paint.set_color(Color::from_rgba8(56,189,248,32));
        bloom_paint.anti_alias = true;
        if let Some(rect) = Rect::from_xywh(card_x, card_y, card_w*0.4, card_h*0.3) {
            pixmap.fill_rect(rect, &bloom_paint, Transform::identity(), None);
        }

        // Frosted sapphire body
        let mut card_paint = Paint::default();
        card_paint.set_color(Color::from_rgba8(14,20,34,28));
        card_paint.anti_alias = true;
        self.fill_rounded_rect(pixmap, card_x, card_y, card_w, card_h, radius, &card_paint);

        // Dual-stop specular caustic rim
        self.draw_liquid_glass_rim(pixmap, card_x, card_y, card_w, card_h, radius);

        // Arched upper lens sheen
        self.draw_lens_sheen(pixmap, card_x, card_y, card_w, card_h);

        // Header pills + watermark
        let header_h = layout.header_h;
        let pill_h = layout.pill_h;
        let pill_w = layout.pill_w;
        for (i, name) in TAB_NAMES.iter().enumerate() {
            let is_active = i == state.current_tab;
            let px = card_x + 16.0 + i as f32 * (pill_w + 8.0);
            let py = card_y + 16.0;
            self.render_liquid_pill(pixmap, px, py, pill_w, pill_h, name, is_active);
        }
        // Watermark right-aligned
        let wm = if !config.custom_driver_name.trim().is_empty() { config.custom_driver_name.clone() } else { "ØG『死』ŦJ乂".to_string() };
        let wm_x = layout.watermark.x();
        let wm_y = layout.watermark.y();
        self.draw_simple_text(pixmap, wm_x + 2.0, wm_y + 2.0, &wm, Color::from_rgba8(255,255,255,40), 1.8);
        self.draw_simple_text(pixmap, wm_x, wm_y, &wm, Color::from_rgba8(255,255,255,210), 1.8);

        // Body rows
        let rows_start_y = card_y + header_h + 16.0;
        let row_h = 52.0;
        let max_rows = UnifiedOsdState::max_rows_for_tab(state.current_tab);
        let visible_count = 8.min(max_rows);
        let first_row = if state.selected_row >= visible_count { (state.selected_row + 1 - visible_count).min(max_rows.saturating_sub(visible_count)) } else { 0 };
        for i in 0..visible_count {
            let row_idx = first_row + i;
            if row_idx >= max_rows { break; }
            let is_sel = row_idx == state.selected_row;
            let ry = rows_start_y + (i as f32) * (row_h + 6.0);
            let label = get_row_label(state.current_tab, row_idx);
            let val = get_row_value(state.current_tab, row_idx, config);
            self.render_row_item_liquid(pixmap, card_x + 16.0, ry, card_w - 32.0, row_h, label, &val, is_sel);
        }

        // Footer hints
        let footer_y = layout.footer.y() + layout.footer.height() / 2.0;
        self.draw_simple_text(pixmap, layout.footer.x() + 64.0, footer_y, "LB / RB  Tabs", Color::from_rgba8(148,165,190,200), 1.4);
        self.draw_simple_text(pixmap, layout.footer.x() + layout.footer.width()*0.5 - 120.0, footer_y, "D-Pad  Select / Adjust", Color::from_rgba8(148,165,190,200), 1.4);
        self.draw_simple_text(pixmap, layout.footer.x() + layout.footer.width() - 300.0, footer_y, "View + Menu  Close", Color::from_rgba8(148,165,190,200), 1.4);
    }

    // ---- Liquid Glass helpers ----
    fn fill_rounded_rect(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, _radius: f32, paint: &Paint) {
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            pixmap.fill_rect(rect, paint, Transform::identity(), None);
        }
    }
    fn fill_rect_gradient(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) {
        let steps = h as u32;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let r = (top.red() * (1.0 - t) + bottom.red() * t) as u8;
            let g = (top.green() * (1.0 - t) + bottom.green() * t) as u8;
            let b = (top.blue() * (1.0 - t) + bottom.blue() * t) as u8;
            let a = (top.alpha() * (1.0 - t) + bottom.alpha() * t) as u8;
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(r,g,b,a));
            if let Some(rect) = Rect::from_xywh(x, y + i as f32, w, 1.0) {
                pixmap.fill_rect(rect, &p, Transform::identity(), None);
            }
        }
    }
    fn draw_liquid_glass_rim(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, _radius: f32) {
        // Approximate rim with two strokes
        let mut top_paint = Paint::default();
        top_paint.set_color(Color::from_rgba8(255,255,255,178));
        top_paint.anti_alias = true;
        let mut bot_paint = Paint::default();
        bot_paint.set_color(Color::from_rgba8(56,189,248,102));
        bot_paint.anti_alias = true;
        let stroke = Stroke { width: 2.0, ..Default::default() };
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &top_paint, &stroke, Transform::identity(), None);
            pixmap.stroke_path(&path, &bot_paint, &stroke, Transform::identity(), None);
        }
    }
    fn draw_lens_sheen(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32) {
        let sheen_h = h * 0.28;
        for i in 0..(sheen_h as u32) {
            let t = i as f32 / sheen_h;
            let a = (38.0 * (1.0 - t)) as u8;
            if a == 0 { continue; }
            let mut p = Paint::default();
            p.set_color(Color::from_rgba8(255,255,255,a));
            p.anti_alias = true;
            if let Some(rect) = Rect::from_xywh(x, y + i as f32, w, 1.0) {
                pixmap.fill_rect(rect, &p, Transform::identity(), None);
            }
        }
    }
    fn render_liquid_pill(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, name: &str, is_active: bool) {
        let mut bg = Paint::default();
        bg.anti_alias = true;
        if is_active {
            bg.set_color(Color::from_rgba8(14,116,144,60));
        } else {
            bg.set_color(Color::from_rgba8(255,255,255,20));
        }
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            pixmap.fill_rect(rect, &bg, Transform::identity(), None);
        }
        let mut border = Paint::default();
        border.anti_alias = true;
        border.set_color(if is_active { Color::from_rgba8(56,189,248,255) } else { Color::from_rgba8(255,255,255,45) });
        let stroke = Stroke { width: if is_active {1.5} else {1.0}, ..Default::default() };
        let mut pb = PathBuilder::new();
        pb.move_to(x,y); pb.line_to(x+w,y); pb.line_to(x+w,y+h); pb.line_to(x,y+h); pb.close();
        if let Some(path)=pb.finish() { pixmap.stroke_path(&path, &border, &stroke, Transform::identity(), None); }
        let text_color = if is_active { Color::from_rgba8(255,255,255,255) } else { Color::from_rgba8(175,192,214,210) };
        self.draw_simple_text(pixmap, x+16.0, y+14.0, name, text_color, 1.4);
    }
    #[expect(clippy::too_many_arguments, reason = "legacy tiny-skia helper, replaced by the fh6-kit GPU renderer in Phase 5")]
    fn render_row_item_liquid(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, label: &str, val: &str, is_sel: bool) {
        if is_sel {
            let mut bg = Paint::default();
            bg.set_color(Color::from_rgba8(14,116,144,90));
            bg.anti_alias = true;
            if let Some(rect)=Rect::from_xywh(x,y,w,h) { pixmap.fill_rect(rect,&bg,Transform::identity(),None); }
        }
        let label_color = if is_sel { Color::from_rgba8(255,255,255,255) } else { Color::from_rgba8(200,215,232,220) };
        self.draw_simple_text(pixmap, x+16.0, y+15.0, label, label_color, 1.3);
        let val_color = Color::from_rgba8(0,229,255,255);
        let val_x = x + w - (val.len() as f32 * 10.0 + 20.0).max(120.0);
        self.draw_simple_text(pixmap, val_x, y+15.0, val, val_color, 1.4);
    }

    fn render_panel_box(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, _is_live: bool) {
        let mut bg_paint = Paint::default();
        bg_paint.set_color(Color::from_rgba8(14, 22, 34, 38));
        bg_paint.anti_alias = true;

        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
        }

        let mut stroke_paint = Paint::default();
        stroke_paint.set_color(Color::from_rgba8(56, 189, 248, 75));
        stroke_paint.anti_alias = true;
        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }

    #[expect(dead_code, reason = "OSD renderer rebuilt in Phase 9")]
    fn render_tab_item(&self, pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, name: &str, is_active: bool) {
        let mut bg_paint = Paint::default();
        bg_paint.anti_alias = true;
        let mut border_paint = Paint::default();
        border_paint.anti_alias = true;

        if is_active {
            bg_paint.set_color(Color::from_rgba8(14, 116, 144, 160));
            border_paint.set_color(Color::from_rgba8(56, 189, 248, 255));
        } else {
            bg_paint.set_color(Color::from_rgba8(255, 255, 255, 20));
            border_paint.set_color(Color::from_rgba8(255, 255, 255, 45));
        }

        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
        }

        let stroke = Stroke {
            width: if is_active { 1.5 } else { 1.0 },
            ..Default::default()
        };
        let mut pb = PathBuilder::new();
        pb.move_to(x, y);
        pb.line_to(x + w, y);
        pb.line_to(x + w, y + h);
        pb.line_to(x, y + h);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
        }

        let text_color = if is_active {
            Color::from_rgba8(255, 255, 255, 255)
        } else {
            Color::from_rgba8(175, 192, 214, 210)
        };
        self.draw_simple_text(pixmap, x + 16.0, y + 14.0, name, text_color, 1.4);
    }

    #[allow(clippy::too_many_arguments)]
    #[expect(dead_code, reason = "OSD renderer rebuilt in Phase 9")]
    fn render_row_item(
        &self,
        pixmap: &mut PixmapMut,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        val: &str,
        is_sel: bool,
    ) {
        if is_sel {
            let mut bg_paint = Paint::default();
            bg_paint.set_color(Color::from_rgba8(14, 116, 144, 90));
            bg_paint.anti_alias = true;
            if let Some(rect) = Rect::from_xywh(x, y, w, h) {
                pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);
            }

            let mut border_paint = Paint::default();
            border_paint.set_color(Color::from_rgba8(56, 189, 248, 180));
            border_paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.0,
                ..Default::default()
            };
            let mut pb = PathBuilder::new();
            pb.move_to(x, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h);
            pb.line_to(x, y + h);
            pb.close();
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
            }
        }

        let label_color = if is_sel {
            Color::from_rgba8(255, 255, 255, 255)
        } else {
            Color::from_rgba8(200, 215, 232, 220)
        };
        self.draw_simple_text(pixmap, x + 16.0, y + 15.0, label, label_color, 1.3);

        let val_color = Color::from_rgba8(0, 229, 255, 255); // Neon Cyan
        let val_x = x + w - (val.len() as f32 * 10.0 + 20.0).max(120.0);
        self.draw_simple_text(pixmap, val_x, y + 15.0, val, val_color, 1.4);
    }

    fn render_dyno_cluster(
        &self,
        pixmap: &mut PixmapMut,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        config: &AppConfig,
        telemetry: Option<&FH6TelemetryData>,
    ) {
        let dyno_fallback = dyno_telemetry(4.0);
        let telemetry_data = telemetry.unwrap_or(&dyno_fallback);
        let theme = config.cockpit_hud_style_typed;
        let bounds = Rect::from_xywh(x, y, w, h).expect("valid bounds");
        if let Some(mut renderer) = fh6_clusters::create_cluster(theme) {
            renderer.render(pixmap, bounds, telemetry_data, config);
        }
    }

    /// Text renderer using bundled font
    fn draw_simple_text(&self, pixmap: &mut PixmapMut, x: f32, y: f32, text: &str, color: Color, scale: f32) {
        let size_px = 14.0 * scale;
        let style = crate::text::TextStyle { size_px, bold: false, letter_spacing_px: 0.0, color };
        crate::text::draw(pixmap, text, x, y, &style, crate::text::Align::Left);
    }

    #[expect(dead_code, reason = "OSD renderer rebuilt in Phase 9")]
    fn draw_vector_char(&self, pixmap: &mut PixmapMut, x: f32, y: f32, ch: char, paint: &Paint, scale: f32) {
        let stroke = Stroke {
            width: (1.2 * scale).max(1.0),
            ..Default::default()
        };
        let mut pb = PathBuilder::new();
        let w = 5.0 * scale;
        let h = 8.0 * scale;

        match ch.to_ascii_uppercase() {
            'A' => {
                pb.move_to(x, y + h);
                pb.line_to(x, y + h * 0.3);
                pb.line_to(x + w * 0.5, y);
                pb.line_to(x + w, y + h * 0.3);
                pb.line_to(x + w, y + h);
                pb.move_to(x, y + h * 0.55);
                pb.line_to(x + w, y + h * 0.55);
            }
            'B' => {
                pb.move_to(x, y);
                pb.line_to(x + w * 0.8, y);
                pb.line_to(x + w, y + h * 0.25);
                pb.line_to(x, y + h * 0.5);
                pb.line_to(x + w, y + h * 0.75);
                pb.line_to(x + w * 0.8, y + h);
                pb.line_to(x, y + h);
                pb.close();
            }
            'C' => {
                pb.move_to(x + w, y);
                pb.line_to(x, y);
                pb.line_to(x, y + h);
                pb.line_to(x + w, y + h);
            }
            'D' => {
                pb.move_to(x, y);
                pb.line_to(x + w * 0.7, y);
                pb.line_to(x + w, y + h * 0.5);
                pb.line_to(x + w * 0.7, y + h);
                pb.line_to(x, y + h);
                pb.close();
            }
            'E' => {
                pb.move_to(x + w, y);
                pb.line_to(x, y);
                pb.line_to(x, y + h);
                pb.line_to(x + w, y + h);
                pb.move_to(x, y + h * 0.5);
                pb.line_to(x + w * 0.7, y + h * 0.5);
            }
            '0'..='9' => {
                // Digits: draw high-legibility box contours
                pb.move_to(x, y);
                pb.line_to(x + w, y);
                pb.line_to(x + w, y + h);
                pb.line_to(x, y + h);
                pb.close();
                if ch == '1' {
                    pb = PathBuilder::new();
                    pb.move_to(x + w * 0.5, y);
                    pb.line_to(x + w * 0.5, y + h);
                } else if ch == '4' {
                    pb = PathBuilder::new();
                    pb.move_to(x + w, y + h);
                    pb.line_to(x + w, y);
                    pb.line_to(x, y + h * 0.5);
                    pb.line_to(x + w, y + h * 0.5);
                }
            }
            '*' | '•' => {
                pb.move_to(x + w * 0.3, y + h * 0.5);
                pb.line_to(x + w * 0.7, y + h * 0.5);
            }
            '/' => {
                pb.move_to(x, y + h);
                pb.line_to(x + w, y);
            }
            ':' => {
                pb.move_to(x + w * 0.5, y + h * 0.25);
                pb.line_to(x + w * 0.5, y + h * 0.35);
                pb.move_to(x + w * 0.5, y + h * 0.65);
                pb.line_to(x + w * 0.5, y + h * 0.75);
            }
            _ => {
                // Default box marker for unrecognized glyphs
                pb.move_to(x, y);
                pb.line_to(x + w, y);
                pb.line_to(x + w, y + h);
                pb.line_to(x, y + h);
                pb.close();
            }
        }

        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    #[test]
    fn test_deck_layout_scales() {
        let l4k = deck_layout(3840, 2160);
        let l1080 = deck_layout(1920, 1080);
        let s = 1920.0 / 3840.0;
        // every rect scales ~ by s
        assert!((l1080.deck_x - 0.6 * 1920.0).abs() < 1.0);
        assert!((l1080.card_x - l4k.card_x * s).abs() < 2.0);
        assert!((l1080.card_w - l4k.card_w * s).abs() < 2.0);
        assert!((l1080.card_h - l4k.card_h * s).abs() < 2.0);
        assert!((l1080.header_h - l4k.header_h * s).abs() < 1.0);
        assert!((l1080.pill_w - l4k.pill_w * s).abs() < 1.0);
        // inside canvas
        let w = 1920.0; let h = 1080.0;
        assert!(l1080.deck_x >= 0.0 && l1080.deck_x + l1080.deck_w <= w + 1.0);
        assert!(l1080.card_x >= l1080.deck_x && l1080.card_x + l1080.card_w <= w);
        assert!(l1080.card_y >= 0.0 && l1080.card_y + l1080.card_h <= h);
        // pills inside header
        let pill_right = l1080.card_x + 4.0 * l1080.pill_w + 3.0 * 8.0;
        assert!(pill_right <= l1080.card_x + l1080.card_w + 1.0);
        // footer inside card
        assert!(l1080.footer.y() + l1080.footer.height() <= h + 1.0);
        // gutter inside card
        assert!(l1080.gutter.x() >= l1080.card_x && l1080.gutter.x() + l1080.gutter.width() <= l1080.card_x + l1080.card_w);
    }

    #[test]
    fn test_osd_render_allocates_and_draws() {
        let renderer = OsdRenderer::default();
        let mut pixmap = Pixmap::new(renderer.width, renderer.height).expect("pixmap allocate");
        let state = UnifiedOsdState::default();
        let config = AppConfig::default();

        renderer.render_frame(&mut pixmap.as_mut(), &state, &config, None);

        // Check that pixels were modified (not all zero)
        let non_zero_pixels = pixmap.data().iter().filter(|&&b| b != 0).count();
        assert!(non_zero_pixels > 1000, "OSD frame should render visible graphics");
    }

    #[test]
    fn test_deck_dump() {
        use std::fs;
        let out_dir = std::path::Path::new("target/osd_preview");
        let _ = fs::create_dir_all(out_dir);
        let renderer = OsdRenderer::new(3840, 2160);
        let mut pixmap = tiny_skia::Pixmap::new(3840, 2160).expect("pixmap");
        let state = UnifiedOsdState::default();
        let config = AppConfig::default();
        renderer.render_frame(&mut pixmap.as_mut(), &state, &config, None);
        assert_eq!(pixmap.width(), 3840);
        assert_eq!(pixmap.height(), 2160);
        // boundary differ (deck starts at 2304, card inset 48)
        let idx_left = (1080 * 3840 + 2303) * 4;
        let idx_right = (1080 * 3840 + 2352) * 4;
        let left = &pixmap.data()[idx_left..idx_left+4];
        let right = &pixmap.data()[idx_right..idx_right+4];
        assert_ne!(left, right);
        let path = out_dir.join("UnifiedOsd_4K_deck.png");
        let png_data = pixmap.encode_png().expect("encode");
        std::fs::write(&path, &png_data).expect("write");
        assert!(path.exists());
        // crops via tiny-skia copy
        let w = 1536u32;
        let h_header = 420u32;
        let h_body = 680u32;
        fn crop_save(src: &tiny_skia::Pixmap, x: u32, y: u32, w: u32, h: u32, out_path: &std::path::Path) {
            let mut dst = tiny_skia::Pixmap::new(w, h).unwrap();
            let src_data = src.data();
            let dst_data = dst.data_mut();
            let src_stride = src.width() as usize * 4;
            let dst_stride = w as usize * 4;
            for row in 0..h as usize {
                let src_start = ((y as usize + row) * src_stride) + (x as usize * 4);
                let dst_start = row * dst_stride;
                dst_data[dst_start..dst_start+dst_stride].copy_from_slice(&src_data[src_start..src_start+dst_stride]);
            }
            let png_data = dst.encode_png().unwrap();
            std::fs::write(out_path, &png_data).unwrap();
        }
        crop_save(&pixmap, 2304, 0, w, h_header, &out_dir.join("deck_header_crop.png"));
        crop_save(&pixmap, 2304, 420, w, h_body, &out_dir.join("deck_body_crop.png"));
    }

}
