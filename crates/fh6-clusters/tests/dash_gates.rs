//! The dash gates every cluster in the registry plugs into, run on wgpu's no-op backend so they
//! need neither a GPU nor the GPU lock:
//!
//! - G2 and G3: each input that `docs/TELEMETRY_USAGE_MATRIX.md` lists for a dash must change
//!   its frame, and no other input may. Every registry entry has exactly one row.
//! - G5: telemetry decoded from hostile bytes (NaN, infinities, huge values, noise) and tiny or
//!   zero-size targets never put a NaN or an infinity into the frame, and drop nothing.
//! - Warm frames allocate nothing: after one pass through the review sequence, `update` and
//!   `draw` make no heap allocation (AGENTS.md: no per-frame allocation).
//! - Every dash builds and draws with no asset folder (the fallback path).
//!
//! "Changes its frame" means the scene's fingerprint (every instance, paint, clip and draw, bit
//! for bit) differs. A rebuilt dash plugs in by updating its row in the matrix. Each gate
//! checks every dash and reports all failures at once.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test harness: a setup step that fails should stop the test with its message"
)]

use fh6_clusters::registry::REGISTRY;
use fh6_clusters::review::{sequence_frame, DriveContext, Driver, SEQUENCE_DT, SEQUENCE_FRAMES, STATES};
use fh6_core::telemetry::{decode, PACKET_LEN};
use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData};
use fh6_kit::testing::{Offscreen, OFFSCREEN_FORMAT};
use fh6_kit::{wgpu, Cluster, ClusterEntry, FontChoice, Kit, Layout, Logo, Palette, Tokens, View};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::task::{Context, Poll, Waker};

/// Updates before the frame a probe fingerprints (springs mostly settle; probes run in lockstep,
/// so settling is not required for the comparison to be fair).
const SETTLE_FRAMES: u32 = 20;
/// The 4K target every gate draws for, except the tiny-target cases.
const TARGET: (u32, u32) = (3840, 2160);

/// One column of the matrix: a group of packet fields and the change its probe makes.
struct Input {
    key: &'static str,
    fields: &'static str,
    change: fn(&mut FH6TelemetryData),
}

/// Every matrix column. Together they change every field of [`FH6TelemetryData`]; `all_fields`
/// below breaks the build when a field is added without a column.
const INPUTS: &[Input] = &[
    Input {
        key: "race_on",
        fields: "is_race_on",
        change: |t| t.is_race_on = 0,
    },
    Input {
        key: "timestamp",
        fields: "timestamp_ms",
        change: |t| t.timestamp_ms = 123_456,
    },
    Input {
        key: "rpm",
        fields: "current_engine_rpm",
        change: |t| t.current_engine_rpm = 7_600.0,
    },
    Input {
        key: "max_rpm",
        fields: "engine_max_rpm",
        change: |t| t.engine_max_rpm = 6_500.0,
    },
    Input {
        key: "idle_rpm",
        fields: "engine_idle_rpm",
        change: |t| t.engine_idle_rpm = 1_500.0,
    },
    Input {
        key: "gear",
        fields: "gear",
        change: |t| t.gear = 2,
    },
    Input {
        key: "speed",
        fields: "speed_ms",
        change: |t| t.speed_ms = 30.0,
    },
    Input {
        key: "throttle",
        fields: "accel",
        change: |t| t.accel = 255,
    },
    Input {
        key: "brake",
        fields: "brake",
        change: |t| t.brake = 200,
    },
    Input {
        key: "clutch",
        fields: "clutch",
        change: |t| t.clutch = 255,
    },
    Input {
        key: "handbrake",
        fields: "hand_brake",
        change: |t| t.hand_brake = 255,
    },
    Input {
        key: "steer",
        fields: "steer",
        change: |t| t.steer = 90,
    },
    Input {
        key: "g_lat",
        fields: "acceleration_x",
        change: |t| t.acceleration_x = -8.8,
    },
    Input {
        key: "g_long",
        fields: "acceleration_z",
        change: |t| t.acceleration_z = -7.8,
    },
    Input {
        key: "g_vert",
        fields: "acceleration_y",
        change: |t| t.acceleration_y = 4.0,
    },
    Input {
        key: "velocity",
        fields: "velocity_x, velocity_y, velocity_z",
        change: |t| {
            let (s, c) = 20f32.to_radians().sin_cos();
            t.velocity_x = t.speed_ms * s;
            t.velocity_y = 1.0;
            t.velocity_z = t.speed_ms * c;
        },
    },
    Input {
        key: "yaw_rate",
        fields: "angular_velocity_y",
        change: |t| t.angular_velocity_y = 0.8,
    },
    Input {
        key: "orientation",
        fields: "yaw, pitch, roll",
        change: |t| {
            t.yaw = 1.2;
            t.pitch = 0.1;
            t.roll = -0.1;
        },
    },
    Input {
        key: "tyre_slip",
        fields: "tire_slip_angle_fl, _fr, _rl, _rr",
        change: |t| {
            t.tire_slip_angle_fl = 0.15;
            t.tire_slip_angle_fr = 0.2;
            t.tire_slip_angle_rl = 0.25;
            t.tire_slip_angle_rr = 0.3;
        },
    },
    Input {
        key: "suspension",
        fields: "normalized_suspension_travel_fl, _fr, _rl, _rr",
        change: |t| {
            t.normalized_suspension_travel_fl = 0.1;
            t.normalized_suspension_travel_fr = 0.9;
            t.normalized_suspension_travel_rl = 0.3;
            t.normalized_suspension_travel_rr = 0.7;
        },
    },
    Input {
        key: "suspension_m",
        fields: "suspension_travel_meters_fl, _fr, _rl, _rr",
        change: |t| {
            t.suspension_travel_meters_fl = 0.01;
            t.suspension_travel_meters_fr = 0.09;
            t.suspension_travel_meters_rl = 0.03;
            t.suspension_travel_meters_rr = 0.07;
        },
    },
    Input {
        key: "car_ordinal",
        fields: "car_ordinal",
        change: |t| t.car_ordinal = 2_200,
    },
    Input {
        key: "drivetrain",
        fields: "drivetrain_type",
        change: |t| t.drivetrain_type = 2,
    },
    Input {
        key: "position",
        fields: "position_x, position_y, position_z",
        change: |t| {
            t.position_x = 120.0;
            t.position_y = 5.0;
            t.position_z = -340.0;
        },
    },
    Input {
        key: "power",
        fields: "power_watts",
        change: |t| t.power_watts = 250_000.0,
    },
    Input {
        key: "boost",
        fields: "boost_psi",
        change: |t| t.boost_psi = 21.0,
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
/// robustness gate interleaves.
fn base_packet() -> FH6TelemetryData {
    with_extras(STATES[1].telemetry())
}

/// A packet sequence a probe drives: the packet for update `i`.
struct Scenario {
    name: &'static str,
    packet: fn(u32) -> FH6TelemetryData,
}

/// The scenarios every dash is probed in: the four review states and the cruise base.
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
/// then run in [`COMMON`] and in these.
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

/// Names every field, so a new telemetry field fails to compile here until it has a column.
#[test]
fn every_telemetry_field_has_a_matrix_column() {
    let FH6TelemetryData {
        is_race_on: _,
        timestamp_ms: _,
        engine_max_rpm: _,
        engine_idle_rpm: _,
        current_engine_rpm: _,
        acceleration_x: _,
        acceleration_y: _,
        acceleration_z: _,
        velocity_x: _,
        velocity_y: _,
        velocity_z: _,
        angular_velocity_y: _,
        yaw: _,
        pitch: _,
        roll: _,
        tire_slip_angle_fl: _,
        tire_slip_angle_fr: _,
        tire_slip_angle_rl: _,
        tire_slip_angle_rr: _,
        normalized_suspension_travel_fl: _,
        normalized_suspension_travel_fr: _,
        normalized_suspension_travel_rl: _,
        normalized_suspension_travel_rr: _,
        suspension_travel_meters_fl: _,
        suspension_travel_meters_fr: _,
        suspension_travel_meters_rl: _,
        suspension_travel_meters_rr: _,
        car_ordinal: _,
        drivetrain_type: _,
        position_x: _,
        position_y: _,
        position_z: _,
        speed_ms: _,
        power_watts: _,
        boost_psi: _,
        accel: _,
        brake: _,
        clutch: _,
        hand_brake: _,
        gear: _,
        steer: _,
    } = base_packet();
    // Each probe really changes its packet.
    let base = base_packet();
    for input in INPUTS {
        let mut t = base.clone();
        (input.change)(&mut t);
        assert_ne!(t, base, "the {} probe changes nothing", input.key);
    }
    let keys: BTreeSet<_> = INPUTS.iter().map(|i| i.key).collect();
    assert_eq!(keys.len(), INPUTS.len(), "duplicate input key");
}

// --- The shared no-op kit -------------------------------------------------------------------

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
}

/// One kit for every gate: building a dash the second time reuses its decoded images.
fn shared(fonts: bool) -> MutexGuard<'static, NoopKit> {
    static WITH_FONTS: OnceLock<Mutex<NoopKit>> = OnceLock::new();
    static WITHOUT_FONTS: OnceLock<Mutex<NoopKit>> = OnceLock::new();
    let cell = if fonts { &WITH_FONTS } else { &WITHOUT_FONTS };
    cell.get_or_init(|| Mutex::new(NoopKit::new(fonts)))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// What a driver passes besides telemetry, owned so it does not borrow the kit.
struct Drive {
    config: AppConfig,
    palette: Palette,
    tokens: Tokens,
    fonts: FontChoice,
    logo: Logo,
}

impl Drive {
    fn new(nk: &NoopKit, entry: &ClusterEntry) -> Self {
        Self {
            config: AppConfig::default(),
            palette: Palette::family(&nk.tokens, entry.family),
            tokens: nk.tokens.clone(),
            fonts: nk.kit.font_choice(None),
            logo: Logo::default(),
        }
    }

    fn ctx(&self, view: View) -> DriveContext<'_> {
        DriveContext {
            config: &self.config,
            palette: &self.palette,
            tokens: &self.tokens,
            fonts: self.fonts,
            logo: &self.logo,
            view,
            reduced_flash: false,
        }
    }
}

/// Fingerprint of a fresh `entry` after [`SETTLE_FRAMES`] updates on `scenario`, with `change`
/// applied to every packet.
fn probe(
    nk: &mut NoopKit,
    entry: &ClusterEntry,
    scenario: &Scenario,
    change: Option<fn(&mut FH6TelemetryData)>,
) -> u64 {
    let mut cluster = nk.make(entry, true);
    let drive = Drive::new(nk, entry);
    let layout = Layout::from_config(&drive.config, &nk.tokens, View::Cockpit, TARGET);
    {
        let ctx = drive.ctx(View::Cockpit);
        let mut driver = Driver::new();
        for i in 0..=SETTLE_FRAMES {
            let mut packet = (scenario.packet)(i);
            if let Some(change) = change {
                change(&mut packet);
            }
            driver.step(cluster.as_mut(), &packet, &ctx, SEQUENCE_DT);
        }
    }
    nk.draw_complete(cluster.as_ref(), &layout);
    nk.kit.scene().fingerprint()
}

// --- G2 and G3: the matrix ------------------------------------------------------------------

fn matrix_path() -> PathBuf {
    repo_root().join("docs/TELEMETRY_USAGE_MATRIX.md")
}

/// The rows of the markdown table in the matrix section headed `heading`, as trimmed cells with
/// the header and separator rows left out.
fn table(heading: &str) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(matrix_path()).expect("docs/TELEMETRY_USAGE_MATRIX.md");
    let section = text
        .split(&format!("\n## {heading}\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("the matrix has a `## {heading}` section"));
    let mut rows = Vec::new();
    let mut header = true;
    for line in section.lines().map(str::trim) {
        if line.starts_with("## ") {
            break;
        }
        let Some(inner) = line.strip_prefix('|').and_then(|l| l.strip_suffix('|')) else {
            continue;
        };
        let cells: Vec<String> = inner.split('|').map(|c| c.trim().to_owned()).collect();
        if header || cells.iter().all(|c| c.starts_with("---")) {
            header = false;
            continue;
        }
        rows.push(cells);
    }
    rows
}

fn unquote(cell: &str) -> String {
    cell.trim().trim_matches('`').to_owned()
}

/// The `## Dashes` table: theme name -> declared input keys.
fn declared_matrix() -> BTreeMap<String, BTreeSet<String>> {
    let mut rows = BTreeMap::new();
    for cells in table("Dashes") {
        let [theme, _name, reads] = &cells[..] else {
            panic!("a Dashes row needs 3 cells (theme, dash, reads): {cells:?}");
        };
        let keys: BTreeSet<String> = if reads == "none" {
            BTreeSet::new()
        } else {
            reads.split(',').map(unquote).collect()
        };
        let theme = unquote(theme);
        assert!(
            rows.insert(theme.clone(), keys).is_none(),
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

/// The inputs whose probe changes `entry`'s frame in at least one of its scenarios.
fn observed_inputs(nk: &mut NoopKit, entry: &ClusterEntry) -> Result<BTreeSet<String>, String> {
    let own = DASH_SCENARIOS
        .iter()
        .filter(|(theme, _)| *theme == entry.theme)
        .map(|(_, s)| s);
    let mut reads = BTreeSet::new();
    for scenario in COMMON.iter().chain(own) {
        let reference = probe(nk, entry, scenario, None);
        for input in INPUTS {
            if !reads.contains(input.key) && probe(nk, entry, scenario, Some(input.change)) != reference {
                reads.insert(input.key.to_owned());
            }
        }
        // The same packets must give the same frame, or the probes above mean nothing.
        if probe(nk, entry, scenario, None) != reference {
            return Err(format!(
                "{}, {}: two fresh builds fed the same packets drew different frames (wall \
                 clock or unseeded randomness?)",
                entry.theme.name(),
                scenario.name
            ));
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
    for (theme, keys) in &declared {
        if !REGISTRY.iter().any(|e| e.theme.name() == theme) {
            failures.push(format!("{theme}: matrix row for a dash not in the registry"));
        }
        for k in keys.iter().filter(|k| !known.contains(k.as_str())) {
            failures.push(format!("{theme}: unknown input `{k}`"));
        }
    }
    let mut nk = shared(true);
    for entry in REGISTRY {
        let theme = entry.theme.name();
        let Some(claimed) = declared.get(theme) else {
            failures.push(format!("{theme}: no row in docs/TELEMETRY_USAGE_MATRIX.md"));
            continue;
        };
        match observed_inputs(&mut nk, entry) {
            Err(e) => failures.push(e),
            Ok(seen) if &seen != claimed => {
                let mut why = Vec::new();
                let unkept: BTreeSet<_> = claimed.difference(&seen).cloned().collect();
                if !unkept.is_empty() {
                    why.push(format!("listed but never change the frame: {}", join(&unkept)));
                }
                let unlisted: BTreeSet<_> = seen.difference(claimed).cloned().collect();
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
    }
    assert!(failures.is_empty(), "G2/G3 failures:\n{}", failures.join("\n"));
}

// --- G5: hostile telemetry and tiny targets -------------------------------------------------

/// Packets decoded from hostile bytes: NaN, both infinities, huge and tiny floats, all-zero and
/// all-ones, and seeded noise. Production telemetry only reaches a dash through `decode`.
fn hostile_packets() -> Vec<(String, FH6TelemetryData)> {
    let mut out = Vec::new();
    let mut add = |name: String, bytes: Vec<u8>| {
        out.push((name, decode(&bytes).expect("packet length")));
    };
    for (name, word) in [
        ("NaN", f32::NAN.to_le_bytes()),
        ("+Inf", f32::INFINITY.to_le_bytes()),
        ("-Inf", f32::NEG_INFINITY.to_le_bytes()),
        ("f32::MAX", f32::MAX.to_le_bytes()),
        ("-f32::MAX", f32::MIN.to_le_bytes()),
        ("subnormal", f32::from_bits(1).to_le_bytes()),
    ] {
        add(name.to_owned(), word.iter().copied().cycle().take(PACKET_LEN).collect());
    }
    for byte in [0x00, 0xFF, 0x7F, 0x80] {
        add(format!("all 0x{byte:02X}"), vec![byte; PACKET_LEN]);
    }
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for seed in 0..4 {
        let bytes = (0..PACKET_LEN)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x >> 56) as u8
            })
            .collect();
        add(format!("noise {seed}"), bytes);
    }
    out
}

fn check_frame(nk: &NoopKit, what: &str, failures: &mut Vec<String>) {
    let scene = nk.kit.scene();
    let bad = scene.non_finite_count();
    if bad > 0 {
        failures.push(format!("{what}: {bad} NaN or infinite values in the frame"));
    }
    if scene.dropped() > 0 {
        failures.push(format!("{what}: {} items dropped (capacity reached)", scene.dropped()));
    }
}

#[test]
fn g5_hostile_telemetry_and_tiny_targets_stay_finite() {
    let packets = hostile_packets();
    let mut failures = Vec::new();
    let mut nk = shared(true);
    for entry in REGISTRY {
        let theme = entry.theme.name();
        for (name, packet) in &packets {
            let mut cluster = nk.make(entry, true);
            let drive = Drive::new(&nk, entry);
            let layout = Layout::from_config(&drive.config, &nk.tokens, View::Cockpit, TARGET);
            {
                let ctx = drive.ctx(View::Cockpit);
                let mut driver = Driver::new();
                // Hostile from a cold start, then hostile after a normal reading.
                for _ in 0..=SETTLE_FRAMES {
                    driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
                }
                {
                    let mut p = nk.kit.begin_frame(&layout);
                    cluster.draw(&mut p, &layout);
                }
                check_frame(&nk, &format!("{theme}, {name}, cold"), &mut failures);
                let ctx = drive.ctx(View::Cockpit);
                driver.step(cluster.as_mut(), &base_packet(), &ctx, SEQUENCE_DT);
                driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
                // A zero dt (two packets in one frame) must not divide by zero.
                driver.step(cluster.as_mut(), packet, &ctx, 0.0);
            }
            {
                let mut p = nk.kit.begin_frame(&layout);
                cluster.draw(&mut p, &layout);
            }
            check_frame(&nk, &format!("{theme}, {name}, after a reading"), &mut failures);
        }
        for view in [View::Cockpit, View::Hood, View::Chase] {
            for size in [(0, 0), (1, 1), (64, 36)] {
                let mut cluster = nk.make(entry, true);
                let drive = Drive::new(&nk, entry);
                let layout = Layout::from_config(&drive.config, &nk.tokens, view, size);
                {
                    let ctx = drive.ctx(view);
                    Driver::new().settle(cluster.as_mut(), &STATES[2], &ctx, SETTLE_FRAMES, SEQUENCE_DT);
                }
                {
                    let mut p = nk.kit.begin_frame(&layout);
                    cluster.draw(&mut p, &layout);
                }
                check_frame(
                    &nk,
                    &format!("{theme}, {view:?} at {}x{}", size.0, size.1),
                    &mut failures,
                );
            }
        }
    }
    assert!(failures.is_empty(), "G5 failures:\n{}", failures.join("\n"));
}

// --- Warm frames allocate nothing -----------------------------------------------------------

#[test]
fn warm_frames_do_not_allocate() {
    let mut failures = Vec::new();
    let mut nk = shared(true);
    for entry in REGISTRY {
        let mut cluster = nk.make(entry, true);
        let drive = Drive::new(&nk, entry);
        let layout = Layout::from_config(&drive.config, &nk.tokens, View::Cockpit, TARGET);
        let mut driver = Driver::new();
        // Pass 1 warms caches and the glyph atlas on every review frame and state.
        let frames: Vec<FH6TelemetryData> = (0..SEQUENCE_FRAMES)
            .map(sequence_frame)
            .chain(STATES.iter().map(|s| s.telemetry()))
            .collect();
        for packet in &frames {
            let ctx = drive.ctx(View::Cockpit);
            driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
            nk.draw_complete(cluster.as_ref(), &layout);
        }
        // Pass 2 is measured.
        let ctx = drive.ctx(View::Cockpit);
        let mut worst = (0, 0);
        for (i, packet) in frames.iter().enumerate() {
            let info = allocation_counter::measure(|| {
                driver.step(cluster.as_mut(), packet, &ctx, SEQUENCE_DT);
                let mut p = nk.kit.begin_frame(&layout);
                cluster.draw(&mut p, &layout);
            });
            if info.count_total > worst.1 {
                worst = (i, info.count_total);
            }
        }
        if worst.1 > 0 {
            failures.push(format!(
                "{}: {} heap allocations in warm frame {} of the review sequence",
                entry.theme.name(),
                worst.1,
                worst.0
            ));
        }
    }
    assert!(failures.is_empty(), "allocating dashes:\n{}", failures.join("\n"));
}

// --- No asset folder ------------------------------------------------------------------------

#[test]
fn dashes_build_and_draw_without_assets() {
    let mut failures = Vec::new();
    let mut nk = shared(false);
    for entry in REGISTRY {
        let theme = entry.theme.name();
        let mut cluster = nk.make(entry, false);
        let drive = Drive::new(&nk, entry);
        let layout = Layout::from_config(&drive.config, &nk.tokens, View::Cockpit, TARGET);
        for state in &STATES {
            {
                let ctx = drive.ctx(View::Cockpit);
                Driver::new().settle(cluster.as_mut(), state, &ctx, SETTLE_FRAMES, SEQUENCE_DT);
            }
            nk.draw_complete(cluster.as_ref(), &layout);
            check_frame(&nk, &format!("{theme} without assets, {}", state.name), &mut failures);
            if nk.kit.scene().instance_count() == 0 {
                failures.push(format!("{theme} without assets, {}: draws nothing", state.name));
            }
        }
    }
    assert!(failures.is_empty(), "no-asset failures:\n{}", failures.join("\n"));
}

/// The allocation gate is only as good as the counter: it must see an allocation on this thread.
#[test]
fn the_allocation_counter_sees_allocations() {
    let info = allocation_counter::measure(|| {
        std::hint::black_box(vec![0u8; 64]);
    });
    assert!(info.count_total >= 1, "allocation-counter is not the global allocator");
}
