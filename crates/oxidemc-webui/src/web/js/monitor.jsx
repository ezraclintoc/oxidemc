// ============================================================================
// monitor.jsx -- per-server Monitor tab: gauges, TPS graph, players, live console
//   layout: 'grid' | 'focus'   (a variation dimension)
// ============================================================================
const { useState: useStateM, useEffect: useEffectM, useRef: useRefM } = React;

const LVL_COLOR = { INFO: 'var(--text-mid)', WARN: 'var(--amber)', ERROR: 'var(--red)', CMD: 'var(--accent-bright)' };

// Module-level log cache -- survives component unmounts so navigating
// away and back to a server restores the previous console output.
const _logCache = {};

function useLiveServer(server) {
  const running = server.status === 'running';
  const [cpu, setCpu] = useStateM(0);
  const [ram, setRam] = useStateM(0);
  const [tps, setTps] = useStateM(null);
  // tpsAvailable becomes true only when the backend confirms TPS data arrived.
  // It stays false (hiding the TPS panel) until the server reports it, and is
  // reset to false when the server stops.
  const [tpsAvailable, setTpsAvailable] = useStateM(false);
  const [players, setPlayers] = useStateM([]);
  const [uptimeSecs, setUptimeSecs] = useStateM(0);
  const [tpsHist, setTpsHist] = useStateM(() => Array(48).fill(0));
  const [loadHist, setLoadHist] = useStateM(() => Array(48).fill(0));
  const [log, setLog] = useStateM(() => _logCache[server.id] || []);
  const wsRef = useRefM(null);

  const appendLog = (line) => {
    setLog(prev => {
      const next = [...prev.slice(-300), line];
      _logCache[server.id] = next;
      return next;
    });
  };

  useEffectM(() => {
    if (!running) {
      setCpu(0); setRam(0); setTps(null); setTpsAvailable(false);
      setPlayers([]); setUptimeSecs(0);
      return;
    }

    const ws = new WebSocket(wsUrl(`/ws/servers/${server.id}`));
    wsRef.current = ws;

    ws.onmessage = (e) => {
      const msg = JSON.parse(e.data);
      if (msg.type === 'console') {
        appendLog({ t: msg.t, lvl: msg.level, src: msg.source, msg: msg.msg });
      } else if (msg.type === 'metrics') {
        const avail = !!msg.tps_available;
        setCpu(msg.cpu);
        setRam(msg.ram);
        setTpsAvailable(avail);
        // Only show a TPS value once the backend confirms it has real data.
        // 0 is a valid (if alarming) reading when tps_available is true.
        setTps(avail ? (msg.tps ?? 0) : null);
        setPlayers(msg.players || []);
        setUptimeSecs(msg.uptime_secs || 0);
        setTpsHist(h => [...h.slice(1), avail ? (msg.tps ?? 0) : 0]);
        setLoadHist(h => [...h.slice(1), msg.cpu]);
      } else if (msg.type === 'status' && msg.status === 'stopped') {
        // Server stopped while we were watching — clear live state.
        setCpu(0); setRam(0); setTps(null); setTpsAvailable(false);
        setPlayers([]); setUptimeSecs(0);
      }
    };

    ws.onerror = () => ws.close();

    return () => { ws.close(); wsRef.current = null; };
  }, [server.id]); // reconnect only when the server identity changes

  const sendCommand = (cmd) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: 'command', cmd }));
    }
  };

  return { cpu, ram, tps, tpsAvailable, players, uptimeSecs, tpsHist, loadHist, log, sendCommand, running };
}

// ── console panel ───────────────────────────────────────────────────────────
function Console({ log, onSend, running, rconEnabled = true, height = 'auto', flush }) {
  const [cmd, setCmd] = useStateM('');
  const bodyRef = useRefM(null);
  useEffectM(() => { if (bodyRef.current) bodyRef.current.scrollTop = bodyRef.current.scrollHeight; }, [log.length]);

  const canSend = running && rconEnabled;
  const statusDot = !running ? 'stop' : rconEnabled ? 'run' : 'boot';
  const statusLabel = !running ? 'offline' : rconEnabled ? 'RCON live' : 'RCON disabled';

  const submit = (e) => {
    e.preventDefault();
    if (!cmd.trim() || !canSend) return;
    onSend(cmd.trim()); setCmd('');
  };
  return (
    <div className="panel" style={{ display: 'flex', flexDirection: 'column', minHeight: 0, height: flush ? '100%' : 'auto', overflow: 'hidden' }}>
      <div className="panel-h">
        <span style={{ color: 'var(--accent-bright)' }}><Icon name="terminal" size={15} /></span>
        <span className="t">Console</span>
        <span className="row" style={{ marginLeft: 'auto', gap: 7, fontSize: 11, color: 'var(--text-faint)' }}>
          <span className={`dot ${statusDot}`} />{statusLabel}
        </span>
      </div>
      <div ref={bodyRef} style={{ flex: 1, minHeight: 0, overflowY: 'auto', padding: '10px 14px', background: 'var(--bg-sink)',
        fontFamily: 'var(--mono)', fontSize: 12, lineHeight: 1.65, height: height }}>
        {log.length === 0 && <div style={{ color: 'var(--text-faint)', padding: '20px 0', textAlign: 'center' }}>
          {running ? 'Waiting for console output...' : 'Server is offline. Start it to stream the console.'}
        </div>}
        {log.map((l, i) => (
          <div key={i} className="row" style={{ gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
            <span style={{ color: 'var(--text-faint)', flex: 'none' }}>{l.t}</span>
            <span style={{ color: LVL_COLOR[l.lvl] || 'var(--text-faint)', flex: 'none', minWidth: 38, fontSize: 10.5 }}>[{l.lvl}]</span>
            <span style={{ color: l.lvl === 'CMD' ? 'var(--accent-bright)' : l.src === 'Chat' ? 'var(--text)' : 'var(--text-mid)', wordBreak: 'break-word' }}>{l.msg}</span>
          </div>
        ))}
      </div>
      <form onSubmit={submit} className="row" style={{ gap: 8, padding: 10, borderTop: '1px solid var(--border)', background: 'var(--panel)' }}>
        <span className="mono" style={{ color: canSend ? 'var(--accent-bright)' : 'var(--text-faint)', fontSize: 14, paddingLeft: 4 }}>/</span>
        <input className="input" value={cmd} onChange={e => setCmd(e.target.value)} disabled={!canSend}
          placeholder={!running ? 'start the server to send commands' : !rconEnabled ? 'RCON disabled -- enable it in Configure to send commands' : 'type an RCON command -- try "list" or "say hi"'}
          style={{ border: 0, background: 'transparent', height: 28, paddingLeft: 0 }} />
        <button className="btn sm primary" type="submit" disabled={!canSend || !cmd.trim()}><Icon name="send" size={13} />Send</button>
      </form>
    </div>
  );
}

// ── players panel ───────────────────────────────────────────────────────────
function Players({ players = [], maxPlayers }) {
  return (
    <div className="panel" style={{ overflow: 'hidden' }}>
      <div className="panel-h">
        <span style={{ color: 'var(--accent-bright)' }}><Icon name="users" size={15} /></span>
        <span className="t">Players</span>
        <span className="val mono-num" style={{ marginLeft: 'auto', fontSize: 13 }}>{players.length}<span style={{ color: 'var(--text-faint)' }}>/{maxPlayers}</span></span>
      </div>
      <div className="col" style={{ padding: players.length ? 6 : 0 }}>
        {players.length === 0
          ? <div style={{ padding: '22px 0', textAlign: 'center', color: 'var(--text-faint)', fontSize: 12 }}>No players online</div>
          : players.map(p => (
            <div key={p} className="row" style={{ gap: 10, padding: '7px 9px', borderRadius: 6 }}>
              <PlayerHead name={p} />
              <span style={{ fontSize: 13 }}>{p}</span>
            </div>
          ))}
      </div>
    </div>
  );
}

// ── monitor view ────────────────────────────────────────────────────────────
function fmtUptime(secs) {
  if (!secs) return '--';
  if (secs < 60) return `${secs}s`;
  const d = Math.floor(secs / 86400), h = Math.floor((secs % 86400) / 3600), m = Math.floor((secs % 3600) / 60), s = secs % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m ${s}s`;
}

function MonitorView({ server, layout = 'grid' }) {
  const live = useLiveServer(server);
  const running = server.status === 'running';
  const rconEnabled = server.rcon_enabled !== false;
  const playerCount = live.players.length;
  const maxRam = server.max_ram || '2G';

  // Show TPS panel only once the backend confirms TPS data has arrived.
  // The backend handles the probe timeout (OXIDEMC_TPS_TIMEOUT env var, default 20s)
  // and persists incapability to oxide.json (OXIDEMC_TPS_PERSIST env var).
  const showTps = live.tpsAvailable;

  const rconWarning = running && !rconEnabled && (
    <div className="row" style={{ gap: 10, padding: '10px 14px', borderRadius: 7, background: 'color-mix(in oklch, var(--amber) 10%, transparent)',
      border: '1px solid color-mix(in oklch, var(--amber) 35%, transparent)', fontSize: 12.5, color: 'var(--amber)' }}>
      <Icon name="alert" size={14} />
      <span><strong>RCON is disabled</strong> -- command input unavailable.
        Enable RCON in the <strong>Configure</strong> tab to unlock live management.</span>
    </div>
  );

  const statItems = [
    ['power', 'Status', STATUS_LABEL[server.status]],
    ['clock', 'Uptime', running ? fmtUptime(live.uptimeSecs) : '--'],
    ['chip', 'Version', `${server.type} ${server.version}`],
    ['globe', 'Port', server.port],
  ];
  if (running && showTps) statItems.push(['monitor', 'TPS', live.tps != null ? live.tps.toFixed(1) : '--']);

  const statStrip = (
    <div className="panel" style={{ display: 'flex', gap: 0, flexWrap: 'wrap' }}>
      {statItems.map(([ic, lb, v], i) => (
        <div key={lb} style={{ flex: '1 1 130px', padding: '14px 18px', borderLeft: i ? '1px solid var(--border-soft)' : 0 }}>
          <Stat icon={ic} label={lb} value={v} accent={lb === 'TPS' || lb === 'Port'} />
        </div>
      ))}
    </div>
  );

  const tpsPanel = showTps && (
    <div className="panel" style={{ overflow: 'hidden' }}>
      <div className="panel-h">
        <span style={{ color: 'var(--accent-bright)' }}><Icon name="monitor" size={15} /></span>
        <span className="t">Ticks / second</span>
        <span className="row" style={{ marginLeft: 'auto', gap: 10 }}>
          <span className="val mono-num" style={{ fontSize: 17, fontWeight: 600, color: live.tps == null ? 'var(--text-faint)' : undefined }}>
            {live.tps != null ? live.tps.toFixed(1) : '...'}
          </span>
          {live.tps != null && <span style={{ fontSize: 11, color: 'var(--text-faint)' }}>/ 20.0</span>}
        </span>
      </div>
      <div style={{ padding: '14px 8px 6px' }}>
        <AreaChart data={live.tpsHist} min={10} max={20} height={120} target={20} tone="var(--accent)" />
        <div className="row" style={{ justifyContent: 'space-between', padding: '4px 8px 0', fontSize: 10, color: 'var(--text-faint)' }}>
          <span>-60s</span><span className="mono">target 20</span><span>now</span>
        </div>
      </div>
    </div>
  );

  const gaugesPanel = (
    <div className="panel">
      <div className="panel-h"><span style={{ color: 'var(--accent-bright)' }}><Icon name="cpu" size={15} /></span><span className="t">Resources</span></div>
      <div className="row" style={{ justifyContent: 'space-around', padding: '18px 12px 10px' }}>
        <Gauge value={running ? live.cpu : 0} label="CPU" accentByLoad size={104} />
        <Gauge value={running ? live.ram : 0} label="RAM" sub="%" accentByLoad size={104} />
      </div>
      <div className="col" style={{ gap: 12, padding: '6px 18px 18px' }}>
        <BarMeter label="Heap (-Xmx)" value={running ? live.ram : 0} max={100}
          valueLabel={`${maxRam.replace(/[gG]/,'').replace(/[mM]/,'')}${maxRam.match(/[gG]/) ? 'G' : 'M'} . ${Math.round(live.ram/100*parseInt(maxRam))}G used`} />
        <BarMeter label="Players" value={playerCount} max={server.max_players}
          valueLabel={`${playerCount}/${server.max_players}`} tone="var(--amber)" />
      </div>
    </div>
  );

  if (layout === 'focus') {
    return (
      <div className="col fade-in" style={{ gap: 16, height: '100%', minHeight: 0 }}>
        {rconWarning}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 300px', gap: 18, flex: 1, minHeight: 0 }}>
          <div className="col" style={{ gap: 16, minHeight: 0 }}>
            {statStrip}
            <div style={{ flex: 1, minHeight: 320 }}>
              <Console log={live.log} onSend={live.sendCommand} running={running} rconEnabled={rconEnabled} flush />
            </div>
          </div>
          <div className="col" style={{ gap: 16, minHeight: 0, overflowY: 'auto' }}>
            {tpsPanel}
            <Players players={live.players} maxPlayers={server.max_players} />
          </div>
        </div>
      </div>
    );
  }

  // grid: balanced card dashboard
  // When TPS is hidden, console moves up into the TPS slot
  if (!tpsPanel) {
    return (
      <div className="col fade-in" style={{ gap: 18 }}>
        {rconWarning}
        {statStrip}
        <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: 18, alignItems: 'start' }}>
          <div style={{ height: 420 }}>
            <Console log={live.log} onSend={live.sendCommand} running={running} rconEnabled={rconEnabled} flush />
          </div>
          <div className="col" style={{ gap: 18 }}>
            {gaugesPanel}
            <Players players={live.players} maxPlayers={server.max_players} />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="col fade-in" style={{ gap: 18 }}>
      {rconWarning}
      {statStrip}
      <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: 18 }}>
        {tpsPanel}
        {gaugesPanel}
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: 18, alignItems: 'start' }}>
        <div style={{ height: 360 }}><Console log={live.log} onSend={live.sendCommand} running={running} rconEnabled={rconEnabled} flush /></div>
        <Players players={live.players} maxPlayers={server.max_players} />
      </div>
    </div>
  );
}

Object.assign(window, { MonitorView, Console, Players, useLiveServer });

