//! `visual-review`: the review images a dash thread looks at before it answers its checklist.
//!
//! CPU only, so no GPU lock: PNG or JPEG in, PNG out, every input composited over OLED black
//! first, exactly as the panel shows it. The procedure that uses it is the `fh6-visual-review`
//! skill (`.claude/skills/fh6-visual-review/SKILL.md`); `--help` lists every command and flag.

/// `println!` that ignores a closed pipe (`visual-review ... | head`) instead of panicking.
macro_rules! out {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

/// `eprintln!` that ignores a closed pipe.
macro_rules! warn {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stderr(), $($arg)*);
    }};
}

mod args;
mod blind;
mod cli;
mod components;
mod cvd;
mod diff;
mod error;
mod flash;
mod frame;
mod geom;
mod glance;
mod motion;
mod overlay;
mod probe;
mod sheet;
mod steady;
mod stretch;
mod tiles;
mod tokens;
mod unwrap;

use args::Args;
use error::{usage, Error, Result};
use std::process::ExitCode;

const USAGE: &str = "\
visual-review: review images for dash renders (CPU only, no GPU lock)

usage: cargo run --release -p visual-review -- <command> ... [--straight]

  tiles    IMAGE [--view V | --region X,Y,W,H] [--anchor V] [--zoom N] [--max WxH] [--out DIR]
           1:1 tiles Claude can see without downscaling (default: the view envelope of a full frame)
  crop     IMAGE (--view V | --region X,Y,W,H) [--anchor V] [--zoom N] [--mirror] [--mask X,Y,W,H ...]
           [--out FILE.png|DIR]
           one region, optionally enlarged with nearest-neighbour pixels, mirrored or with areas masked
  glance   FRAME --zone NAME:X,Y,W,H[:MIN] ... [--anchor V] [--over BACKDROP|#RRGGBB]
           [--sigma S] [--squint-sigma S] [--out DIR]
           the blur gate: 960x540 at sigma 1.5, contrast per zone (gear 7:1, speed 4.5:1, others 4.5:1);
           FRAME is a full render, a <state>_crop.png or a sequence frame
  diff     A B [--view V | --region X,Y,W,H] [--anchor V] [--fit] [--delta 3] [--max-fraction 0.005]
           [--pairs 3] [--labels \"A|B\"] [--out DIR]
           OKLab difference as the golden harness measures it: heat map, crop pairs, a 50% blend
  sheet    IMAGE[@X,Y,W,H|@view] ... [--labels \"A|B|C\"] [--cols N] [--width PX] [--blur S] [--title T]
           [--view V | --region X,Y,W,H] [--anchor V] [--out FILE.png|DIR]
           pictures side by side under labels
  flash    SEQUENCE_DIR [--frames 30-41] [--eye-m 0.6] [--scale 1.0] [--fps 60] [--out FILE.png|DIR]
           WCAG 2.3.1 general and red flash audit over a frame sequence (30-41: the limiter hold)
  motion   SEQUENCE_DIR [--frames A-B] [--out FILE.png|DIR]
           change per frame: stalls, spikes and whether the idle is still
  cvd      IMAGE [--view V | --region X,Y,W,H] [--anchor V] [--out DIR]
           protanopia, deuteranopia, tritanopia and luminance-only views
  stretch  IMAGE [--max 8] [--view V | --region X,Y,W,H] [--anchor V] [--out FILE.png|DIR]
           near-black levels stretched: haze, target edges and faint ink an OLED shows
  static   STATE_RENDER STATE_RENDER ... [--view V | --region X,Y,W,H] [--anchor V] [--mask X,Y,W,H ...]
           [--out FILE.png|DIR]
           the parts that never change: how bright they are against the live data
  blind    MOCKUPS_DIR [--count 8] [--seed N] [--anchor V] [--mask X,Y,W,H ...] [--out DIR] [--key FILE]
           seeded, neutral-named frames for a blind glance read (logo masked), answer key kept apart
  key      the review telemetry of all 60 sequence frames and the 4 states (the answer key)
  unwrap   IMAGE --center CX,CY --radius INNER,OUTER [--from DEG] [--to DEG] [--ppd N] [--anchor V]
           [--out FILE.png|DIR]
           a dial ring unrolled into a strip, to count ticks and read the needle angle
  annotate IMAGE (--box X,Y,W,H:LABEL | --ring CX,CY,R:LABEL | --arrow X1,Y1,X2,Y2:LABEL
           | --protractor CX,CY,R | --note TEXT) ...
           [--view V | --region X,Y,W,H] [--zoom N] [--anchor V] [--color #RRGGBB] [--out FILE.png|DIR]
           numbered findings (labels in a legend) drawn on an image or a crop of it
  probe    IMAGE [--view V | --region X,Y,W,H] [--anchor V]
           true-black share, luminance spread, median colour and hues of a region

Coordinates are 4K frame pixels (3840x2160). --anchor cockpit|hood|hud makes them local to that
view's anchor (a Geom table's numbers); on a <state>_crop.png or a sequence frame they are mapped
onto the crop. --view cockpit|hood|hud picks the envelope plus 40 px.
--straight reads inputs with transparency as straight alpha (a design-tool export); renders are
premultiplied.
Outputs go to mockups/<Name>/review/ for an input that belongs to a dash, else mockups/review/;
--out FILE.png|DIR names a file, DIR a folder.";

fn main() -> ExitCode {
    let argv: Vec<String> = match std::env::args_os().skip(1).map(|a| a.into_string()).collect() {
        Ok(v) => v,
        Err(bad) => {
            warn!("error: the argument {bad:?} is not valid Unicode");
            return ExitCode::from(2);
        }
    };
    let mut it = argv.into_iter();
    let Some(cmd) = it.next() else {
        warn!("{USAGE}");
        return ExitCode::from(2);
    };
    if matches!(cmd.as_str(), "-h" | "--help" | "help") {
        out!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match run(&cmd, it) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Usage(msg)) => {
            warn!("error: {msg}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(e) => {
            warn!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cmd: &str, rest: impl Iterator<Item = String>) -> Result<()> {
    let args = Args::parse(rest, &["--fit", "--mirror", "--straight", "--help"])?;
    // Help is asked for as a switch (`--help` or `-h` before any `--`), never found inside a
    // flag's value.
    if args.has("--help") {
        out!("{USAGE}");
        return Ok(());
    }
    let ctx = cli::Ctx::new(args.has("--straight"))?;
    match cmd {
        "tiles" => tiles::tiles(&args, &ctx),
        "crop" => tiles::crop(&args, &ctx),
        "glance" => glance::run(&args, &ctx),
        "diff" => diff::run(&args, &ctx),
        "sheet" => sheet::run(&args, &ctx),
        "flash" => flash::run(&args, &ctx),
        "motion" => motion::run(&args, &ctx),
        "cvd" => cvd::run(&args, &ctx),
        "stretch" => stretch::run(&args, &ctx),
        "static" => steady::run(&args, &ctx),
        "blind" => blind::run(&args, &ctx),
        "key" => blind::key(&args),
        "unwrap" => unwrap::run(&args, &ctx),
        "annotate" => overlay::annotate(&args, &ctx),
        "probe" => probe::run(&args, &ctx),
        other => Err(usage(format!("unknown command {other:?}"))),
    }
}
