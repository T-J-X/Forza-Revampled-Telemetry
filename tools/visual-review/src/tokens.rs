//! The values the checks come from: `data/design_tokens.json` (layout, blur gate, strobe strip).
use crate::error::{usage, Error, Result};
use crate::geom::Rect;
use serde_json::Value;
use std::path::Path;

/// A view's anchor and envelope, as in the tokens' `layout` (4K px; the envelope is local to the
/// anchor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    /// The anchor in frame pixels.
    pub anchor: (i64, i64),
    /// The envelope, local to the anchor.
    pub envelope: Rect,
}

impl View {
    /// The envelope in frame pixels, grown by `margin`.
    pub fn region(self, margin: i64) -> Rect {
        self.envelope.offset(self.anchor.0, self.anchor.1).grow(margin)
    }
}

/// The tokens the review tool reads.
#[derive(Debug, Clone)]
pub struct Tokens {
    /// The full frame size (`layout.canvas`).
    pub canvas: (u32, u32),
    /// Cockpit view `[3]`.
    pub cockpit: View,
    /// Hood view `[4]`.
    pub hood: View,
    /// The raised HUD (`layout.hudRaised`): its anchor, with the cockpit envelope.
    pub hud: View,
    /// Blur gate: the size the frame is reduced to.
    pub glance_size: (u32, u32),
    /// Blur gate: the Gaussian sigma at that size.
    pub glance_sigma: f32,
    /// Blur gate: the gear's minimum contrast.
    pub gear_min: f32,
    /// Blur gate: the speed's minimum contrast.
    pub speed_min: f32,
    /// The tallest the strobing shift strip may be at scale 1.0.
    pub strip_max_px: f64,
}

impl Tokens {
    /// Reads `data/design_tokens.json` under `root`.
    ///
    /// # Errors
    /// [`Error::Tokens`] when the file is missing, is not JSON or lacks a value.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("data").join("design_tokens.json");
        let fail = |msg: String| Error::Tokens {
            path: path.display().to_string(),
            msg,
        };
        let text = std::fs::read_to_string(&path).map_err(|e| fail(e.to_string()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| fail(e.to_string()))?;
        let n = |pointer: &str| -> Result<f64> {
            v.pointer(pointer)
                .and_then(Value::as_f64)
                .ok_or_else(|| fail(format!("no number at {pointer}")))
        };
        let i = |pointer: &str| -> Result<i64> { Ok(n(pointer)?.round() as i64) };
        let u = |pointer: &str| -> Result<u32> {
            u32::try_from(i(pointer)?).map_err(|_| fail(format!("{pointer} is not a positive size")))
        };
        let view = |key: &str| -> Result<View> {
            Ok(View {
                anchor: (
                    i(&format!("/layout/{key}/anchor/x"))?,
                    i(&format!("/layout/{key}/anchor/y"))?,
                ),
                envelope: Rect::new(
                    i(&format!("/layout/{key}/envelope/x"))?,
                    i(&format!("/layout/{key}/envelope/y"))?,
                    i(&format!("/layout/{key}/envelope/w"))?,
                    i(&format!("/layout/{key}/envelope/h"))?,
                ),
            })
        };
        let cockpit = view("cockpit")?;
        let hood = view("hood")?;
        let hud = View {
            anchor: (i("/layout/hudRaised/anchor/x")?, i("/layout/hudRaised/anchor/y")?),
            envelope: cockpit.envelope,
        };
        Ok(Self {
            canvas: (u("/layout/canvas/width")?, u("/layout/canvas/height")?),
            cockpit,
            hood,
            hud,
            glance_size: (
                u("/accessibility/blurGate/downscaleTo/0")?,
                u("/accessibility/blurGate/downscaleTo/1")?,
            ),
            glance_sigma: n("/accessibility/blurGate/sigma")? as f32,
            gear_min: n("/accessibility/blurGate/gearMinContrast")? as f32,
            speed_min: n("/accessibility/blurGate/speedMinContrast")? as f32,
            strip_max_px: n("/layout/zones/z1_gearAndShift/strobeStripMaxPx")?,
        })
    }

    /// The view called `name`: `cockpit`, `hood` or `hud`.
    ///
    /// # Errors
    /// A usage error for any other name.
    pub fn view(&self, name: &str) -> Result<View> {
        match name {
            "cockpit" => Ok(self.cockpit),
            "hood" => Ok(self.hood),
            "hud" => Ok(self.hud),
            other => Err(usage(format!("unknown view {other:?}: use cockpit, hood or hud"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_repo_tokens() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let t = Tokens::load(&root).expect("tokens load");
        assert_eq!(t.canvas, (3840, 2160));
        assert_eq!(t.cockpit.anchor, (1920, 1835));
        assert_eq!(t.cockpit.region(0), Rect::new(1200, 1515, 1440, 560));
        assert_eq!(t.hood.region(0), Rect::new(1040, 1540, 1760, 400));
        assert_eq!(t.hud.anchor, (1920, 1595));
        assert_eq!(t.glance_size, (960, 540));
        assert!((t.gear_min - 7.0).abs() < 1e-6 && (t.speed_min - 4.5).abs() < 1e-6);
        assert!((t.strip_max_px - 42.0).abs() < 1e-9);
        assert!(t.view("hood").is_ok() && t.view("bonnet").is_err());
    }

    #[test]
    fn a_missing_file_is_an_error() {
        assert!(Tokens::load(Path::new("no/such/root")).is_err());
    }
}
