use crate::error::{ApiError, ApiResult};
use oxidemc_core::rcon::RconClient;
use oxidemc_core::schema::ServerState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::{broadcast, Mutex, RwLock};

/// A single console line pushed to every WebSocket subscriber of a server.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsoleLine {
    pub t: String,      // "HH:MM:SS"
    pub level: String,  // INFO | WARN | ERROR | CMD
    pub source: String, // Server | RCON | Chat
    pub msg: String,
}

/// Live sample for the Monitor gauges + TPS chart.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Metrics {
    pub cpu: f32,           // % of one core (or normalized)
    pub ram: f32,           // % of -Xmx
    pub ram_mb: u64,
    pub tps: f32,           // parsed from `tps`/forge, else estimated
    pub players: Vec<String>,
}

/// Everything we track for a server that is currently running.
pub struct RunningServer {
    pub child: Child,
    /// fan-out of console lines to all connected WebSockets
    pub console: broadcast::Sender<ConsoleLine>,
    /// latest metrics sample (updated by the poller task)
    pub metrics: Arc<RwLock<Metrics>>,
    /// player list maintained by parsing console join/leave events
    pub players: Arc<Mutex<Vec<String>>>,
    pub start_time: std::time::Instant,
    #[allow(dead_code)]
    pub pid: u32,
}

#[derive(Clone)]
pub struct AppState {
    /// parent dir that holds one sub-folder (with oxide.json) per server
    pub servers_dir: PathBuf,
    pub java_path: String,
    /// name -> running handle
    pub running: Arc<Mutex<HashMap<String, RunningServer>>>,
}

impl AppState {
    /// Build state from the global manage.json (servers_directory, java_path).
    pub fn from_manage() -> ApiResult<Self> {
        let manage = oxidemc_core::config::load_manage()?;
        Ok(Self {
            servers_dir: expand(&manage.servers_directory.default),
            java_path: manage.java_path.default,
            running: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn server_dir(&self, name: &str) -> PathBuf {
        self.servers_dir.join(name)
    }

    /// Load a server's oxide.json, or 404.
    pub fn load_state(&self, name: &str) -> ApiResult<ServerState> {
        let dir = self.server_dir(name);
        if !dir.join("oxide.json").exists() {
            return Err(ApiError::NotFound(name.to_string()));
        }
        Ok(oxidemc_core::config::load_server_state(&dir)?)
    }

    /// Enumerate every server folder that contains an oxide.json.
    pub fn list_states(&self) -> ApiResult<Vec<ServerState>> {
        let mut out = Vec::new();
        if !self.servers_dir.exists() {
            return Ok(out);
        }
        for entry in std::fs::read_dir(&self.servers_dir)? {
            let dir = entry?.path();
            if dir.join("oxide.json").exists() {
                if let Ok(state) = oxidemc_core::config::load_server_state(&dir) {
                    out.push(state);
                }
            }
        }
        Ok(out)
    }

    pub async fn is_running(&self, name: &str) -> bool {
        self.running.lock().await.contains_key(name)
    }

    /// Launch a server with piped stdout so we can stream the console.
    ///
    /// NOTE: `oxidemc_core::server::launch` inherits stdio, which is right for
    /// the TUI but means we can't capture output. Here we mirror its command
    /// construction but pipe stdout/stderr. Consider adding a `launch_piped`
    /// (or a `Stdio` parameter) to core so this logic lives in one place.
    pub async fn start(&self, name: &str) -> ApiResult<()> {
        if self.is_running(name).await {
            return Err(ApiError::AlreadyRunning);
        }
        let state = self.load_state(name)?;
        let dir = self.server_dir(name);

        oxidemc_core::server::accept_eula(&dir)?;

        let jar = oxidemc_core::downloader::jar_name(&state.server_type, &state.minecraft_version);
        let java_bin = if self.java_path.is_empty() { "java" } else { &self.java_path };

        let mut cmd = tokio::process::Command::new(java_bin);
        cmd.current_dir(&dir)
            .arg(format!("-Xms{}", state.performance.min_ram))
            .arg(format!("-Xmx{}", state.performance.max_ram))
            .args(state.performance.jvm_flags.split_whitespace())
            .arg("-jar")
            .arg(&jar)
            .arg("--nogui")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let pid = child.id().unwrap_or(0);

        let (console_tx, _rx) = broadcast::channel::<ConsoleLine>(512);
        let metrics = Arc::new(RwLock::new(Metrics {
            cpu: 0.0, ram: 0.0, ram_mb: 0, tps: 0.0, players: vec![],
        }));
        let players: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        // pump stdout -> broadcast + track join/leave for player list
        if let Some(stdout) = child.stdout.take() {
            let tx = console_tx.clone();
            let pl = players.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let parsed = parse_log_line(&line);
                    if parsed.msg.ends_with(" joined the game") {
                        if let Some(n) = parsed.msg.strip_suffix(" joined the game") {
                            let n = n.trim().to_string();
                            if !n.is_empty() { pl.lock().await.push(n); }
                        }
                    } else if parsed.msg.ends_with(" left the game")
                        || parsed.msg.contains(" lost connection:")
                    {
                        let n = parsed.msg
                            .split_once(" left the game")
                            .or_else(|| parsed.msg.split_once(" lost connection:"))
                            .map(|(n, _)| n.trim().to_string());
                        if let Some(n) = n {
                            pl.lock().await.retain(|p| p != &n);
                        }
                    }
                    let _ = tx.send(parsed);
                }
            });
        }

        // metrics poller (sysinfo + optional RCON TPS) -> RwLock
        let xmx = state.performance.max_ram.clone();
        let rcon_cfg = if state.network.rcon.enabled {
            Some((state.network.rcon.port, state.network.rcon.password.clone()))
        } else {
            None
        };
        spawn_metrics_poller(pid, metrics.clone(), players.clone(), xmx, rcon_cfg);

        self.running.lock().await.insert(
            name.to_string(),
            RunningServer {
                child, console: console_tx, metrics, players,
                start_time: std::time::Instant::now(), pid,
            },
        );
        Ok(())
    }

    /// Graceful stop via RCON `stop`, falling back to SIGKILL.
    pub async fn stop(&self, name: &str) -> ApiResult<()> {
        let mut guard = self.running.lock().await;
        let mut running = guard.remove(name).ok_or(ApiError::NotRunning)?;

        let state = self.load_state(name)?;
        if state.network.rcon.enabled {
            if let Ok(mut rcon) = self.connect_rcon(&state).await {
                let _ = oxidemc_core::server::stop(&mut rcon).await;
            }
        }
        // give it a moment, then ensure it's gone
        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), running.child.wait()).await;
        let _ = running.child.kill().await;
        Ok(())
    }

    pub async fn restart(&self, name: &str) -> ApiResult<()> {
        if self.is_running(name).await {
            self.stop(name).await?;
        }
        self.start(name).await
    }

    /// Open an authenticated RCON connection from a server's oxide.json.
    pub async fn connect_rcon(&self, state: &ServerState) -> ApiResult<RconClient> {
        if !state.network.rcon.enabled {
            return Err(ApiError::RconDisabled);
        }
        Ok(RconClient::connect(
            "127.0.0.1",
            state.network.rcon.port,
            &state.network.rcon.password,
        )
        .await?)
    }
}

/// Spawn a 1.5s sysinfo poller. Every ~10s also polls RCON for TPS if enabled.
fn spawn_metrics_poller(
    pid: u32,
    metrics: Arc<RwLock<Metrics>>,
    players: Arc<Mutex<Vec<String>>>,
    max_ram: String,
    rcon_cfg: Option<(u16, String)>, // (port, password) if RCON enabled
) {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
    tokio::spawn(async move {
        let mut sys = System::new();
        let xmx_mb = parse_ram_mb(&max_ram);
        let mut tick: u32 = 0;
        loop {
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
                true,
                ProcessRefreshKind::everything(),
            );
            if let Some(proc_) = sys.process(Pid::from_u32(pid)) {
                let ram_mb = proc_.memory() / 1_048_576;
                let current_players = players.lock().await.clone();
                let mut m = metrics.write().await;
                m.cpu = proc_.cpu_usage();
                m.ram_mb = ram_mb;
                m.ram = if xmx_mb > 0 { (ram_mb as f32 / xmx_mb as f32) * 100.0 } else { 0.0 };
                m.players = current_players;
            } else {
                break; // process gone
            }

            // poll RCON for TPS every ~10s (every 7 ticks of 1.5s ≈ 10.5s)
            tick += 1;
            if tick % 7 == 0 {
                if let Some((port, ref password)) = rcon_cfg {
                    if let Ok(mut rcon) = oxidemc_core::rcon::RconClient::connect(
                        "127.0.0.1", port, password,
                    ).await {
                        if let Ok(reply) = rcon.send_command("tps").await {
                            if let Some(tps) = parse_tps(&reply) {
                                metrics.write().await.tps = tps;
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }
    });
}

/// Parse TPS from vanilla/Paper RCON `tps` reply.
/// "TPS from last 1m, 5m, 15m: 19.96, 19.97, 19.97" → 19.96
fn parse_tps(reply: &str) -> Option<f32> {
    reply.split(':').nth(1)?.split(',').next()?.trim().parse().ok()
}

/// Parse a vanilla log line: "[12:04:51] [Server thread/INFO]: Done"
fn parse_log_line(line: &str) -> ConsoleLine {
    let now = now_hms();
    let level = if line.contains("/ERROR") || line.contains("ERROR]") {
        "ERROR"
    } else if line.contains("/WARN") || line.contains("WARN]") {
        "WARN"
    } else {
        "INFO"
    };
    let source = if line.contains('<') && line.contains('>') { "Chat" } else { "Server" };
    let msg = line.split_once("]: ").map_or(line, |x| x.1).to_string();
    ConsoleLine { t: now, level: level.into(), source: source.into(), msg }
}

pub fn now_hms() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn parse_ram_mb(s: &str) -> u64 {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: u64 = digits.parse().unwrap_or(0);
    match s.chars().last() {
        Some('G') | Some('g') => n * 1024,
        _ => n,
    }
}

fn expand(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(p)
}
