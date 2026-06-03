// ============================================================================
// app.jsx — OxideMC web shell: sidebar nav, router, servers list, server
//           detail (Monitor/Configure tabs), global settings, Tweaks.
// ============================================================================
const { useState: useStateA, useEffect: useEffectA, useMemo: useMemoA } = React;

// ── server card (home) ──────────────────────────────────────────────────────
function ServerCard({ server, onOpen, onAction }) {
  const running = server.status === 'running';
  const plat = PLATFORMS.find(p => p.id === server.type);
  return (
    <div className="panel server-card" onClick={() => onOpen(server.id)} style={{ cursor: 'pointer', overflow: 'hidden', transition: 'border-color .14s, transform .08s' }}>
      <div style={{ padding: 16, display: 'flex', gap: 14 }}>
        <PlatformLogo id={server.type} tone={plat?.tone} size={46} />
        <div className="col grow" style={{ gap: 5, minWidth: 0 }}>
          <div className="row" style={{ justifyContent: 'space-between', gap: 10, minWidth: 0 }}>
            <span className="sans truncate" style={{ fontSize: 16, fontWeight: 600, flex: 1, minWidth: 0 }}>{server.name}</span>
            <StatusPill status={server.status} />
          </div>
          <span className="truncate" style={{ fontSize: 12, color: 'var(--text-dim)' }}>{cleanMotd(server.motd)}</span>
        </div>
      </div>
      <div className="row" style={{ gap: 0, padding: '0 16px 4px', flexWrap: 'wrap' }}>
        {[['chip', `${plat?.name} ${server.version}`], ['users', `${server.players.length}/${server.max_players}`], ['globe', `:${server.port}`], ['monitor', running ? `${server.tps.toFixed(1)} tps` : 'idle']].map(([ic, v]) => (
          <span key={v} className="row" style={{ gap: 6, fontSize: 11.5, color: 'var(--text-dim)', marginRight: 16, marginBottom: 8 }}>
            <span style={{ color: 'var(--text-faint)' }}><Icon name={ic} size={13} /></span>{v}
          </span>
        ))}
      </div>
      <div className="row" style={{ gap: 8, padding: 12, borderTop: '1px solid var(--border-soft)', background: 'var(--bg-sink)' }}
        onClick={e => e.stopPropagation()}>
        {running ? (
          <>
            <button className="btn sm danger grow" onClick={() => onAction(server.id, 'stop')}><Icon name="stop" size={13} />Stop</button>
            <button className="btn sm grow" onClick={() => onAction(server.id, 'restart')}><Icon name="restart" size={13} />Restart</button>
          </>
        ) : server.status === 'booting' ? (
          <button className="btn sm grow" disabled><span className="spin" style={{ width: 12, height: 12, border: '2px solid var(--border)', borderTopColor: 'var(--accent)', borderRadius: '50%' }} />starting…</button>
        ) : (
          <button className="btn sm primary grow" onClick={() => onAction(server.id, 'start')}><Icon name="play" size={13} />Start</button>
        )}
        <button className="btn sm icon" onClick={() => onOpen(server.id)}><Icon name="chevR" size={15} /></button>
      </div>
    </div>
  );
}

// ── servers home ────────────────────────────────────────────────────────────
function ServersHome({ servers, onOpen, onAction, onNew }) {
  const [q, setQ] = useStateA('');
  const list = servers.filter(s => s.name.includes(q.toLowerCase()) || s.type.includes(q.toLowerCase()));
  const runCount = servers.filter(s => s.status === 'running').length;
  const players = servers.reduce((n, s) => n + s.players.length, 0);
  return (
    <div className="content-pad fade-in">
      <div className="pagehead">
        <div>
          <div className="kicker" style={{ marginBottom: 8 }}>OxideMC</div>
          <h1>Servers</h1>
          <p>Every Minecraft server OxideMC manages. Open one to monitor or configure it.</p>
        </div>
        <button className="btn primary" onClick={onNew}><Icon name="plus" size={15} />New server</button>
      </div>

      <div className="row" style={{ gap: 12, marginBottom: 20, flexWrap: 'wrap' }}>
        <div className="panel row" style={{ gap: 22, padding: '12px 18px' }}>
          <Stat icon="servers" label="Servers" value={servers.length} />
          <div style={{ width: 1, height: 28, background: 'var(--border-soft)' }} />
          <Stat icon="power" label="Running" value={`${runCount}`} accent />
          <div style={{ width: 1, height: 28, background: 'var(--border-soft)' }} />
          <Stat icon="users" label="Players online" value={players} accent />
        </div>
        <div className="grow" />
        <div className="row panel" style={{ gap: 8, padding: '0 12px', height: 40, minWidth: 220 }}>
          <span style={{ color: 'var(--text-faint)' }}><Icon name="search" size={15} /></span>
          <input className="input" value={q} onChange={e => setQ(e.target.value)} placeholder="filter servers…"
            style={{ border: 0, background: 'transparent', height: 38, paddingLeft: 0 }} />
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(330px, 1fr))', gap: 16 }}>
        {list.map(s => <ServerCard key={s.id} server={s} onOpen={onOpen} onAction={onAction} />)}
        <button className="panel" onClick={onNew} style={{
          cursor: 'pointer', minHeight: 150, display: 'grid', placeItems: 'center', color: 'var(--text-dim)',
          border: '1px dashed var(--border)', background: 'transparent', gap: 8,
        }}>
          <div className="col" style={{ alignItems: 'center', gap: 9 }}>
            <span style={{ color: 'var(--accent-bright)' }}><Icon name="plus" size={22} /></span>
            <span style={{ fontSize: 13 }}>Install a new server</span>
          </div>
        </button>
      </div>
    </div>
  );
}

// ── server detail (tabs) ────────────────────────────────────────────────────
function ServerDetail({ server, tab, setTab, onAction, onBack, onChange, onSave, monitorLayout, settingsTreatment }) {
  const running = server.status === 'running';
  const plat = PLATFORMS.find(p => p.id === server.type);
  return (
    <div className="col" style={{ height: '100%', minHeight: 0 }}>
      <div style={{ padding: '22px 38px 0' }}>
        <div className="row" style={{ gap: 8, marginBottom: 14, fontSize: 12, color: 'var(--text-dim)' }}>
          <button className="btn sm ghost" onClick={onBack} style={{ paddingLeft: 8 }}><Icon name="arrowL" size={14} />Servers</button>
        </div>
        <div className="row" style={{ gap: 16, alignItems: 'flex-start' }}>
          <PlatformLogo id={server.type} tone={plat?.tone} size={50} />
          <div className="col grow" style={{ gap: 6, minWidth: 0 }}>
            <div className="row" style={{ gap: 12, minWidth: 0 }}>
              <h1 className="sans" style={{ margin: 0, fontSize: 25, fontWeight: 600, letterSpacing: '-.02em', whiteSpace: 'nowrap', flex: '0 1 auto' }}>{server.name}</h1>
              <StatusPill status={server.status} />
            </div>
            <div className="row" style={{ gap: 16, fontSize: 12, color: 'var(--text-dim)', flexWrap: 'wrap' }}>
              <span className="row" style={{ gap: 6 }}><Icon name="chip" size={13} />{plat?.name} {server.version}</span>
              <span className="row" style={{ gap: 6 }}><Icon name="globe" size={13} />localhost:{server.port}</span>
              <span className="row" style={{ gap: 6 }}><Icon name="folder" size={13} />~/servers/{server.name}</span>
            </div>
          </div>
          <div className="row" style={{ gap: 8 }}>
            {running ? (
              <>
                <button className="btn danger" onClick={() => onAction(server.id, 'stop')}><Icon name="stop" size={15} />Stop</button>
                <button className="btn" onClick={() => onAction(server.id, 'restart')}><Icon name="restart" size={15} />Restart</button>
              </>
            ) : server.status === 'booting' ? (
              <button className="btn" disabled><span className="spin" style={{ width: 13, height: 13, border: '2px solid var(--border)', borderTopColor: 'var(--accent)', borderRadius: '50%' }} />starting…</button>
            ) : (
              <button className="btn primary" onClick={() => onAction(server.id, 'start')}><Icon name="play" size={15} />Start</button>
            )}
          </div>
        </div>
        <div className="row" style={{ gap: 4, marginTop: 18, borderBottom: '1px solid var(--border)' }}>
          {[['monitor', 'Monitor', 'monitor'], ['sliders', 'Configure', 'configure']].map(([ic, label, key]) => (
            <button key={key} onClick={() => setTab(key)} style={{
              display: 'flex', alignItems: 'center', gap: 8, padding: '10px 14px', border: 0, cursor: 'pointer',
              background: 'transparent', fontFamily: 'var(--mono)', fontSize: 13,
              color: tab === key ? 'var(--accent-bright)' : 'var(--text-dim)',
              borderBottom: '2px solid ' + (tab === key ? 'var(--accent)' : 'transparent'), marginBottom: -1,
            }}><Icon name={ic} size={15} />{label}</button>
          ))}
        </div>
      </div>
      <div className="content" style={{ padding: tab === 'monitor' ? '20px 38px 30px' : '24px 38px 40px' }}>
        {tab === 'monitor'
          ? <MonitorView key={server.id + server.status} server={server} layout={monitorLayout} />
          : (
            <div style={{ maxWidth: 920 }}>
              <div className="row" style={{ marginBottom: 16, gap: 10 }}>
                <span className="kicker">Editing {server.name} · oxide.json</span>
                <div className="grow" />
                <button className="btn sm primary" onClick={() => onSave(server.id)}><Icon name="check" size={13} />Save</button>
              </div>
              <SettingsForm values={server} onChange={(k, v) => onChange(server.id, k, v)} treatment={settingsTreatment} />
            </div>
          )}
      </div>
    </div>
  );
}

// ── global settings (Manage) ────────────────────────────────────────────────
function GlobalSettings() {
  const [m, setM] = useStateA(null);
  const [err, setErr] = useStateA(null);
  const [saving, setSaving] = useStateA(false);
  const [saved, setSaved] = useStateA(false);

  useEffectA(() => {
    apiFetch('/api/manage').then(setM).catch(e => setErr(e.message));
  }, []);

  if (err) return <div className="content-pad" style={{ color: 'var(--red)', fontSize: 13 }}><Icon name="alert" size={14} /> Failed to load settings: {err}</div>;
  if (!m) return <div className="content-pad" style={{ color: 'var(--text-dim)', fontSize: 13 }}>Loading…</div>;

  const set = (k, v) => setM(s => ({ ...s, [k]: { ...s[k], default: v } }));
  const save = () => {
    setSaving(true);
    apiFetch('/api/manage', { method: 'PUT', body: JSON.stringify(m) })
      .then(updated => { setM(updated); setSaved(true); setTimeout(() => setSaved(false), 2000); })
      .catch(console.error)
      .finally(() => setSaving(false));
  };

  const Row = ({ label, note, children }) => (
    <div style={{ display: 'grid', gridTemplateColumns: '230px 1fr', gap: 20, padding: '16px 18px', borderTop: '1px solid var(--border-soft)', alignItems: 'center' }}>
      <div className="col" style={{ gap: 3 }}>
        <span style={{ fontSize: 13, color: 'var(--text)' }}>{label}</span>
        {note && <span style={{ fontSize: 11, color: 'var(--text-faint)', lineHeight: 1.4 }}>{note}</span>}
      </div>
      <div>{children}</div>
    </div>
  );

  return (
    <div className="content-pad fade-in" style={{ maxWidth: 820 }}>
      <div className="pagehead"><div>
        <div className="kicker" style={{ marginBottom: 8 }}>Manage · manage.json</div>
        <h1>OxideMC settings</h1>
        <p>Global preferences applied to every server OxideMC creates and manages.</p>
      </div>
      <button className="btn primary" onClick={save} disabled={saving}>
        {saved ? <><Icon name="check" size={14} />Saved</> : saving ? 'Saving…' : 'Save changes'}
      </button>
      </div>

      <div className="panel" style={{ overflow: 'hidden' }}>
        <div className="panel-h"><span style={{ color: 'var(--accent-bright)' }}><Icon name="folder" size={15} /></span><span className="t">Paths</span></div>
        <Row label="Servers directory" note="Parent folder for new servers."><input className="input" value={m.servers_directory.default} onChange={e => set('servers_directory', e.target.value)} /></Row>
        <Row label="Java path" note="Empty = use java found on $PATH."><input className="input" value={m.java_path.default} placeholder="auto-detect" onChange={e => set('java_path', e.target.value)} /></Row>
        <Row label="Backup directory"><input className="input" value={m.backup_directory.default} onChange={e => set('backup_directory', e.target.value)} /></Row>
        <Row label="Backup count" note="Rolling backups kept per server. 0 = unlimited.">
          <input className="input" value={m.backup_count.default} onChange={e => set('backup_count', parseInt(e.target.value.replace(/\D/g,''),10)||0)} style={{ maxWidth: 120 }} /></Row>
      </div>

      <div className="panel" style={{ overflow: 'hidden', marginTop: 16 }}>
        <div className="panel-h"><span style={{ color: 'var(--accent-bright)' }}><Icon name="settings" size={15} /></span><span className="t">Behavior</span></div>
        <Row label="Theme" note="TUI color theme.">
          <select className="select" value={m.theme.default} onChange={e => set('theme', e.target.value)} style={{ maxWidth: 200 }}>
            {['default', 'green', 'ocean', 'lava'].map(o => <option key={o}>{o}</option>)}
          </select></Row>
        <Row label="Auto-update check" note="Check for OxideMC updates on launch."><Toggle on={m.auto_update_check.default} onChange={v => set('auto_update_check', v)} /></Row>
        <Row label="Java version check" note="Warn if Java is incompatible with the MC version."><Toggle on={m.check_java_version.default} onChange={v => set('check_java_version', v)} /></Row>
      </div>
    </div>
  );
}

// ── root app ────────────────────────────────────────────────────────────────
const ACCENT_HEX = '#E8561A';

function App() {
  const [servers, setServers] = useStateA([]);
  const [route, setRoute] = useStateA({ view: 'servers', id: null, tab: 'monitor' });
  const [hints, setHints] = useStateA([]);

  const current = servers.find(s => s.id === route.id);

  const loadServers = () =>
    apiFetch('/api/servers')
      .then(list => setServers(prev => list.map(s => {
        const existing = prev.find(p => p.id === s.name);
        // preserve live metrics/players from WS; only update status + static fields
        return summaryToServer({ ...s, ...(existing ? { players: existing.players, cpu: existing.cpu, ram: existing.ram, tps: existing.tps, uptimeH: existing.uptimeH } : {}) });
      })))
      .catch(console.error);

  useEffectA(() => {
    loadServers();
    const iv = setInterval(loadServers, 5000);
    return () => clearInterval(iv);
  }, []);

  const doAction = (id, action) => {
    setServers(prev => prev.map(s => {
      if (s.id !== id) return s;
      if (action === 'stop') return { ...s, status: 'stopped', players: [], cpu: 0, ram: 0, tps: 0 };
      if (action === 'start' || action === 'restart') return { ...s, status: 'booting' };
      return s;
    }));
    apiFetch(`/api/servers/${id}/${action}`, { method: 'POST' })
      .then(() => loadServers())
      .catch(err => { console.error(err); loadServers(); });
  };

  const changeServer = (id, k, v) => setServers(prev => prev.map(s => s.id === id ? { ...s, [k]: v } : s));

  const saveServer = (id) => {
    const s = servers.find(p => p.id === id);
    if (!s) return;
    const state = flatToServerState(s.name, s.type, s.version, s);
    apiFetch(`/api/servers/${id}`, { method: 'PUT', body: JSON.stringify(state) }).catch(console.error);
  };

  // default footer hints per view
  useEffectA(() => {
    if (route.view === 'servers') setHints([{ keys: ['↑↓'], label: 'navigate' }, { keys: ['↵'], label: 'open' }, { keys: ['N'], label: 'new server' }]);
    else if (route.view === 'server') setHints([{ keys: ['Esc'], label: 'back' }, { keys: ['Tab'], label: 'switch tab' }, { keys: ['S'], label: route.tab === 'monitor' ? 'start / stop' : 'save' }]);
    else if (route.view === 'settings') setHints([{ keys: ['Esc'], label: 'back' }, { keys: ['↵'], label: 'apply' }]);
  }, [route.view, route.tab]);

  const go = (view, extra = {}) => setRoute(r => ({ ...r, view, ...extra }));

  useEffectA(() => {
    const handler = (e) => {
      const tag = e.target.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || e.target.isContentEditable) return;
      if (route.view === 'servers') {
        if (e.key === 'n' || e.key === 'N') go('new');
      } else if (route.view === 'server') {
        if (e.key === 'Escape') go('servers', { id: null });
        if (e.key === 'Tab') { e.preventDefault(); setRoute(r => ({ ...r, tab: r.tab === 'monitor' ? 'configure' : 'monitor' })); }
        if ((e.key === 's' || e.key === 'S') && current) {
          if (route.tab === 'monitor') doAction(current.id, current.status === 'running' ? 'stop' : 'start');
          else saveServer(current.id);
        }
      } else if (route.view === 'settings' || route.view === 'new') {
        if (e.key === 'Escape') go('servers', { id: null });
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [route, current]);

  return (
    <div className="app" data-bg="deep" data-border="boxed" style={{ ['--accent']: ACCENT_HEX }}>
      {/* sidebar */}
      <aside className="sidebar">
        <div className="brand">
          <svg width="34" height="34" viewBox="0 0 34 34" fill="none" style={{ flex: 'none' }}>
            <rect width="34" height="34" rx="8" fill="var(--accent)" opacity="0.15"/>
            <rect x="1" y="1" width="32" height="32" rx="7" stroke="var(--accent)" strokeWidth="1.2" opacity="0.5"/>
            <rect x="7" y="8" width="20" height="7" rx="2" fill="var(--accent)" opacity="0.85"/>
            <rect x="7" y="19" width="20" height="7" rx="2" fill="var(--accent)" opacity="0.55"/>
            <circle cx="10.5" cy="11.5" r="1.5" fill="var(--accent-bright)"/>
            <circle cx="10.5" cy="22.5" r="1.5" fill="var(--accent-bright)" opacity="0.6"/>
          </svg>
          <div className="col">
            <span className="brand-name">Oxide<b>MC</b></span>
            <span className="brand-sub">server manager</span>
          </div>
        </div>
        <nav className="nav">
          <div className="nav-group-label">Workspace</div>
          <div className={'nav-item' + (route.view === 'servers' || route.view === 'server' ? ' active' : '')} onClick={() => go('servers', { id: null })}>
            <Icon name="servers" size={16} /> Servers <span className="count">{servers.length}</span>
          </div>
          <div className={'nav-item' + (route.view === 'settings' ? ' active' : '')} onClick={() => go('settings')}>
            <Icon name="settings" size={16} /> Settings
          </div>
          <div className="nav-cta" onClick={() => go('new')}>
            <Icon name="plus" size={15} /> New Server
          </div>

          {servers.length > 0 && <>
            <div className="nav-group-label">Quick access</div>
            {servers.slice(0, 5).map(s => (
              <div key={s.id} className={'nav-item' + (route.id === s.id ? ' active' : '')} onClick={() => go('server', { id: s.id, tab: 'monitor' })}>
                <span className={`dot ${STATUS_CLASS[s.status]}`} style={{ marginLeft: 3, marginRight: 2 }} />
                <span className="truncate" style={{ fontSize: 12.5 }}>{s.name}</span>
              </div>
            ))}
          </>}
        </nav>
        <div className="sidebar-foot">
          <div className="divider" />
          <div className="sb-meta"><span>v1.0.0</span><span className="row" style={{ gap: 5 }}><span className="dot run" />daemon ok</span></div>
        </div>
      </aside>

      {/* main */}
      <div className="main">
        <div className="content">
          {route.view === 'servers' && <ServersHome servers={servers} onOpen={id => go('server', { id, tab: 'monitor' })} onAction={doAction} onNew={() => go('new')} />}
          {route.view === 'server' && current && (
            <ServerDetail server={current} tab={route.tab} setTab={tab => setRoute(r => ({ ...r, tab }))}
              onAction={doAction} onBack={() => go('servers', { id: null })} onChange={changeServer}
              onSave={saveServer} monitorLayout="grid" settingsTreatment="grouped" />
          )}
          {route.view === 'new' && (
            <NewServerWizard layout="stepped" settingsTreatment="grouped"
              setFooterHints={setHints}
              onCancel={() => go('servers')}
              onDone={({ name }) => {
                go('server', { id: name, tab: 'monitor' });
                doAction(name, 'start');
              }} />
          )}
          {route.view === 'settings' && <GlobalSettings />}
        </div>
        <Footer hints={hints} right={
          <div className="status-mini"><span className="dot run" />OxideMC daemon · localhost:7878</div>
        } />
      </div>

    </div>
  );
}

function bootUp(s) {
  const had = s.players && s.players.length;
  const players = had ? s.players : MC_NAMES.slice(0, 2 + Math.floor(Math.random() * 3));
  return { ...s, status: 'running', players, cpu: 18 + Math.floor(Math.random() * 30), ram: 30 + Math.floor(Math.random() * 35), tps: +(19.4 + Math.random() * 0.6).toFixed(1), uptimeH: s.uptimeH || 1 };
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
