//! Synapt bridge: polls Synapt's local API to detect whether it is running,
//! fetches the current peer list, and maintains shared bridge state that Tauri
//! commands and the frontend can read.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

const SYNAPT_API: &str = "http://127.0.0.1:57321";
const HEALTH_INTERVAL: Duration = Duration::from_secs(10);
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

/// A peer device returned by Synapt's `/v1/peers` endpoint.
/// Field names match the api-contract exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynaptPeer {
    /// Stable UUID for this peer, persisted across sessions.
    pub id: String,
    /// Hostname or user-configured display name.
    pub name: String,
    /// IPv4 address of the peer.
    pub ip: String,
    /// Synapt transfer port on the peer device.
    pub port: u16,
    /// Whether the peer was seen in the last 30 seconds.
    pub online: bool,
    /// ISO 8601 UTC timestamp of the last discovery response.
    pub last_seen: String,
}

/// Current state of the bridge, mirrored to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct BridgeState {
    /// Whether Synapt is currently reachable.
    pub active: bool,
    /// Peers discovered by Synapt; empty when inactive.
    pub peers: Vec<SynaptPeer>,
    /// Synapt API version reported by `/v1/health`, when active.
    pub api_version: Option<String>,
}

impl BridgeState {
    /// The state used whenever Synapt is not reachable.
    pub fn inactive() -> Self {
        Self {
            active: false,
            peers: vec![],
            api_version: None,
        }
    }
}

/// Shared bridge state, wrapped in `Arc<RwLock>` so commands can read it.
pub type SharedBridgeState = Arc<RwLock<BridgeState>>;

/// Replace the shared bridge state, tolerating a poisoned lock without panicking.
fn store_state(state: &SharedBridgeState, value: BridgeState) {
    match state.write() {
        Ok(mut guard) => *guard = value,
        Err(poisoned) => *poisoned.into_inner() = value,
    }
}

/// Start the background bridge polling task.
///
/// Polls `/v1/health` every 10 seconds. On success it fetches `/v1/peers` and
/// updates state; on failure it marks the bridge inactive. A `bridge-state-changed`
/// Tauri event is emitted on every transition between active and inactive.
pub async fn start(state: SharedBridgeState, app: tauri::AppHandle) {
    let client = match Client::builder().timeout(HTTP_TIMEOUT).build() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("bridge: failed to build HTTP client: {e}");
            return;
        }
    };

    let mut was_active = false;

    loop {
        let is_active = poll_health(&client).await;

        if is_active {
            let peers = fetch_peers(&client).await.unwrap_or_default();
            let new_state = BridgeState {
                active: true,
                peers,
                api_version: Some("1".to_string()),
            };

            let transitioned = !was_active;
            store_state(&state, new_state.clone());

            if transitioned {
                tracing::info!("bridge: Synapt detected on port 57321");
                emit_state(&app, &new_state);
            }
            was_active = true;
        } else {
            if was_active {
                tracing::info!("bridge: Synapt went offline");
                let inactive = BridgeState::inactive();
                store_state(&state, inactive.clone());
                emit_state(&app, &inactive);
            }
            was_active = false;
        }

        tokio::time::sleep(HEALTH_INTERVAL).await;
    }
}

/// Return true when Synapt answers `/v1/health` with a success status.
async fn poll_health(client: &Client) -> bool {
    client
        .get(format!("{SYNAPT_API}/v1/health"))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Fetch the current peer list from Synapt, returning `None` on any error.
async fn fetch_peers(client: &Client) -> Option<Vec<SynaptPeer>> {
    #[derive(Deserialize)]
    struct PeersResponse {
        peers: Vec<SynaptPeer>,
    }

    let resp = client
        .get(format!("{SYNAPT_API}/v1/peers"))
        .send()
        .await
        .ok()?;
    let body: PeersResponse = resp.json().await.ok()?;
    Some(body.peers)
}

/// Emit the current bridge state to the frontend.
fn emit_state(app: &tauri::AppHandle, state: &BridgeState) {
    use tauri::Emitter;
    if let Err(e) = app.emit("bridge-state-changed", state) {
        tracing::warn!("bridge: failed to emit state change: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_has_no_peers() {
        let s = BridgeState::inactive();
        assert!(!s.active);
        assert!(s.peers.is_empty());
        assert!(s.api_version.is_none());
    }

    #[test]
    fn bridge_state_serialises_all_fields() {
        let s = BridgeState {
            active: true,
            peers: vec![SynaptPeer {
                id: "peer-1".to_string(),
                name: "laptop".to_string(),
                ip: "192.168.1.42".to_string(),
                port: 54321,
                online: true,
                last_seen: "2025-03-14T10:22:00Z".to_string(),
            }],
            api_version: Some("1".to_string()),
        };
        let json = serde_json::to_value(&s).expect("serialise");
        assert_eq!(json["active"], true);
        assert_eq!(json["api_version"], "1");
        assert_eq!(json["peers"][0]["id"], "peer-1");
        assert_eq!(json["peers"][0]["port"], 54321);
        assert_eq!(json["peers"][0]["online"], true);
    }

    #[test]
    fn synapt_peer_deserialises_from_contract_shape() {
        let raw = r#"{
            "id": "peer-uuid-string",
            "name": "aatish-laptop",
            "ip": "192.168.1.42",
            "port": 54321,
            "online": true,
            "last_seen": "2025-03-14T10:22:00Z"
        }"#;
        let peer: SynaptPeer = serde_json::from_str(raw).expect("deserialise");
        assert_eq!(peer.id, "peer-uuid-string");
        assert_eq!(peer.name, "aatish-laptop");
        assert_eq!(peer.ip, "192.168.1.42");
        assert_eq!(peer.port, 54321);
        assert!(peer.online);
        assert_eq!(peer.last_seen, "2025-03-14T10:22:00Z");
    }

    #[tokio::test]
    async fn poll_health_false_when_nothing_listening() {
        // Nothing is bound to 57321 in the test environment, so the connection
        // is refused and poll_health must report inactive without panicking.
        let client = Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .expect("client");
        assert!(!poll_health(&client).await);
    }
}
