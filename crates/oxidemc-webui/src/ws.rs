use crate::state::{now_hms, AppState, ConsoleLine, InstallMsg};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio::sync::broadcast;

// ── Monitor WebSocket  GET /ws/servers/:name ─────────────────────────────────
//
// Backs the entire Monitor tab:
//   server → client:  {type:"console", ...ConsoleLine}
//                      {type:"metrics", cpu, ram, tps, tps_available, players}
//                      {type:"status",  status:"stopped"}  (when server exits)
//   client → server:  {type:"command", cmd:"list"}

pub async fn monitor(
    ws: WebSocketUpgrade,
    State(st): State<AppState>,
    Path(name): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_monitor(socket, st, name))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMsg {
    Command { cmd: String },
}

async fn handle_monitor(mut socket: WebSocket, st: AppState, name: String) {
    let (mut console_rx, metrics, start_time) = {
        let running = st.running.lock().await;
        match running.get(&name) {
            Some(r) => (r.console.subscribe(), r.metrics.clone(), r.start_time),
            None => {
                let _ = socket
                    .send(Message::Text(
                        json!({ "type": "status", "status": "stopped" }).to_string(),
                    ))
                    .await;
                return;
            }
        }
    };

    let mut ticker = tokio::time::interval(Duration::from_millis(1500));

    loop {
        tokio::select! {
            line = console_rx.recv() => {
                match line {
                    Ok(l) => {
                        if socket.send(Message::Text(console_frame(&l))).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => {
                        // Server stopped — notify the client then close cleanly.
                        let _ = socket.send(Message::Text(
                            json!({ "type": "status", "status": "stopped" }).to_string()
                        )).await;
                        break;
                    }
                }
            }

            _ = ticker.tick() => {
                let m = metrics.read().await.clone();
                let frame = json!({
                    "type": "metrics",
                    "cpu": m.cpu, "ram": m.ram, "ram_mb": m.ram_mb,
                    "tps": m.tps, "tps_available": m.tps_available,
                    "players": m.players,
                    "uptime_secs": start_time.elapsed().as_secs(),
                }).to_string();
                if socket.send(Message::Text(frame)).await.is_err() { break; }
            }

            inbound = socket.recv() => {
                match inbound {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(ClientMsg::Command { cmd }) = serde_json::from_str::<ClientMsg>(&txt) {
                            run_command(&st, &name, &cmd, &mut socket).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

async fn run_command(st: &AppState, name: &str, cmd: &str, socket: &mut WebSocket) {
    let echo = console_frame(&ConsoleLine {
        t: now_hms(), level: "CMD".into(), source: "RCON".into(), msg: format!("/{cmd}"),
    });
    let _ = socket.send(Message::Text(echo)).await;

    let state = match st.load_state(name) {
        Ok(s) => s,
        Err(e) => return send_err(socket, &e.to_string()).await,
    };
    match st.connect_rcon(&state).await {
        Ok(mut rcon) => match rcon.send_command(cmd).await {
            Ok(reply) if !reply.is_empty() => {
                let frame = console_frame(&ConsoleLine {
                    t: now_hms(), level: "INFO".into(), source: "RCON".into(), msg: reply,
                });
                let _ = socket.send(Message::Text(frame)).await;
            }
            Ok(_) => {}
            Err(e) => send_err(socket, &e.to_string()).await,
        },
        Err(e) => send_err(socket, &e.to_string()).await,
    }
}

fn console_frame(l: &ConsoleLine) -> String {
    json!({
        "type": "console",
        "t": l.t, "level": l.level, "source": l.source, "msg": l.msg,
    })
    .to_string()
}

async fn send_err(socket: &mut WebSocket, msg: &str) {
    let frame = json!({ "type": "console", "t": now_hms(), "level": "ERROR", "source": "RCON", "msg": msg }).to_string();
    let _ = socket.send(Message::Text(frame)).await;
}

// ── Install progress WebSocket  GET /ws/install/:job ────────────────────────
//
// Streams InstallMsg frames until Done or Error, then closes.

pub async fn install_progress(
    ws: WebSocketUpgrade,
    State(st): State<AppState>,
    Path(job): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_install(socket, st, job))
}

async fn handle_install(mut socket: WebSocket, st: AppState, job: String) {
    let mut rx = {
        let guard = st.install_jobs.lock().await;
        match guard.get(&job) {
            Some(tx) => tx.subscribe(),
            None => {
                let _ = socket.send(Message::Text(
                    json!({ "type": "error", "msg": "job not found" }).to_string(),
                )).await;
                return;
            }
        }
    };

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let terminal = matches!(msg, InstallMsg::Done | InstallMsg::Error { .. });
                let frame = serde_json::to_string(&msg).unwrap_or_default();
                if socket.send(Message::Text(frame)).await.is_err() { break; }
                if terminal { break; }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}
