//! Axum HTTP listener that receives clips forwarded by Synapt from a remote
//! peer. It always starts on port 57322 when SynaptClip launches, regardless of
//! whether Synapt is currently running.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router,
};
use serde::{Deserialize, Serialize};

const LISTENER_PORT: u16 = 57322;

/// Body Synapt POSTs to `/v1/clips/receive`. Field names match the api-contract.
#[derive(Deserialize)]
struct ReceiveRequest {
    sender_peer_id: String,
    sender_name: String,
    content: String,
    content_type: String,
    #[allow(dead_code)]
    received_at: String,
}

/// Response SynaptClip returns to Synapt.
#[derive(Serialize)]
struct ReceiveResponse {
    status: &'static str,
}

/// Shared state for the listener's request handler.
#[derive(Clone)]
struct ListenerState {
    db: Arc<crate::storage::Db>,
    app: tauri::AppHandle,
}

/// Start the incoming-clip listener. Binds `127.0.0.1:57322` and serves until
/// the process exits. Logs and returns if the port cannot be bound.
pub async fn start(db: Arc<crate::storage::Db>, app: tauri::AppHandle) {
    let state = ListenerState { db, app };
    let router = Router::new()
        .route("/v1/clips/receive", post(receive_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], LISTENER_PORT));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("listener: failed to bind port {LISTENER_PORT}: {e}");
            return;
        }
    };
    tracing::info!("listener: accepting incoming clips on port {LISTENER_PORT}");

    if let Err(e) = axum::serve(listener, router).await {
        tracing::error!("listener: server error: {e}");
    }
}

/// Return the rejection reason when a request must be answered with 422, else
/// `None`. In v1 only non-empty text content is accepted.
fn rejection_reason(content_type: &str, content: &str) -> Option<&'static str> {
    if content_type != "text" {
        return Some("unsupported content_type");
    }
    if content.is_empty() {
        return Some("empty content");
    }
    None
}

async fn receive_handler(
    State(state): State<ListenerState>,
    Json(body): Json<ReceiveRequest>,
) -> impl IntoResponse {
    if let Some(reason) = rejection_reason(&body.content_type, &body.content) {
        tracing::debug!("listener: rejecting clip ({reason})");
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ReceiveResponse { status: "rejected" }),
        )
            .into_response();
    }

    let result = state
        .db
        .insert_received_clip(&body.content, &body.sender_peer_id, &body.sender_name)
        .await;

    match result {
        Ok(clip_id) => {
            tracing::info!(
                "listener: clip {clip_id} received from {} ({})",
                body.sender_name,
                body.sender_peer_id
            );
            use tauri::Emitter;
            let _ = state.app.emit(
                "clip-received",
                serde_json::json!({
                    "clip_id": clip_id,
                    "sender_name": body.sender_name,
                    "content": body.content,
                }),
            );
            (StatusCode::OK, Json(ReceiveResponse { status: "accepted" })).into_response()
        }
        Err(e) => {
            tracing::error!("listener: db insert failed: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ReceiveResponse { status: "error" }),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fresh on-disk database in a unique temp directory.
    async fn temp_db() -> crate::storage::Db {
        let dir = std::env::temp_dir().join(format!(
            "synaptclip-test-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        crate::storage::Db::new(&dir).await.expect("temp db")
    }

    #[tokio::test]
    async fn insert_received_clip_tags_source_app_synapt() {
        let db = temp_db().await;
        let id = db
            .insert_received_clip("hello from peer", "peer-1", "laptop")
            .await
            .expect("insert");
        let clip = db.get_clip_by_id(id).await.expect("query").expect("exists");
        assert_eq!(clip.source_app.as_deref(), Some("synapt"));
        assert_eq!(clip.content, "hello from peer");
        assert_eq!(clip.content_type, "text");
    }

    #[test]
    fn rejects_non_text_content_type() {
        // content_type 'image' -> 422
        assert!(rejection_reason("image", "data").is_some());
    }

    #[test]
    fn rejects_empty_content() {
        // empty content -> 422
        assert!(rejection_reason("text", "").is_some());
    }

    #[test]
    fn accepts_valid_text() {
        assert!(rejection_reason("text", "hello").is_none());
    }

    #[tokio::test]
    async fn listener_port_is_bindable() {
        // Confirms the listener port constant is correct and loopback-bindable.
        let addr = SocketAddr::from(([127, 0, 0, 1], LISTENER_PORT));
        let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
        let bound = listener.local_addr().expect("local addr");
        assert_eq!(bound.port(), 57322);
        assert!(bound.ip().is_loopback());
        // A client can establish a TCP connection to the bound port.
        let connect = tokio::net::TcpStream::connect(addr).await;
        assert!(connect.is_ok());
    }
}
