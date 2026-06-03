use crate::state::{now_hms, AppState, ConsoleLine};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

/// GET /ws/servers/:name
///
/// Backs the entire Monitor tab over one socket:
///   • server → client:  {type:"console", ...ConsoleLine}
///                        {type:"metrics", cpu, ram, tps, players}
///   • client → server:  {type:"command", cmd:"list"}   (RCON command input)
pub async fn monitor(
    ws: WebSocketUpgrade,
    State(st): State<AppState>,
    Path(name): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle(socket, st, name))
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMsg {
    Command { cmd: String },
}

async fn handle(mut socket: WebSocket, st: AppState, name: String) {
    // Subscribe to the live console + grab the metrics handle, if running.
    let (mut console_rx, metrics) = {
        let running = st.running.lock().await;
        match running.get(&name) {
            Some(r) => (r.console.subscribe(), r.metrics.clone()),
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
            // console line → client
            line = console_rx.recv() => {
                match line {
                    Ok(l) => {
                        let frame = console_frame(&l);
                        if socket.send(Message::Text(frame)).await.is_err() { break; }
                    }
                    Err(_) => continue, // lagged; skip
                }
            }

            // periodic metrics push → client
            _ = ticker.tick() => {
                let m = metrics.read().await.clone();
                let frame = json!({
                    "type": "metrics",
                    "cpu": m.cpu, "ram": m.ram, "ram_mb": m.ram_mb,
                    "tps": m.tps, "players": m.players,
                }).to_string();
                if socket.send(Message::Text(frame)).await.is_err() { break; }
            }

            // command from client (RCON input box)
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

/// Open a one-shot RCON connection, send the command, echo command + reply back.
async fn run_command(st: &AppState, name: &str, cmd: &str, socket: &mut WebSocket) {
    // echo the typed command first
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
