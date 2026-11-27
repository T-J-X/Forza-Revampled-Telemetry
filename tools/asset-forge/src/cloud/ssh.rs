//! SSH to the pod through the OpenSSH client that ships with Windows (`ssh.exe`, `scp.exe`,
//! `ssh-keygen.exe`), the same way `blender.rs` drives `blender.exe`. Scripts go to the pod's
//! `bash -s` on standard input, so nothing has to survive two layers of quoting. Every script
//! first marks the pod as in use, which is what its idle watchdog looks at.
use crate::error::{io_err, tool_err, Result};
use std::ffi::OsString;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Output, Stdio};

/// Marks the pod as in use, then makes the pod's own variables (`RUNPOD_POD_ID` and friends)
/// visible, which a non-interactive SSH shell does not get by default.
const PREAMBLE: &str =
    "mkdir -p /run/forge && touch /run/forge/activity\nif [ -f /etc/rp_environment ]; then . /etc/rp_environment; fi\n";

/// What the tunnel's remote side runs: keep marking the pod as in use until this end closes
/// its input, which happens when asset-forge exits for any reason.
const KEEPALIVE: &str =
    "mkdir -p /run/forge; while :; do touch /run/forge/activity; sleep 15; done & cat >/dev/null; kill $!";

/// The line that ends a script on its way to the pod. A script must not contain it.
const SCRIPT_END: &str = "FORGE_SCRIPT_END";

/// What is sent to the pod's `bash -s` for `script`: save it to a file, run that file with no
/// standard input, and pass its exit status back. Running from a file keeps a command inside
/// the script (apt, Blender) from reading the rest of the script as its own input.
fn wrap(script: &str) -> String {
    format!(
        "f=$(mktemp /tmp/forge.XXXXXX)\ncat > \"$f\" <<'{SCRIPT_END}'\n{PREAMBLE}{script}\n{SCRIPT_END}\nbash \"$f\" </dev/null\nrc=$?\nrm -f \"$f\"\nexit $rc\n"
    )
}

/// Finds an OpenSSH program: `FH6_OPENSSH_DIR`, then Windows' own copy, then the `PATH`.
fn program(name: &str) -> PathBuf {
    let exe = format!("{name}.exe");
    if let Some(dir) = std::env::var_os("FH6_OPENSSH_DIR").map(PathBuf::from) {
        return dir.join(exe);
    }
    let system = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("System32")
        .join("OpenSSH")
        .join(&exe);
    if system.is_file() {
        system
    } else {
        PathBuf::from(name)
    }
}

/// A path in the form OpenSSH takes inside an `-o Name=value` option: forward slashes, and
/// quotes when it holds a space.
fn option_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.contains(' ') {
        format!("\"{text}\"")
    } else {
        text
    }
}

/// Creates an ed25519 key pair without a passphrase at `key` (and `key.pub`), unless it exists.
pub fn ensure_key(key: &Path) -> Result<()> {
    if key.is_file() && key.with_extension("pub").is_file() {
        return Ok(());
    }
    if let Some(dir) = key.parent() {
        std::fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    }
    let out = Command::new(program("ssh-keygen"))
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "asset-forge-cloud", "-f"])
        .arg(key)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| tool_err("ssh-keygen", e.to_string()))?;
    if out.status.success() && key.is_file() {
        Ok(())
    } else {
        Err(tool_err(
            "ssh-keygen",
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ))
    }
}

/// A pod's SSH address plus the key and host-key file to use with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ssh {
    /// Host name or address.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// User name.
    pub user: String,
    /// The private key.
    pub key: PathBuf,
    /// The host-key file for this pod. Each pod gets its own, so the first contact is trusted
    /// and any later change of host key is refused.
    pub known_hosts: PathBuf,
}

impl Ssh {
    fn target(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }

    /// Options shared by `ssh` and `scp` (the port flag differs between them).
    fn options(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = vec!["-i".into(), self.key.clone().into()];
        for option in [
            "IdentitiesOnly=yes".to_owned(),
            "BatchMode=yes".to_owned(),
            "StrictHostKeyChecking=accept-new".to_owned(),
            format!("UserKnownHostsFile={}", option_path(&self.known_hosts)),
            "ConnectTimeout=20".to_owned(),
            "ServerAliveInterval=20".to_owned(),
            "ServerAliveCountMax=6".to_owned(),
            "LogLevel=ERROR".to_owned(),
        ] {
            args.push("-o".into());
            args.push(option.into());
        }
        args
    }

    fn ssh(&self) -> Command {
        let mut cmd = Command::new(program("ssh"));
        cmd.args(self.options()).arg("-T").arg("-p").arg(self.port.to_string());
        cmd
    }

    fn scp(&self) -> Command {
        let mut cmd = Command::new(program("scp"));
        cmd.args(self.options()).arg("-q").arg("-P").arg(self.port.to_string());
        cmd.stdin(Stdio::null());
        cmd
    }

    fn feed(mut child: Child, script: &str) -> Result<Child> {
        if let Some(mut stdin) = child.stdin.take() {
            // A script the far end stops reading early is reported by its exit status instead.
            let _ = stdin.write_all(wrap(script).as_bytes());
        }
        Ok(child)
    }

    /// Runs `script` in the pod's bash and returns what it printed and how it ended.
    pub fn run(&self, script: &str) -> Result<Output> {
        let child = self
            .ssh()
            .arg(self.target())
            .arg("bash -s")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| tool_err("ssh", e.to_string()))?;
        Self::feed(child, script)?
            .wait_with_output()
            .map_err(|e| tool_err("ssh", e.to_string()))
    }

    /// Runs `script` with its output going straight to this console (for long steps), and
    /// returns whether it succeeded.
    pub fn run_live(&self, script: &str) -> Result<bool> {
        let child = self
            .ssh()
            .arg(self.target())
            .arg("bash -s")
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| tool_err("ssh", e.to_string()))?;
        let status = Self::feed(child, script)?
            .wait()
            .map_err(|e| tool_err("ssh", e.to_string()))?;
        Ok(status.success())
    }

    /// Runs `script`, requires it to succeed, and returns its standard output.
    pub fn check(&self, script: &str) -> Result<String> {
        let out = self.run(script)?;
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        if out.status.success() {
            return Ok(stdout);
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let tail: Vec<&str> = stdout
            .lines()
            .chain(stderr.lines())
            .filter(|l| !l.trim().is_empty())
            .collect();
        let tail = tail[tail.len().saturating_sub(25)..].join("\n");
        Err(tool_err(
            "pod",
            format!("a command on the pod failed ({}):\n{tail}", out.status),
        ))
    }

    /// True when the pod's sshd answers and accepts the key.
    #[must_use]
    pub fn reachable(&self) -> bool {
        self.run("true").is_ok_and(|out| out.status.success())
    }

    fn finish_scp(mut cmd: Command, what: &str) -> Result<()> {
        let out = cmd.output().map_err(|e| tool_err("scp", e.to_string()))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(tool_err(
                "scp",
                format!("{what}: {}", String::from_utf8_lossy(&out.stderr).trim()),
            ))
        }
    }

    /// Copies one local file or folder to `remote` (a path on the pod with no spaces).
    pub fn upload(&self, local: &Path, remote: &str) -> Result<()> {
        let mut cmd = self.scp();
        if local.is_dir() {
            cmd.arg("-r");
        }
        cmd.arg(local).arg(format!("{}:{remote}", self.target()));
        Self::finish_scp(cmd, &format!("upload {}", local.display()))
    }

    /// Copies the folder `remote` on the pod into `local`, merging over what is there. `scratch`
    /// is an empty-able folder on the same disk as `local`, used for the incoming copy.
    pub fn fetch_dir(&self, remote: &str, local: &Path, scratch: &Path) -> Result<usize> {
        let _ = std::fs::remove_dir_all(scratch);
        std::fs::create_dir_all(scratch).map_err(|e| io_err(scratch, e))?;
        let mut cmd = self.scp();
        cmd.arg("-r").arg(format!("{}:{remote}", self.target())).arg(scratch);
        Self::finish_scp(cmd, &format!("download {remote}"))?;
        // scp puts the folder itself inside `scratch`; its contents are what `local` wants.
        let name = remote.trim_end_matches('/').rsplit('/').next().unwrap_or_default();
        let moved = move_tree(&scratch.join(name), local)?;
        let _ = std::fs::remove_dir_all(scratch);
        Ok(moved)
    }

    /// Opens a forward from `127.0.0.1:<local_port>` here to `127.0.0.1:<remote_port>` on the
    /// pod, and keeps the pod marked as in use for as long as the returned value lives.
    pub fn tunnel(&self, local_port: u16, remote_port: u16) -> Result<Tunnel> {
        let mut child = self
            .ssh()
            .arg("-o")
            .arg("ExitOnForwardFailure=yes")
            .arg("-L")
            .arg(format!("127.0.0.1:{local_port}:127.0.0.1:{remote_port}"))
            .arg(self.target())
            .arg(KEEPALIVE)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| tool_err("ssh", e.to_string()))?;
        let stdin = child.stdin.take();
        Ok(Tunnel { child, stdin })
    }
}

/// A running port forward. Dropping it closes the forward; so does the end of this process,
/// because the far side stops when its input closes.
#[derive(Debug)]
pub struct Tunnel {
    child: Child,
    stdin: Option<ChildStdin>,
}

impl Tunnel {
    /// True while the SSH process behind the forward is still running.
    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        drop(self.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Moves every file under `from` to the same relative place under `to`, replacing files that
/// are already there. Returns the number of files moved.
pub fn move_tree(from: &Path, to: &Path) -> Result<usize> {
    std::fs::create_dir_all(to).map_err(|e| io_err(to, e))?;
    let mut moved = 0;
    for entry in std::fs::read_dir(from).map_err(|e| io_err(from, e))? {
        let entry = entry.map_err(|e| io_err(from, e))?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            moved += move_tree(&src, &dst)?;
            continue;
        }
        let _ = std::fs::remove_file(&dst);
        if std::fs::rename(&src, &dst).is_err() {
            // A rename cannot cross disks; copy instead.
            std::fs::copy(&src, &dst).map_err(|e| io_err(&dst, e))?;
            let _ = std::fs::remove_file(&src);
        }
        moved += 1;
    }
    Ok(moved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pod() -> Ssh {
        Ssh {
            host: "213.173.108.12".into(),
            port: 17445,
            user: "root".into(),
            key: PathBuf::from(r"C:\Tools\cloud\id_ed25519"),
            known_hosts: PathBuf::from(r"C:\Tools Folder\cloud\known_hosts_p1"),
        }
    }

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn ssh_uses_the_key_only_and_trusts_a_pod_on_first_contact() {
        let a = args(&pod().ssh());
        assert!(a.windows(2).any(|w| w == ["-i", r"C:\Tools\cloud\id_ed25519"]));
        assert!(a.contains(&"IdentitiesOnly=yes".to_owned()));
        assert!(a.contains(&"BatchMode=yes".to_owned()));
        assert!(a.contains(&"StrictHostKeyChecking=accept-new".to_owned()));
        assert!(a.windows(2).any(|w| w == ["-p", "17445"]));
    }

    #[test]
    fn the_host_key_file_is_quoted_when_its_path_has_a_space() {
        let a = args(&pod().ssh());
        assert!(
            a.contains(&"UserKnownHostsFile=\"C:/Tools Folder/cloud/known_hosts_p1\"".to_owned()),
            "{a:?}"
        );
        assert_eq!(option_path(Path::new(r"C:\a\b")), "C:/a/b");
    }

    #[test]
    fn scp_takes_the_port_with_a_capital_p() {
        let a = args(&pod().scp());
        assert!(a.windows(2).any(|w| w == ["-P", "17445"]));
        assert!(!a.contains(&"-p".to_owned()));
    }

    #[test]
    fn every_script_marks_the_pod_as_in_use_first() {
        assert!(PREAMBLE.starts_with("mkdir -p /run/forge && touch /run/forge/activity"));
        assert!(KEEPALIVE.contains("touch /run/forge/activity"));
        assert!(KEEPALIVE.contains("cat >/dev/null"));
    }

    #[test]
    fn a_script_runs_from_a_file_with_no_input_and_returns_its_status() {
        let sent = wrap("echo hello");
        let body_at = sent.find("echo hello").unwrap();
        assert!(sent.find("<<'FORGE_SCRIPT_END'").unwrap() < body_at);
        assert!(sent.find(PREAMBLE).unwrap() < body_at);
        assert!(sent[body_at..].contains("\nFORGE_SCRIPT_END\nbash \"$f\" </dev/null\n"));
        assert!(sent.ends_with("exit $rc\n"));
    }

    #[test]
    fn move_tree_merges_and_replaces() {
        let root = std::env::temp_dir().join("forge_ssh_move_tree");
        let _ = std::fs::remove_dir_all(&root);
        let (from, to) = (root.join("from"), root.join("to"));
        std::fs::create_dir_all(from.join("sub")).unwrap();
        std::fs::create_dir_all(&to).unwrap();
        std::fs::write(from.join("a.png"), b"new").unwrap();
        std::fs::write(from.join("sub").join("b.exr"), b"deep").unwrap();
        std::fs::write(to.join("a.png"), b"old").unwrap();
        std::fs::write(to.join("keep.txt"), b"kept").unwrap();
        assert_eq!(move_tree(&from, &to).ok(), Some(2));
        assert_eq!(std::fs::read(to.join("a.png")).unwrap(), b"new");
        assert_eq!(std::fs::read(to.join("sub").join("b.exr")).unwrap(), b"deep");
        assert_eq!(std::fs::read(to.join("keep.txt")).unwrap(), b"kept");
        let _ = std::fs::remove_dir_all(&root);
    }
}
