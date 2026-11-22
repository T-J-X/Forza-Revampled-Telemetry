//! HTTP: one agent (native TLS through the OS, no C crypto), a project user agent and size limits.
use crate::error::{ForgeError, Result};
use std::time::Duration;
use ureq::tls::{TlsConfig, TlsProvider};
use ureq::Agent;

/// The user agent Wikimedia and GitHub ask automated clients to send.
pub const USER_AGENT: &str = "FH6-asset-forge/0.1 (+https://github.com/T-J-X/Forza-Revampled-Telemetry)";
/// Largest body read into memory.
const MAX_BODY: u64 = 256 * 1024 * 1024;

/// Builds the shared agent.
pub fn agent() -> Agent {
    Agent::config_builder()
        .tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build())
        .user_agent(USER_AGENT)
        .timeout_global(Some(Duration::from_secs(900)))
        .http_status_as_error(true)
        .build()
        .into()
}

fn http_err(url: &str, e: impl std::fmt::Display) -> ForgeError {
    ForgeError::Http {
        url: url.to_owned(),
        message: e.to_string(),
    }
}

/// GETs `url` into memory.
pub fn get_bytes(agent: &Agent, url: &str) -> Result<Vec<u8>> {
    let mut resp = agent.get(url).call().map_err(|e| http_err(url, e))?;
    resp.body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_vec()
        .map_err(|e| http_err(url, e))
}

/// GETs `url` as JSON.
pub fn get_json(agent: &Agent, url: &str) -> Result<serde_json::Value> {
    let bytes = get_bytes(agent, url)?;
    serde_json::from_slice(&bytes).map_err(|source| ForgeError::Json {
        context: url.to_owned(),
        source,
    })
}

/// POSTs JSON and parses the JSON answer.
pub fn post_json(agent: &Agent, url: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
    let mut resp = agent.post(url).send_json(body).map_err(|e| http_err(url, e))?;
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_vec()
        .map_err(|e| http_err(url, e))?;
    serde_json::from_slice(&bytes).map_err(|source| ForgeError::Json {
        context: url.to_owned(),
        source,
    })
}

/// Percent-encodes a query value.
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn encodes_query_values() {
        assert_eq!(super::encode("a b/c"), "a%20b%2Fc");
    }
}
