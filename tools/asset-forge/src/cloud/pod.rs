//! What runs on the pod. The pod is Linux and only takes shell, so its scripts live here as
//! Rust string constants (the repo holds no `.sh` files), the same way the Blender recipes hold
//! bpy. Every version is pinned to what this PC runs, and every download is checked against a
//! SHA-256 that was compared with the local file on 2026-11-27.
//!
//! Three scripts:
//! - the **start command** (the container's command): the idle watchdog, then ComfyUI and
//!   Blender's libraries when the disk is already set up, then the image's own start script,
//!   which brings up sshd;
//! - the **provisioner**, run over SSH when the disk is new or the pins changed;
//! - the **doctor**, which reports what is installed.
use crate::ledger::fingerprint;
use serde_json::{json, Value};

/// Where the pod's disk is mounted.
pub const WORKSPACE: &str = "/workspace";
/// The pod's name. There is only ever one.
pub const POD_NAME: &str = "asset-forge-gpu";
/// The Runpod template that carries the image, the ports and the start command.
pub const TEMPLATE_NAME: &str = "asset-forge-gpu";
/// The network volume that keeps the models and tools between pods.
pub const VOLUME_NAME: &str = "asset-forge";
/// Runpod's own Ubuntu 24.04 base image (its `/start.sh` starts sshd from `PUBLIC_KEY`).
/// Python, torch and the CUDA libraries live on the pod's disk, not in the image.
pub const DEFAULT_IMAGE: &str = "runpod/base:2.1.1-ubuntu2404";
/// ComfyUI's port on the pod. It listens on loopback only and is reached through SSH.
pub const COMFY_PORT: u16 = 8188;
/// Where job folders go: the pod's own disk, which is fast, free and new on every pod. The
/// results are copied back to this PC, so nothing in them has to outlive the pod.
pub const JOBS: &str = "/root/forge-jobs";

/// Blender, as on this PC.
pub const BLENDER_VERSION: &str = "5.3.0";
/// The official Linux build.
pub const BLENDER_URL: &str = "https://download.blender.org/release/Blender5.3/blender-5.3.0-linux-x64.tar.xz";
/// From `blender-5.3.0.sha256` in the same folder.
pub const BLENDER_SHA256: &str = "dacb8264197a3f7fda6001285a5a506fc97ee943881261c2c3bd63c882f22e05";

/// ComfyUI's repository.
pub const COMFY_REPO: &str = "https://github.com/Comfy-Org/ComfyUI";
/// The release this PC's portable install runs.
pub const COMFY_TAG: &str = "v0.46.0";
/// The commit that tag points at.
pub const COMFY_COMMIT: &str = "a277b5bf213e8a44fa34817b7770b54f86ef4cea";

/// The Python version of this PC's ComfyUI.
pub const PYTHON: &str = "3.13";
/// The torch packages of this PC's ComfyUI.
pub const TORCH: &str = "torch==2.14.1 torchvision==0.29.1 torchaudio==2.11.0";
/// The wheel index for them: the CUDA 13.0 build, as on this PC. It is also the build that
/// knows the Blackwell cards, which the CUDA 12.6 build does not.
pub const TORCH_INDEX: &str = "https://download.pytorch.org/whl/cu130";
/// The lowest host CUDA version a pod may land on (it matches [`TORCH_INDEX`]). Runpod's
/// secure hosts reported 13.2 on 2026-11-27.
pub const MIN_CUDA: &str = "13.0";

/// The Python packages ComfyUI shares with this PC, one `name==version` per line, at the
/// versions in this PC's `python_embeded` on 2026-11-27 (read from its `*.dist-info` folders).
/// Names are lowercase with hyphens, as `uv pip freeze` prints them. ComfyUI's own
/// `requirements.txt` leaves most of these open, so without this list a pod installed on
/// another day gets other versions.
pub const PINS: &str = r"
aiohappyeyeballs==2.7.1
aiohttp==3.14.5
aiosignal==1.4.0
alembic==1.20.1
annotated-doc==0.0.5
annotated-types==0.8.0
anyio==4.15.1
attrs==26.1.0
av==19.0.2
blake3==1.0.11
certifi==2026.10.23
charset-normalizer==3.5.2
click==8.5.0
comfy-aimdo==0.6.0
comfy-angle==0.1.1
comfy-kitchen==0.2.41
comfyui-embedded-docs==0.5.17
comfyui-frontend-package==1.59.6
comfyui-workflow-templates==0.11.111
comfyui-workflow-templates-core==0.3.402
comfyui-workflow-templates-json==0.1.137
comfyui-workflow-templates-media-api==0.3.84
comfyui-workflow-templates-media-assets-01==0.1.48
comfyui-workflow-templates-media-assets-02==0.1.34
einops==0.8.2
filelock==4.2.4
frozenlist==1.8.0
fsspec==2026.9.0
h11==0.16.0
hf-xet==1.7.0
httpcore2==2.16.0
httpx2==2.16.0
huggingface-hub==2.3.0
idna==3.20
jinja2==3.1.6
kornia==0.9.0
kornia-rs==0.2.0
mako==1.4.3
markdown-it-py==4.2.0
markupsafe==3.0.4
mdurl==0.1.2
mpmath==1.3.0
multidict==7.2.1
networkx==3.7
numpy==2.5.4
packaging==26.3
pillow==13.0.0
propcache==0.5.4
psutil==7.2.2
pydantic==2.14.1
pydantic-core==2.50.1
pydantic-settings==2.16.1
pygments==2.21.0
pyopengl==3.1.10
python-dotenv==1.2.4
pyyaml==6.0.3
regex==2026.9.29
requests==2.34.2
rich==15.0.0
safetensors==0.8.0
scipy==1.18.1
sentencepiece==0.2.2
setuptools==84.2.1
shellingham==1.5.4
simpleeval==1.0.8
spandrel==0.4.2
sqlalchemy==2.1.6
sympy==1.14.0
tokenizers==0.23.3
torchsde==0.2.6
tqdm==4.70.1
trampoline==0.1.2
transformers==5.21.0
truststore==0.10.5
typer==0.27.3
typing-extensions==4.16.0
typing-inspection==0.4.4
urllib3==2.8.0
yarl==1.25.1
";

/// Packages only the Linux install has, at the versions the first pod resolved on 2026-11-27:
/// the CUDA runtime wheels and triton (inside the torch wheel on Windows, and fixed by torch's
/// own requirements), and the workflow-template media packages, which this PC carries under
/// other names. They are pinned so that a rebuilt disk is the same as this one.
pub const PINS_LINUX: &str = r"
comfyui-workflow-templates-media-image==0.3.160
comfyui-workflow-templates-media-other==0.3.229
comfyui-workflow-templates-media-video==0.3.101
cuda-bindings==13.4.4
cuda-pathfinder==1.8.4
cuda-toolkit==13.0.3.0
nvidia-cublas==13.1.1.3
nvidia-cuda-cupti==13.0.85
nvidia-cuda-nvrtc==13.0.88
nvidia-cuda-runtime==13.0.96
nvidia-cudnn-cu13==9.24.0.43
nvidia-cufft==12.0.0.61
nvidia-cufile==1.15.1.6
nvidia-curand==10.4.0.35
nvidia-cusolver==12.0.4.66
nvidia-cusparse==12.6.3.3
nvidia-cusparselt-cu13==0.8.1
nvidia-nccl-cu13==2.30.7
nvidia-nvjitlink==13.4.92
nvidia-nvshmem-cu13==3.4.5
nvidia-nvtx==13.0.85
triton==3.8.0
";

/// A model file ComfyUI loads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Model {
    /// The folder under `models/`.
    pub folder: &'static str,
    /// The file name ComfyUI lists, the same as on this PC.
    pub file: &'static str,
    /// Where the pod downloads it from.
    pub url: &'static str,
    /// Its size in bytes.
    pub bytes: u64,
    /// Its SHA-256.
    pub sha256: &'static str,
}

/// The models of the local install: SDXL base and refiner, the depth and canny ControlNets and
/// 4x-UltraSharp.
pub const MODELS: &[Model] = &[
    Model {
        folder: "checkpoints",
        file: "sd_xl_base_1.0.safetensors",
        url: "https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/resolve/main/sd_xl_base_1.0.safetensors",
        bytes: 6_938_078_334,
        sha256: "31e35c80fc4829d14f90153f4c74cd59c90b779f6afe05a74cd6120b893f7e5b",
    },
    Model {
        folder: "checkpoints",
        file: "sd_xl_refiner_1.0.safetensors",
        url: "https://huggingface.co/stabilityai/stable-diffusion-xl-refiner-1.0/resolve/main/sd_xl_refiner_1.0.safetensors",
        bytes: 6_075_981_930,
        sha256: "7440042bbdc8a24813002c09b6b69b64dc90fded4472613437b7f55f9b7d9c5f",
    },
    Model {
        folder: "controlnet",
        file: "controlnet-canny-sdxl-1.0-small.fp16.safetensors",
        url: "https://huggingface.co/diffusers/controlnet-canny-sdxl-1.0-small/resolve/main/diffusion_pytorch_model.fp16.safetensors",
        bytes: 320_237_179,
        sha256: "fde4888a5f0a5648118991cc50e0ac4d60a2356dbaddf5e0649dd69c1119a2f9",
    },
    Model {
        folder: "controlnet",
        file: "controlnet-depth-sdxl-1.0-small.fp16.safetensors",
        url: "https://huggingface.co/diffusers/controlnet-depth-sdxl-1.0-small/resolve/main/diffusion_pytorch_model.fp16.safetensors",
        bytes: 320_237_179,
        sha256: "6f23c58fd632f52238a7b35ebdc02f9f596fd13dbaa121f9b37b9f4689c2b1e9",
    },
    Model {
        folder: "upscale_models",
        file: "4x-UltraSharp.pth",
        url: "https://huggingface.co/Kim2091/UltraSharp/resolve/main/4x-UltraSharp.pth",
        bytes: 66_961_958,
        sha256: "a5812231fc936b42af08a5edba784195495d303d5b3248c24489ef0c4021fe01",
    },
];

/// How often the watchdog looks, in seconds (the `sleep` in [`WATCHDOG`]).
pub const WATCHDOG_POLL_SECS: u64 = 30;

/// The watchdog. It deletes the pod when nothing has used it for `FORGE_IDLE_SECS`, or at the
/// end of its life: `FORGE_MAX_SECS` after this container started, or `FORGE_DEADLINE` (Unix
/// seconds, set when the pod was created, so it also holds across restarts), whichever is
/// earlier. "Used" means a script from this PC ran, the tunnel from this PC is open, Blender is
/// running, or the GPU is busy. It starts with the container, so it guards the pod even if this
/// PC never connects.
///
/// Nothing in it may wait for long: the GPU probe and the delete commands have time limits, the
/// log line goes to the network volume in the background, the two limits are checked before the
/// probes, and a second loop that needs nothing but the clock enforces the end of life on its
/// own, in case the first one hangs anyway.
const WATCHDOG: &str = r#"# asset-forge cloud lane: the pod's start command (asset-forge, src/cloud/pod.rs).
mkdir -p /run/forge
date +%s > /run/forge/boot
touch /run/forge/activity
forge_stop() {
  ( mkdir -p /workspace/forge && echo "$(date -u +%FT%TZ) $RUNPOD_POD_ID $1" >> /workspace/forge/stops.log ) &
  timeout -k 5 60 runpodctl pod delete "$RUNPOD_POD_ID" && return
  timeout -k 5 60 runpodctl remove pod "$RUNPOD_POD_ID" && return
  curl -fsS -m 30 -X DELETE -H "Authorization: Bearer $RUNPOD_API_KEY" "https://api.runpod.io/v2/pods/$RUNPOD_POD_ID" && return
  timeout -k 5 60 runpodctl pod stop "$RUNPOD_POD_ID" && return
  timeout -k 5 60 runpodctl stop pod "$RUNPOD_POD_ID" && return
  curl -fsS -m 30 -X POST -H "Authorization: Bearer $RUNPOD_API_KEY" -H "Content-Type: application/json" -d '{"action":"stop"}' "https://api.runpod.io/v2/pods/$RUNPOD_POD_ID/action"
}
forge_end=$(( $(cat /run/forge/boot) + ${FORGE_MAX_SECS:-14400} ))
case "${FORGE_DEADLINE:-}" in
  ''|*[!0-9]*) ;;
  *) if [ "$FORGE_DEADLINE" -lt "$forge_end" ]; then forge_end=$FORGE_DEADLINE; fi ;;
esac
forge_backstop() {
  while [ "$(date +%s)" -lt "$forge_end" ]; do sleep 30; done
  while :; do forge_stop "end of life (backstop)"; sleep 60; done
}
forge_watch() {
  idle=${FORGE_IDLE_SECS:-600}
  while sleep 30; do
    now=$(date +%s)
    if [ "$now" -ge "$forge_end" ]; then forge_stop "end of life"; continue; fi
    last=$(stat -c %Y /run/forge/activity 2>/dev/null || echo "$now")
    if [ $((now - last)) -ge "$idle" ]; then forge_stop "idle for ${idle}s"; continue; fi
    if grep -qx blender /proc/[0-9]*/comm 2>/dev/null; then touch /run/forge/activity; fi
    util=$(timeout -k 5 20 nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null | sort -n | tail -n 1)
    case "$util" in ''|*[!0-9]*) util=0 ;; esac
    if [ "$util" -ge 5 ]; then touch /run/forge/activity; fi
  done
}
forge_backstop > /run/forge/backstop.log 2>&1 &
forge_watch > /run/forge/watchdog.log 2>&1 &
"#;

/// Brings the pod's services up: Blender's shared libraries (they sit on the container disk,
/// which is new on every pod), then ComfyUI on loopback. It ends by writing `/run/forge/ready`,
/// or `/run/forge/failed` with the reason.
const SERVICES: &str = r#"log() { echo "$(date -u +%T) [services] $*"; }
fail() { log "$1"; echo "$1" > /run/forge/failed; exit 1; }
comfy_pid() {
  for p in /proc/[0-9]*; do
    if tr '\0\n' '  ' 2>/dev/null < "$p/cmdline" | grep -q '^/workspace/venv/bin/python main.py '; then echo "${p##*/}"; return 0; fi
  done
  return 1
}
mkdir -p /run/forge /workspace/forge
# One copy at a time. The pod's start command and a command from the PC can both ask for the
# services; the second one waits here and then finds them up.
exec 9> /run/forge/services.lock
flock 9
rm -f /run/forge/ready /run/forge/failed
if ! command -v curl >/dev/null 2>&1 || ! /workspace/blender/blender -b --version >/dev/null 2>&1; then
  log "installing Blender's shared libraries"
  export DEBIAN_FRONTEND=noninteractive
  # The installer may be using apt at the same moment; /run/forge/apt.lock makes one wait.
  (flock 8 && apt-get update -qq && apt-get install -y -qq --no-install-recommends curl ca-certificates libx11-6 libxi6 libxxf86vm1 libxfixes3 libxrender1 libgl1 libegl1 libxkbcommon0 libsm6 libice6 libgomp1) 8> /run/forge/apt.lock > /run/forge/apt.log 2>&1 || fail "apt could not install Blender's libraries (see /run/forge/apt.log)"
fi
version=$(/workspace/blender/blender -b --version 2>/dev/null | head -n 1)
[ -n "$version" ] || fail "Blender does not start"
log "$version"
if ! comfy_pid >/dev/null; then
  log "starting ComfyUI"
  cd /workspace/ComfyUI || fail "ComfyUI is not installed"
  # 9>&- : ComfyUI must not inherit the lock, or it would hold it for as long as it runs.
  setsid nohup /workspace/venv/bin/python main.py --listen 127.0.0.1 --port __COMFY_PORT__ > /workspace/forge/comfyui.log 2>&1 < /dev/null 9>&- &
fi
n=0
until curl -fsS -m 3 http://127.0.0.1:__COMFY_PORT__/system_stats >/dev/null 2>&1; do
  n=$((n + 1))
  if [ "$n" -gt 180 ]; then fail "ComfyUI did not answer within 6 minutes (see /workspace/forge/comfyui.log)"; fi
  if [ "$n" -gt 5 ] && ! comfy_pid >/dev/null; then fail "ComfyUI stopped while starting (see /workspace/forge/comfyui.log)"; fi
  sleep 2
done
find /workspace/cache -type f -mtime +14 -delete 2>/dev/null
touch /run/forge/ready
log ready
"#;

/// Hands over to the image's own start script, which sets up sshd from `PUBLIC_KEY` and keeps
/// the container alive. The fallback does the same by hand for an image without one.
const HANDOVER: &str = r#"if [ -x /start.sh ]; then exec /start.sh; fi
mkdir -p ~/.ssh && chmod 700 ~/.ssh
echo "$PUBLIC_KEY" >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys
service ssh start || /usr/sbin/sshd
exec sleep infinity
"#;

/// Installs everything on the pod's disk. Each step is skipped when its result is already
/// there, so a second run only checks.
const PROVISION: &str = r#"set -euo pipefail
W=/workspace
say() { echo "$(date -u +%T) [provision] $*"; }
mkdir -p "$W/forge" "$W/cache" "$W/models" /run/forge
# One installer or services script at a time: the pod's start command may be bringing the
# services up from this disk, and ComfyUI must never start while its files are being replaced.
# The services script takes the same lock and waits here.
exec 9> /run/forge/services.lock
flock 9
( while :; do touch /run/forge/activity; sleep 20; done ) 9>&- &
KEEP=$!
trap 'kill $KEEP 2>/dev/null || true' EXIT
# Stops a running ComfyUI before its files are replaced. It matches ComfyUI's own command line
# from the start: the container init holds this whole start command in its arguments, and a
# looser match would stop the container. It is called only by the steps that change files, so
# a pass that finds everything in place leaves ComfyUI running.
stop_comfyui() {
  for p in /proc/[0-9]*; do
    if tr '\0\n' '  ' 2>/dev/null < "$p/cmdline" | grep -q '^/workspace/venv/bin/python main.py '; then kill "${p##*/}" 2>/dev/null || true; fi
  done
  rm -f /run/forge/ready
}
# Job folders live on the pod's own disk now; an older install kept them on the volume.
rm -rf "$W/jobs"
export DEBIAN_FRONTEND=noninteractive
need=""
for tool in curl git xz; do command -v "$tool" >/dev/null 2>&1 || need="$need $tool"; done
if [ -n "$need" ]; then
  say "installing tools:$need"
  # The pod's start command may be using apt at the same moment (Blender's libraries);
  # /run/forge/apt.lock makes one wait for the other.
  ( flock 8; apt-get update -qq; apt-get install -y -qq --no-install-recommends curl ca-certificates git xz-utils ) 8> /run/forge/apt.lock
fi

B="$W/blender-__BLENDER_VERSION__"
if [ ! -x "$B/blender" ]; then
  say "Blender __BLENDER_VERSION__"
  curl -fsSL --retry 5 --retry-delay 5 -o /tmp/blender.tar.xz "__BLENDER_URL__"
  echo "__BLENDER_SHA256__  /tmp/blender.tar.xz" | sha256sum -c -
  rm -rf "$B.part" && mkdir -p "$B.part"
  tar -xJf /tmp/blender.tar.xz -C "$B.part" --strip-components=1
  mv "$B.part" "$B"
  rm -f /tmp/blender.tar.xz
fi
ln -sfn "$B" "$W/blender"

if [ "$(git -C "$W/ComfyUI" rev-parse HEAD 2>/dev/null || true)" != "__COMFY_COMMIT__" ]; then
  say "ComfyUI __COMFY_TAG__"
  stop_comfyui
  rm -rf "$W/ComfyUI" "$W/ComfyUI.part"
  git -c advice.detachedHead=false clone -q --depth 1 --branch "__COMFY_TAG__" "__COMFY_REPO__" "$W/ComfyUI.part"
  test "$(git -C "$W/ComfyUI.part" rev-parse HEAD)" = "__COMFY_COMMIT__"
  mv "$W/ComfyUI.part" "$W/ComfyUI"
fi

cat > "$W/forge/constraints.txt" <<'PINS'
__PINS__
PINS
PYBASE="python __PYTHON__"
PYSTAMP="$PYBASE with __TORCH__ from __TORCH_INDEX__, ComfyUI __COMFY_COMMIT__, pins __PINS_FP__"
HAVE="$(cat "$W/venv/.forge" 2>/dev/null || true)"
if [ "$HAVE" != "$PYSTAMP" ]; then
  say "Python __PYTHON__, torch and ComfyUI's requirements, at the pinned versions"
  stop_comfyui
  # Writing this environment to the volume takes about a quarter of an hour, so it is rebuilt
  # from nothing only when it has to be:
  #   - nothing is there yet;
  #   - it was made for another Python version;
  #   - an earlier attempt to bring it up to date did not finish (the .resumed mark is left).
  # Otherwise the files stay, and the two installs below change only the packages that are
  # missing or at another version. That covers an install that was cut off and a changed pin.
  case "$HAVE" in "$PYBASE "*|"") SAME_PYTHON=1 ;; *) SAME_PYTHON=0 ;; esac
  if [ ! -x "$W/venv/bin/python" ] || [ -f "$W/venv/.resumed" ] || [ "$SAME_PYTHON" = 0 ]; then
    rm -rf "$W/venv"
  else
    say "bringing the environment that is there to the pinned versions"
    touch "$W/venv/.resumed"
    # While it is being changed it carries no stamp, so a half-changed environment can never
    # be taken for a finished one.
    rm -f "$W/venv/.forge"
  fi
  if command -v uv >/dev/null 2>&1; then
    export UV_PYTHON_INSTALL_DIR="$W/python" UV_CACHE_DIR=/tmp/uv-cache UV_LINK_MODE=copy
    uv python install __PYTHON__
    [ -x "$W/venv/bin/python" ] || uv venv -q --python __PYTHON__ "$W/venv"
    PIP="uv pip install -q --python $W/venv/bin/python"
  else
    [ -x "$W/venv/bin/python" ] || python__PYTHON__ -m venv "$W/venv"
    PIP="$W/venv/bin/pip install -q --no-cache-dir"
  fi
  # torch comes from its own index, which does not carry the other pinned packages, so the
  # pins are applied by the second install. That one also corrects what torch brought along.
  $PIP --index-url "__TORCH_INDEX__" __TORCH__
  $PIP --constraint "$W/forge/constraints.txt" -r "$W/ComfyUI/requirements.txt"
  # The environment is stamped as good only after torch has seen the GPU.
  "$W/venv/bin/python" - <<'PY'
import torch
ok = torch.cuda.is_available()
print("[provision] torch", torch.__version__, "on", torch.cuda.get_device_name(0) if ok else "no GPU")
raise SystemExit(0 if ok else 1)
PY
  rm -f "$W/venv/.resumed"
  echo "$PYSTAMP" > "$W/venv/.forge"
fi

fetch() {
  d="$W/models/$1"
  f="$d/$2"
  mkdir -p "$d" "$W/ComfyUI/models/$1"
  if [ ! -f "$f" ] || [ "$(stat -c %s "$f")" != "$5" ]; then
    say "model $2"
    rm -f "$f.part"
    curl -fsSL --retry 8 --retry-delay 5 -o "$f.part" "$3"
    echo "$4  $f.part" | sha256sum -c -
    mv "$f.part" "$f"
  fi
  ln -sfn "$f" "$W/ComfyUI/models/$1/$2"
}
__MODELS__
printf '%s\n' "__STAMP__" > "$W/forge/provisioned"
say done
"#;

/// Reports what the pod has, one `key=value` per line.
const DOCTOR: &str = r#"echo "gpu=$(nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader 2>&1 | head -n 1)"
echo "blender=$(/workspace/blender/blender -b --version 2>&1 | head -n 1)"
cat > /tmp/forge_devices.py <<'PY'
import bpy
prefs = bpy.context.preferences.addons['cycles'].preferences
for kind in ('OPTIX', 'CUDA'):
    try:
        prefs.compute_device_type = kind
        prefs.get_devices()
        names = [d.name for d in prefs.devices if d.type == kind]
    except Exception:
        names = []
    print('cycles_%s=%s' % (kind.lower(), '; '.join(names)))
PY
/workspace/blender/blender -b --factory-startup -P /tmp/forge_devices.py 2>/dev/null | grep '^cycles_'
echo "comfyui=$(curl -fsS -m 5 http://127.0.0.1:__COMFY_PORT__/system_stats 2>/dev/null | tr -d '\n')"
echo "torch=$(/workspace/venv/bin/python -c 'import torch; print(torch.__version__)' 2>&1 | tail -n 1)"
echo "packages=$(UV_CACHE_DIR=/tmp/uv-cache uv pip freeze --python /workspace/venv/bin/python 2>/dev/null | tr '\n' ' ')"
for f in /workspace/models/*/*; do echo "model=$(stat -c %s "$f") ${f#/workspace/models/}"; done
echo "stamp=$(cat /workspace/forge/provisioned 2>/dev/null)"
echo "disk_mb=$(du -sm /workspace 2>/dev/null | cut -f1)"
echo "stops=$(tail -n 3 /workspace/forge/stops.log 2>/dev/null | tr '\n' '|')"
echo "services=$(tail -n 4 /workspace/forge/services.log 2>/dev/null | tr '\n' '|')"
echo "tunnels=$(for p in /proc/[0-9]*; do tr '\0' ' ' < "$p/cmdline" 2>/dev/null; echo; done | grep -c '[r]ead -r -t 60')"
"#;

fn fill(text: &str) -> String {
    text.replace("__COMFY_PORT__", &COMFY_PORT.to_string())
        .replace("__BLENDER_VERSION__", BLENDER_VERSION)
        .replace("__BLENDER_URL__", BLENDER_URL)
        .replace("__BLENDER_SHA256__", BLENDER_SHA256)
        .replace("__COMFY_REPO__", COMFY_REPO)
        .replace("__COMFY_TAG__", COMFY_TAG)
        .replace("__COMFY_COMMIT__", COMFY_COMMIT)
        .replace("__PYTHON__", PYTHON)
        .replace("__TORCH_INDEX__", TORCH_INDEX)
        .replace("__TORCH__", TORCH)
}

/// Every pinned package as `(name, version)`: [`PINS`], then [`PINS_LINUX`].
pub fn pins() -> impl Iterator<Item = (&'static str, &'static str)> {
    PINS.lines()
        .chain(PINS_LINUX.lines())
        .filter_map(|line| line.trim().split_once("=="))
}

/// The pins as the constraints file the installer gives to uv, one `name==version` per line.
fn constraints() -> String {
    let lines: Vec<String> = pins().map(|(name, version)| format!("{name}=={version}")).collect();
    lines.join("\n")
}

fn provision_body() -> String {
    let models: Vec<String> = MODELS
        .iter()
        .map(|m| format!("fetch {} {} \"{}\" {} {}", m.folder, m.file, m.url, m.sha256, m.bytes))
        .collect();
    let constraints = constraints();
    fill(PROVISION)
        .replace("__MODELS__", &models.join("\n"))
        .replace("__PINS_FP__", &fingerprint(&[constraints.as_bytes()]))
        .replace("__PINS__", &constraints)
}

/// The marker a provisioned disk carries. It changes whenever the provisioner's text changes,
/// which is whenever a pin changes, so an out-of-date disk is provisioned again.
#[must_use]
pub fn stamp() -> String {
    format!("forge-pod-{}", fingerprint(&[provision_body().as_bytes()]))
}

/// The provisioner, ready to run on the pod.
#[must_use]
pub fn provision_script() -> String {
    provision_body().replace("__STAMP__", &stamp())
}

/// The services script, ready to run on the pod.
#[must_use]
pub fn services_script() -> String {
    fill(SERVICES)
}

/// The doctor script, ready to run on the pod.
#[must_use]
pub fn doctor_script() -> String {
    fill(DOCTOR)
}

/// The container's command: watchdog first, then the services when the disk is provisioned,
/// then the image's own start script.
#[must_use]
pub fn start_command() -> String {
    // The services start here only on a disk this version installed. After a pin change the
    // PC runs the installer first and then the services itself.
    format!(
        "{WATCHDOG}if [ \"$(cat /workspace/forge/provisioned 2>/dev/null)\" = \"{}\" ]; then\n(\n{}) >> /workspace/forge/services.log 2>&1 &\nfi\n{HANDOVER}",
        stamp(),
        services_script()
    )
}

/// The Runpod template: image, container disk, the SSH port and the start command.
#[must_use]
pub fn template(image: &str, disk_gb: u32) -> Value {
    json!({
        "name": TEMPLATE_NAME,
        "image": image,
        "disk": disk_gb,
        "ports": ["22/tcp"],
        "env": { "NVIDIA_DRIVER_CAPABILITIES": "all" },
        "cmd": ["bash", "-c", start_command()],
        "category": "NVIDIA",
        "public": false,
        "serverless": false,
        "startSsh": true,
        "startJupyter": false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// True when `text` still holds a `__NAME__` placeholder (capitals, digits, underscores).
    fn has_placeholder(text: &str) -> bool {
        text.split("__").skip(1).step_by(2).any(|name| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        })
    }

    #[test]
    fn no_placeholder_survives_in_any_script() {
        assert!(has_placeholder("a __COMFY_PORT__ b"));
        assert!(!has_placeholder("print(torch.__version__)"));
        for (name, text) in [
            ("start", start_command()),
            ("provision", provision_script()),
            ("services", services_script()),
            ("doctor", doctor_script()),
        ] {
            assert!(!has_placeholder(&text), "{name} still has a placeholder");
            assert!(!text.contains('\r'), "{name} has a carriage return");
        }
    }

    #[test]
    fn the_watchdog_starts_before_anything_else_and_uses_the_two_limits() {
        let start = start_command();
        assert!(start.find("forge_watch >").unwrap() < start.find("[services]").unwrap());
        assert!(start.find("[services]").unwrap() < start.find("exec /start.sh").unwrap());
        assert!(start.contains("${FORGE_IDLE_SECS:-600}"));
        assert!(start.contains("${FORGE_MAX_SECS:-14400}"));
        assert!(start.contains("runpodctl pod delete \"$RUNPOD_POD_ID\""));
        // The PC's estimate of a watchdog deletion counts on this interval.
        assert!(start.contains(&format!("while sleep {WATCHDOG_POLL_SECS}; do")));
    }

    #[test]
    fn nothing_in_the_watchdog_can_hold_up_a_deletion() {
        let w = WATCHDOG;
        // Every command that talks to the GPU or to the provider has a time limit.
        assert!(w.contains("timeout -k 5 20 nvidia-smi"));
        for line in w.lines().filter(|l| l.contains("runpodctl")) {
            assert!(line.trim_start().starts_with("timeout -k 5 60 runpodctl"), "{line}");
        }
        for line in w.lines().filter(|l| l.contains("curl ")) {
            assert!(line.contains("curl -fsS -m 30 "), "{line}");
        }
        // The log line on the network volume is written in the background.
        assert!(w.contains(">> /workspace/forge/stops.log ) &\n"));
        assert!(!w.contains("mkdir -p /run/forge /workspace"));
        // Both limits are checked before the probes.
        let probe = w.find("nvidia-smi").unwrap();
        assert!(w.find("forge_stop \"end of life\"").unwrap() < probe);
        assert!(w.find("forge_stop \"idle for").unwrap() < probe);
        // A second loop ends the pod on time with nothing but the clock, and the deadline set
        // at creation holds across restarts of the container.
        assert!(w.contains("forge_backstop > /run/forge/backstop.log 2>&1 &\n"));
        assert!(w.contains("if [ \"$FORGE_DEADLINE\" -lt \"$forge_end\" ]; then forge_end=$FORGE_DEADLINE; fi"));
    }

    #[test]
    fn the_provisioner_stops_comfyui_without_matching_the_containers_init() {
        // The container's init runs as `docker-init -- bash -c <the start command>`, so its
        // command line holds every word of the start command, newlines included. Both scripts
        // turn the whole command line into one line before matching, so `^` can only match
        // its start: ComfyUI's own command.
        let own = "/workspace/venv/bin/python main.py ";
        let matcher = format!("tr '\\0\\n' '  ' 2>/dev/null < \"$p/cmdline\" | grep -q '^{own}'");
        assert_eq!(provision_script().matches(&matcher).count(), 1);
        assert_eq!(services_script().matches(&matcher).count(), 1);
        assert!(services_script().contains(&format!("setsid nohup {own}--listen")));
        let init = format!("/sbin/docker-init\0--\0bash\0-c\0{}", start_command()).replace(['\0', '\n'], " ");
        assert!(init.contains("main.py --listen"));
        assert!(!init.starts_with(own));
    }

    #[test]
    fn the_pod_starts_the_services_only_on_a_disk_this_version_installed() {
        let start = start_command();
        assert!(start.contains(&format!(
            "if [ \"$(cat /workspace/forge/provisioned 2>/dev/null)\" = \"{}\" ]; then\n(\n",
            stamp()
        )));
        // The installer holds the services' lock from before its first step to its end, and
        // its keep-alive loop does not inherit the lock.
        let script = provision_script();
        let lock = script.find("exec 9> /run/forge/services.lock\nflock 9\n").unwrap();
        assert!(lock < script.find("B=\"$W/blender-").unwrap());
        assert!(script.contains("done ) 9>&- &\nKEEP=$!"));
        assert!(services_script().contains("exec 9> /run/forge/services.lock\nflock 9\n"));
    }

    #[test]
    fn every_script_is_valid_bash() {
        // `bash -n` reads each script from standard input and checks its syntax. Without bash
        // on this PC (Git for Windows brings one) there is nothing to run, and the test passes.
        let Ok(probe) = std::process::Command::new("bash").arg("--version").output() else {
            return;
        };
        if !probe.status.success() {
            return;
        }
        for (name, text) in [
            ("start", start_command()),
            ("provision", provision_script()),
            ("services", services_script()),
            ("doctor", doctor_script()),
            ("keepalive", super::super::ssh::KEEPALIVE.to_owned()),
        ] {
            let mut child = std::process::Command::new("bash")
                .arg("-n")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            {
                use std::io::Write as _;
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(text.as_bytes()).unwrap();
            }
            let out = child.wait_with_output().unwrap();
            assert!(out.status.success(), "{name}: {}", String::from_utf8_lossy(&out.stderr));
        }
    }

    #[test]
    fn the_python_environment_is_stamped_only_after_torch_sees_the_gpu() {
        let script = provision_script();
        let check = script.find("torch.cuda.is_available()").unwrap();
        let stamp = script.find("echo \"$PYSTAMP\" > \"$W/venv/.forge\"").unwrap();
        assert!(check < stamp, "the stamp must come after the check");
    }

    #[test]
    fn the_environment_is_rebuilt_from_nothing_only_when_it_has_to_be() {
        let script = provision_script();
        // Rebuild when nothing is there, when an earlier attempt did not finish, or when it
        // was made for another Python version. A changed pin alone keeps the files.
        assert!(script.contains(
            "if [ ! -x \"$W/venv/bin/python\" ] || [ -f \"$W/venv/.resumed\" ] || [ \"$SAME_PYTHON\" = 0 ]; then\n    rm -rf \"$W/venv\""
        ));
        assert!(script.contains("case \"$HAVE\" in \"$PYBASE \"*|\"\") SAME_PYTHON=1 ;; *) SAME_PYTHON=0 ;; esac"));
        // The stamp starts with the Python version and a space, which is what that test reads.
        assert!(script.contains("PYBASE=\"python 3.13\"\nPYSTAMP=\"$PYBASE with torch==2.14.1 "));
        // While the files are being changed there is a mark and no stamp.
        let mark = script.find("touch \"$W/venv/.resumed\"").unwrap();
        let unstamp = script.find("rm -f \"$W/venv/.forge\"").unwrap();
        let install = script.find("$PIP --index-url").unwrap();
        assert!(mark < unstamp && unstamp < install);
        // The mark goes, and the stamp comes back, only at the very end.
        let unmark = script.find("rm -f \"$W/venv/.resumed\"").unwrap();
        assert!(install < unmark && unmark < script.find("echo \"$PYSTAMP\"").unwrap());
        // An environment that is there is never created a second time.
        assert!(script.contains("[ -x \"$W/venv/bin/python\" ] || uv venv -q --python 3.13 \"$W/venv\""));
    }

    #[test]
    fn the_pins_are_well_formed_and_reach_the_installer() {
        let all: Vec<(&str, &str)> = pins().collect();
        assert_eq!(PINS.lines().filter(|l| !l.trim().is_empty()).count(), 79);
        assert_eq!(PINS_LINUX.lines().filter(|l| !l.trim().is_empty()).count(), 22);
        assert_eq!(all.len(), 101, "a line without == was dropped");
        for (name, version) in &all {
            let plain = name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.');
            assert!(plain && !name.is_empty(), "{name}");
            assert!(version.starts_with(|c: char| c.is_ascii_digit()), "{name}=={version}");
            assert_eq!(
                all.iter().filter(|(n, _)| n == name).count(),
                1,
                "{name} is pinned twice"
            );
        }
        // The three packages of the torch line have their own index and exact builds, so they
        // are not in the lists. (`torchsde` is a different package and is pinned like the rest.)
        for own in ["torch", "torchvision", "torchaudio"] {
            assert!(
                !all.iter().any(|(n, _)| *n == own),
                "{own} must not be in the pin lists"
            );
        }
        assert!(all.contains(&("torchsde", "0.2.6")));
        // The installer writes exactly these lines into its constraints file.
        let script = provision_script();
        let start = script.find("<<'PINS'\n").unwrap() + "<<'PINS'\n".len();
        let end = start + script[start..].find("\nPINS\n").unwrap();
        let written: Vec<&str> = script[start..end].lines().collect();
        assert_eq!(written.len(), 101);
        assert!(written.contains(&"kornia==0.9.0"));
        assert!(written.contains(&"transformers==5.21.0"));
        assert!(written.contains(&"triton==3.8.0"));
    }

    #[test]
    fn only_the_requirements_install_takes_the_pins() {
        let script = provision_script();
        // The PyTorch index does not carry the other pinned packages, so a constrained torch
        // install cannot be resolved there (checked with a dry run on 2026-11-27).
        assert!(script.contains("\n  $PIP --index-url \"https://download.pytorch.org/whl/cu130\" torch==2.14.1 "));
        assert!(
            script.contains("\n  $PIP --constraint \"$W/forge/constraints.txt\" -r \"$W/ComfyUI/requirements.txt\"\n")
        );
    }

    #[test]
    fn a_changed_pin_changes_both_stamps() {
        let body = provision_body();
        let other = body.replace("kornia==0.9.0", "kornia==0.9.2");
        assert_ne!(fingerprint(&[body.as_bytes()]), fingerprint(&[other.as_bytes()]));
        assert!(body.contains(&format!("pins {}\"", fingerprint(&[constraints().as_bytes()]))));
    }

    #[test]
    fn the_services_run_one_copy_at_a_time_and_start_comfyui_once() {
        let s = services_script();
        // The lock comes before anything is changed, and ComfyUI is started without it.
        let lock = s.find("exec 9> /run/forge/services.lock\nflock 9\n").unwrap();
        assert!(lock < s.find("rm -f /run/forge/ready /run/forge/failed").unwrap());
        assert!(
            s.contains("main.py --listen 127.0.0.1 --port 8188 > /workspace/forge/comfyui.log 2>&1 < /dev/null 9>&- &")
        );
        // A ComfyUI that is already starting is waited for, not started a second time.
        assert!(s.contains("if ! comfy_pid >/dev/null; then\n  log \"starting ComfyUI\""));
        // One that died is reported at once instead of after the six-minute limit.
        assert!(s.contains("! comfy_pid >/dev/null; then fail \"ComfyUI stopped while starting"));
    }

    #[test]
    fn the_installer_stops_comfyui_only_in_the_steps_that_replace_its_files() {
        let script = provision_script();
        let calls: Vec<usize> = script.match_indices("\n  stop_comfyui\n").map(|(at, _)| at).collect();
        assert_eq!(
            calls.len(),
            2,
            "one call in the ComfyUI step and one in the Python step"
        );
        let comfy_step = script.find("say \"ComfyUI v0.46.0\"").unwrap();
        let python_step = script.find("say \"Python 3.13, torch").unwrap();
        assert!(comfy_step < calls[0] && calls[0] < script.find("rm -rf \"$W/ComfyUI\"").unwrap());
        assert!(python_step < calls[1] && calls[1] < script.find("$PIP --index-url").unwrap());
        // Nothing outside that function stops a process.
        assert_eq!(script.matches("kill \"${p##*/}\"").count(), 1);
    }

    #[test]
    fn job_folders_are_on_the_pods_own_disk() {
        assert!(JOBS.starts_with("/root/"));
        assert!(!JOBS.starts_with(WORKSPACE));
        // The installer clears the folder an older install kept on the volume.
        assert!(provision_script().contains("rm -rf \"$W/jobs\""));
    }

    #[test]
    fn the_start_command_fits_runpods_request_limit() {
        // A pod-create request, with the template folded in, may be 102,400 bytes at most.
        assert!(start_command().len() < 16_000, "{}", start_command().len());
    }

    #[test]
    fn comfyui_listens_on_loopback_only() {
        assert!(services_script().contains("--listen 127.0.0.1 --port 8188"));
        assert!(!start_command().contains("0.0.0.0"));
    }

    #[test]
    fn the_provisioner_downloads_and_checks_every_model() {
        let script = provision_script();
        for m in MODELS {
            let line = format!("fetch {} {} \"{}\" {} {}", m.folder, m.file, m.url, m.sha256, m.bytes);
            assert!(script.contains(&line), "{}", m.file);
            assert_eq!(m.sha256.len(), 64);
            assert!(m.url.starts_with("https://huggingface.co/"));
        }
        assert!(script.contains("sha256sum -c -"));
        assert!(script.contains(BLENDER_SHA256));
        assert!(script.contains(COMFY_COMMIT));
        assert!(script.contains(&stamp()));
    }

    #[test]
    fn the_models_match_the_sizes_the_local_doctor_checks() {
        let size = |file: &str| MODELS.iter().find(|m| m.file == file).map(|m| m.bytes);
        assert_eq!(size(crate::paths::SDXL_CKPT), Some(crate::paths::SDXL_BYTES));
        assert_eq!(size(crate::paths::UPSCALER), Some(crate::paths::UPSCALER_BYTES));
        assert_eq!(MODELS.len(), 5);
    }

    #[test]
    fn the_stamp_is_stable_and_names_no_placeholder() {
        assert_eq!(stamp(), stamp());
        assert!(stamp().starts_with("forge-pod-"));
        assert_eq!(stamp().len(), "forge-pod-".len() + 16);
    }

    #[test]
    fn the_template_exposes_only_ssh_and_runs_the_start_command() {
        let t = template(DEFAULT_IMAGE, 20);
        assert_eq!(t["ports"], json!(["22/tcp"]));
        assert_eq!(t["cmd"][0], json!("bash"));
        assert_eq!(t["cmd"][1], json!("-c"));
        assert_eq!(t["cmd"][2], json!(start_command()));
        assert_eq!(t["public"], json!(false));
        assert_eq!(t["env"]["NVIDIA_DRIVER_CAPABILITIES"], json!("all"));
    }
}
