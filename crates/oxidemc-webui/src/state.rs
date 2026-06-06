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
    pub cpu: f32,
    pub ram: f32,
    pub ram_mb: u64,
    pub tps: f32,
    /// True once the first successful RCON TPS reply has arrived this session.
    pub tps_available: bool,
    pub players: Vec<String>,
}

/// Progress events broadcast to the install WebSocket.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InstallMsg {
    Progress { downloaded: u64, total: u64 },
    Done,
    Error { msg: String },
}

/// Everything we track for a server that is currently running.
pub struct RunningServer {
    pub child: Child,
    pub console: broadcast::Sender<ConsoleLine>,
    pub metrics: Arc<RwLock<Metrics>>,
    #[allow(dead_code)]
    pub players: Arc<Mutex<Vec<String>>>,
    pub start_time: std::time::Instant,
    #[allow(dead_code)]
    pub pid: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub servers_dir: PathBuf,
    pub java_path: String,
    pub running: Arc<Mutex<HashMap<String, RunningServer>>>,
    /// job-id → broadcast sender for install progress WebSocket
    pub install_jobs: Arc<Mutex<HashMap<String, broadcast::Sender<InstallMsg>>>>,
}

impl AppState {
    pub fn from_manage() -> ApiResult<Self> {
        let manage = oxidemc_core::config::load_manage()?;
        Ok(Self {
            servers_dir: expand(&manage.servers_directory.default),
            java_path: manage.java_path.default,
            running: Arc::new(Mutex::new(HashMap::new())),
            install_jobs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn server_dir(&self, name: &str) -> PathBuf {
        self.servers_dir.join(name)
    }

    pub fn load_state(&self, name: &str) -> ApiResult<ServerState> {
        let dir = self.server_dir(name);
        if !dir.join("oxide.json").exists() {
            return Err(ApiError::NotFound(name.to_string()));
        }
        Ok(oxidemc_core::config::load_server_state(&dir)?)
    }

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
            cpu: 0.0, ram: 0.0, ram_mb: 0, tps: 0.0, tps_available: false, players: vec![],
        }));
        let players: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

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
                            if !n.is_empty() {
                                let mut pl = pl.lock().await;
                                if !pl.contains(&n) { pl.push(n); }
                            }
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

        // Skip TPS probing if this server was previously confirmed as incapable.
        let tps_cmd = if state.tps_capable == Some(false) {
            None
        } else {
            state.network.rcon.enabled
                .then(|| tps_rcon_cfg(&state.server_type))
                .flatten()
        };
        let xmx = state.performance.max_ram.clone();
        spawn_metrics_poller(pid, metrics.clone(), players.clone(), xmx, tps_cmd, dir.clone());

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
    /// The running Mutex is released immediately after removing the entry so
    /// concurrent start/list/stop calls are not blocked during I/O.
    pub async fn stop(&self, name: &str) -> ApiResult<()> {
        let mut running = {
            let mut guard = self.running.lock().await;
            guard.remove(name).ok_or(ApiError::NotRunning)?
        }; // lock dropped here

        let state = self.load_state(name)?;
        if state.network.rcon.enabled {
            if let Ok(mut rcon) = self.connect_rcon(&state).await {
                let _ = oxidemc_core::server::stop(&mut rcon).await;
            }
        }
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            running.child.wait(),
        )
        .await;
        let _ = running.child.kill().await;
        Ok(())
    }

    pub async fn restart(&self, name: &str) -> ApiResult<()> {
        // A concurrent stop might beat us — treat NotRunning as non-fatal here.
        match self.stop(name).await {
            Ok(()) | Err(ApiError::NotRunning) => {}
            Err(e) => return Err(e),
        }
        self.start(name).await
    }

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

fn tps_rcon_cfg(server_type: &str) -> Option<&'static str> {
    match server_type {
        "paper" | "purpur" | "spigot" => Some("tps"),
        "forge" => Some("forge tps"),
        "neoforge" => Some("neoforge tps"),
        _ => None,
    }
}

/// Spawn the 1.5 s sysinfo poller.
///
/// Every ~10.5 s it also polls RCON for TPS — but re-reads credentials from
/// oxide.json each time so a Configure save (new password/port) takes effect
/// without a restart.
///
/// ENV vars:
///   OXIDEMC_TPS_TIMEOUT  — seconds before giving up on TPS probe (default 20)
///   OXIDEMC_TPS_PERSIST  — "false"/"0" to skip writing tps_capable=false to
///                          oxide.json when the probe times out (default true)
fn spawn_metrics_poller(
    pid: u32,
    metrics: Arc<RwLock<Metrics>>,
    players: Arc<Mutex<Vec<String>>>,
    max_ram: String,
    tps_cmd: Option<&'static str>,
    server_dir: PathBuf,
) {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

    let tps_timeout = std::env::var("OXIDEMC_TPS_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(20);
    let tps_persist = std::env::var("OXIDEMC_TPS_PERSIST")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    tokio::spawn(async move {
        let mut sys = System::new();
        let xmx_mb = parse_ram_mb(&max_ram);
        let mut tick: u32 = 0;
        let probe_start = std::time::Instant::now();
        // If tps_cmd is None we have nothing to probe; mark as already given up.
        let mut tps_giving_up = tps_cmd.is_none();
        // Persistent RCON connection — reconnect only on failure or credential change.
        let mut rcon_client: Option<RconClient> = None;
        let mut rcon_creds: Option<(u16, String)> = None;

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
                break;
            }

            tick += 1;

            // TPS timeout: give up if no data arrived within the probe window.
            if !tps_giving_up {
                let has_data = metrics.read().await.tps_available;
                if !has_data && probe_start.elapsed().as_secs() >= tps_timeout {
                    tps_giving_up = true;
                    tracing::debug!(
                        "TPS probe timeout ({}s) for {:?}; giving up",
                        tps_timeout, server_dir
                    );
                    if tps_persist {
                        if let Ok(mut st) = oxidemc_core::config::load_server_state(&server_dir) {
                            st.tps_capable = Some(false);
                            let _ = oxidemc_core::config::save_server_state(&st, &server_dir);
                        }
                    }
                }
            }

            // Poll RCON for TPS every ~10.5 s (every 7 × 1.5 s ticks).
            // Persistent connection: reconnect only on failure or credential change.
            if !tps_giving_up && tick % 7 == 0 {
                if let Some(cmd) = tps_cmd {
                    match oxidemc_core::config::load_server_state(&server_dir) {
                        Err(e) => tracing::debug!("TPS: could not reload state: {}", e),
                        Ok(st) if !st.network.rcon.enabled => {
                            rcon_client = None;
                            rcon_creds = None;
                        }
                        Ok(st) => {
                            let creds = (st.network.rcon.port, st.network.rcon.password.clone());
                            if rcon_creds.as_ref() != Some(&creds) {
                                rcon_client = None;
                                rcon_creds = None;
                            }
                            if rcon_client.is_none() {
                                match RconClient::connect("127.0.0.1", creds.0, &creds.1).await {
                                    Err(e) => tracing::debug!("TPS RCON connect failed: {}", e),
                                    Ok(client) => {
                                        rcon_client = Some(client);
                                        rcon_creds = Some(creds);
                                    }
                                }
                            }
                            if let Some(ref mut rcon) = rcon_client {
                                match rcon.send_command(cmd).await {
                                    Err(e) => {
                                        tracing::debug!("TPS RCON cmd '{}' failed: {}", cmd, e);
                                        rcon_client = None;
                                        rcon_creds = None;
                                    }
                                    Ok(reply) => {
                                        let clean = strip_mc_colors(&reply);
                                        if let Some(tps) = parse_tps(cmd, &clean) {
                                            let mut m = metrics.write().await;
                                            m.tps = tps;
                                            m.tps_available = true;
                                        } else {
                                            tracing::debug!("TPS parse failed on: {:?}", clean);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }
    });
}

fn strip_mc_colors(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{00A7}' { chars.next(); } else { out.push(c); }
    }
    out
}

/// Parse TPS from an RCON reply.
///
/// Paper/Purpur/Spigot (`tps`):
///   "TPS from last 1m, 5m, 15m: 19.96, 19.97, 19.97"  → 19.96
///
/// Forge/NeoForge (`forge tps` / `neoforge tps`):
///   "Overall: Mean tick time: 44.71 ms. Mean TPS: 20.000"  → 20.0
fn parse_tps(cmd: &str, reply: &str) -> Option<f32> {
    if cmd.contains("forge") {
        for line in reply.lines() {
            if line.contains("Overall") && line.contains("Mean TPS:") {
                let after = line.split("Mean TPS:").nth(1)?;
                return after.trim().parse().ok();
            }
        }
        None
    } else {
        let after_colon = reply.split(':').nth(1)?;
        let first = after_colon.split(',').next()?.trim();
        let clean: String = first.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
        clean.parse().ok()
    }
}

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
