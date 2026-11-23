//! Studio light rigs, in the kit's screen convention.
//!
//! Azimuths are degrees **clockwise from 3 o'clock on screen** (y down), the same convention as
//! `common::VirtualLight`: −135° is upper-left, the kit's resting key. Elevation is degrees above
//! the dial plane. Distances and sizes are local px; Blender gets BU.
//!
//! Power is calibrated, not guessed: an area light of power 1.0 straight above a white diffuse
//! surface (albedo 0.8) renders it at about 0.8 linear, whatever the distance (Blender energy =
//! `power × 10 × d²` with `d` in BU, measured in Cycles with the Standard view). So power 1 is
//! "full exposure" and a rig's powers add up the way a photographer's stops do.
//!
//! Why the defaults look the way they do: in a top-down ortho render a flat polished face
//! mirrors only what is straight above it. Without a big overhead source, metal dials render
//! black and the result looks like vector UI. Every preset therefore carries a broad overhead
//! softbox (the "sky" that flat faces reflect, graded so faces show a sheen gradient), a hard
//! key at the kit's light azimuth for chamfers, ridges and needles, and low rim strips that
//! draw thin highlights round every machined edge.
use super::frame::PX;
use serde_json::{json, Value};

/// One light.
#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    /// Name (shows in logs).
    pub name: &'static str,
    /// Screen azimuth (degrees, clockwise from 3 o'clock).
    pub az: f32,
    /// Elevation above the dial plane (degrees).
    pub el: f32,
    /// Distance from the target (local px).
    pub dist: f32,
    /// Softbox width and height (local px). Big = soft.
    pub size: [f32; 2],
    /// Calibrated power (1.0 = full exposure on white).
    pub power: f32,
    /// Linear RGB tint.
    pub color: [f32; 3],
    /// Rotate the softbox's long side to face the target tangentially (strips).
    pub strip: bool,
}

impl Light {
    /// A softbox.
    #[must_use]
    pub fn soft(name: &'static str, az: f32, el: f32, dist: f32, size: f32, power: f32) -> Self {
        Self {
            name,
            az,
            el,
            dist,
            size: [size, size],
            power,
            color: [1.0, 1.0, 1.0],
            strip: false,
        }
    }

    /// A strip box `len × wid` px.
    #[must_use]
    pub fn strip(name: &'static str, az: f32, el: f32, dist: f32, len: f32, wid: f32, power: f32) -> Self {
        Self {
            name,
            az,
            el,
            dist,
            size: [len, wid],
            power,
            color: [1.0, 1.0, 1.0],
            strip: true,
        }
    }

    /// With a tint.
    #[must_use]
    pub fn tint(mut self, rgb: [f32; 3]) -> Self {
        self.color = rgb;
        self
    }

    /// Position in Blender (BU) relative to the target, and the energy in W.
    #[must_use]
    pub fn place(&self) -> ([f32; 3], f32) {
        let (a, e) = (self.az.to_radians(), self.el.to_radians());
        let d = self.dist * PX;
        let pos = [a.cos() * e.cos() * d, -a.sin() * e.cos() * d, e.sin() * d];
        (pos, self.power * 10.0 * d * d)
    }

    fn to_json(&self, target: [f32; 2], z: f32) -> Value {
        let (p, energy) = self.place();
        json!({
            "name": self.name,
            "loc": [target[0] * PX + p[0], -target[1] * PX + p[1], z * PX + p[2]],
            "target": [target[0] * PX, -target[1] * PX, z * PX],
            "size": [self.size[0] * PX, self.size[1] * PX],
            "energy": energy, "color": self.color, "strip": self.strip,
        })
    }
}

/// A light rig: lights plus a dim, camera-invisible studio HDRI for reflections in curved parts.
#[derive(Debug, Clone, PartialEq)]
pub struct Rig {
    /// The lights.
    pub lights: Vec<Light>,
    /// HDRI strength (Blender's bundled `studio.exr`; 0 = black world).
    pub hdri: f32,
    /// HDRI rotation about z (degrees).
    pub hdri_rot: f32,
    /// Where the lights aim (local px) and at what height.
    pub target: [f32; 3],
    /// Global exposure in stops (applied as Blender view exposure).
    pub exposure: f32,
}

impl Rig {
    /// The hero product rig, aimed at a part of about `span` px across, with the key at screen
    /// azimuth `key_az` (the kit's rest light is −135°; render −160° and −110° too for the
    /// lateral-G swing). The key is hot (1.6) so chamfers and needle facets glint; the sky is
    /// dim (0.14) because a big source is seen *directly* in every flat mirror-like face: at
    /// 0.14 a piano-black face reflects ≈ 2 % grey and a dark sunray dial a deep anthracite, so
    /// the key's anisotropic wedge reads. Raise the sky and every metal face goes white.
    #[must_use]
    pub fn hero(span: f32, key_az: f32) -> Self {
        let s = span.max(100.0);
        Self {
            lights: vec![
                // Hard key at the kit's light azimuth: chamfers, ridges, needle facets.
                Light::soft("key", key_az, 38.0, s * 2.2, s * 0.35, 1.6).tint([1.0, 0.97, 0.92]),
                // Broad overhead sky: what flat polished and brushed faces mirror.
                Light::soft("sky", key_az + 20.0, 76.0, s * 2.0, s * 1.6, 0.14),
                // Second, smaller overhead source offset the other way: a sheen gradient
                // rather than a flat grey.
                Light::soft("sky2", key_az + 180.0, 64.0, s * 2.2, s * 0.6, 0.06).tint([0.9, 0.95, 1.0]),
                // Low cool rim opposite the key: thin edge highlights round every bezel.
                Light::strip("rim", key_az + 180.0, 14.0, s * 1.8, s * 1.4, s * 0.08, 1.2).tint([0.85, 0.93, 1.0]),
                // Warm low kicker from the side: separates stacked parts.
                Light::strip("kick", key_az + 95.0, 18.0, s * 1.8, s * 1.0, s * 0.08, 0.6).tint([1.0, 0.9, 0.8]),
            ],
            hdri: 0.08,
            hdri_rot: key_az,
            target: [0.0, 0.0, 0.0],
            exposure: 0.0,
        }
    }

    /// One raking light (low elevation) for texture and relief proofs.
    #[must_use]
    pub fn raking(span: f32, az: f32) -> Self {
        let s = span.max(100.0);
        Self {
            lights: vec![Light::soft("rake", az, 12.0, s * 2.0, s * 0.2, 3.0)],
            hdri: 0.05,
            hdri_rot: 0.0,
            target: [0.0, 0.0, 0.0],
            exposure: 0.0,
        }
    }

    /// Aims the rig at `(x, y)` local px, height `z` px.
    #[must_use]
    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.target = [x, y, z];
        self
    }

    /// Scales every light's power (and the HDRI) by `k`.
    #[must_use]
    pub fn power(mut self, k: f32) -> Self {
        for l in &mut self.lights {
            l.power *= k;
        }
        self.hdri *= k;
        self
    }

    /// JSON for Blender.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let t = [self.target[0], self.target[1]];
        json!({
            "lights": self.lights.iter().map(|l| l.to_json(t, self.target[2])).collect::<Vec<_>>(),
            "hdri": self.hdri, "hdri_rot": self.hdri_rot.to_radians(), "exposure": self.exposure,
        })
    }
}

/// The azimuths to render for the kit's lateral-G light swing (rest, then ±25°).
pub const SWING: [f32; 3] = [-135.0, -160.0, -110.0];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_left_key_sits_upper_left_in_blender() {
        let l = Light::soft("k", -135.0, 40.0, 1000.0, 100.0, 1.0);
        let (p, energy) = l.place();
        // Screen upper-left = Blender −x, +y (y flipped), above the plane.
        assert!(p[0] < 0.0 && p[1] > 0.0 && p[2] > 0.0, "{p:?}");
        // 10 BU away at power 1 → 10 × 10² W.
        assert!((energy - 1000.0).abs() < 1e-2);
    }

    #[test]
    fn hero_rig_has_an_overhead_sky_and_a_rim() {
        let r = Rig::hero(500.0, -135.0);
        assert!(r.lights.iter().any(|l| l.el > 70.0));
        assert!(r.lights.iter().any(|l| l.el < 20.0 && l.strip));
        let j = r.to_json();
        assert_eq!(j["lights"].as_array().map(Vec::len), Some(5));
    }
}
