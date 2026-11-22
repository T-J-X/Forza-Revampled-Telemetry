//! ComfyUI over HTTP: start the portable server if it is not up, queue an API-format workflow
//! from `tools/asset-forge/workflows/*.json` with parameters patched in, poll `/history`, and
//! download the outputs through `/view`.
use crate::error::{io_err, tool_err, ForgeError, Result};
use crate::net;
use crate::paths::Paths;
use serde_json::Value;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A connected ComfyUI.
pub struct Comfy {
    base: String,
    agent: ureq::Agent,
}

impl Comfy {
    /// Connects, starting the portable server when needed (waits up to 4 minutes).
    pub fn connect(paths: &Paths) -> Result<Self> {
        let agent = local_agent();
        let base = paths.comfy_url.trim_end_matches('/').to_owned();
        let me = Self { base, agent };
        if me.alive() {
            return Ok(me);
        }
        let py = paths.comfy_dir.join("python_embeded").join("python.exe");
        let main = paths.comfy_dir.join("ComfyUI").join("main.py");
        if !py.is_file() || !main.is_file() {
            return Err(tool_err("comfyui", format!("portable install not found in {}", paths.comfy_dir.display())));
        }
        let log_path = paths.scratch.join("comfyui.log");
        if let Some(p) = log_path.parent() {
            std::fs::create_dir_all(p).map_err(|e| io_err(p, e))?;
        }
        let log = std::fs::File::create(&log_path).map_err(|e| io_err(&log_path, e))?;
        let err = log.try_clone().map_err(|e| io_err(&log_path, e))?;
        Command::new(&py)
            .arg("-s")
            .arg(&main)
            .arg("--windows-standalone-build")
            .arg("--listen")
            .arg("127.0.0.1")
            .arg("--port")
            .arg("8188")
            .current_dir(&paths.comfy_dir)
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(err)
            .spawn()
            .map_err(|e| tool_err("comfyui", e.to_string()))?;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(240) {
            if me.alive() {
                return Ok(me);
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        Err(tool_err("comfyui", format!("server did not come up; see {}", log_path.display())))
    }

    /// True when `/system_stats` answers.
    pub fn alive(&self) -> bool {
        net::get_bytes(&self.agent, &format!("{}/system_stats", self.base)).is_ok()
    }

    /// `/system_stats` as JSON.
    pub fn stats(&self) -> Result<Value> {
        net::get_json(&self.agent, &format!("{}/system_stats", self.base))
    }

    /// Queues `workflow` and waits for it; returns every output image's bytes.
    pub fn run(&self, workflow: &Value, timeout: Duration) -> Result<Vec<Vec<u8>>> {
        let body = serde_json::json!({ "prompt": workflow, "client_id": "asset-forge" });
        let resp = net::post_json(&self.agent, &format!("{}/prompt", self.base), &body)?;
        let id = resp
            .get("prompt_id")
            .and_then(Value::as_str)
            .ok_or_else(|| tool_err("comfyui", format!("no prompt_id in {resp}")))?
            .to_owned();
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err(tool_err("comfyui", format!("prompt {id} timed out")));
            }
            std::thread::sleep(Duration::from_millis(1500));
            let hist = net::get_json(&self.agent, &format!("{}/history/{id}", self.base))?;
            let Some(entry) = hist.get(&id) else { continue };
            if let Some(status) = entry.get("status") {
                if status.get("status_str").and_then(Value::as_str) == Some("error") {
                    return Err(tool_err("comfyui", format!("prompt {id} failed: {status}")));
                }
            }
            let Some(outputs) = entry.get("outputs").and_then(Value::as_object) else { continue };
            let mut images = Vec::new();
            for node in outputs.values() {
                for img in node.get("images").and_then(Value::as_array).into_iter().flatten() {
                    let filename = img.get("filename").and_then(Value::as_str).unwrap_or_default();
                    let sub = img.get("subfolder").and_then(Value::as_str).unwrap_or_default();
                    let kind = img.get("type").and_then(Value::as_str).unwrap_or("output");
                    let url = format!(
                        "{}/view?filename={}&subfolder={}&type={}",
                        self.base,
                        net::encode(filename),
                        net::encode(sub),
                        net::encode(kind)
                    );
                    images.push(net::get_bytes(&self.agent, &url)?);
                }
            }
            if images.is_empty() {
                return Err(ForgeError::Tool {
                    tool: "comfyui".into(),
                    message: format!("prompt {id} finished without images"),
                });
            }
            return Ok(images);
        }
    }
}

fn local_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .http_status_as_error(true)
        .build()
        .into()
}

/// Loads a workflow and replaces every string value `"{{key}}"` with the parameter (keeping
/// numbers numeric) and every `{{key}}` inside longer strings with its text.
pub fn patch(workflow: &str, params: &serde_json::Map<String, Value>) -> Result<Value> {
    let mut v: Value = serde_json::from_str(workflow).map_err(|source| ForgeError::Json {
        context: "workflow".into(),
        source,
    })?;
    fn walk(v: &mut Value, p: &serde_json::Map<String, Value>) {
        match v {
            Value::String(s) => {
                if let Some(key) = s.strip_prefix("{{").and_then(|r| r.strip_suffix("}}")) {
                    if let Some(rep) = p.get(key) {
                        *v = rep.clone();
                        return;
                    }
                }
                for (k, rep) in p {
                    let token = format!("{{{{{k}}}}}");
                    if s.contains(&token) {
                        let text = match rep {
                            Value::String(t) => t.clone(),
                            other => other.to_string(),
                        };
                        *s = s.replace(&token, &text);
                    }
                }
            }
            Value::Array(a) => a.iter_mut().for_each(|x| walk(x, p)),
            Value::Object(o) => o.values_mut().for_each(|x| walk(x, p)),
            _ => {}
        }
    }
    walk(&mut v, params);
    Ok(v)
}

#[cfg(test)]
mod tests {
    #[test]
    fn patch_keeps_numbers_numeric() {
        let wf = r#"{"3":{"inputs":{"seed":"{{seed}}","text":"a {{subject}} plate"}}}"#;
        let mut p = serde_json::Map::new();
        p.insert("seed".into(), serde_json::json!(42));
        p.insert("subject".into(), serde_json::json!("plasma"));
        let v = super::patch(wf, &p).unwrap_or_default();
        assert_eq!(v["3"]["inputs"]["seed"], serde_json::json!(42));
        assert_eq!(v["3"]["inputs"]["text"], serde_json::json!("a plasma plate"));
    }
}
