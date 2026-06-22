use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use mestier_core::infrastructure::realtime::wire::GatewayEvent;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use tracing::{error, info, warn};

use handlers::AppState;

use crate::paths::GatewayPath;

const IDENTIFY_TIMEOUT_SECS: u64 = 10;
const HEARTBEAT_INTERVAL_SECS: u64 = 30;
const HEARTBEAT_ACK_TIMEOUT_SECS: u64 = 10;

// ── Client → Server messages ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientMessage {
    Identify { token: String },
    HeartbeatAck,
}

// ── Server → Client messages ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ServerMessage {
    Ready {
        user_id: String,
    },
    Dispatch {
        #[serde(rename = "t")]
        event_type: String,
        #[serde(rename = "d")]
        data: serde_json::Value,
    },
    Heartbeat,
    Close {
        reason: String,
    },
}

// ── Route handler ─────────────────────────────────────────────────────────────

/// WebSocket gateway endpoint.  Authentication is performed via the `identify`
/// control message — this route must NOT be placed behind `auth_middleware`.
pub async fn handler(
    _path: GatewayPath,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// ── Socket lifecycle ──────────────────────────────────────────────────────────

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // ── Phase 1: identify within 10 s ────────────────────────────────────────
    let token = match timeout(
        Duration::from_secs(IDENTIFY_TIMEOUT_SECS),
        wait_for_identify(&mut socket),
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            warn!("gateway: identify parse error: {e}");
            let _ = send_close(&mut socket, "identify failed").await;
            return;
        }
        Err(_) => {
            warn!("gateway: identify timeout");
            let _ = send_close(&mut socket, "identify timeout").await;
            return;
        }
    };

    // ── Phase 2: validate token ───────────────────────────────────────────────
    let identity = match state.auth.get_identity(&token).await {
        Ok(id) => id,
        Err(e) => {
            warn!("gateway: invalid token: {e}");
            let _ = send_close(&mut socket, "unauthorized").await;
            return;
        }
    };

    // ── Phase 3: resolve user ─────────────────────────────────────────────────
    let user = match state.usecase.find_user_by_sub(identity.id()).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            warn!("gateway: user not found for sub {}", identity.id());
            let _ = send_close(&mut socket, "user not found").await;
            return;
        }
        Err(e) => {
            error!("gateway: find_user_by_sub failed: {e}");
            let _ = send_close(&mut socket, "internal error").await;
            return;
        }
    };

    // ── Phase 4: resolve org memberships ─────────────────────────────────────
    let orgs = match state
        .usecase
        .list_organizations_for_user(identity.id())
        .await
    {
        Ok(o) => o,
        Err(e) => {
            error!("gateway: list_organizations_for_user failed: {e}");
            let _ = send_close(&mut socket, "internal error").await;
            return;
        }
    };

    // ── Phase 5: subscribe to EventHub — one Receiver per org ────────────────
    // Merge all org streams into a single mpsc channel so the dispatch loop
    // only ever blocks on one receiver.  Each spawned forwarder task drops its
    // broadcast::Receiver when tx is closed (connection ends), unsubscribing
    // automatically.
    let (event_tx, mut event_rx) = mpsc::channel::<Result<GatewayEvent, RecvError>>(64);

    for org in &orgs {
        let mut rx = state.events.subscribe(org.id);
        let tx = event_tx.clone();
        tokio::spawn(async move {
            loop {
                let item = rx.recv().await;
                let is_closed = matches!(item, Err(RecvError::Closed));
                if tx.send(item).await.is_err() || is_closed {
                    break;
                }
            }
        });
    }
    // Drop the original sender — only the forwarder clones keep it alive.
    drop(event_tx);

    let user_id_str = user.id.to_string();

    // ── Phase 6: send ready ───────────────────────────────────────────────────
    if send_json(
        &mut socket,
        &ServerMessage::Ready {
            user_id: user_id_str.clone(),
        },
    )
    .await
    .is_err()
    {
        return;
    }

    info!(
        "gateway: user {} connected, {} org(s)",
        user_id_str,
        orgs.len()
    );

    // ── Phase 7: dispatch loop with heartbeat ─────────────────────────────────
    let mut heartbeat_interval =
        tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
    // Consume the immediate first tick so we don't send a heartbeat right away.
    heartbeat_interval.tick().await;

    loop {
        tokio::select! {
            // Outbound: gateway events forwarded from broadcast receivers
            maybe_event = event_rx.recv() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        let full = serde_json::to_value(&event)
                            .unwrap_or(serde_json::Value::Null);
                        let event_type = full
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("UNKNOWN")
                            .to_owned();
                        let data = full
                            .get("data")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        let msg = ServerMessage::Dispatch { event_type, data };
                        if send_json(&mut socket, &msg).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(RecvError::Lagged(n))) => {
                        warn!(
                            "gateway: user {user_id_str} lagged {n} event(s) — \
                             client must reconcile via REST"
                        );
                        // Best-effort: skip and continue rather than kill the connection.
                    }
                    Some(Err(RecvError::Closed)) | None => {
                        // All hub senders dropped — hub shutdown.
                        break;
                    }
                }
            }

            // Heartbeat tick
            _ = heartbeat_interval.tick() => {
                if send_json(&mut socket, &ServerMessage::Heartbeat).await.is_err() {
                    break;
                }
                let ack = timeout(
                    Duration::from_secs(HEARTBEAT_ACK_TIMEOUT_SECS),
                    wait_for_ack(&mut socket),
                )
                .await;
                if ack.is_err() || ack.unwrap().is_err() {
                    warn!("gateway: user {user_id_str} missed heartbeat ack — closing");
                    break;
                }
            }

            // Inbound: client frames (only Close or errors expected here;
            // HeartbeatAck is consumed inside wait_for_ack above)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {} // ping/pong auto-handled by axum; text frames ignored
                    Some(Err(e)) => {
                        warn!("gateway: ws receive error for user {user_id_str}: {e}");
                        break;
                    }
                }
            }
        }
    }

    info!("gateway: user {} disconnected", user_id_str);
    // Presence is NOT changed on disconnect (spec §4.6 — self-declared).
    // event_rx drops here; all forwarder tasks detect the closed mpsc sender
    // and exit, dropping their broadcast::Receiver in turn.
}

// ── Low-level send helpers ────────────────────────────────────────────────────

async fn send_json(socket: &mut WebSocket, msg: &impl Serialize) -> Result<(), ()> {
    let json = serde_json::to_string(msg).map_err(|_| ())?;
    socket
        .send(Message::Text(json.into()))
        .await
        .map_err(|_| ())
}

async fn send_close(socket: &mut WebSocket, reason: &str) -> Result<(), ()> {
    send_json(
        socket,
        &ServerMessage::Close {
            reason: reason.to_owned(),
        },
    )
    .await
}

async fn wait_for_identify(socket: &mut WebSocket) -> Result<String, String> {
    while let Some(msg) = socket.recv().await {
        let msg = msg.map_err(|e| e.to_string())?;
        if let Message::Text(text) = msg {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Identify { token }) => return Ok(token),
                Ok(_) => return Err("expected identify op".into()),
                Err(e) => return Err(e.to_string()),
            }
        }
    }
    Err("connection closed before identify".into())
}

async fn wait_for_ack(socket: &mut WebSocket) -> Result<(), String> {
    while let Some(msg) = socket.recv().await {
        let msg = msg.map_err(|e| e.to_string())?;
        if let Message::Text(text) = msg
            && let Ok(ClientMessage::HeartbeatAck) = serde_json::from_str::<ClientMessage>(&text)
        {
            return Ok(());
        }
    }
    Err("connection closed while waiting for ack".into())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identify_op_deserializes() {
        let msg = r#"{"op":"identify","token":"tok123"}"#;
        let parsed: ClientMessage = serde_json::from_str(msg).unwrap();
        assert!(matches!(parsed, ClientMessage::Identify { token } if token == "tok123"));
    }

    #[test]
    fn heartbeat_ack_deserializes() {
        let msg = r#"{"op":"heartbeat_ack"}"#;
        let parsed: ClientMessage = serde_json::from_str(msg).unwrap();
        assert!(matches!(parsed, ClientMessage::HeartbeatAck));
    }

    #[test]
    fn ready_server_message_serializes() {
        let msg = ServerMessage::Ready {
            user_id: "00000000-0000-0000-0000-000000000000".to_owned(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"op\":\"ready\""));
        assert!(json.contains("\"user_id\""));
    }

    #[test]
    fn heartbeat_server_message_serializes() {
        let json = serde_json::to_string(&ServerMessage::Heartbeat).unwrap();
        assert!(json.contains("\"op\":\"heartbeat\""));
    }

    #[test]
    fn close_server_message_serializes() {
        let msg = ServerMessage::Close {
            reason: "timeout".to_owned(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"op\":\"close\""));
        assert!(json.contains("timeout"));
    }
}
