//! Where the tools and the repo live. Defaults follow ONBOARDING.md; `FH6_BLENDER`,
//! `FH6_COMFYUI` and `FH6_COMFYUI_URL` override them. The GPU lane and the masters folder come
//! from the command line (`--lane`, `--masters`) or from `FH6_GPU_LANE` and `FH6_MASTERS`.
use crate::lane::Lane;
use std::path::PathBuf;

/// Locations of the external tools and the repo.
#[derive(Debug, Clone)]
pub struct Paths {
    /// The workspace root (holds `Cargo.toml` and `assets/`).
    pub root: PathBuf,
    /// The tools folder (`Documents\Dash_Tools`): Blender, ComfyUI, the lane locks and the cloud
    /// lane's own folder.
    pub tools: PathBuf,
    /// `blender.exe`.
    pub blender: PathBuf,
    /// The ComfyUI portable folder (holds `python_embeded/` and `ComfyUI/`).
    pub comfy_dir: PathBuf,
    /// The ComfyUI API base URL.
    pub comfy_url: String,
    /// Scratch space for generated bpy scripts and intermediate renders (never committed).
    pub scratch: PathBuf,
    /// The GPU this run's ComfyUI and Blender jobs go to.
    pub lane: Lane,
    /// A dash's folder in the Drive masters folder; when set, every Blender output folder and
    /// every master-sized ComfyUI image is also copied there.
    pub masters: Option<PathBuf>,
}

/// The SDXL checkpoint file name in `models/checkpoints`.
pub const SDXL_CKPT: &str = "sd_xl_base_1.0.safetensors";
/// Its expected size in bytes (the official file).
pub const SDXL_BYTES: u64 = 6_938_078_334;
/// The upscaler in `models/upscale_models`.
pub const UPSCALER: &str = "4x-UltraSharp.pth";
/// Its expected size in bytes.
pub const UPSCALER_BYTES: u64 = 66_961_958;

impl Paths {
    /// Resolves every path from the environment and the current directory. The lane starts as
    /// local and no masters folder is set; `main` applies the flags.
    pub fn discover() -> Self {
        let root = find_root();
        let tools = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("Documents")
            .join("Dash_Tools");
        let blender = std::env::var_os("FH6_BLENDER")
            .map(PathBuf::from)
            .unwrap_or_else(|| tools.join("blender-5.3.0-windows-x64").join("blender.exe"));
        let comfy_dir = std::env::var_os("FH6_COMFYUI")
            .map(PathBuf::from)
            .unwrap_or_else(|| tools.join("ComfyUI_windows_portable"));
        let comfy_url = std::env::var("FH6_COMFYUI_URL").unwrap_or_else(|_| "http://127.0.0.1:8188".to_owned());
        let scratch = root.join("target").join("asset-forge");
        Self {
            root,
            tools,
            blender,
            comfy_dir,
            comfy_url,
            scratch,
            lane: Lane::Local,
            masters: None,
        }
    }

    /// `assets/`.
    pub fn assets(&self) -> PathBuf {
        self.root.join("assets")
    }

    /// `tools/asset-forge/`.
    pub fn forge_dir(&self) -> PathBuf {
        self.root.join("tools").join("asset-forge")
    }

    /// The ComfyUI models folder.
    pub fn comfy_models(&self) -> PathBuf {
        self.comfy_dir.join("ComfyUI").join("models")
    }
}

/// Walks up from the current directory to the folder that holds the workspace `Cargo.toml`.
fn find_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_default();
    let mut dir = start.as_path();
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            if let Ok(text) = std::fs::read_to_string(&manifest) {
                if text.contains("[workspace]") {
                    return dir.to_path_buf();
                }
            }
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return start,
        }
    }
}
