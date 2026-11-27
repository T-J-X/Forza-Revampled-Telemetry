//! The Runpod REST API v2 (`https://api.runpod.io/v2`), limited to the calls the cloud lane
//! makes: the catalog, the balance and billing history, network volumes, templates and pods.
//! Field names follow the published OpenAPI schema (`/v2/openapi.json`, read 2026-11-27).
use crate::error::{ForgeError, Result};
use crate::net;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

/// The API root.
pub const BASE: &str = "https://api.runpod.io/v2";
/// The client name sent with every request. It names the tool and nothing else.
const USER_AGENT: &str = "asset-forge/0.1";
/// How often a request is sent again after a rate limit or a failed read.
const TRIES: u32 = 4;
/// The longest wait the client accepts from a `Retry-After` header, in seconds.
const MAX_RETRY_AFTER: u64 = 60;

/// Stock levels the catalog reports, lowest first.
const LEVELS: [&str; 4] = ["NONE", "LOW", "MEDIUM", "HIGH"];

/// A stock level as a number: 0 for none (or unknown) up to 3 for high.
#[must_use]
pub fn stock_rank(level: &str) -> u8 {
    LEVELS
        .iter()
        .position(|l| l.eq_ignore_ascii_case(level))
        .and_then(|i| u8::try_from(i).ok())
        .unwrap_or(0)
}

/// A GPU type's hourly list prices in USD.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct Price {
    /// Secure cloud (Runpod's own data centres).
    #[serde(default)]
    pub secure: f64,
    /// Community cloud (hosted hardware).
    #[serde(default)]
    pub community: f64,
}

/// One data centre's stock of a GPU type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DcStock {
    /// The data centre's id.
    pub id: String,
    /// `NONE`, `LOW`, `MEDIUM` or `HIGH`.
    #[serde(default)]
    pub availability: String,
}

/// A GPU type from the catalog.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuType {
    /// The id pod creation takes.
    pub id: String,
    /// The display name.
    #[serde(default)]
    pub name: String,
    /// VRAM in GB.
    #[serde(default)]
    pub memory: u32,
    /// Offered on the secure cloud.
    #[serde(default)]
    pub secure: bool,
    /// List prices per hour.
    #[serde(default)]
    pub price: Price,
    /// Stock per data centre (only data centres that offer it).
    #[serde(default)]
    pub data_centers: Vec<DcStock>,
}

/// A data centre from the catalog.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCenter {
    /// Its id, such as `EU-RO-1`.
    pub id: String,
    /// Its display name.
    #[serde(default)]
    pub name: String,
    /// `EUROPE`, `NORTH_AMERICA` and so on.
    #[serde(default)]
    pub region: String,
    /// Network volume tiers it supports (`STANDARD`, `HIGH_PERFORMANCE`); empty means none.
    #[serde(default)]
    pub network_volume_types: Vec<String>,
}

/// A network volume.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    /// Its id.
    pub id: String,
    /// Its name (not unique on the provider's side).
    #[serde(default)]
    pub name: String,
    /// Size in GB.
    #[serde(default)]
    pub size: u32,
    /// The data centre it lives in.
    #[serde(default)]
    pub data_center: String,
}

/// One way to reach a pod over SSH.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SshEndpoint {
    /// Host name or address.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// User name.
    pub username: String,
}

/// A pod's SSH block.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PodSsh {
    /// The pod's own sshd on its published `22/tcp` port; this is the one that carries file
    /// transfers and port forwards.
    #[serde(default)]
    pub direct: Option<SshEndpoint>,
}

/// The GPU a pod got.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PodGpu {
    /// The GPU type id.
    #[serde(default)]
    pub id: String,
}

/// Live numbers for a running pod.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PodRuntime {
    /// Seconds since the container started.
    #[serde(default)]
    pub uptime: u64,
}

/// A pod.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pod {
    /// Its id.
    pub id: String,
    /// Its name.
    #[serde(default)]
    pub name: String,
    /// `PROVISIONING`, `STARTING`, `RUNNING`, `EXITED`, `ERROR` or `TERMINATED`.
    #[serde(default)]
    pub status: String,
    /// The hourly compute rate in USD.
    #[serde(default)]
    pub cost: f64,
    /// Where it runs.
    #[serde(default)]
    pub data_center_id: Option<String>,
    /// Its GPU.
    #[serde(default)]
    pub gpu: Option<PodGpu>,
    /// How to reach it.
    #[serde(default)]
    pub ssh: PodSsh,
    /// When it was created (RFC 3339).
    #[serde(default)]
    pub created_at: String,
    /// Live numbers; `None` unless it is running and was fetched by id.
    #[serde(default)]
    pub runtime: Option<PodRuntime>,
    /// The environment the pod was created with (an object of strings), when the answer has it.
    #[serde(default)]
    pub env: Value,
}

impl Pod {
    /// True while the pod is starting or running, which is also while it is billed for compute.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        matches!(self.status.as_str(), "PROVISIONING" | "STARTING" | "RUNNING")
    }

    /// The GPU type id, or `?`.
    #[must_use]
    pub fn gpu_id(&self) -> &str {
        self.gpu.as_ref().map_or("?", |g| g.id.as_str())
    }

    /// A whole-number value from the pod's environment, such as `FORGE_MAX_SECS`.
    #[must_use]
    pub fn env_number(&self, name: &str) -> Option<u64> {
        self.env.get(name)?.as_str()?.trim().parse().ok()
    }
}

/// The account balance.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// USD left.
    #[serde(default)]
    pub balance_amount: f64,
    /// `PREPAID` (spends the balance, then stops) or `POSTPAID` (invoiced, no cap).
    #[serde(default)]
    pub billing_mode: String,
}

/// What a pod-create call came back with.
#[derive(Debug, Clone, PartialEq)]
pub enum Created {
    /// The pod exists.
    Pod(Box<Pod>),
    /// This GPU and data centre had no room (or the account may not use that pool); try the
    /// next candidate. The text is the provider's explanation.
    Unavailable(String),
}

/// Everything a pod-create call needs.
#[derive(Debug, Clone, PartialEq)]
pub struct PodRequest {
    /// The pod's name.
    pub name: String,
    /// The template that carries the image, start command, ports and disk.
    pub template_id: String,
    /// The GPU type id.
    pub gpu_id: String,
    /// `SECURE` or `COMMUNITY`.
    pub cloud: String,
    /// The data centre to place it in.
    pub data_center: String,
    /// Lowest host CUDA version (`major.minor`), or empty for any.
    pub min_cuda: String,
    /// Environment variables added to the template's.
    pub env: Vec<(String, String)>,
    /// The network volume to mount at `/workspace`; `None` gives the pod a disk of its own.
    pub volume_id: Option<String>,
    /// Size of that own disk in GB, used only without a volume.
    pub own_disk_gb: u32,
}

impl PodRequest {
    /// The JSON body for `POST /v2/pods`.
    #[must_use]
    pub fn body(&self) -> Value {
        let env: serde_json::Map<String, Value> = self.env.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        let mounts = match &self.volume_id {
            Some(id) => json!({ "network": [{ "volumeId": id, "path": super::pod::WORKSPACE }] }),
            None => json!({ "persistent": { "size": self.own_disk_gb.max(10), "path": super::pod::WORKSPACE } }),
        };
        let mut gpu = json!({ "id": self.gpu_id, "count": 1 });
        if !self.min_cuda.is_empty() {
            gpu["minCudaVersion"] = json!(self.min_cuda);
        }
        json!({
            "name": self.name,
            "templateId": self.template_id,
            "cloud": self.cloud,
            "gpu": gpu,
            "dataCenterIds": [self.data_center],
            "env": env,
            "mounts": mounts,
        })
    }
}

/// An answer from the API: the status and the parsed body (`Null` when it had none).
#[derive(Debug, Clone, PartialEq)]
struct Reply {
    status: u16,
    body: Value,
}

impl Reply {
    /// The provider's explanation of an error (`detail`, plus any validation messages).
    fn detail(&self) -> String {
        let detail = self.body.get("detail").and_then(Value::as_str).unwrap_or_default();
        let errors: Vec<&str> = self
            .body
            .get("errors")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        match (detail.is_empty(), errors.is_empty()) {
            (true, true) => format!("status {}", self.status),
            (false, true) => format!("status {}: {detail}", self.status),
            (_, false) => format!("status {}: {detail} [{}]", self.status, errors.join("; ")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

/// A client for one account.
pub struct Runpod {
    agent: ureq::Agent,
    auth: String,
}

impl std::fmt::Debug for Runpod {
    // The key never appears in debug output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runpod").finish_non_exhaustive()
    }
}

impl Runpod {
    /// A client that signs every request with `key`.
    #[must_use]
    pub fn new(key: &str) -> Self {
        let agent = ureq::Agent::config_builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    // Windows' own certificate store. The bundled list does not hold the root this API chains to.
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .user_agent(USER_AGENT)
            .timeout_global(Some(Duration::from_secs(60)))
            // Statuses are read here, because the error bodies say what went wrong.
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            agent,
            auth: format!("Bearer {}", key.trim()),
        }
    }

    fn once(
        &self,
        method: Method,
        url: &str,
        body: Option<&Value>,
    ) -> std::result::Result<(Reply, Option<u64>), String> {
        let sent = match method {
            Method::Get => self.agent.get(url).header("Authorization", self.auth.as_str()).call(),
            Method::Delete => self
                .agent
                .delete(url)
                .header("Authorization", self.auth.as_str())
                .call(),
            Method::Post => self
                .agent
                .post(url)
                .header("Authorization", self.auth.as_str())
                .send_json(body.unwrap_or(&Value::Null)),
            Method::Patch => self
                .agent
                .patch(url)
                .header("Authorization", self.auth.as_str())
                .send_json(body.unwrap_or(&Value::Null)),
        };
        let mut resp = sent.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok());
        let text = resp.body_mut().read_to_string().map_err(|e| e.to_string())?;
        let body = if text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };
        Ok((Reply { status, body }, retry_after))
    }

    /// Sends a request. Rate limits are waited out. Failed reads are sent again; a failed write
    /// is not, because it may have gone through (the callers look before they create).
    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Reply> {
        let url = format!("{BASE}{path}");
        let http = |message: String| ForgeError::Http {
            url: url.clone(),
            message,
        };
        let mut last = String::new();
        for attempt in 0..TRIES {
            match self.once(method, &url, body) {
                Ok((reply, retry_after)) if reply.status == 429 => {
                    last = reply.detail();
                    std::thread::sleep(Duration::from_secs(retry_after.unwrap_or(5).clamp(1, MAX_RETRY_AFTER)));
                }
                Ok((reply, _)) if reply.status >= 500 && method == Method::Get => {
                    last = reply.detail();
                    std::thread::sleep(Duration::from_secs(2 << attempt));
                }
                Ok((reply, _)) => return Ok(reply),
                Err(message) if method == Method::Get => {
                    last = message;
                    std::thread::sleep(Duration::from_secs(2 << attempt));
                }
                Err(message) => return Err(http(message)),
            }
        }
        Err(http(format!("gave up after {TRIES} tries: {last}")))
    }

    /// Sends a request and requires a 2xx answer.
    fn ok(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let reply = self.send(method, path, body)?;
        if (200..300).contains(&reply.status) {
            return Ok(reply.body);
        }
        let hint = match reply.status {
            401 => " (the API key is missing, wrong or expired; save a new one with `asset-forge cloud key`, which asks for it)",
            403 => " (the API key lacks this permission)",
            _ => "",
        };
        Err(ForgeError::Http {
            url: format!("{BASE}{path}"),
            message: format!("{}{hint}", reply.detail()),
        })
    }

    fn list<T: for<'de> Deserialize<'de>>(&self, path: &str, field: &str) -> Result<Vec<T>> {
        let body = self.ok(Method::Get, path, None)?;
        let items = body.get(field).cloned().unwrap_or(Value::Array(Vec::new()));
        serde_json::from_value(items).map_err(|source| ForgeError::Json {
            context: format!("{BASE}{path}"),
            source,
        })
    }

    fn one<T: for<'de> Deserialize<'de>>(path: &str, body: Value) -> Result<T> {
        serde_json::from_value(body).map_err(|source| ForgeError::Json {
            context: format!("{BASE}{path}"),
            source,
        })
    }

    /// The account balance and billing mode.
    pub fn balance(&self) -> Result<Balance> {
        let path = "/billing/balance";
        Self::one(path, self.ok(Method::Get, path, None)?)
    }

    /// GPU types with prices and, per data centre, current pod stock on `cloud`.
    pub fn gpus(&self, cloud: &str) -> Result<Vec<GpuType>> {
        self.list(
            &format!(
                "/catalog/gpus?include=AVAILABILITY&product=POD&cloud={}",
                net::encode(cloud)
            ),
            "gpus",
        )
    }

    /// Every data centre.
    pub fn data_centers(&self) -> Result<Vec<DataCenter>> {
        self.list("/catalog/datacenters", "dataCenters")
    }

    /// The account's network volumes.
    pub fn volumes(&self) -> Result<Vec<Volume>> {
        self.list("/network-volumes", "networkVolumes")
    }

    /// Creates a standard network volume. This starts a monthly storage charge.
    pub fn create_volume(&self, name: &str, size_gb: u32, data_center: &str) -> Result<Volume> {
        let path = "/network-volumes";
        let body = json!({ "name": name, "size": size_gb, "dataCenter": data_center, "type": "STANDARD" });
        Self::one(path, self.ok(Method::Post, path, Some(&body))?)
    }

    /// The id of the account's template called `name`, if it has one.
    pub fn template_id(&self, name: &str) -> Result<Option<String>> {
        let body = self.ok(Method::Get, "/templates", None)?;
        Ok(body
            .get("templates")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|t| t.get("name").and_then(Value::as_str) == Some(name))
            .and_then(|t| t.get("id").and_then(Value::as_str))
            .map(str::to_owned))
    }

    /// Creates the template, or brings an existing one up to date. Returns its id.
    pub fn put_template(&self, existing: Option<&str>, template: &Value) -> Result<String> {
        let body = match existing {
            Some(id) => self.ok(
                Method::Patch,
                &format!("/templates/{}", net::encode(id)),
                Some(template),
            )?,
            None => self.ok(Method::Post, "/templates", Some(template))?,
        };
        body.get("id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| ForgeError::Http {
                url: format!("{BASE}/templates"),
                message: format!("no id in {body}"),
            })
    }

    /// The account's pods (no live SSH ports or runtime numbers; fetch one by id for those).
    pub fn pods(&self) -> Result<Vec<Pod>> {
        self.list("/pods", "pods")
    }

    /// One pod, or `None` when it no longer exists.
    pub fn pod(&self, id: &str) -> Result<Option<Pod>> {
        let path = format!("/pods/{}", net::encode(id));
        let reply = self.send(Method::Get, &path, None)?;
        match reply.status {
            404 => Ok(None),
            200..=299 => Self::one(&path, reply.body).map(Some),
            _ => Err(ForgeError::Http {
                url: format!("{BASE}{path}"),
                message: reply.detail(),
            }),
        }
    }

    /// Asks for one pod on one GPU type in one data centre. A lack of room is an answer, not an
    /// error, so the caller can try its next candidate.
    pub fn create_pod(&self, request: &PodRequest) -> Result<Created> {
        let path = "/pods";
        let reply = self.send(Method::Post, path, Some(&request.body()))?;
        let fail = |message: String| ForgeError::Http {
            url: format!("{BASE}{path}"),
            message,
        };
        match reply.status {
            200..=299 => Self::one(path, reply.body).map(|pod| Created::Pod(Box::new(pod))),
            400 | 403 => Ok(Created::Unavailable(reply.detail())),
            402 => Err(fail(format!(
                "{}: the Runpod balance is too low for one hour of this GPU; add credit on the billing page",
                reply.detail()
            ))),
            _ => Err(fail(reply.detail())),
        }
    }

    /// Deletes a pod for good. A pod that is already gone counts as deleted.
    pub fn terminate(&self, id: &str) -> Result<()> {
        let path = format!("/pods/{}", net::encode(id));
        let reply = self.send(Method::Delete, &path, None)?;
        if (200..300).contains(&reply.status) || reply.status == 404 {
            Ok(())
        } else {
            Err(ForgeError::Http {
                url: format!("{BASE}{path}"),
                message: reply.detail(),
            })
        }
    }

    fn billed(&self, path: &str) -> Result<f64> {
        let body = self.ok(Method::Get, path, None)?;
        Ok(body
            .pointer("/metadata/totals/totalAmount")
            .and_then(Value::as_f64)
            .unwrap_or(0.0))
    }

    /// USD billed for pods over `window`; with `pod`, for that pod only. The billing history
    /// runs behind the balance: an hour after the first pod it still listed nothing.
    pub fn pod_billing(&self, window: Window, pod: Option<&str>) -> Result<f64> {
        let filter = pod.map_or_else(String::new, |id| format!("&podId={}", net::encode(id)));
        self.billed(&format!("/billing/pods?{}{filter}", window.query()))
    }

    /// USD billed for network volume storage over `window`.
    pub fn volume_billing(&self, window: Window) -> Result<f64> {
        self.billed(&format!("/billing/network-volumes?{}", window.query()))
    }
}

/// The provider calls that find, start, delete and bill pods. [`Runpod`] makes them over HTTP;
/// the lane's tests use a fake, so the code that spends money is tested without spending any.
pub trait Api {
    /// The account balance and billing mode.
    fn balance(&self) -> Result<Balance>;
    /// The id of the account's template called `name`, if it has one.
    fn template_id(&self, name: &str) -> Result<Option<String>>;
    /// Creates the template, or brings an existing one up to date. Returns its id.
    fn put_template(&self, existing: Option<&str>, template: &Value) -> Result<String>;
    /// The account's pods, without their SSH ports or live numbers.
    fn pods(&self) -> Result<Vec<Pod>>;
    /// One pod, or `None` when it no longer exists.
    fn pod(&self, id: &str) -> Result<Option<Pod>>;
    /// Asks for one pod; a lack of room is an answer, not an error.
    fn create_pod(&self, request: &PodRequest) -> Result<Created>;
    /// Deletes a pod for good; one that is already gone counts as deleted.
    fn terminate(&self, id: &str) -> Result<()>;
    /// USD billed for pods over `window`; with `pod`, for that pod only.
    fn pod_billing(&self, window: Window, pod: Option<&str>) -> Result<f64>;
    /// USD billed for network volume storage over `window`.
    fn volume_billing(&self, window: Window) -> Result<f64>;
}

impl Api for Runpod {
    fn balance(&self) -> Result<Balance> {
        Runpod::balance(self)
    }

    fn template_id(&self, name: &str) -> Result<Option<String>> {
        Runpod::template_id(self, name)
    }

    fn put_template(&self, existing: Option<&str>, template: &Value) -> Result<String> {
        Runpod::put_template(self, existing, template)
    }

    fn pods(&self) -> Result<Vec<Pod>> {
        Runpod::pods(self)
    }

    fn pod(&self, id: &str) -> Result<Option<Pod>> {
        Runpod::pod(self, id)
    }

    fn create_pod(&self, request: &PodRequest) -> Result<Created> {
        Runpod::create_pod(self, request)
    }

    fn terminate(&self, id: &str) -> Result<()> {
        Runpod::terminate(self, id)
    }

    fn pod_billing(&self, window: Window, pod: Option<&str>) -> Result<f64> {
        Runpod::pod_billing(self, window, pod)
    }

    fn volume_billing(&self, window: Window) -> Result<f64> {
        Runpod::volume_billing(self, window)
    }
}

/// How far back a billing total reaches. The current day or month counts as the last one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    /// The last `n` days.
    Days(u32),
    /// The last `n` calendar months.
    Months(u32),
}

impl Window {
    fn query(self) -> String {
        match self {
            Self::Days(n) => format!("bucketSize=day&lastN={}", n.max(1)),
            Self::Months(n) => format!("bucketSize=month&lastN={}", n.max(1)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(volume: Option<&str>) -> PodRequest {
        PodRequest {
            name: "asset-forge-gpu".into(),
            template_id: "tpl1".into(),
            gpu_id: "NVIDIA GeForce RTX 4090".into(),
            cloud: "SECURE".into(),
            data_center: "EU-RO-1".into(),
            min_cuda: "12.6".into(),
            env: vec![("PUBLIC_KEY".into(), "ssh-ed25519 AAAA test".into())],
            volume_id: volume.map(str::to_owned),
            own_disk_gb: 40,
        }
    }

    #[test]
    fn pod_request_mounts_the_volume_at_the_workspace() {
        let body = request(Some("vol9")).body();
        assert_eq!(body["mounts"]["network"][0]["volumeId"], json!("vol9"));
        assert_eq!(body["mounts"]["network"][0]["path"], json!("/workspace"));
        assert!(body["mounts"].get("persistent").is_none());
        assert_eq!(
            body["gpu"],
            json!({ "id": "NVIDIA GeForce RTX 4090", "count": 1, "minCudaVersion": "12.6" })
        );
        assert_eq!(body["dataCenterIds"], json!(["EU-RO-1"]));
        assert_eq!(body["templateId"], json!("tpl1"));
        assert_eq!(body["env"]["PUBLIC_KEY"], json!("ssh-ed25519 AAAA test"));
    }

    #[test]
    fn pod_request_without_a_volume_asks_for_its_own_disk() {
        let mut req = request(None);
        req.min_cuda.clear();
        let body = req.body();
        assert_eq!(
            body["mounts"]["persistent"],
            json!({ "size": 40, "path": "/workspace" })
        );
        assert!(body["mounts"].get("network").is_none());
        assert!(body["gpu"].get("minCudaVersion").is_none());
    }

    #[test]
    fn pod_json_from_the_api_parses() {
        let text = r#"{"id":"p1","name":"asset-forge-gpu","status":"RUNNING","cost":0.69,
            "dataCenterId":"EU-RO-1","gpu":{"id":"NVIDIA GeForce RTX 4090","count":1,"vcpuCount":16,"memory":62},
            "ssh":{"proxy":null,"direct":{"host":"213.173.108.12","port":17445,"username":"root","command":"ssh ..."}},
            "createdAt":"2026-11-27T14:00:00Z","startedAt":null,"runtime":{"uptime":42},"actions":["stop"]}"#;
        let pod: Pod = serde_json::from_str(text).unwrap();
        assert!(pod.is_alive());
        assert_eq!(pod.gpu_id(), "NVIDIA GeForce RTX 4090");
        assert_eq!(pod.ssh.direct.as_ref().map(|d| d.port), Some(17445));
        assert_eq!(pod.runtime.as_ref().map(|r| r.uptime), Some(42));
        assert!((pod.cost - 0.69).abs() < 1e-9);
        assert_eq!(pod.env_number("FORGE_MAX_SECS"), None);
    }

    #[test]
    fn the_pods_environment_is_read_back() {
        // The shape of the answer for the third test pod, 2026-11-27 (key shortened).
        let text = r#"{"id":"p3","status":"RUNNING","cost":0.89,
            "env":{"FORGE_IDLE_SECS":"600","FORGE_MAX_SECS":"944","PUBLIC_KEY":"ssh-ed25519 AAAA test"}}"#;
        let pod: Pod = serde_json::from_str(text).unwrap();
        assert_eq!(pod.env_number("FORGE_MAX_SECS"), Some(944));
        assert_eq!(pod.env_number("FORGE_IDLE_SECS"), Some(600));
        assert_eq!(pod.env_number("PUBLIC_KEY"), None);
        assert_eq!(pod.env_number("MISSING"), None);
    }

    #[test]
    fn a_stopped_or_listed_pod_parses_with_its_gaps() {
        let text = r#"{"id":"p2","name":"x","status":"EXITED","cost":0.34,"dataCenterId":null,
            "ssh":{"proxy":null,"direct":null},"createdAt":"2026-11-27T14:00:00Z","runtime":null}"#;
        let pod: Pod = serde_json::from_str(text).unwrap();
        assert!(!pod.is_alive());
        assert_eq!(pod.gpu_id(), "?");
        assert_eq!(pod.ssh.direct, None);
    }

    #[test]
    fn stock_levels_rank_in_order() {
        assert!(stock_rank("HIGH") > stock_rank("MEDIUM"));
        assert!(stock_rank("MEDIUM") > stock_rank("LOW"));
        assert!(stock_rank("LOW") > stock_rank("NONE"));
        assert_eq!(stock_rank("whatever"), 0);
    }

    #[test]
    fn error_detail_includes_the_validation_messages() {
        let reply = Reply {
            status: 422,
            body: json!({ "title": "Unprocessable", "status": 422, "detail": "invalid body", "errors": ["$.gpu: required"] }),
        };
        assert_eq!(reply.detail(), "status 422: invalid body [$.gpu: required]");
        let bare = Reply {
            status: 502,
            body: Value::Null,
        };
        assert_eq!(bare.detail(), "status 502");
    }

    #[test]
    fn a_billing_window_is_a_count_of_days_or_months() {
        assert_eq!(Window::Days(3).query(), "bucketSize=day&lastN=3");
        assert_eq!(Window::Months(12).query(), "bucketSize=month&lastN=12");
        // The API refuses a count of 0.
        assert_eq!(Window::Days(0).query(), "bucketSize=day&lastN=1");
    }

    #[test]
    fn debug_output_hides_the_key() {
        let text = format!("{:?}", Runpod::new("rpa_SECRET"));
        assert!(!text.contains("SECRET"), "{text}");
    }
}
