//! The Phase 4 test cluster: deliberately simple, driven by live telemetry, proving the whole
//! packet-to-pixel path. Real clusters arrive in Phases 5 and 7.
//!
//! Layout is authored in 4K reference pixels around the anchor and multiplied by
//! [`TestClusterInput::scale`]. Building it allocates nothing and reads no clock.
use crate::quad::{QuadBatch, QuadInstance, QuadKind};
use crate::seven_seg::{push_glyph, push_text, SegStyle, StackStr};
use std::fmt::Write as _;

/// Panel size in reference pixels.
const PANEL_W: f32 = 760.0;
const PANEL_H: f32 = 300.0;
/// Number of LEDs in the RPM bar.
const RPM_LEDS: usize = 30;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GHOST: [f32; 4] = [1.0, 1.0, 1.0, 0.07];
const AMBER: [f32; 4] = [1.0, 0.702, 0.0, 1.0];
const CRIMSON: [f32; 4] = [1.0, 0.09, 0.267, 1.0];
const THROTTLE: [f32; 4] = [0.463, 1.0, 0.012, 0.95];
/// Glass backplate: OLED black at the top of the allowed glass alpha range (64/255).
const GLASS: [f32; 4] = [0.0, 0.0, 0.0, 0.25];

/// Everything the test cluster shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestClusterInput {
    /// Speed in display units (mph or km/h).
    pub speed: f32,
    /// Raw telemetry gear: 0 = reverse, 1..=10 = gears, above 10 = neutral.
    pub gear: u8,
    /// Engine speed.
    pub rpm: f32,
    /// Redline engine speed (the bar's full scale).
    pub max_rpm: f32,
    /// Fraction of `max_rpm` where the shift zone (amber) starts.
    pub shift_start: f32,
    /// Fraction of `max_rpm` where the redline zone (crimson) starts.
    pub redline: f32,
    /// Throttle 0..=1.
    pub throttle: f32,
    /// Brake 0..=1.
    pub brake: f32,
    /// Panel centre in target pixels.
    pub anchor: [f32; 2],
    /// Reference-pixel to target-pixel multiplier.
    pub scale: f32,
    /// Accent colour (brand primary), straight alpha, sRGB-encoded.
    pub accent: [f32; 4],
}

fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// The glyph for a raw telemetry gear.
pub fn gear_char(gear: u8) -> char {
    match gear {
        0 => 'r',
        1..=9 => char::from(b'0' + gear),
        10 => '0',
        _ => 'n',
    }
}

/// Appends the test cluster to `batch`.
pub fn build(batch: &mut QuadBatch, input: &TestClusterInput) {
    let s = finite(input.scale).clamp(0.1, 8.0);
    let ox = finite(input.anchor[0]) - PANEL_W * s * 0.5;
    let oy = finite(input.anchor[1]) - PANEL_H * s * 0.5;
    // Reference-space rectangle to target space.
    let r = |x: f32, y: f32, w: f32, h: f32| [ox + x * s, oy + y * s, w * s, h * s];

    batch.push(QuadInstance::new(
        r(0.0, 0.0, PANEL_W, PANEL_H),
        GLASS,
        QuadKind::RoundedRect {
            radius: 40.0 * s,
            rim: 0.0,
        },
    ));
    let mut rim = input.accent;
    rim[3] *= 0.55;
    batch.push(QuadInstance::new(
        r(0.0, 0.0, PANEL_W, PANEL_H),
        rim,
        QuadKind::RoundedRect {
            radius: 40.0 * s,
            rim: (1.5 * s).max(1.0),
        },
    ));

    // RPM LED bar.
    let max_rpm = if finite(input.max_rpm) > 500.0 { input.max_rpm } else { 8000.0 };
    let frac = (finite(input.rpm) / max_rpm).clamp(0.0, 1.0);
    let lit = (frac * RPM_LEDS as f32).round() as usize;
    let (bar_x, bar_w, gap) = (40.0, PANEL_W - 80.0, 6.0);
    let led_w = (bar_w - gap * (RPM_LEDS as f32 - 1.0)) / RPM_LEDS as f32;
    for i in 0..RPM_LEDS {
        let pos = (i as f32 + 0.5) / RPM_LEDS as f32;
        let color = if i >= lit {
            GHOST
        } else if pos >= input.redline {
            CRIMSON
        } else if pos >= input.shift_start {
            AMBER
        } else {
            input.accent
        };
        batch.push(QuadInstance::new(
            r(bar_x + i as f32 * (led_w + gap), 30.0, led_w, 30.0),
            color,
            QuadKind::RoundedRect {
                radius: 4.0 * s,
                rim: 0.0,
            },
        ));
    }

    // Speed: three tabular digits, right-aligned, over ghost 8s.
    let speed_style = SegStyle {
        height: 120.0 * s,
        color: WHITE,
        ghost: Some(GHOST),
    };
    let mut speed = StackStr::<8>::new();
    let shown = finite(input.speed).abs().round().min(999.0) as u32;
    let _ = write!(speed, "{shown:>3}");
    let speed_w = speed_style.text_width(speed.as_str());
    push_text(batch, speed.as_str(), ox + 300.0 * s - speed_w, oy + 100.0 * s, &speed_style);

    // Gear: one large glyph in the centre.
    let gear_style = SegStyle {
        height: 180.0 * s,
        color: WHITE,
        ghost: Some(GHOST),
    };
    let gear_mask = crate::seven_seg::mask_for(gear_char(input.gear)).unwrap_or(0);
    push_glyph(
        batch,
        gear_mask,
        ox + (PANEL_W * 0.5) * s - gear_style.glyph_width() * 0.5,
        oy + 88.0 * s,
        &gear_style,
    );

    // Engine speed, small, right side.
    let rpm_style = SegStyle {
        height: 44.0 * s,
        color: [1.0, 1.0, 1.0, 0.75],
        ghost: None,
    };
    let mut rpm_text = StackStr::<8>::new();
    let _ = write!(rpm_text, "{:>5}", finite(input.rpm).clamp(0.0, 99_999.0).round() as u32);
    let rpm_w = rpm_style.text_width(rpm_text.as_str());
    push_text(batch, rpm_text.as_str(), ox + 600.0 * s - rpm_w, oy + 228.0 * s, &rpm_style);

    // Throttle and brake bars, filling upwards.
    for (i, (value, color)) in [(input.throttle, THROTTLE), (input.brake, CRIMSON)].into_iter().enumerate() {
        let x = 640.0 + i as f32 * 36.0;
        let (top, height) = (100.0, 172.0);
        batch.push(QuadInstance::new(
            r(x, top, 18.0, height),
            GHOST,
            QuadKind::RoundedRect {
                radius: 6.0 * s,
                rim: 0.0,
            },
        ));
        let fill = finite(value).clamp(0.0, 1.0) * height;
        if fill >= 1.0 {
            batch.push(QuadInstance::new(
                r(x, top + height - fill, 18.0, fill),
                color,
                QuadKind::RoundedRect {
                    radius: 6.0 * s,
                    rim: 0.0,
                },
            ));
        }
    }
}

/// One line of the `--stats` overlay: a short seven-segment label and a number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatLine {
    /// Label drawn with seven segments, e.g. "P50", "CPU".
    pub label: &'static str,
    /// The value.
    pub value: f32,
    /// Digits after the decimal point.
    pub decimals: usize,
}

/// Draws the stats lines at `origin` (top-left, target pixels), `line_height` tall each.
pub fn build_stats(batch: &mut QuadBatch, origin: [f32; 2], line_height: f32, lines: &[StatLine]) {
    let label_style = SegStyle {
        height: line_height,
        color: [0.0, 0.824, 1.0, 1.0],
        ghost: None,
    };
    let value_style = SegStyle {
        height: line_height,
        color: WHITE,
        ghost: None,
    };
    let label_w = label_style.advance() * 4.0;
    for (row, line) in lines.iter().enumerate() {
        let y = origin[1] + row as f32 * line_height * 1.5;
        push_text(batch, line.label, origin[0], y, &label_style);
        let mut text = StackStr::<16>::new();
        let _ = write!(text, "{:>7.*}", line.decimals, finite(line.value).clamp(-99_999.0, 99_999.0));
        push_text(batch, text.as_str(), origin[0] + label_w, y, &value_style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> TestClusterInput {
        TestClusterInput {
            speed: 123.4,
            gear: 4,
            rpm: 6500.0,
            max_rpm: 8000.0,
            shift_start: 0.72,
            redline: 0.94,
            throttle: 0.8,
            brake: 0.0,
            anchor: [1920.0, 1835.0],
            scale: 1.45,
            accent: [0.0, 0.824, 1.0, 1.0],
        }
    }

    #[test]
    fn gear_glyphs() {
        assert_eq!(gear_char(0), 'r');
        assert_eq!(gear_char(1), '1');
        assert_eq!(gear_char(9), '9');
        assert_eq!(gear_char(11), 'n');
    }

    #[test]
    fn rpm_drives_lit_leds() {
        let lit_count = |rpm: f32| {
            let mut batch = QuadBatch::new();
            build(&mut batch, &TestClusterInput { rpm, ..input() });
            batch
                .as_slice()
                .iter()
                .filter(|q| q.params[1] == 0.0 && q.rect[3] > 40.0 && q.rect[3] < 45.0 && q.color != GHOST)
                .count()
        };
        assert_eq!(lit_count(0.0), 0);
        assert_eq!(lit_count(8000.0), RPM_LEDS);
        assert!(lit_count(4000.0) > 10 && lit_count(4000.0) < 20);
    }

    #[test]
    fn panel_stays_inside_a_4k_frame() {
        let mut batch = QuadBatch::new();
        build(&mut batch, &input());
        for q in batch.as_slice() {
            assert!(q.rect[0] >= 0.0 && q.rect[0] + q.rect[2] <= 3840.0, "{q:?}");
            assert!(q.rect[1] >= 0.0 && q.rect[1] + q.rect[3] <= 2160.0, "{q:?}");
        }
    }

    #[test]
    fn nan_telemetry_still_builds() {
        let mut batch = QuadBatch::new();
        build(
            &mut batch,
            &TestClusterInput {
                speed: f32::NAN,
                rpm: f32::INFINITY,
                max_rpm: f32::NAN,
                throttle: f32::NAN,
                scale: f32::NAN,
                ..input()
            },
        );
        assert!(!batch.is_empty());
        assert!(batch
            .as_slice()
            .iter()
            .all(|q| q.rect.iter().chain(&q.color).chain(&q.params).all(|v| v.is_finite())));
    }
}
