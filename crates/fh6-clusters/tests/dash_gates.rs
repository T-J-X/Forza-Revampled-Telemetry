//! The dash gates every cluster in the registry plugs into, run on wgpu's no-op backend so they
//! need neither a GPU nor the GPU lock:
//!
//! - G2 and G3: each input that `docs/TELEMETRY_USAGE_MATRIX.md` lists for a dash must change
//!   its frame, and no other input may. Every registry entry has exactly one row.
//! - G5: hostile telemetry (whole packets of NaN, infinities, huge values, noise, with racing on
//!   and off; each field of a normal packet in turn at its extremes; inconsistent readings) and
//!   tiny or odd targets never put a NaN or an infinity into the frame, drop nothing, make the
//!   kit repair nothing the dash computed (`fh6_kit::math::repairs`), and never panic.
//! - Warm frames allocate nothing: after one pass through the review sequence, `update`,
//!   `animating` and `draw` make no heap allocation (AGENTS.md: no per-frame allocation).
//!   `Kit::render` is not measured here; the host CPU phase measures the whole frame on a GPU.
//! - Every dash builds and draws with no asset folder (the fallback path), hostile packets
//!   included.
//!
//! "Changes its frame" means the scene's fingerprint differs (`Scene::fingerprint`: every visible
//! instance, paint, clip, point, draw and mesh, bit for bit) at one of the probe's checks. A
//! probe runs a fresh build on its scenario, switches the input in for [`HOLD`] frames and back
//! out, and compares four frames (the first and last with the input, the first after it and the
//! last) with a fresh build that never saw the input. So an input that only shows while it
//! changes (a reset, a start banner) counts too. Each gate gives every dash its own work and
//! reports all failures at once; each runs on its own kits, so no gate depends on another's
//! order.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test harness: a setup step that fails should stop the test with its message"
)]

use fh6_clusters::catalog;
use fh6_clusters::registry::REGISTRY;
use fh6_clusters::review::{sequence_frame, DriveContext, Driver, SEQUENCE_DT, SEQUENCE_FRAMES, STATES};
use fh6_core::telemetry::{decode, encode, MAX_ABS, PACKET_LEN};
use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData, PowerUnit, SpeedUnit};
use fh6_kit::math::repairs;
use fh6_kit::testing::{Offscreen, OFFSCREEN_FORMAT};
use fh6_kit::{wgpu, Cluster, ClusterEntry, FontChoice, Kit, Layout, Logo, Palette, Tokens, View};
use serde_json::{Map, Value};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::task::{Context, Poll, Waker};

/// The 4K target every gate draws for, except the tiny-target cases.
const TARGET: (u32, u32) = (3840, 2160);
/// Updates on the scenario before a probe switches its input in.
const LEAD: u32 = 8;
/// Updates with the input switched in (long enough for a smoothed reading to move well away).
const HOLD: u32 = 20;
/// Updates after it switches back.
const TAIL: u32 = 10;
/// The updates after which a probe's frame is fingerprinted: the first and last with the input,
/// the first after it, and the last.
const CHECKS: [u32; 4] = [LEAD, LEAD + HOLD - 1, LEAD + HOLD, LEAD + HOLD + TAIL - 1];
/// Updates before a robustness check draws a cold build.
const COLD_FRAMES: u32 = 8;
/// Failures reported per dash; the rest are counted.
const MAX_REPORTED: usize = 12;

type Change = fn(&mut FH6TelemetryData);

/// One column of the matrix: a group of packet fields and the probe values that move them.
struct Input {
    key: &'static str,
    /// The fields, as the matrix writes them (`_fr` repeats the previous field's stem).
    fields: &'static str,
    /// Each probe value is its own probe; discrete inputs get several.
    changes: &'static [Change],
}

/// Every matrix column. Together they change every field of [`FH6TelemetryData`], each probe
/// exactly its own fields (`every_telemetry_field_has_exactly_one_input`).
const INPUTS: &[Input] = &[
    Input {
        key: "race_on",
        fields: "is_race_on",
        changes: &[|t| t.is_race_on = 0],
    },
    Input {
        key: "timestamp",
        fields: "timestamp_ms",
        // An hour's jump in, and back.
        changes: &[|t| t.timestamp_ms = t.timestamp_ms.wrapping_add(3_600_000)],
    },
    Input {
        key: "rpm",
        fields: "current_engine_rpm",
        changes: &[|t| t.current_engine_rpm = 7_600.0],
    },
    Input {
        key: "max_rpm",
        fields: "engine_max_rpm",
        changes: &[|t| t.engine_max_rpm = 6_500.0],
    },
    Input {
        key: "idle_rpm",
        fields: "engine_idle_rpm",
        changes: &[|t| t.engine_idle_rpm = 1_500.0],
    },
    Input {
        key: "gear",
        fields: "gear",
        // A forward gear, reverse and neutral.
        changes: &[|t| t.gear = 2, |t| t.gear = 0, |t| t.gear = 11],
    },
    Input {
        key: "speed",
        fields: "speed_ms",
        changes: &[|t| t.speed_ms = 30.0],
    },
    Input {
        key: "throttle",
        fields: "accel",
        changes: &[|t| t.accel = 255, |t| t.accel = 64],
    },
    Input {
        key: "brake",
        fields: "brake",
        changes: &[|t| t.brake = 200],
    },
    Input {
        key: "clutch",
        fields: "clutch",
        changes: &[|t| t.clutch = 255],
    },
    Input {
        key: "handbrake",
        fields: "hand_brake",
        changes: &[|t| t.hand_brake = 255],
    },
    Input {
        key: "steer",
        fields: "steer",
        changes: &[|t| t.steer = 90, |t| t.steer = -90],
    },
    Input {
        key: "g_lat",
        fields: "acceleration_x",
        changes: &[|t| t.acceleration_x = -8.8],
    },
    Input {
        key: "g_long",
        fields: "acceleration_z",
        changes: &[|t| t.acceleration_z = -7.8],
    },
    Input {
        key: "g_vert",
        fields: "acceleration_y",
        changes: &[|t| t.acceleration_y = 4.0],
    },
    Input {
        key: "velocity",
        fields: "velocity_x, velocity_y, velocity_z",
        changes: &[|t| {
            let (s, c) = 20f32.to_radians().sin_cos();
            t.velocity_x = t.speed_ms * s;
            t.velocity_y = 1.0;
            t.velocity_z = t.speed_ms * c;
        }],
    },
    Input {
        key: "yaw_rate",
        fields: "angular_velocity_y",
        changes: &[|t| t.angular_velocity_y = 0.8],
    },
    Input {
        key: "orientation",
        fields: "yaw, pitch, roll",
        changes: &[|t| {
            t.yaw = 1.2;
            t.pitch = 0.1;
            t.roll = -0.1;
        }],
    },
    Input {
        key: "tyre_slip",
        fields: "tire_slip_angle_fl, _fr, _rl, _rr",
        changes: &[|t| {
            t.tire_slip_angle_fl = 0.15;
            t.tire_slip_angle_fr = 0.2;
            t.tire_slip_angle_rl = 0.25;
            t.tire_slip_angle_rr = 0.3;
        }],
    },
    Input {
        key: "suspension",
        fields: "normalized_suspension_travel_fl, _fr, _rl, _rr",
        changes: &[|t| {
            t.normalized_suspension_travel_fl = 0.1;
            t.normalized_suspension_travel_fr = 0.9;
            t.normalized_suspension_travel_rl = 0.3;
            t.normalized_suspension_travel_rr = 0.7;
        }],
    },
    Input {
        key: "suspension_m",
        fields: "suspension_travel_meters_fl, _fr, _rl, _rr",
        changes: &[|t| {
            t.suspension_travel_meters_fl = 0.01;
            t.suspension_travel_meters_fr = 0.09;
            t.suspension_travel_meters_rl = 0.03;
            t.suspension_travel_meters_rr = 0.07;
        }],
    },
    Input {
        key: "car_ordinal",
        fields: "car_ordinal",
        changes: &[|t| t.car_ordinal = 2_200],
    },
    Input {
        key: "drivetrain",
        fields: "drivetrain_type",
        // The scenarios are front drive (0): rear and all-wheel drive.
        changes: &[|t| t.drivetrain_type = 1, |t| t.drivetrain_type = 2],
    },
    Input {
        key: "position",
        fields: "position_x, position_y, position_z",
        changes: &[|t| {
            t.position_x += 120.0;
            t.position_y += 5.0;
            t.position_z -= 340.0;
        }],
    },
    Input {
        key: "power",
        fields: "power_watts",
        changes: &[|t| t.power_watts = 250_000.0],
    },
    Input {
        key: "boost",
        fields: "boost_psi",
        changes: &[|t| t.boost_psi = 21.0],
    },
];

/// `t` with every field a review state leaves at zero set to a plausible reading, so a probe
/// moves each field away from a real value.
fn with_extras(mut t: FH6TelemetryData) -> FH6TelemetryData {
    t.timestamp_ms = 1_000;
    t.normalized_suspension_travel_fl = 0.5;
    t.normalized_suspension_travel_fr = 0.5;
    t.normalized_suspension_travel_rl = 0.5;
    t.normalized_suspension_travel_rr = 0.5;
    t.suspension_travel_meters_fl = 0.05;
    t.suspension_travel_meters_fr = 0.05;
    t.suspension_travel_meters_rl = 0.05;
    t.suspension_travel_meters_rr = 0.05;
    t.car_ordinal = 1_000;
    t
}

/// The mid-cruise review state with extras: the first scenario, and the "normal reading" the
/// robustness gate starts from.
fn base_packet() -> FH6TelemetryData {
    with_extras(STATES[1].telemetry())
}

/// A packet sequence a probe drives: the packet for update `i`, before the clock runs.
struct Scenario {
    name: &'static str,
    packet: fn(u32) -> FH6TelemetryData,
}

/// Update `i` of `scenario` as the game would send it: the timestamp counts up and the car moves
/// along its velocity.
fn packet_at(scenario: &Scenario, i: u32) -> FH6TelemetryData {
    let mut t = (scenario.packet)(i);
    let secs = i as f32 * SEQUENCE_DT;
    t.timestamp_ms = t.timestamp_ms.wrapping_add((secs * 1_000.0).round() as u32);
    t.position_x += t.velocity_x * secs;
    t.position_y += t.velocity_y * secs;
    t.position_z += t.velocity_z * secs;
    t
}

/// The scenarios every dash is probed in: the four review states, cruise first, and a lift off
/// from the limiter to cruise (so a peak or a falling reading differs from the live one).
const COMMON: &[Scenario] = &[
    Scenario {
        name: "cruise",
        packet: |_| base_packet(),
    },
    Scenario {
        name: "idle",
        packet: |_| with_extras(STATES[0].telemetry()),
    },
    Scenario {
        name: "redline",
        packet: |_| with_extras(STATES[2].telemetry()),
    },
    Scenario {
        name: "drift braking",
        packet: |_| with_extras(STATES[3].telemetry()),
    },
    Scenario {
        name: "lift off",
        packet: |i| {
            if i < LEAD / 2 {
                with_extras(STATES[2].telemetry())
            } else {
                base_packet()
            }
        },
    },
];

/// A standing start in first gear at 4,500 rpm: the launch scenarios start from it.
fn standing() -> FH6TelemetryData {
    let mut t = with_extras(STATES[0].telemetry());
    t.gear = 1;
    t.current_engine_rpm = 4_500.0;
    t
}

/// Scenarios for inputs a dash reads only in combination with others (a lamp that needs two
/// conditions). A dash adds its own here when its matrix row lists such an input; its probes
/// then run in [`COMMON`] and in these. The matrix's `## Scenarios` table lists them.
const DASH_SCENARIOS: &[(ClusterTheme, Scenario)] = &[
    // LNCH: standing, throttle above 50% with the brake, handbrake or clutch held.
    (
        ClusterTheme::BoschDdu,
        Scenario {
            name: "launch, nothing held",
            packet: |_| FH6TelemetryData {
                accel: 204,
                ..standing()
            },
        },
    ),
    (
        ClusterTheme::BoschDdu,
        Scenario {
            name: "launch, brake held, throttle off",
            packet: |_| FH6TelemetryData {
                brake: 255,
                ..standing()
            },
        },
    ),
    // ABS: braking harder than 30% while decelerating at 0.85 g or more.
    (
        ClusterTheme::BoschDdu,
        Scenario {
            name: "threshold braking, brake off",
            packet: |_| {
                let mut t = base_packet();
                t.acceleration_z = -0.95 * 9.806_65;
                t.accel = 0;
                t
            },
        },
    ),
    // TC: throttle with the body slip growing faster than 8 deg/s, unless front-wheel drive.
    (
        ClusterTheme::BoschDdu,
        Scenario {
            name: "power oversteer, front drive",
            packet: |i| {
                let mut t = base_packet();
                t.speed_ms = 25.0;
                t.accel = 230;
                t.drivetrain_type = 0;
                let (s, c) = (2.0 + 0.6 * i as f32).to_radians().sin_cos();
                t.velocity_x = t.speed_ms * s;
                t.velocity_z = t.speed_ms * c;
                t
            },
        },
    ),
];

// --- Field coverage --------------------------------------------------------------------------

/// The field names in a matrix `fields` cell: `tire_slip_angle_fl, _fr` names
/// `tire_slip_angle_fl` and `tire_slip_angle_fr`.
fn expand(fields: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in fields.split(',').map(str::trim) {
        let name = match (part.strip_prefix('_'), out.last()) {
            (Some(suffix), Some(prev)) => match prev.rsplit_once('_') {
                Some((stem, _)) => format!("{stem}_{suffix}"),
                None => part.to_owned(),
            },
            _ => part.to_owned(),
        };
        out.push(name);
    }
    out
}

/// `t` as `serde_json` writes it: field name -> value.
fn json_fields(t: &FH6TelemetryData) -> Map<String, Value> {
    match serde_json::to_value(t).expect("serialise telemetry") {
        Value::Object(fields) => fields,
        other => panic!("telemetry serialises as an object, not {other}"),
    }
}

/// The fields whose values differ between `a` and `b`.
fn changed_fields(a: &FH6TelemetryData, b: &FH6TelemetryData) -> BTreeSet<String> {
    let (a, b) = (json_fields(a), json_fields(b));
    a.iter()
        .filter(|(k, v)| b.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .collect()
}

/// Each field of [`FH6TelemetryData`] belongs to exactly one input, and each probe value changes
/// exactly its input's fields, so a new field fails here until it has a column.
#[test]
fn every_telemetry_field_has_exactly_one_input() {
    let base = base_packet();
    let all: BTreeSet<String> = json_fields(&base).keys().cloned().collect();
    let mut covered = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for input in INPUTS {
        assert!(keys.insert(input.key), "two inputs are called `{}`", input.key);
        let fields: BTreeSet<String> = expand(input.fields).into_iter().collect();
        for field in &fields {
            assert!(
                all.contains(field),
                "`{}` names {field}, which is not a telemetry field",
                input.key
            );
            assert!(covered.insert(field.clone()), "{field} is in two inputs");
        }
        assert!(!input.changes.is_empty(), "`{}` has no probe value", input.key);
        for (n, change) in input.changes.iter().enumerate() {
            let mut t = base.clone();
            change(&mut t);
            assert_eq!(
                changed_fields(&base, &t),
                fields,
                "probe value {n} of `{}` must change exactly its fields",
                input.key
            );
        }
    }
    let missing: Vec<_> = all.difference(&covered).collect();
    assert!(missing.is_empty(), "telemetry fields with no input: {missing:?}");
}

#[test]
fn field_lists_expand_suffixes() {
    assert_eq!(expand("gear"), ["gear"]);
    assert_eq!(expand("yaw, pitch, roll"), ["yaw", "pitch", "roll"]);
    assert_eq!(
        expand("tire_slip_angle_fl, _fr, _rl, _rr"),
        [
            "tire_slip_angle_fl",
            "tire_slip_angle_fr",
            "tire_slip_angle_rl",
            "tire_slip_angle_rr"
        ]
    );
}

// --- The no-op kit ---------------------------------------------------------------------------

fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = pin!(f);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::yield_now();
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A kit on wgpu's no-op device: clusters build, update and record frames as on a GPU, and
/// `render` records (and discards) the passes, which is what generates missing glyphs.
struct NoopKit {
    device: wgpu::Device,
    queue: wgpu::Queue,
    kit: Kit,
    tokens: Tokens,
    target: Offscreen,
}

/// A probe's fingerprints, one per entry of [`CHECKS`], and the glyph atlas generation they were
/// recorded at (frames compare only at the same generation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Timeline {
    prints: [u64; CHECKS.len()],
    generation: u32,
}

impl NoopKit {
    fn new(fonts: bool) -> Self {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::NOOP;
        desc.backend_options.noop.enable = true;
        let instance = wgpu::Instance::new(desc);
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("wgpu no-op adapter (is the `noop` feature on?)");
        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("no-op device");
        let tokens = Tokens::load(&repo_root().join("data/design_tokens.json")).expect("tokens");
        let mut kit = Kit::new(&device, &queue, tokens.clone()).expect("kit");
        if fonts {
            kit.fonts_mut()
                .load_dir(&repo_root().join("assets/fonts"))
                .expect("fonts");
            kit.bind_font_roles();
        }
        let target = Offscreen::new(&device, TARGET.0, TARGET.1);
        Self {
            device,
            queue,
            kit,
            tokens,
            target,
        }
    }

    fn make(&mut self, entry: &ClusterEntry, assets: bool) -> Box<dyn Cluster> {
        let dir = repo_root().join("assets");
        let mut setup = self
            .kit
            .setup(&self.device, &self.queue, assets.then_some(dir.as_path()));
        (entry.make)(&mut setup)
    }

    /// Records `cluster`'s frame, rendering (on the no-op device) until every glyph it uses is
    /// in the atlas, so the scene holds the complete frame.
    fn draw_complete(&mut self, cluster: &dyn Cluster, layout: &Layout) {
        for _ in 0..64 {
            {
                let mut p = self.kit.begin_frame(layout);
                cluster.draw(&mut p, layout);
            }
            if self.kit.scene().missing_glyph_count() == 0 {
                return;
            }
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            self.kit.render(
                &self.device,
                &self.queue,
                &mut encoder,
                self.target.view(),
                OFFSCREEN_FORMAT,
                TARGET,
                1.0,
            );
            self.queue.submit([encoder.finish()]);
            self.kit.frame_submitted();
        }
        panic!("glyphs still missing after 64 frames");
    }

    /// A fresh `entry` driven through `scenario` in `view`, with `change` applied to the packets
    /// of the hold window, fingerprinted after each update in [`CHECKS`]. Recorded again when the
    /// atlas grew on the way, so all four prints share one generation.
    fn timeline(
        &mut self,
        entry: &ClusterEntry,
        drive: &Drive,
        view: View,
        scenario: &Scenario,
        change: Option<Change>,
    ) -> Timeline {
        let frames = LEAD + HOLD + TAIL;
        for _ in 0..4 {
            let start = self.kit.fonts().atlas_generation();
            let mut cluster = self.make(entry, true);
            let layout = Layout::from_config(&drive.config, &self.tokens, view, TARGET);
            let ctx = drive.ctx(view);
            let mut driver = Driver::new();
            let mut prints = [0; CHECKS.len()];
            let mut next = 0;
            for i in 0..frames {
                let mut packet = packet_at(scenario, i);
                if let Some(change) = change.filter(|_| (LEAD..LEAD + HOLD).contains(&i)) {
                    change(&mut packet);
                }
                driver.step(cluster.as_mut(), &packet, &ctx, SEQUENCE_DT);
                if CHECKS.get(next) == Some(&i) {
                    self.draw_complete(cluster.as_ref(), &layout);
                    prints[next] = self.kit.scene().fingerprint();
                    next += 1;
                }
            }
            let generation = self.kit.fonts().atlas_generation();
            if generation == start {
                return Timeline { prints, generation };
            }
        }
        panic!("the glyph atlas grew on four runs of the same probe");
    }
}

/// What a driver passes besides telemetry, owned so it does not borrow the kit.
struct Drive {
    config: AppConfig,
    palette: Palette,
    tokens: Tokens,
    fonts: FontChoice,
    logo: Logo,
    reduced_flash: bool,
}

impl Drive {
    fn new(nk: &NoopKit, entry: &ClusterEntry) -> Self {
        Self {
            config: AppConfig::default(),
            palette: Palette::family(&nk.tokens, entry.family.key()),
            tokens: nk.tokens.clone(),
            fonts: nk.kit.font_choice(None),
            logo: Logo::default(),
            reduced_flash: false,
        }
    }

    /// Speed in km/h, power in kW and reduced flash on, or the defaults (mph, hp, full flash).
    fn set_kmh_and_reduced_flash(&mut self, on: bool) {
        self.config.speed_unit = if on { SpeedUnit::Kmh } else { SpeedUnit::Mph };
        self.config.power_unit = if on { PowerUnit::Kw } else { PowerUnit::Hp };
        self.reduced_flash = on;
    }

    fn ctx(&self, view: View) -> DriveContext<'_> {
        DriveContext {
            config: &self.config,
            palette: &self.palette,
            tokens: &self.tokens,
            fonts: self.fonts,
            logo: &self.logo,
            view,
            reduced_flash: self.reduced_flash,
        }
    }
}

/// The views the catalog designs `entry` for (a raised HUD draws in the cockpit layout).
fn designed_views(entry: &ClusterEntry) -> Vec<View> {
    let Some(row) = catalog::entry(entry.theme) else {
        return vec![View::Cockpit];
    };
    let mut views = Vec::new();
    if row.views.cockpit || row.views.raised_hud {
        views.push(View::Cockpit);
    }
    if row.views.hood {
        views.push(View::Hood);
    }
    views
}

fn panic_text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a non-text panic".to_owned())
}

/// Runs `gate` for every registry dash on `threads` threads, each with its own no-op kit (with
/// or without the repo's fonts), and returns every failure in registry order. A panic fails
/// that dash, not the whole gate.
fn for_each_dash(
    fonts: bool,
    threads: usize,
    gate: impl Fn(&mut NoopKit, &ClusterEntry) -> Vec<String> + Sync,
) -> Vec<String> {
    let mut results: Vec<(usize, Vec<String>)> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|k| {
                let gate = &gate;
                s.spawn(move || {
                    let mut nk = NoopKit::new(fonts);
                    REGISTRY
                        .iter()
                        .enumerate()
                        .skip(k)
                        .step_by(threads)
                        .map(|(i, entry)| {
                            let failures =
                                catch_unwind(AssertUnwindSafe(|| gate(&mut nk, entry))).unwrap_or_else(|p| {
                                    vec![format!("{}: panicked: {}", entry.theme.name(), panic_text(&*p))]
                                });
                            (i, failures)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("a gate worker"))
            .collect()
    });
    results.sort_by_key(|(i, _)| *i);
    // A shared helper that breaks fails every dash at once: keep the report readable.
    for (i, failures) in &mut results {
        if failures.len() > MAX_REPORTED {
            let more = failures.len() - MAX_REPORTED;
            failures.truncate(MAX_REPORTED);
            failures.push(format!("{}: and {more} more", REGISTRY[*i].theme.name()));
        }
    }
    results.into_iter().flat_map(|(_, f)| f).collect()
}

// --- G2 and G3: the matrix -------------------------------------------------------------------

fn matrix_path() -> PathBuf {
    repo_root().join("docs/TELEMETRY_USAGE_MATRIX.md")
}

/// The rows of the Markdown tables in the matrix section headed `## {heading}`, as trimmed
/// cells, without each table's header and separator rows and outside fenced code. The file may
/// have CRLF line endings (a Windows checkout).
fn table(heading: &str) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(matrix_path())
        .expect("docs/TELEMETRY_USAGE_MATRIX.md")
        .replace("\r\n", "\n");
    let (mut found, mut inside, mut fenced, mut header) = (false, false, false, true);
    let mut rows = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(title) = line.strip_prefix("## ") {
            inside = title.trim() == heading;
            found |= inside;
            header = true;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(inner) = line.strip_prefix('|').and_then(|l| l.strip_suffix('|')) else {
            header = true; // anything between tables ends the table
            continue;
        };
        let cells: Vec<String> = inner.split('|').map(|c| c.trim().to_owned()).collect();
        if header {
            header = false;
            continue;
        }
        let separator = cells
            .iter()
            .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')));
        if !separator {
            rows.push(cells);
        }
    }
    assert!(found, "docs/TELEMETRY_USAGE_MATRIX.md has no `## {heading}` section");
    rows
}

fn unquote(cell: &str) -> String {
    cell.trim().trim_matches('`').to_owned()
}

/// A declared row: the dash's display name and the input keys it reads.
struct Row {
    name: String,
    reads: BTreeSet<String>,
}

/// The `## Dashes` table: theme name -> row.
fn declared_matrix() -> BTreeMap<String, Row> {
    let mut rows = BTreeMap::new();
    for cells in table("Dashes") {
        let [theme, name, reads] = &cells[..] else {
            panic!("a Dashes row needs 3 cells (theme, dash, reads): {cells:?}");
        };
        let reads: BTreeSet<String> = if reads == "none" {
            BTreeSet::new()
        } else {
            reads.split(',').map(unquote).collect()
        };
        let theme = unquote(theme);
        let row = Row {
            name: name.clone(),
            reads,
        };
        assert!(
            rows.insert(theme.clone(), row).is_none(),
            "{theme} has two rows in the matrix"
        );
    }
    rows
}

#[test]
fn g3_the_matrix_inputs_table_matches_the_test() {
    let doc: Vec<(String, String)> = table("Inputs")
        .into_iter()
        .map(|cells| {
            let [key, fields] = &cells[..] else {
                panic!("an Inputs row needs 2 cells (key, fields): {cells:?}");
            };
            (unquote(key), fields.clone())
        })
        .collect();
    let code: Vec<(String, String)> = INPUTS.iter().map(|i| (i.key.to_owned(), i.fields.to_owned())).collect();
    assert_eq!(
        doc, code,
        "docs/TELEMETRY_USAGE_MATRIX.md `## Inputs` and the test's INPUTS differ"
    );
}

#[test]
fn g3_the_matrix_scenarios_table_matches_the_test() {
    let doc: Vec<(String, String)> = table("Scenarios")
        .into_iter()
        .map(|cells| {
            let [dash, scenario, _reachable] = &cells[..] else {
                panic!("a Scenarios row needs 3 cells (dash, scenario, makes reachable): {cells:?}");
            };
            (dash.clone(), scenario.clone())
        })
        .collect();
    let code: Vec<(String, String)> = DASH_SCENARIOS
        .iter()
        .map(|(theme, s)| {
            let entry = REGISTRY.iter().find(|e| e.theme == *theme).expect("a registry dash");
            (entry.name.to_owned(), s.name.to_owned())
        })
        .collect();
    assert_eq!(
        doc, code,
        "docs/TELEMETRY_USAGE_MATRIX.md `## Scenarios` and the test's DASH_SCENARIOS differ"
    );
}

/// The inputs that change `entry`'s frame in any of its designed views and scenarios.
fn observed_inputs(nk: &mut NoopKit, entry: &ClusterEntry) -> Result<BTreeSet<String>, String> {
    let drive = Drive::new(nk, entry);
    let own = DASH_SCENARIOS
        .iter()
        .filter(|(theme, _)| *theme == entry.theme)
        .map(|(_, s)| s);
    let scenarios: Vec<&Scenario> = COMMON.iter().chain(own).collect();
    let mut reads = BTreeSet::new();
    for view in designed_views(entry) {
        for scenario in &scenarios {
            let mut control = nk.timeline(entry, &drive, view, scenario, None);
            // The same packets must give the same frames, or the probes below mean nothing.
            let again = nk.timeline(entry, &drive, view, scenario, None);
            if again.generation == control.generation && again.prints != control.prints {
                return Err(format!(
                    "{}, {view:?}, {}: two fresh builds fed the same packets drew different frames \
                     (wall clock or unseeded randomness?)",
                    entry.theme.name(),
                    scenario.name
                ));
            }
            for input in INPUTS {
                if reads.contains(input.key) {
                    continue;
                }
                for &change in input.changes {
                    let mut probed = nk.timeline(entry, &drive, view, scenario, Some(change));
                    // The atlas only grows, so this settles on one generation.
                    while probed.generation != control.generation {
                        if probed.generation > control.generation {
                            control = nk.timeline(entry, &drive, view, scenario, None);
                        } else {
                            probed = nk.timeline(entry, &drive, view, scenario, Some(change));
                        }
                    }
                    if probed.prints != control.prints {
                        reads.insert(input.key.to_owned());
                        break;
                    }
                }
            }
        }
    }
    Ok(reads)
}

/// `keys` in matrix order and in the matrix's format (`a`, `b`), or "none".
fn join(keys: &BTreeSet<String>) -> String {
    if keys.is_empty() {
        "none".to_owned()
    } else {
        INPUTS
            .iter()
            .filter(|i| keys.contains(i.key))
            .map(|i| format!("`{}`", i.key))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[test]
fn g2_g3_dashes_read_exactly_the_inputs_the_matrix_declares() {
    let declared = declared_matrix();
    let known: BTreeSet<&str> = INPUTS.iter().map(|i| i.key).collect();
    let mut failures = Vec::new();
    for (theme, row) in &declared {
        if !REGISTRY.iter().any(|e| e.theme.name() == theme) {
            failures.push(format!("{theme}: matrix row for a dash not in the registry"));
        }
        for k in row.reads.iter().filter(|k| !known.contains(k.as_str())) {
            failures.push(format!("{theme}: unknown input `{k}`"));
        }
    }
    failures.extend(for_each_dash(true, 3, |nk, entry| {
        let theme = entry.theme.name();
        let Some(row) = declared.get(theme) else {
            return vec![format!("{theme}: no row in docs/TELEMETRY_USAGE_MATRIX.md")];
        };
        let mut failures = Vec::new();
        if row.name != entry.name {
            failures.push(format!(
                "{theme}: the matrix calls it {}, the registry {}",
                row.name, entry.name
            ));
        }
        match observed_inputs(nk, entry) {
            Err(e) => failures.push(e),
            Ok(seen) if seen != row.reads => {
                let mut why = Vec::new();
                let unkept: BTreeSet<_> = row.reads.difference(&seen).cloned().collect();
                if !unkept.is_empty() {
                    why.push(format!("listed but never change the frame: {}", join(&unkept)));
                }
                let unlisted: BTreeSet<_> = seen.difference(&row.reads).cloned().collect();
                if !unlisted.is_empty() {
                    why.push(format!("change the frame but are not listed: {}", join(&unlisted)));
                }
                failures.push(format!(
                    "{theme}: {}. Observed row: | `{theme}` | {} | {} |",
                    why.join("; "),
                    entry.name,
                    join(&seen)
                ));
            }
            Ok(_) => {}
        }
        failures
    }));
    assert!(failures.is_empty(), "G2/G3 failures:\n{}", failures.join("\n"));
}

// --- G5: hostile telemetry and odd targets ---------------------------------------------------

/// Seeded xorshift bytes.
fn noise(seed: u64) -> Vec<u8> {
    let mut x = 0x9E37_79B9_7F4A_7C15 ^ seed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    (0..PACKET_LEN)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x >> 56) as u8
        })
        .collect()
}

/// Whole hostile packets, each as sent and with `IsRaceOn` forced to 1 (most fill patterns read
/// as racing off, and a dash may draw little then): every float word NaN, an infinity, the
/// largest or a subnormal float; every byte 0x00, 0xFF, 0x7F or 0x80; and seeded noise.
/// Production telemetry only reaches a dash through `decode`.
fn whole_packets() -> Vec<(String, FH6TelemetryData)> {
    let mut raw: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, word) in [
        ("NaN", f32::NAN),
        ("+Inf", f32::INFINITY),
        ("-Inf", f32::NEG_INFINITY),
        ("f32::MAX", f32::MAX),
        ("-f32::MAX", f32::MIN),
        ("subnormal", f32::from_bits(1)),
    ] {
        let bytes = word.to_le_bytes().iter().copied().cycle().take(PACKET_LEN).collect();
        raw.push((format!("every word {name}"), bytes));
    }
    for byte in [0x00, 0xFF, 0x7F, 0x80] {
        raw.push((format!("every byte 0x{byte:02X}"), vec![byte; PACKET_LEN]));
    }
    for seed in 0..8 {
        raw.push((format!("noise {seed}"), noise(seed)));
    }
    let mut out: Vec<(String, FH6TelemetryData)> = Vec::new();
    for (name, mut bytes) in raw {
        out.push((name.clone(), decode(&bytes).expect("packet length")));
        bytes[0..4].copy_from_slice(&1i32.to_le_bytes());
        let racing = decode(&bytes).expect("packet length");
        if out.last().map(|(_, t)| t) != Some(&racing) {
            out.push((format!("{name}, racing"), racing));
        }
    }
    out
}

/// Packets that differ from the base packet in one place: each 4-byte word in turn (every float,
/// integer and byte field) set to the largest float, its negative, zero or the smallest
/// subnormal; integer extremes; and inconsistent readings a corrupt or modded car can send.
fn field_packets() -> Vec<(String, FH6TelemetryData)> {
    let base = base_packet();
    let base_bytes = encode(&base);
    let mut out: Vec<(String, FH6TelemetryData)> = Vec::new();
    let mut add = |name: String, t: FH6TelemetryData| {
        if t != base && !out.iter().any(|(_, o)| *o == t) {
            out.push((name, t));
        }
    };
    for off in (0..PACKET_LEN).step_by(4) {
        for (label, v) in [
            ("f32::MAX", f32::MAX),
            ("-f32::MAX", f32::MIN),
            ("0", 0.0),
            ("a subnormal", f32::from_bits(1)),
        ] {
            let mut bytes = base_bytes;
            bytes[off..off + 4].copy_from_slice(&v.to_le_bytes());
            let t = decode(&bytes).expect("packet length");
            let fields: Vec<String> = changed_fields(&base, &t).into_iter().collect();
            add(format!("{} from {label}", fields.join(", ")), t);
        }
    }
    let edits: [(&str, Change); 22] = [
        ("gear 255", |t| t.gear = 255),
        ("gear 12", |t| t.gear = 12),
        ("gear 10", |t| t.gear = 10),
        ("car_ordinal i32::MIN", |t| t.car_ordinal = i32::MIN),
        ("car_ordinal i32::MAX", |t| t.car_ordinal = i32::MAX),
        ("car_ordinal -1", |t| t.car_ordinal = -1),
        ("drivetrain -1", |t| t.drivetrain_type = -1),
        ("drivetrain 3", |t| t.drivetrain_type = 3),
        ("drivetrain i32::MAX", |t| t.drivetrain_type = i32::MAX),
        ("steer -128", |t| t.steer = -128),
        ("steer 127", |t| t.steer = 127),
        ("is_race_on -1", |t| t.is_race_on = -1),
        ("is_race_on 2", |t| t.is_race_on = 2),
        ("timestamp u32::MAX", |t| t.timestamp_ms = u32::MAX),
        ("every pedal 255", |t| {
            t.accel = 255;
            t.brake = 255;
            t.clutch = 255;
            t.hand_brake = 255;
        }),
        ("max_rpm and idle_rpm 0", |t| {
            t.engine_max_rpm = 0.0;
            t.engine_idle_rpm = 0.0;
        }),
        ("idle_rpm above max_rpm", |t| {
            t.engine_idle_rpm = 9_000.0;
            t.engine_max_rpm = 7_000.0;
        }),
        ("idle_rpm equal to max_rpm", |t| {
            t.engine_idle_rpm = 7_000.0;
            t.engine_max_rpm = 7_000.0;
        }),
        ("rpm far above max_rpm", |t| {
            t.current_engine_rpm = 3.0 * t.engine_max_rpm + 1_000.0
        }),
        ("max_rpm at the bound", |t| t.engine_max_rpm = MAX_ABS),
        ("speed negative", |t| t.speed_ms = -30.0),
        ("speed at the bound, velocity 0", |t| {
            t.speed_ms = MAX_ABS;
            t.velocity_x = 0.0;
            t.velocity_y = 0.0;
            t.velocity_z = 0.0;
        }),
    ];
    for (name, edit) in edits {
        let mut t = base_packet();
        edit(&mut t);
        add(name.to_owned(), t);
    }
    out
}

#[test]
fn the_robustness_packets_cover_every_field() {
    let base = base_packet();
    let mut touched = BTreeSet::new();
    for (_, t) in field_packets() {
        touched.extend(changed_fields(&base, &t));
    }
    let all: BTreeSet<String> = json_fields(&base).keys().cloned().collect();
    assert_eq!(touched, all, "a field the per-field sweep never sets");
    let whole = whole_packets();
    assert!(
        whole.iter().any(|(_, t)| t.is_race_on == 0),
        "a whole packet with racing off"
    );
    assert!(
        whole.iter().filter(|(_, t)| t.is_race_on == 1).count() >= 18,
        "racing variants"
    );
}

/// What a finished frame must hold: finite values, nothing dropped and no repairs since `since`.
fn check_frame(nk: &NoopKit, what: &str, since: u64, failures: &mut Vec<String>) {
    let scene = nk.kit.scene();
    let bad = scene.non_finite_count();
    if bad > 0 {
        failures.push(format!("{what}: {bad} NaN or infinite values in the frame"));
    }
    if scene.dropped() > 0 {
        failures.push(format!("{what}: {} items dropped (capacity reached)", scene.dropped()));
    }
    let repaired = repairs().wrapping_sub(since);
    if repaired > 0 {
        failures.push(format!(
            "{what}: the kit repaired {repaired} NaN or infinite values the dash computed"
        ));
    }
}

/// The G5 cases for one dash. `stage` names the case running, for a panic's message.
fn robustness(
    nk: &mut NoopKit,
    entry: &ClusterEntry,
    whole: &[(String, FH6TelemetryData)],
    fields: &[(String, FH6TelemetryData)],
    stage: &RefCell<String>,
    failures: &mut Vec<String>,
) {
    let theme = entry.theme.name();
    let mut drive = Drive::new(nk, entry);
    // Whole packets from a cold start, then after a normal reading at odd frame times, in every
    // view and with km/h, kW and reduced flash.
    for (variant, view, kmh) in [
        ("cockpit", View::Cockpit, false),
        ("cockpit, km/h, kW, reduced flash", View::Cockpit, true),
        ("hood", View::Hood, false),
        ("chase", View::Chase, false),
    ] {
        drive.set_kmh_and_reduced_flash(kmh);
        let layout = Layout::from_config(&drive.config, &nk.tokens, view, TARGET);
        for (name, packet) in whole {
            let what = format!("{theme}, {name}, {variant}");
            stage.replace(what.clone());
            let since = repairs();
            let mut cluster = nk.make(entry, true);
            let ctx = drive.ctx(view);
            let mut driver = Driver::new();
            for _ in 0..COLD_FRAMES {
                driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
            }
            std::hint::black_box(cluster.animating());
            nk.draw_complete(cluster.as_ref(), &layout);
            check_frame(nk, &format!("{what}, cold"), since, failures);
            let since = repairs();
            driver.step(cluster.as_mut(), &base_packet(), &ctx, SEQUENCE_DT);
            // A zero dt is two packets in one frame; 5 s is a hitch.
            for dt in [SEQUENCE_DT, 0.0, 0.1, 0.25, 5.0] {
                driver.step(cluster.as_mut(), packet, &ctx, dt);
            }
            std::hint::black_box(cluster.animating());
            nk.draw_complete(cluster.as_ref(), &layout);
            check_frame(nk, &format!("{what}, after a reading"), since, failures);
        }
    }
    drive.set_kmh_and_reduced_flash(false);
    // One field at a time on a warm dash, in each view it is designed for.
    for view in designed_views(entry) {
        let layout = Layout::from_config(&drive.config, &nk.tokens, view, TARGET);
        let ctx = drive.ctx(view);
        let mut cluster = nk.make(entry, true);
        let mut driver = Driver::new();
        for _ in 0..COLD_FRAMES {
            driver.step(cluster.as_mut(), &base_packet(), &ctx, SEQUENCE_DT);
        }
        for (name, packet) in fields {
            let what = format!("{theme}, {view:?}, {name}");
            stage.replace(what.clone());
            let since = repairs();
            driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
            driver.step(cluster.as_mut(), packet, &ctx, 0.0);
            std::hint::black_box(cluster.animating());
            nk.draw_complete(cluster.as_ref(), &layout);
            check_frame(nk, &what, since, failures);
            driver.step(cluster.as_mut(), &base_packet(), &ctx, SEQUENCE_DT);
        }
    }
    // Tiny, empty and very thin targets in every view.
    for view in [View::Cockpit, View::Hood, View::Chase] {
        for size in [(0, 0), (1, 1), (64, 36), (3840, 1), (1, 2160)] {
            let what = format!("{theme}, {view:?} at {}x{}", size.0, size.1);
            stage.replace(what.clone());
            let since = repairs();
            let mut cluster = nk.make(entry, true);
            let layout = Layout::from_config(&drive.config, &nk.tokens, view, size);
            {
                let ctx = drive.ctx(view);
                Driver::new().settle(cluster.as_mut(), &STATES[2], &ctx, COLD_FRAMES, SEQUENCE_DT);
            }
            nk.draw_complete(cluster.as_ref(), &layout);
            check_frame(nk, &what, since, failures);
        }
    }
}

#[test]
fn g5_hostile_telemetry_and_odd_targets_stay_finite() {
    let whole = whole_packets();
    let fields = field_packets();
    let failures = for_each_dash(true, 2, |nk, entry| {
        let stage = RefCell::new(String::from("setup"));
        let mut failures = Vec::new();
        let run = catch_unwind(AssertUnwindSafe(|| {
            robustness(nk, entry, &whole, &fields, &stage, &mut failures);
        }));
        if let Err(p) = run {
            failures.push(format!("{}: panicked: {}", stage.borrow(), panic_text(&*p)));
        }
        failures
    });
    assert!(failures.is_empty(), "G5 failures:\n{}", failures.join("\n"));
}

// --- Warm frames allocate nothing ------------------------------------------------------------

#[test]
fn warm_frames_do_not_allocate() {
    let failures = for_each_dash(true, 1, |nk, entry| {
        let mut failures = Vec::new();
        let drive = Drive::new(nk, entry);
        let own = DASH_SCENARIOS
            .iter()
            .filter(|(theme, _)| *theme == entry.theme)
            .flat_map(|(_, s)| (0..4).map(move |i| packet_at(s, i)));
        let frames: Vec<FH6TelemetryData> = (0..SEQUENCE_FRAMES)
            .map(sequence_frame)
            .chain(STATES.iter().map(|s| with_extras(s.telemetry())))
            .chain(own)
            .collect();
        // Pass 2 reads slightly different values, so a cache keyed by value would miss.
        let perturbed: Vec<FH6TelemetryData> = frames
            .iter()
            .map(|t| FH6TelemetryData {
                current_engine_rpm: t.current_engine_rpm + 7.0,
                speed_ms: t.speed_ms * 1.003,
                ..t.clone()
            })
            .collect();
        for view in designed_views(entry) {
            let mut cluster = nk.make(entry, true);
            let layout = Layout::from_config(&drive.config, &nk.tokens, view, TARGET);
            let ctx = drive.ctx(view);
            let mut driver = Driver::new();
            // Pass 1 warms caches and the glyph atlas on every frame.
            for packet in &frames {
                driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
                nk.draw_complete(cluster.as_ref(), &layout);
            }
            // Pass 2 is measured.
            let mut worst = (0, 0);
            for (i, packet) in perturbed.iter().enumerate() {
                let info = allocation_counter::measure(|| {
                    driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
                    std::hint::black_box(cluster.animating());
                    let mut p = nk.kit.begin_frame(&layout);
                    cluster.draw(&mut p, &layout);
                });
                if info.count_total > worst.1 {
                    worst = (i, info.count_total);
                }
            }
            if worst.1 > 0 {
                failures.push(format!(
                    "{}, {view:?}: {} heap allocations in warm frame {} of the review sequence",
                    entry.theme.name(),
                    worst.1,
                    worst.0
                ));
            }
        }
        failures
    });
    assert!(failures.is_empty(), "allocating dashes:\n{}", failures.join("\n"));
}

/// The allocation gate is only as good as the counter: it must see an allocation on this thread.
#[test]
fn the_allocation_counter_sees_allocations() {
    let info = allocation_counter::measure(|| {
        std::hint::black_box(vec![0u8; 64]);
    });
    assert!(info.count_total >= 1, "allocation-counter is not the global allocator");
}

// --- No asset folder -------------------------------------------------------------------------

#[test]
fn dashes_build_and_draw_without_assets() {
    let whole = whole_packets();
    let failures = for_each_dash(false, 1, |nk, entry| {
        let theme = entry.theme.name();
        let mut failures = Vec::new();
        let drive = Drive::new(nk, entry);
        let layout = Layout::from_config(&drive.config, &nk.tokens, View::Cockpit, TARGET);
        let ctx = drive.ctx(View::Cockpit);
        for state in &STATES {
            let since = repairs();
            let mut cluster = nk.make(entry, false);
            Driver::new().settle(cluster.as_mut(), state, &ctx, COLD_FRAMES, SEQUENCE_DT);
            nk.draw_complete(cluster.as_ref(), &layout);
            let what = format!("{theme} without assets, {}", state.name);
            check_frame(nk, &what, since, &mut failures);
            if nk.kit.scene().instance_count() == 0 {
                failures.push(format!("{what}: draws nothing"));
            }
        }
        for (name, packet) in &whole {
            let since = repairs();
            let mut cluster = nk.make(entry, false);
            let mut driver = Driver::new();
            for _ in 0..COLD_FRAMES {
                driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
            }
            nk.draw_complete(cluster.as_ref(), &layout);
            check_frame(nk, &format!("{theme} without assets, {name}"), since, &mut failures);
        }
        failures
    });
    assert!(failures.is_empty(), "no-asset failures:\n{}", failures.join("\n"));
}
