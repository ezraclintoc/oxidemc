// ============================================================================
// data.jsx -- mock data + icon set, shared across OxideMC web screens
// ============================================================================

// ── Icon set (line icons, 1.6 stroke) ──────────────────────────────────────
function Icon({ name, size = 16, style }) {
  const p = ICONS[name] || '';
  return (
    <svg className="ico" width={size} height={size} viewBox="0 0 24 24" fill="none"
      stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" style={style}>
      {p}
    </svg>
  );
}
const ICONS = {
  servers:  <><rect x="3" y="4" width="18" height="7" rx="1.5"/><rect x="3" y="13" width="18" height="7" rx="1.5"/><path d="M7 7.5h.01M7 16.5h.01"/></>,
  plus:     <><path d="M12 5v14M5 12h14"/></>,
  settings: <><circle cx="12" cy="12" r="3.2"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-2.7 1.1V21a2 2 0 1 1-4 0v-.1A1.6 1.6 0 0 0 7 19.4a1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0-1.1-2.7H1a2 2 0 1 1 0-4h.1A1.6 1.6 0 0 0 2.6 7a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1A1.6 1.6 0 0 0 7 2.6h.1A1.6 1.6 0 0 0 8.2 1.1V1a2 2 0 1 1 4 0v.1A1.6 1.6 0 0 0 15 2.6a1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8V7a1.6 1.6 0 0 0 1.5 1.1h.1a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.4 1z" transform="scale(.86) translate(2 2)"/></>,
  monitor:  <><path d="M3 12h4l2 6 4-14 2 8h6"/></>,
  sliders:  <><path d="M4 6h10M18 6h2M4 12h2M10 12h10M4 18h7M15 18h5"/><circle cx="16" cy="6" r="2"/><circle cx="8" cy="12" r="2"/><circle cx="13" cy="18" r="2"/></>,
  play:     <><path d="M7 5v14l11-7z"/></>,
  stop:     <><rect x="6" y="6" width="12" height="12" rx="1.5"/></>,
  restart:  <><path d="M3 12a9 9 0 1 0 3-6.7"/><path d="M3 4v4h4"/></>,
  terminal: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M13 15h4"/></>,
  cpu:      <><rect x="7" y="7" width="10" height="10" rx="1.5"/><path d="M10 2v3M14 2v3M10 19v3M14 19v3M2 10h3M2 14h3M19 10h3M19 14h3"/></>,
  ram:      <><rect x="2" y="7" width="20" height="10" rx="1.5"/><path d="M6 17v2M10 17v2M14 17v2M18 17v2M6 11v2M10 11v2M14 11v2"/></>,
  users:    <><circle cx="9" cy="8" r="3.2"/><path d="M3 20c0-3.3 2.7-5.5 6-5.5s6 2.2 6 5.5"/><path d="M16 5.2a3.2 3.2 0 0 1 0 6M21 20c0-2.6-1.6-4.6-4-5.2"/></>,
  clock:    <><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/></>,
  globe:    <><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.5 2.5 3.8 5.8 3.8 9s-1.3 6.5-3.8 9c-2.5-2.5-3.8-5.8-3.8-9S9.5 5.5 12 3z"/></>,
  chip:     <><path d="M12 2l8 4.5v9L12 20l-8-4.5v-9z"/><path d="M12 11l8-4.5M12 11v9M12 11L4 6.5"/></>,
  check:    <><path d="M20 6L9 17l-5-5"/></>,
  chevR:    <><path d="M9 6l6 6-6 6"/></>,
  chevD:    <><path d="M6 9l6 6 6-6"/></>,
  edit:     <><path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/></>,
  arrowL:   <><path d="M19 12H5M12 19l-7-7 7-7"/></>,
  arrowR:   <><path d="M5 12h14M12 5l7 7-7 7"/></>,
  download: <><path d="M12 3v12M7 10l5 5 5-5M5 21h14"/></>,
  folder:   <><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></>,
  shield:   <><path d="M12 3l8 3v6c0 4.5-3.2 7.8-8 9-4.8-1.2-8-4.5-8-9V6z"/></>,
  zap:      <><path d="M13 2L4 14h7l-1 8 9-12h-7z"/></>,
  search:   <><circle cx="11" cy="11" r="7"/><path d="M21 21l-4-4"/></>,
  copy:     <><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/></>,
  alert:    <><path d="M12 9v4M12 17h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/></>,
  dot:      <><circle cx="12" cy="12" r="3"/></>,
  send:     <><path d="M22 2 11 13M22 2l-7 20-4-9-9-4z"/></>,
  power:    <><path d="M12 3v9M6.4 6.4a8 8 0 1 0 11.2 0"/></>,
};

// ── platform catalogue ─────────────────────────────────────────────────────
const PLATFORMS = [
  { id: 'vanilla',  name: 'Vanilla',  blurb: 'Official Mojang server. No plugins or mods.', tone: '#7bbf6a' },
  { id: 'paper',    name: 'Paper',    blurb: 'High-performance Spigot fork. Plugins + great defaults.', tone: '#d8d2c6' },
  { id: 'fabric',   name: 'Fabric',   blurb: 'Lightweight modding toolchain. Fast updates.', tone: '#c9a87a' },
  { id: 'forge',    name: 'Forge',    blurb: 'The classic mod loader. Massive ecosystem.', tone: '#9aa3ad' },
  { id: 'neoforge', name: 'NeoForge', blurb: 'Community-driven Forge successor.', tone: '#e0863f' },
  { id: 'purpur',   name: 'Purpur',   blurb: 'Paper fork with deep gameplay config.', tone: '#a585d8' },
];

// ── version lists (per platform, async-feel) ───────────────────────────────
const VERSIONS = ['1.21.4','1.21.3','1.21.1','1.21','1.20.6','1.20.4','1.20.2','1.20.1','1.19.4','1.19.2','1.18.2','1.17.1','1.16.5'];

// ── default settings (mirrors SettingsState::with_defaults) ─────────────────
function defaultSettings() {
  return {
    motd: 'A Minecraft Server', max_players: 20, gamemode: 'survival', difficulty: 'normal',
    pvp: true, whitelist: false, online_mode: true, spawn_protection: 16,
    port: 25565, max_ram: '2G', min_ram: '1G', render_distance: 10, simulation_distance: 10,
    jvm_flags: '', rcon_enabled: true, rcon_port: 25575, rcon_password: '',
  };
}

// ── settings field schema (drives forms + editors) ──────────────────────────
// kind: text | number | ram | choice | toggle | password | textarea
const SETTINGS_SCHEMA = [
  { group: 'Server', icon: 'chip', fields: [
    { key: 'motd',        label: 'MOTD',        kind: 'text',   note: 'Shown in the multiplayer server list.' },
    { key: 'max_players', label: 'Max Players', kind: 'number', min: 1, max: 10000 },
    { key: 'gamemode',    label: 'Gamemode',    kind: 'choice', options: ['survival','creative','adventure','spectator'] },
    { key: 'difficulty',  label: 'Difficulty',  kind: 'choice', options: ['peaceful','easy','normal','hard'] },
    { key: 'pvp',         label: 'PVP',         kind: 'toggle', note: 'Allow player-vs-player combat.' },
    { key: 'whitelist',   label: 'Whitelist',   kind: 'toggle' },
    { key: 'online_mode', label: 'Online Mode', kind: 'toggle', note: 'Verify accounts with Mojang. Disable for LAN only.' },
    { key: 'spawn_protection', label: 'Spawn Protection', kind: 'number', min: 0, max: 256, unit: 'blocks' },
  ]},
  { group: 'Network', icon: 'globe', fields: [
    { key: 'port',          label: 'Port',          kind: 'number', min: 1, max: 65535 },
    { key: 'rcon_enabled',  label: 'RCON',          kind: 'toggle', note: 'Required for OxideMC live management.' },
    { key: 'rcon_port',     label: 'RCON Port',     kind: 'number', min: 1, max: 65535, dep: 'rcon_enabled' },
    { key: 'rcon_password', label: 'RCON Password', kind: 'password', dep: 'rcon_enabled', note: 'Required when RCON is enabled.' },
  ]},
  { group: 'Performance', icon: 'zap', fields: [
    { key: 'max_ram',             label: 'Max RAM',             kind: 'ram',    note: 'JVM max heap (-Xmx). Use G / M suffix.' },
    { key: 'min_ram',             label: 'Min RAM',             kind: 'ram' },
    { key: 'render_distance',     label: 'Render Distance',     kind: 'number', min: 2, max: 32, unit: 'chunks' },
    { key: 'simulation_distance', label: 'Simulation Distance', kind: 'number', min: 2, max: 32, unit: 'chunks' },
    { key: 'jvm_flags',           label: 'JVM Flags',           kind: 'text',   note: 'Extra arguments appended to the launch command.' },
  ]},
];

function cleanMotd(s) { return (s || '').replace(/\u00A7[0-9a-fk-or]/gi, ''); }

function displayVal(f, v) {
  if (f.kind === 'toggle') return v ? 'enabled' : 'disabled';
  if (f.kind === 'password') return v ? '********' : '(not set)';
  if (f.key === 'motd') return cleanMotd(v) || '(empty)';
  if (v === '' || v == null) return '(empty)';
  return f.unit ? `${v} ${f.unit}` : String(v);
}

// ── mock servers ────────────────────────────────────────────────────────────
const MC_NAMES = ['Notch','jeb_','Dinnerbone','Grumm','Steve','Alex','Herobrine','Captain_S','redstone_rex','EnderQueen','pixelmancer','oak_log'];

function makeServer(o) {
  return {
    motd: 'A Minecraft Server', gamemode: 'survival', difficulty: 'normal', pvp: true,
    whitelist: false, online_mode: true, spawn_protection: 16, render_distance: 10,
    simulation_distance: 10, jvm_flags: '', rcon_enabled: true, rcon_port: 25575,
    rcon_password: 'hunter2', min_ram: '1G', max_ram: '2G', ...o,
  };
}

const SERVERS = [
  makeServer({ id:'survival',  name:'survival-smp',  type:'paper',    version:'1.21.4', status:'running', port:25565,
    motd:'\u00A76Oxide SMP \u00A77* season 4', max_players:40, players:['Notch','jeb_','EnderQueen','pixelmancer','oak_log'],
    max_ram:'6G', cpu:34, ram:62, tps:19.9, uptimeH:142, difficulty:'hard', pvp:true, whitelist:true }),
  makeServer({ id:'creative',  name:'build-world',   type:'purpur',   version:'1.21.4', status:'running', port:25566,
    motd:'Creative flat * /plot', max_players:20, players:['Dinnerbone','Grumm'], gamemode:'creative',
    max_ram:'4G', cpu:12, ram:38, tps:20.0, uptimeH:71, difficulty:'peaceful', pvp:false }),
  makeServer({ id:'modded',    name:'create-pack',   type:'forge',    version:'1.20.1', status:'stopped', port:25567,
    motd:'Create: Above & Beyond', max_players:12, players:[], max_ram:'8G', cpu:0, ram:0, tps:0, uptimeH:0, difficulty:'normal' }),
  makeServer({ id:'snapshot',  name:'snapshot-test', type:'fabric',   version:'1.21.3', status:'booting', port:25568,
    motd:'Snapshot testing', max_players:8, players:[], max_ram:'3G', cpu:48, ram:21, tps:0, uptimeH:0, difficulty:'normal' }),
  makeServer({ id:'vanilla',   name:'pure-vanilla',  type:'vanilla',  version:'1.21.4', status:'stopped', port:25569,
    motd:'No mods. No plugins. Just blocks.', max_players:20, players:[], max_ram:'2G', cpu:0, ram:0, tps:0, uptimeH:0 }),
];

// ── console log seed (RCON-style) ───────────────────────────────────────────
const LOG_SEED = [
  { t:'12:04:51', lvl:'INFO',  src:'Server',     msg:'Starting minecraft server version 1.21.4' },
  { t:'12:04:51', lvl:'INFO',  src:'Server',     msg:'Loading properties' },
  { t:'12:04:52', lvl:'INFO',  src:'Server',     msg:'Default game type: SURVIVAL' },
  { t:'12:04:52', lvl:'INFO',  src:'RCON',       msg:'RCON running on 0.0.0.0:25575' },
  { t:'12:04:53', lvl:'INFO',  src:'Server',     msg:'Preparing level "world"' },
  { t:'12:04:55', lvl:'INFO',  src:'Server',     msg:'Preparing spawn area: 92%' },
  { t:'12:04:56', lvl:'INFO',  src:'Server',     msg:'Done (4.122s)! For help, type "help"' },
  { t:'12:09:14', lvl:'INFO',  src:'Server',     msg:'Notch joined the game' },
  { t:'12:09:40', lvl:'INFO',  src:'Server',     msg:'jeb_ joined the game' },
  { t:'12:11:02', lvl:'WARN',  src:'Server',     msg:'Can\'t keep up! Is the server overloaded? Running 2140ms behind' },
  { t:'12:11:48', lvl:'INFO',  src:'Server',     msg:'EnderQueen joined the game' },
  { t:'12:14:20', lvl:'INFO',  src:'Chat',       msg:'<Notch> anyone got spare iron?' },
  { t:'12:14:55', lvl:'INFO',  src:'Chat',       msg:'<jeb_> coming to you, hold on' },
  { t:'12:16:03', lvl:'INFO',  src:'Server',     msg:'pixelmancer joined the game' },
  { t:'12:18:31', lvl:'INFO',  src:'Server',     msg:'Saving the game (this may take a moment!)' },
  { t:'12:18:32', lvl:'INFO',  src:'Server',     msg:'Saved the game' },
];

// little deterministic-ish jitter for live charts
function jitter(base, amp) { return base + (Math.random() - 0.5) * amp; }

// ── API helpers ─────────────────────────────────────────────────────────────
async function apiFetch(path, opts = {}) {
  const r = await fetch(path, {
    headers: { 'Content-Type': 'application/json' },
    ...opts,
  });
  if (!r.ok) {
    const err = await r.json().catch(() => ({ error: r.statusText }));
    throw new Error(err.error || r.statusText);
  }
  return r.json();
}

function wsUrl(path) {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${location.host}${path}`;
}

// Map flat UI settings → nested ServerState for API
function flatToServerState(name, type, version, s) {
  return {
    server_name: name,
    server_type: type,
    minecraft_version: version,
    directory: `~/servers/${name}`,
    server: {
      motd: s.motd, max_players: s.max_players, gamemode: s.gamemode,
      difficulty: s.difficulty, pvp: s.pvp, whitelist: s.whitelist,
      online_mode: s.online_mode, level_name: 'world', seed: '',
      spawn_protection: s.spawn_protection,
    },
    network: {
      port: s.port,
      rcon: { enabled: s.rcon_enabled, port: s.rcon_port, password: s.rcon_password || '' },
    },
    performance: {
      max_ram: s.max_ram, min_ram: s.min_ram, jvm_flags: s.jvm_flags || '',
      render_distance: s.render_distance, simulation_distance: s.simulation_distance,
    },
    players: { ops: [], banned_players: [], whitelist_players: [] },
    mods: { mods: [] },
  };
}

// Map nested ServerState → flat UI server object
function serverStateToFlat(state, extra = {}) {
  return makeServer({
    id: state.server_name,
    name: state.server_name,
    type: state.server_type,
    version: state.minecraft_version,
    motd: state.server.motd,
    max_players: state.server.max_players,
    gamemode: state.server.gamemode,
    difficulty: state.server.difficulty,
    pvp: state.server.pvp,
    whitelist: state.server.whitelist,
    online_mode: state.server.online_mode,
    spawn_protection: state.server.spawn_protection,
    port: state.network.port,
    rcon_enabled: state.network.rcon.enabled,
    rcon_port: state.network.rcon.port,
    rcon_password: state.network.rcon.password,
    max_ram: state.performance.max_ram,
    min_ram: state.performance.min_ram,
    jvm_flags: state.performance.jvm_flags,
    render_distance: state.performance.render_distance,
    simulation_distance: state.performance.simulation_distance,
    players: [], cpu: 0, ram: 0, tps: 0, uptimeH: 0,
    ...extra,
  });
}

// Map ServerSummary (list endpoint) → flat UI server object.
// Pass preserved live metrics (players, cpu, ram, tps, uptimeH) via `s` to keep
// them across the 5-second API poll cycle.
function summaryToServer(s) {
  return makeServer({
    id: s.name,
    name: s.name,
    type: s.server_type,
    version: s.minecraft_version,
    status: s.status,
    port: s.port,
    max_players: s.max_players,
    motd: s.motd,
    players: s.players ?? [],
    cpu: s.cpu ?? 0,
    ram: s.ram ?? 0,
    tps: s.tps ?? 0,
    uptimeH: s.uptimeH ?? 0,
  });
}

Object.assign(window, {
  Icon, ICONS, PLATFORMS, VERSIONS, defaultSettings, SETTINGS_SCHEMA, displayVal,
  SERVERS, LOG_SEED, MC_NAMES, jitter, makeServer, cleanMotd,
  apiFetch, wsUrl, flatToServerState, serverStateToFlat, summaryToServer,
});
