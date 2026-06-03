// ============================================================================
// monitor.jsx — per-server Monitor tab: gauges, TPS graph, players, live console
//   layout: 'grid' | 'focus'   (a variation dimension)
// ============================================================================
const { useState: useStateM, useEffect: useEffectM, useRef: useRefM } = React;

const LVL_COLOR = { INFO: 'var(--text-mid)', WARN: 'var(--amber)', ERROR: 'var(--red)', CMD: 'var(--accent-bright)' };

function useLiveServer(server) {
  const running = server.status === 'running';
  const [cpu, setCpu] = useStateM(0);
  const [ram, setRam] = useStateM(0);
  const [tps, setTps] = useStateM(0);
  const [tpsHist, setTpsHist] = useStateM(() => Array(48).fill(0));
  const [loadHist, setLoadHist] = useStateM(() => Array(48).fill(0));
  const [log, setLog] = useStateM([]);
  const wsRef = useRefM(null);

  useEffectM(() => {
    if (!running) { setCpu(0); setRam(0); setTps(0); return; }

    const ws = new WebSocket(wsUrl(`/ws/servers/${server.id}`));
    wsRef.current = ws;

    ws.onmessage = (e) => {
      const msg = JSON.parse(e.data);
      if (msg.type === 'console') {
        setLog(l => [...l.slice(-200), { t: msg.t, lvl: msg.level, src: msg.source, msg: msg.msg }]);
      } else if (msg.type === 'metrics') {
        setCpu(msg.cpu);
        setRam(msg.ram);
        setTps(msg.tps ?? 0);
        setTpsHist(h => [...h.slice(1), msg.tps ?? 0]);
        setLoadHist(h => [...h.slice(1), msg.cpu]);
      }
    };

    ws.onerror = () => ws.close();

    return () => { ws.close(); wsRef.current = null; };
  }, [running, server.id]);

  const sendCommand = (cmd) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: 'command', cmd }));
    }
  };

  return { cpu, ram, tps, tpsHist, loadHist, log, sendCommand, running };
}

// ── console panel ───────────────────────────────────────────────────────────
function Console({ log, onSend, running, height = 'auto', flush }) {
  const [cmd, setCmd] = useStateM('');
  const [hist, setHist] = useStateM([]);
  const bodyRef = useRefM(null);
  useEffectM(() => { if (bodyRef.current) bodyRef.current.scrollTop = bodyRef.current.scrollHeight; }, [log.length]);
  const submit = (e) => {
    e.preventDefault();
    if (!cmd.trim() || !running) return;
    onSend(cmd.trim()); setHist(h => [cmd.trim(), ...h].slice(0, 30)); setCmd('');
  };
  return (
    <div className="panel" style={{ display: 'flex', flexDirection: 'column', minHeight: 0, height: flush ? '100%' : 'auto', overflow: 'hidden' }}>
      <div className="panel-h">
        <span style={{ color: 'var(--accent-bright)' }}><Icon name="terminal" size={15} /></span>
        <span className="t">Console</span>
        <span className="row" style={{ marginLeft: 'auto', gap: 7, fontSize: 11, color: 'var(--text-faint)' }}>
          <span className={`dot ${running ? 'run' : 'stop'}`} />{running ? 'RCON live' : 'offline'}
        </span>
      </div>
      <div ref={bodyRef} style={{ flex: 1, minHeight: 0, overflowY: 'auto', padding: '10px 14px', background: 'var(--bg-sink)',
        fontFamily: 'var(--mono)', fontSize: 12, lineHeight: 1.65, height: height }}>
        {log.length === 0 && <div style={{ color: 'var(--text-faint)', padding: '20px 0', textAlign: 'center' }}>Server is offline. Start it to stream the console.</div>}
        {log.map((l, i) => (
          <div key={i} className="row" style={{ gap: 8, alignItems: 'baseline', flexWrap: 'wrap' }}>
            <span style={{ color: 'var(--text-faint)', flex: 'none' }}>{l.t}</span>
            <span style={{ color: LVL_COLOR[l.lvl] || 'var(--text-faint)', flex: 'none', minWidth: 38, fontSize: 10.5 }}>[{l.lvl}]</span>
            <span style={{ color: l.lvl === 'CMD' ? 'var(--accent-bright)' : l.src === 'Chat' ? 'var(--text)' : 'var(--text-mid)', wordBreak: 'break-word' }}>{l.msg}</span>
          </div>
        ))}
      </div>
      <form onSubmit={submit} className="row" style={{ gap: 8, padding: 10, borderTop: '1px solid var(--border)', background: 'var(--panel)' }}>
        <span className="mono" style={{ color: running ? 'var(--accent-bright)' : 'var(--text-faint)', fontSize: 14, paddingLeft: 4 }}>/</span>
        <input className="input" value={cmd} onChange={e => setCmd(e.target.value)} disabled={!running}
          placeholder={running ? 'type an RCON command — try "list" or "say hi"' : 'start the server to send commands'}
          style={{ border: 0, background: 'transparent', height: 28, paddingLeft: 0 }} />
        <button className="btn sm primary" type="submit" disabled={!running || !cmd.trim()}><Icon name="send" size={13} />Send</button>
      </form>
    </div>
  );
}

// ── players panel ───────────────────────────────────────────────────────────
function Players({ server }) {
  return (
    <div className="panel" style={{ overflow: 'hidden' }}>
      <div className="panel-h">
        <span style={{ color: 'var(--accent-bright)' }}><Icon name="users" size={15} /></span>
        <span className="t">Players</span>
        <span className="val mono-num" style={{ marginLeft: 'auto', fontSize: 13 }}>{server.players.length}<span style={{ color: 'var(--text-faint)' }}>/{server.max_players}</span></span>
      </div>
      <div className="col" style={{ padding: server.players.length ? 6 : 0 }}>
        {server.players.length === 0
          ? <div style={{ padding: '22px 0', textAlign: 'center', color: 'var(--text-faint)', fontSize: 12 }}>No players online</div>
          : server.players.map(p => (
            <div key={p} className="row" style={{ gap: 10, padding: '7px 9px', borderRadius: 6 }}>
              <PlayerHead name={p} />
              <span style={{ fontSize: 13 }}>{p}</span>
              <span className="row" style={{ marginLeft: 'auto', gap: 5, fontSize: 10.5, color: 'var(--text-faint)' }}>
                <Icon name="dot" size={8} style={{ color: 'var(--green)' }} />{Math.floor(jitter(40, 60))}ms
              </span>
            </div>
          ))}
      </div>
    </div>
  );
}

// ── monitor view ────────────────────────────────────────────────────────────
function MonitorView({ server, layout = 'grid' }) {
  const live = useLiveServer(server);
  const running = server.status === 'running';

  const statStrip = (
    <div className="panel" style={{ display: 'flex', gap: 0, flexWrap: 'wrap' }}>
      {[
        ['power', 'Status', STATUS_LABEL[server.status]],
        ['clock', 'Uptime', running ? `${Math.floor(server.uptimeH/24)}d ${server.uptimeH%24}h` : '—'],
        ['chip', 'Version', `${server.type} ${server.version}`],
        ['globe', 'Port', server.port],
        ['monitor', 'TPS', running ? live.tps.toFixed(1) : '—'],
      ].map(([ic, lb, v], i) => (
        <div key={lb} style={{ flex: '1 1 130px', padding: '14px 18px', borderLeft: i ? '1px solid var(--border-soft)' : 0 }}>
          <Stat icon={ic} label={lb} value={v} accent={lb === 'TPS' || lb === 'Port'} />
        </div>
      ))}
    </div>
  );

  const tpsPanel = (
    <div className="panel" style={{ overflow: 'hidden' }}>
      <div className="panel-h">
        <span style={{ color: 'var(--accent-bright)' }}><Icon name="monitor" size={15} /></span>
        <span className="t">Ticks / second</span>
        <span className="row" style={{ marginLeft: 'auto', gap: 10 }}>
          <span className="val mono-num" style={{ fontSize: 17, fontWeight: 600 }}>{running ? live.tps.toFixed(1) : '0.0'}</span>
          <span style={{ fontSize: 11, color: 'var(--text-faint)' }}>/ 20.0</span>
        </span>
      </div>
      <div style={{ padding: '14px 8px 6px' }}>
        <AreaChart data={live.tpsHist} min={10} max={20} height={120} target={20} tone="var(--accent)" />
        <div className="row" style={{ justifyContent: 'space-between', padding: '4px 8px 0', fontSize: 10, color: 'var(--text-faint)' }}>
          <span>−60s</span><span className="mono">target 20</span><span>now</span>
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
        <BarMeter label="Heap (-Xmx)" value={running ? live.ram : 0} max={100} valueLabel={`${(server.max_ram).replace('G','')}·${Math.round(live.ram/100*parseInt(server.max_ram))}G`} />
        <BarMeter label="Players" value={server.players.length} max={server.max_players} valueLabel={`${server.players.length}/${server.max_players}`} tone="var(--amber)" />
      </div>
    </div>
  );

  if (layout === 'focus') {
    // console-led single column, stats + small charts in a side rail
    return (
      <div className="fade-in" style={{ display: 'grid', gridTemplateColumns: '1fr 300px', gap: 18, height: '100%', minHeight: 0 }}>
        <div className="col" style={{ gap: 16, minHeight: 0 }}>
          {statStrip}
          <div style={{ flex: 1, minHeight: 320 }}>
            <Console log={live.log} onSend={live.sendCommand} running={running} flush />
          </div>
        </div>
        <div className="col" style={{ gap: 16, minHeight: 0, overflowY: 'auto' }}>
          {tpsPanel}
          <Players server={server} />
        </div>
      </div>
    );
  }

  // grid: balanced card dashboard
  return (
    <div className="col fade-in" style={{ gap: 18 }}>
      {statStrip}
      <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: 18 }}>
        {tpsPanel}
        {gaugesPanel}
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: 18, alignItems: 'start' }}>
        <div style={{ height: 360 }}><Console log={live.log} onSend={live.sendCommand} running={running} flush /></div>
        <Players server={server} />
      </div>
    </div>
  );
}

Object.assign(window, { MonitorView, Console, Players, useLiveServer });
