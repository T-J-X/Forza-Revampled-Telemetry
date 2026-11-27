//! Blender on the pod. A recipe's parameters name folders and files on this PC, so they are
//! rewritten for the pod: `out` becomes the job's output folder there, and every other value
//! that is the path of an existing file or folder here is uploaded and replaced by where it
//! landed. After the render the output folder is copied back into the local `out`, so the
//! recipe's Rust code finds its files exactly where a local run would have put them.
//!
//! Uploaded files are kept on the pod's disk under a name made from their content, so a
//! texture that many renders use is sent once.
use super::pod::{JOBS, WORKSPACE};
use super::ssh::Ssh;
use crate::error::{io_err, tool_err, Result};
use crate::lane::safe_name;
use crate::ledger::fingerprint;
use crate::paths::Paths;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// What a parameter string turned out to be on this PC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Local {
    /// An existing file, with a key made from its content.
    File(String),
    /// An existing folder.
    Folder,
    /// Not a path, or nothing is there.
    Other,
}

/// Something to send to the pod before the render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upload {
    /// The file or folder here.
    pub local: PathBuf,
    /// Where it goes on the pod.
    pub remote: String,
    /// Kept between jobs under its content name (files); folders are sent every time.
    pub cached: bool,
}

/// A recipe's parameters rewritten for the pod.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// The parameters Blender reads on the pod.
    pub params: Value,
    /// What has to be there first.
    pub uploads: Vec<Upload>,
    /// The local output folder the results come back to.
    pub out: Option<PathBuf>,
    /// The job's folder on the pod.
    pub dir: String,
    /// Parameters that name a file or folder on this PC that is not there. The job is refused
    /// rather than sent with a path of this PC in it.
    pub missing: Vec<PathBuf>,
}

impl Plan {
    /// Rewrites `params` for a job called `name`. `look` says what a path is on this PC.
    pub fn new(name: &str, params: &Value, look: &mut dyn FnMut(&Path) -> Local) -> Self {
        let dir = format!("{JOBS}/{}", safe_name(name));
        let mut plan = Self {
            params: params.clone(),
            uploads: Vec::new(),
            out: None,
            dir,
            missing: Vec::new(),
        };
        let mut rewritten = params.clone();
        if let Some(out) = rewritten.get_mut("out") {
            if let Some(local) = out.as_str().map(PathBuf::from) {
                *out = Value::String(format!("{}/out", plan.dir));
                plan.out = Some(local);
            }
        }
        plan.walk(&mut rewritten, true, look);
        plan.params = rewritten;
        plan
    }

    fn walk(&mut self, value: &mut Value, top: bool, look: &mut dyn FnMut(&Path) -> Local) {
        match value {
            Value::String(text) => {
                if let Some(remote) = self.place(text, look) {
                    *text = remote;
                }
            }
            Value::Array(items) => items.iter_mut().for_each(|item| self.walk(item, false, look)),
            Value::Object(map) => {
                for (key, item) in map.iter_mut() {
                    // `out` at the top level is already the pod's output folder.
                    if !(top && key == "out") {
                        self.walk(item, false, look);
                    }
                }
            }
            _ => {}
        }
    }

    /// The pod path for a parameter string, when it names something that exists here.
    fn place(&mut self, text: &str, look: &mut dyn FnMut(&Path) -> Local) -> Option<String> {
        let path = Path::new(text);
        if !path.is_absolute() {
            return None;
        }
        if let Some(known) = self.uploads.iter().find(|u| u.local == path) {
            return Some(known.remote.clone());
        }
        let base = safe_name(
            &path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );
        let (remote, cached) = match look(path) {
            Local::File(key) => (format!("{WORKSPACE}/cache/{key}_{base}"), true),
            Local::Folder => (format!("{}/in/{}_{base}", self.dir, self.uploads.len()), false),
            Local::Other => {
                if !self.missing.iter().any(|m| m == path) {
                    self.missing.push(path.to_path_buf());
                }
                return None;
            }
        };
        self.uploads.push(Upload {
            local: path.to_path_buf(),
            remote: remote.clone(),
            cached,
        });
        Some(remote)
    }

    /// A script that prints the cached uploads the pod does not have yet, one per line.
    #[must_use]
    pub fn missing_script(&self) -> String {
        let names: Vec<&str> = self
            .uploads
            .iter()
            .filter(|u| u.cached)
            .map(|u| u.remote.as_str())
            .collect();
        format!("for f in {}; do [ -s \"$f\" ] || echo \"$f\"; done", names.join(" "))
    }

    /// A script that clears the job's folder on the pod, keeping nothing from an earlier run.
    #[must_use]
    pub fn prepare_script(&self) -> String {
        format!(
            "rm -rf {dir}/out {dir}/in && mkdir -p {dir}/out {dir}/in {WORKSPACE}/cache",
            dir = self.dir
        )
    }

    /// A script that runs Blender on `job.py` with `job.json`, prints its whole log and ends
    /// with Blender's exit status.
    #[must_use]
    pub fn run_script(&self) -> String {
        format!(
            "cd {dir} || exit 97\n{WORKSPACE}/blender/blender -b --factory-startup --python-exit-code 3 -P job.py -- job.json > log.txt 2>&1\nrc=$?\ncat log.txt\nexit $rc",
            dir = self.dir
        )
    }
}

/// Looks at a path on this PC. A file's key is a fingerprint of its bytes plus its length.
fn look_on_disk(path: &Path) -> Local {
    if path.is_dir() {
        return Local::Folder;
    }
    match std::fs::read(path) {
        Ok(bytes) => Local::File(format!("{}_{}", fingerprint(&[&bytes]), bytes.len())),
        Err(_) => Local::Other,
    }
}

/// The last non-empty lines of a log, for an error message.
fn tail(log: &str, lines: usize) -> String {
    let kept: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
    kept[kept.len().saturating_sub(lines)..].join("\n")
}

fn send(ssh: &Ssh, plan: &Plan, stage: &Path) -> Result<()> {
    let missing = if plan.uploads.iter().any(|u| u.cached) {
        ssh.check(&plan.missing_script())?
    } else {
        String::new()
    };
    ssh.check(&plan.prepare_script())?;
    for file in ["job.py", "job.json"] {
        ssh.upload(&stage.join(file), &format!("{}/{file}", plan.dir))?;
    }
    for upload in &plan.uploads {
        if upload.cached {
            if !missing.lines().any(|l| l.trim() == upload.remote) {
                continue;
            }
            // Land under a temporary name, so a broken transfer is never taken for the file.
            let part = format!("{}.part", upload.remote);
            ssh.upload(&upload.local, &part)?;
            ssh.check(&format!("mv {part} {}", upload.remote))?;
        } else {
            ssh.upload(&upload.local, &upload.remote)?;
        }
    }
    Ok(())
}

/// Runs a Blender recipe on the pod. `source` is the whole script (prelude and recipe).
/// Returns Blender's log, as the local run returns its standard output.
pub fn run_blender(paths: &Paths, name: &str, source: &str, params: &Value) -> Result<String> {
    let plan = Plan::new(name, params, &mut look_on_disk);
    if !plan.missing.is_empty() {
        let list: Vec<String> = plan.missing.iter().map(|p| p.display().to_string()).collect();
        return Err(tool_err(
            "blender (cloud)",
            format!(
                "recipe {name} names input(s) that are not on this PC: {}",
                list.join(", ")
            ),
        ));
    }
    super::with_session(paths, |session| {
        let stage = paths.scratch.join("cloud").join(safe_name(name));
        std::fs::create_dir_all(&stage).map_err(|e| io_err(&stage, e))?;
        let script = stage.join("job.py");
        std::fs::write(&script, source).map_err(|e| io_err(&script, e))?;
        let args = stage.join("job.json");
        let text = serde_json::to_string_pretty(&plan.params).unwrap_or_default();
        std::fs::write(&args, text).map_err(|e| io_err(&args, e))?;

        send(&session.ssh, &plan, &stage)?;
        let ran = session.ssh.run(&plan.run_script())?;
        let log = String::from_utf8_lossy(&ran.stdout).into_owned();
        let log_path = stage.join("log.txt");
        let _ = std::fs::write(&log_path, &log);
        if !ran.status.success() || !log.contains("FORGE-OK") {
            let stderr = String::from_utf8_lossy(&ran.stderr);
            return Err(tool_err(
                "blender (cloud)",
                format!(
                    "recipe {name} failed ({}); log {}\n{}\n{}",
                    ran.status,
                    log_path.display(),
                    tail(&log, 25),
                    stderr.trim()
                ),
            ));
        }
        if let Some(out) = &plan.out {
            let files = session
                .ssh
                .fetch_dir(&format!("{}/out", plan.dir), out, &stage.join("incoming"))?;
            println!(
                "  cloud: {name} rendered on the pod; {files} file(s) back in {}",
                out.display()
            );
        }
        Ok(log)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Pretends `C:\in\weave.png` and `C:\in\car.glb` are files and `C:\in\maps` is a folder.
    fn fake(path: &Path) -> Local {
        match path.to_string_lossy().as_ref() {
            r"C:\in\weave.png" => Local::File("aaaa_10".into()),
            r"C:\in\car.glb" => Local::File("bbbb_20".into()),
            r"C:\in\maps" => Local::Folder,
            _ => Local::Other,
        }
    }

    fn plan(params: &Value) -> Plan {
        Plan::new("hero bezel", params, &mut fake)
    }

    #[test]
    fn out_becomes_the_jobs_folder_on_the_pod() {
        let p = plan(&json!({ "out": r"C:\repo\target\asset-forge\hero", "seed": 7 }));
        assert_eq!(p.dir, "/root/forge-jobs/hero_bezel");
        assert_eq!(p.params["out"], json!("/root/forge-jobs/hero_bezel/out"));
        assert_eq!(p.params["seed"], json!(7));
        assert_eq!(p.out.as_deref(), Some(Path::new(r"C:\repo\target\asset-forge\hero")));
        assert!(p.uploads.is_empty());
    }

    #[test]
    fn existing_files_anywhere_in_the_parameters_are_uploaded_once() {
        let p = plan(&json!({
            "out": r"C:\out",
            "weave": r"C:\in\weave.png",
            "materials": [{ "name": "leather", "texture": r"C:\in\weave.png" }],
            "shots": [[r"C:\in\car.glb", 1.5], [r"C:\in\car.glb", 2.5]],
        }));
        assert_eq!(p.uploads.len(), 2);
        assert_eq!(p.params["weave"], json!("/workspace/cache/aaaa_10_weave.png"));
        assert_eq!(
            p.params["materials"][0]["texture"],
            json!("/workspace/cache/aaaa_10_weave.png")
        );
        assert_eq!(p.params["shots"][1][0], json!("/workspace/cache/bbbb_20_car.glb"));
        assert_eq!(p.params["materials"][0]["name"], json!("leather"));
        assert!(p.uploads.iter().all(|u| u.cached));
    }

    #[test]
    fn folders_go_into_the_job_and_other_strings_are_left_alone() {
        let p = plan(&json!({
            "out": r"C:\out",
            "maps": r"C:\in\maps",
            "prefix": "housing",
            "missing": r"C:\in\not-there.png",
            "relative": "weave.png",
        }));
        assert_eq!(p.params["maps"], json!("/root/forge-jobs/hero_bezel/in/0_maps"));
        assert_eq!(p.params["prefix"], json!("housing"));
        assert_eq!(p.params["relative"], json!("weave.png"));
        assert_eq!(p.uploads.len(), 1);
        assert!(!p.uploads[0].cached);
        // A path of this PC with nothing behind it is listed, so the job is refused here
        // instead of failing on the pod with this PC's folder names in it.
        assert_eq!(p.missing, vec![PathBuf::from(r"C:\in\not-there.png")]);
    }

    #[test]
    fn a_nested_out_key_is_an_ordinary_value() {
        let p = plan(&json!({ "out": r"C:\out", "part": { "out": r"C:\in\weave.png" } }));
        assert_eq!(p.params["part"]["out"], json!("/workspace/cache/aaaa_10_weave.png"));
        assert_eq!(p.params["out"], json!("/root/forge-jobs/hero_bezel/out"));
    }

    #[test]
    fn the_scripts_name_only_this_jobs_folder() {
        let p = plan(&json!({ "out": r"C:\out", "weave": r"C:\in\weave.png" }));
        assert_eq!(
            p.prepare_script(),
            "rm -rf /root/forge-jobs/hero_bezel/out /root/forge-jobs/hero_bezel/in && mkdir -p /root/forge-jobs/hero_bezel/out /root/forge-jobs/hero_bezel/in /workspace/cache"
        );
        assert!(p.missing_script().contains("/workspace/cache/aaaa_10_weave.png"));
        let run = p.run_script();
        assert!(run.starts_with("cd /root/forge-jobs/hero_bezel || exit 97\n"));
        assert!(
            run.contains("/workspace/blender/blender -b --factory-startup --python-exit-code 3 -P job.py -- job.json")
        );
        assert!(run.ends_with("exit $rc"));
    }

    #[test]
    fn a_file_on_disk_is_keyed_by_its_content() {
        let dir = std::env::temp_dir().join("forge_remote_look");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.png");
        std::fs::write(&a, b"pixels").unwrap();
        let key = |p: &Path| match look_on_disk(p) {
            Local::File(k) => k,
            other => panic!("{other:?}"),
        };
        let first = key(&a);
        assert!(first.ends_with("_6"), "{first}");
        std::fs::write(&a, b"other!").unwrap();
        assert_ne!(key(&a), first);
        assert_eq!(look_on_disk(&dir), Local::Folder);
        assert_eq!(look_on_disk(&dir.join("nothing")), Local::Other);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tail_keeps_the_last_lines_that_say_something() {
        assert_eq!(tail("a\n\nb\n \nc\n", 2), "b\nc");
        assert_eq!(tail("", 5), "");
    }
}
