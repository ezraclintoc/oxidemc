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

        // pump stdout -> broadcast
        if let Some(stdout) = child.stdout.take() {
            let tx = console_tx.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send(parse_log_line(&line));
                }
            });
        }

        // metrics poller (sysinfo) -> RwLock; see ws.rs for how it's read
        let xmx = state.performance.max_ram.clone();
        spawn_metrics_poller(pid, metrics.clone(), xmx);

        self.running.lock().await.insert(
            name.to_string(),
            RunningServer { child, console: console_tx, metrics, pid },
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

/// Spawn a 1.5s sysinfo poller that updates CPU/RAM and periodically asks RCON
/// for the player list. Kept intentionally small — see comments for extension.
fn spawn_metrics_poller(pid: u32, metrics: Arc<RwLock<Metrics>>, max_ram: String) {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
    tokio::spawn(async move {
        let mut sys = System::new();
        let xmx_mb = parse_ram_mb(&max_ram);
        loop {
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
                true,
                ProcessRefreshKind::everything(),
            );
            if let Some(proc_) = sys.process(Pid::from_u32(pid)) {
                let ram_mb = proc_.memory() / 1_048_576;
                let mut m = metrics.write().await;
                m.cpu = proc_.cpu_usage();
                m.ram_mb = ram_mb;
                m.ram = if xmx_mb > 0 { (ram_mb as f32 / xmx_mb as f32) * 100.0 } else { 0.0 };
            } else {
                break; // process gone
            }
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }
    });
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
