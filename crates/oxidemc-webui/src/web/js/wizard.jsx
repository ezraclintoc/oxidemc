// ============================================================================
// wizard.jsx -- "New Server" install wizard.
//   layout: 'single' | 'split' | 'stepped'   (a variation dimension)
// ============================================================================
const { useState: useStateW, useEffect: useEffectW, useRef: useRefW } = React;

const WIZ_STEPS = [
  { key: 'name',     label: 'Name',     title: 'Name your server',     sub: 'A human-readable name. The directory is derived from it.' },
  { key: 'platform', label: 'Platform', title: 'Choose a platform',    sub: 'Server software to download and run.' },
  { key: 'version',  label: 'Version',  title: 'Choose a version',     sub: 'Minecraft version. "latest" always picks newest stable.' },
  { key: 'settings', label: 'Settings', title: 'Configure settings',   sub: 'Optional -- every field has a sensible default.' },
  { key: 'review',   label: 'Review',   title: 'Review & install',     sub: 'Confirm your choices before downloading.' },
];

function NewServerWizard({ layout = 'split', settingsTreatment = 'grouped', onDone, onCancel, setFooterHints }) {
  const [step, setStep] = useStateW(0);
  const [phase, setPhase] = useStateW('form'); // form | download | done
  const [name, setName] = useStateW('my-server');
  const [platform, setPlatform] = useStateW('paper');
  const [version, setVersion] = useStateW('latest');
  const [versions, setVersions] = useStateW([]);
  const [verLoading, setVerLoading] = useStateW(false);
  const [settings, setSettings] = useStateW(defaultSettings());
  const [progress, setProgress] = useStateW(0);
  const [installErr, setInstallErr] = useStateW(null);
  const maxReached = useRefW(0);

  // fetch real versions when platform changes or we reach the version step
  useEffectW(() => {
    if (step !== 2) return;
    setVerLoading(true);
    apiFetch(`/api/platforms/${platform}/versions`)
      .then(data => { setVersions(data.versions || []); setVersion(data.versions?.[0] || 'latest'); })
      .catch(() => setVersions([]))
      .finally(() => setVerLoading(false));
  }, [platform, step]);

  // kick off real install
  useEffectW(() => {
    if (phase !== 'download') return;
    setProgress(5);
    setInstallErr(null);
    let ws = null;

    const state = flatToServerState(name, platform, version, settings);
    apiFetch('/api/install', { method: 'POST', body: JSON.stringify({ server_name: name, server_type: platform, minecraft_version: version, state }) })
      .then(res => {
        ws = new WebSocket(wsUrl(`/ws/install/${res.job}`));
        ws.onmessage = (e) => {
          const msg = JSON.parse(e.data);
          if (msg.type === 'progress' && msg.total > 0) {
            setProgress(Math.max(5, Math.min(95, Math.round(msg.downloaded / msg.total * 90) + 5)));
          } else if (msg.type === 'done') {
            setProgress(100);
            setTimeout(() => setPhase('done'), 400);
          } else if (msg.type === 'error') {
            setInstallErr(msg.msg);
            setPhase('form');
          }
        };
      })
      .catch(err => { setInstallErr(err.message); setPhase('form'); });

    return () => { if (ws) ws.close(); };
  }, [phase]);

  const nameErr = !/^[a-z0-9][a-z0-9-_]*$/i.test(name) ? 'Use letters, digits, - or _' : null;
  const canNext = step === 0 ? !nameErr : true;
  maxReached.current = Math.max(maxReached.current, step);

  const next = () => { if (step < WIZ_STEPS.length - 1) setStep(step + 1); else setPhase('download'); };
  const back = () => { if (step > 0) setStep(step - 1); else onCancel && onCancel(); };

  // footer hints
  useEffectW(() => {
    if (!setFooterHints) return;
    if (phase === 'done') { setFooterHints([{ keys: ['Enter'], label: 'open server' }, { keys: ['N'], label: 'new server' }]); return; }
    if (phase === 'download') { setFooterHints([{ keys: ['Esc'], label: 'cancel' }]); return; }
    setFooterHints([
      { keys: ['Esc'], label: step === 0 ? 'cancel' : 'back' },
      { keys: ['↵'], label: step === WIZ_STEPS.length - 1 ? 'install' : 'continue' },
    ]);
  }, [step, phase]);

  // ─ terminal phases ─
  if (phase === 'download') return <DownloadView name={name} platform={platform} version={version} progress={progress} />;
  if (phase === 'done') return <DoneView name={name} platform={platform} version={version} onOpen={() => onDone && onDone({ name, platform, version, settings })} onNew={() => { setPhase('form'); setStep(0); maxReached.current = 0; }} />;

  const stepBody = (
    <StepBody step={step} name={name} setName={setName} nameErr={nameErr}
      platform={platform} setPlatform={setPlatform}
      version={version} setVersion={setVersion} versions={versions} verLoading={verLoading}
      settings={settings} setSettings={setSettings} settingsTreatment={settingsTreatment}
      onChange={(k, v) => setSettings(s => ({ ...s, [k]: v }))} />
  );

  const navBtns = (
    <div className="col" style={{ gap: 10, alignItems: 'flex-end' }}>
      {installErr && <span className="row" style={{ gap: 6, fontSize: 12, color: 'var(--red)' }}><Icon name="alert" size={13} />{installErr}</span>}
      <div className="row" style={{ gap: 10 }}>
        <button className="btn ghost" onClick={back}><Icon name="arrowL" size={15} />{step === 0 ? 'Cancel' : 'Back'}</button>
        <button className="btn primary" disabled={!canNext} onClick={next}>
          {step === WIZ_STEPS.length - 1 ? <><Icon name="download" size={15} />Install</> : <>Continue<Icon name="arrowR" size={15} /></>}
        </button>
      </div>
    </div>
  );

  const cur = WIZ_STEPS[step];

  // ── LAYOUT: split ─────────────────────────────────────────────────────────
  if (layout === 'split') {
    return (
      <div className="content-pad fade-in" key="split">
        <div style={{ display: 'grid', gridTemplateColumns: '232px 1fr', gap: 30, alignItems: 'start' }}>
          <div className="panel" style={{ position: 'sticky', top: 0, overflow: 'hidden' }}>
            <div className="panel-h"><span className="t">New Server</span></div>
            <div className="col" style={{ padding: 8, gap: 2 }}>
              {WIZ_STEPS.map((s, i) => (
                <button key={s.key} onClick={() => i <= maxReached.current && setStep(i)}
                  disabled={i > maxReached.current}
                  className="wiz-step" style={{
                    display: 'flex', alignItems: 'center', gap: 11, padding: '9px 10px', borderRadius: 6,
                    background: i === step ? 'var(--accent-faint)' : 'transparent', border: 0, width: '100%', textAlign: 'left',
                    cursor: i <= maxReached.current ? 'pointer' : 'default', fontFamily: 'var(--mono)',
                    color: i === step ? 'var(--accent-bright)' : i < step ? 'var(--text-mid)' : 'var(--text-faint)',
                  }}>
                  <span style={{
                    width: 22, height: 22, borderRadius: 6, flex: 'none', display: 'grid', placeItems: 'center',
                    fontSize: 11, fontWeight: 600,
                    background: i === step ? 'var(--accent)' : i < step ? 'color-mix(in oklch, var(--accent) 20%, transparent)' : 'var(--inset)',
                    color: i === step ? '#1a0f08' : i < step ? 'var(--accent-bright)' : 'var(--text-faint)',
                    border: '1px solid ' + (i <= step ? 'transparent' : 'var(--border-soft)'),
                  }}>{i < step ? <Icon name="check" size={12} /> : i + 1}</span>
                  <span style={{ fontSize: 13 }}>{s.label}</span>
                </button>
              ))}
            </div>
          </div>
          <div style={{ minWidth: 0 }}>
            <div style={{ marginBottom: 22 }}>
              <div className="kicker" style={{ marginBottom: 8 }}>Step {step + 1} / {WIZ_STEPS.length}</div>
              <h1 className="sans" style={{ margin: 0, fontSize: 25, fontWeight: 600, letterSpacing: '-.02em' }}>{cur.title}</h1>
              <p style={{ margin: '7px 0 0', color: 'var(--text-dim)', fontSize: 13 }}>{cur.sub}</p>
            </div>
            {stepBody}
            <div className="row" style={{ justifyContent: 'flex-end', marginTop: 24 }}>{navBtns}</div>
          </div>
        </div>
      </div>
    );
  }

  // ── LAYOUT: stepped (horizontal stepper) ──────────────────────────────────
  if (layout === 'stepped') {
    return (
      <div className="content-pad fade-in" key="stepped" style={{ maxWidth: 880, margin: '0 auto' }}>
        <div className="row" style={{ marginBottom: 28, gap: 0 }}>
          {WIZ_STEPS.map((s, i) => (
            <React.Fragment key={s.key}>
              <button onClick={() => i <= maxReached.current && setStep(i)} disabled={i > maxReached.current}
                className="col" style={{ alignItems: 'center', gap: 7, background: 'none', border: 0, cursor: i <= maxReached.current ? 'pointer' : 'default', flex: 'none' }}>
                <span style={{
                  width: 30, height: 30, borderRadius: '50%', display: 'grid', placeItems: 'center', fontSize: 12, fontWeight: 600,
                  fontFamily: 'var(--mono)',
                  background: i === step ? 'var(--accent)' : i < step ? 'color-mix(in oklch, var(--accent) 22%, transparent)' : 'var(--inset)',
                  color: i === step ? '#1a0f08' : i < step ? 'var(--accent-bright)' : 'var(--text-faint)',
                  border: '1px solid ' + (i <= step ? 'color-mix(in oklch, var(--accent) 50%, transparent)' : 'var(--border)'),
                  boxShadow: i === step ? '0 0 0 4px var(--accent-glow)' : 'none', transition: 'all .15s',
                }}>{i < step ? <Icon name="check" size={14} /> : i + 1}</span>
                <span style={{ fontSize: 11, color: i === step ? 'var(--accent-bright)' : 'var(--text-dim)', fontFamily: 'var(--mono)' }}>{s.label}</span>
              </button>
              {i < WIZ_STEPS.length - 1 &&
                <div style={{ flex: 1, height: 2, margin: '0 6px', marginBottom: 20, borderRadius: 2,
                  background: i < step ? 'color-mix(in oklch, var(--accent) 45%, transparent)' : 'var(--border)' }} />}
            </React.Fragment>
          ))}
        </div>
        <div className="panel" style={{ padding: '26px 28px' }}>
          <div style={{ marginBottom: 20 }}>
            <h1 className="sans" style={{ margin: 0, fontSize: 22, fontWeight: 600, letterSpacing: '-.02em' }}>{cur.title}</h1>
            <p style={{ margin: '6px 0 0', color: 'var(--text-dim)', fontSize: 12.5 }}>{cur.sub}</p>
          </div>
          {stepBody}
        </div>
        <div className="row" style={{ justifyContent: 'space-between', marginTop: 20 }}>
          <button className="btn ghost" onClick={back}><Icon name="arrowL" size={15} />{step === 0 ? 'Cancel' : 'Back'}</button>
          <button className="btn primary" disabled={!canNext} onClick={next}>
            {step === WIZ_STEPS.length - 1 ? <><Icon name="download" size={15} />Install</> : <>Continue<Icon name="arrowR" size={15} /></>}
          </button>
        </div>
      </div>
    );
  }

  // ── LAYOUT: single (centered column, pip indicator) ───────────────────────
  return (
    <div className="content-pad fade-in" key="single" style={{ maxWidth: 640, margin: '0 auto' }}>
      <div className="row" style={{ gap: 6, marginBottom: 22 }}>
        {WIZ_STEPS.map((s, i) => (
          <div key={s.key} onClick={() => i <= maxReached.current && setStep(i)} style={{
            flex: 1, height: 4, borderRadius: 2, cursor: i <= maxReached.current ? 'pointer' : 'default',
            background: i <= step ? 'var(--accent)' : 'var(--border)', transition: 'background .2s',
          }} />
        ))}
      </div>
      <div style={{ marginBottom: 22 }}>
        <div className="kicker" style={{ marginBottom: 8 }}>Step {step + 1} / {WIZ_STEPS.length} -- {cur.label}</div>
        <h1 className="sans" style={{ margin: 0, fontSize: 26, fontWeight: 600, letterSpacing: '-.02em' }}>{cur.title}</h1>
        <p style={{ margin: '7px 0 0', color: 'var(--text-dim)', fontSize: 13 }}>{cur.sub}</p>
      </div>
      {stepBody}
      <div className="row" style={{ justifyContent: 'space-between', marginTop: 26 }}>
        <button className="btn ghost" onClick={back}><Icon name="arrowL" size={15} />{step === 0 ? 'Cancel' : 'Back'}</button>
        {navBtns && <button className="btn primary" disabled={!canNext} onClick={next}>
          {step === WIZ_STEPS.length - 1 ? <><Icon name="download" size={15} />Install</> : <>Continue<Icon name="arrowR" size={15} /></>}
        </button>}
      </div>
    </div>
  );
}

// ── step bodies ─────────────────────────────────────────────────────────────
function StepBody({ step, name, setName, nameErr, platform, setPlatform, version, setVersion, versions, verLoading, settings, setSettings, settingsTreatment, onChange }) {
  if (step === 0) {
    return (
      <div className="col" style={{ gap: 9, maxWidth: 460 }}>
        <label className="kicker" style={{ fontSize: 11 }}>Server name</label>
        <div className="row" style={{ gap: 0, alignItems: 'stretch' }}>
          <span className="mono" style={{ display: 'grid', placeItems: 'center', padding: '0 12px', background: 'var(--inset)',
            border: '1px solid var(--border)', borderRight: 0, borderRadius: '5px 0 0 5px', color: 'var(--text-faint)', fontSize: 13 }}>~/servers/</span>
          <input className="input" autoFocus value={name} onChange={e => setName(e.target.value)}
            style={{ borderRadius: '0 5px 5px 0' }} />
        </div>
        {nameErr ? <span className="row" style={{ gap: 6, fontSize: 12, color: 'var(--red)' }}><Icon name="alert" size={13} />{nameErr}</span>
                 : <span style={{ fontSize: 12, color: 'var(--text-dim)' }}>Directory: <span className="val">~/servers/{name || '...'}</span></span>}
      </div>
    );
  }
  if (step === 1) {
    return (
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(232px, 1fr))', gap: 12 }}>
        {PLATFORMS.map(p => {
          const sel = platform === p.id;
          return (
            <button key={p.id} onClick={() => setPlatform(p.id)} className="plat-card" style={{
              display: 'flex', gap: 13, alignItems: 'flex-start', textAlign: 'left', padding: 14, cursor: 'pointer',
              borderRadius: 9, background: sel ? 'var(--accent-faint)' : 'var(--panel)',
              border: '1px solid ' + (sel ? 'color-mix(in oklch, var(--accent) 55%, transparent)' : 'var(--border)'),
              boxShadow: sel ? '0 0 0 3px var(--accent-glow)' : 'none', transition: 'all .14s',
            }}>
              <PlatformLogo id={p.id} tone={p.tone} />
              <div className="col" style={{ gap: 4, minWidth: 0 }}>
                <span className="sans" style={{ fontSize: 15, fontWeight: 600, color: sel ? 'var(--accent-bright)' : 'var(--text)' }}>{p.name}</span>
                <span style={{ fontSize: 11.5, color: 'var(--text-dim)', lineHeight: 1.4 }}>{p.blurb}</span>
              </div>
            </button>
          );
        })}
      </div>
    );
  }
  if (step === 2) {
    return (
      <div className="panel" style={{ maxWidth: 380, overflow: 'hidden' }}>
        <div className="panel-h"><span className="t">{PLATFORMS.find(p=>p.id===platform)?.name} versions</span>
          {verLoading && <span className="row" style={{ marginLeft: 'auto', gap: 7, color: 'var(--text-dim)', fontSize: 11 }}>
            <span className="spin" style={{ width: 11, height: 11, border: '2px solid var(--border)', borderTopColor: 'var(--accent)', borderRadius: '50%' }} />loading</span>}</div>
        <div style={{ maxHeight: 280, overflowY: 'auto' }}>
          {verLoading ? (
            <div className="col" style={{ padding: 8, gap: 6 }}>
              {[...Array(7)].map((_, i) => <div key={i} style={{ height: 30, borderRadius: 5, background: 'linear-gradient(90deg,var(--inset),var(--panel-2),var(--inset))', backgroundSize: '200% 100%', animation: 'shimmer 1.2s infinite', opacity: 1 - i*0.1 }} />)}
            </div>
          ) : (
            <div className="col" style={{ padding: 6 }}>
              {(versions.length ? versions : VERSIONS).map(v => {
                const sel = version === v;
                return (
                  <button key={v} onClick={() => setVersion(v)} style={{
                    display: 'flex', alignItems: 'center', gap: 10, padding: '9px 11px', borderRadius: 5, border: 0, cursor: 'pointer',
                    background: sel ? 'var(--accent-faint)' : 'transparent', color: sel ? 'var(--accent-bright)' : 'var(--text)',
                    fontFamily: 'var(--mono)', fontSize: 13, width: '100%', textAlign: 'left',
                  }}>
                    <span style={{ color: sel ? 'var(--accent)' : 'transparent' }}><Icon name="chevR" size={13} /></span>
                    {v}{v === 'latest' && <span className="pill tag" style={{ marginLeft: 'auto', height: 18, fontSize: 9.5 }}>recommended</span>}
                  </button>
                );
              })}
            </div>
          )}
        </div>
      </div>
    );
  }
  if (step === 3) {
    return <SettingsForm values={settings} onChange={onChange} treatment={settingsTreatment} />;
  }
  // review
  const plat = PLATFORMS.find(p => p.id === platform);
  const rows = [
    ['Server name', name], ['Platform', plat?.name], ['Version', version], ['Directory', `~/servers/${name}`],
    ['MOTD', settings.motd], ['Max players', settings.max_players], ['Gamemode', settings.gamemode],
    ['Difficulty', settings.difficulty], ['Port', settings.port], ['Max RAM', settings.max_ram],
    ['RCON', settings.rcon_enabled ? 'enabled' : 'disabled'],
  ];
  return (
    <>
      <div className="panel" style={{ maxWidth: 560, overflow: 'hidden' }}>
        <div className="panel-h"><span className="t">Summary</span><span className="row" style={{ marginLeft: 'auto', gap: 8 }}><PlatformLogo id={platform} size={20} /></span></div>
        <div style={{ padding: '6px 0' }}>
          {rows.map(([k, v], i) => (
            <div key={k} style={{ display: 'grid', gridTemplateColumns: '160px 1fr', gap: 14, padding: '8px 16px',
              borderTop: i ? '1px solid var(--border-soft)' : 0 }}>
              <span style={{ fontSize: 13, color: 'var(--text-dim)' }}>{k}</span>
              <span className="val truncate" style={{ fontSize: 13 }}>{String(v)}</span>
            </div>
          ))}
        </div>
      </div>
      <p style={{ fontSize: 11.5, color: 'var(--text-faint)', marginTop: 12, maxWidth: 560, lineHeight: 1.5 }}>
        By clicking <strong style={{ color: 'var(--text-dim)' }}>Install</strong> you agree to{' '}
        <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noopener noreferrer"
          style={{ color: 'var(--accent-bright)' }}>Mojang's End User License Agreement</a>.
      </p>
    </>
  );
}

// ── download view ───────────────────────────────────────────────────────────
function DownloadView({ name, platform, version, progress }) {
  return (
    <div className="content-pad fade-in" style={{ maxWidth: 560, margin: '60px auto 0' }}>
      <div className="col" style={{ alignItems: 'center', gap: 4, textAlign: 'center', marginBottom: 26 }}>
        <PlatformLogo id={platform} size={56} />
        <h1 className="sans" style={{ margin: '14px 0 0', fontSize: 22, fontWeight: 600 }}>Installing {name}</h1>
        <span className="mono" style={{ fontSize: 13, color: 'var(--text-dim)' }}>downloading {platform}-{version}.jar</span>
      </div>
      <div className="panel" style={{ padding: 20 }}>
        <div className="row" style={{ justifyContent: 'space-between', marginBottom: 10 }}>
          <span className="mono" style={{ fontSize: 12, color: 'var(--text-dim)' }}>{progress >= 100 ? 'finalizing...' : 'fetching JAR'}</span>
          <span className="val mono-num" style={{ fontSize: 14, fontWeight: 600 }}>{Math.round(progress)}%</span>
        </div>
        <div style={{ height: 12, borderRadius: 999, background: 'var(--inset)', overflow: 'hidden', border: '1px solid var(--border)' }}>
          <div style={{ width: progress + '%', height: '100%', borderRadius: 999, transition: 'width .18s linear',
            background: 'linear-gradient(90deg, var(--accent-deep), var(--accent), var(--accent-bright))' }} />
        </div>
        <div className="col mono" style={{ gap: 3, marginTop: 16, fontSize: 11.5, color: 'var(--text-faint)' }}>
          <span style={{ color: 'var(--green)' }}>✓ resolved {platform} build for {version}</span>
          <span style={{ color: progress > 30 ? 'var(--green)' : 'var(--text-faint)' }}>{progress > 30 ? '✓' : '.'} accepted EULA</span>
          <span style={{ color: progress > 60 ? 'var(--green)' : 'var(--text-faint)' }}>{progress > 60 ? '✓' : '.'} wrote server.properties</span>
          <span style={{ color: progress >= 100 ? 'var(--green)' : 'var(--text-faint)' }}>{progress >= 100 ? '✓' : '.'} generated start script</span>
        </div>
      </div>
    </div>
  );
}

// ── done view ───────────────────────────────────────────────────────────────
function DoneView({ name, platform, version, onOpen, onNew }) {
  return (
    <div className="content-pad fade-in" style={{ maxWidth: 520, margin: '80px auto 0', textAlign: 'center' }}>
      <div style={{ width: 64, height: 64, borderRadius: '50%', margin: '0 auto 18px', display: 'grid', placeItems: 'center',
        background: 'color-mix(in oklch, var(--green) 16%, transparent)', border: '1px solid color-mix(in oklch, var(--green) 45%, transparent)', color: 'var(--green)' }}>
        <Icon name="check" size={30} />
      </div>
      <h1 className="sans" style={{ margin: 0, fontSize: 26, fontWeight: 600 }}>"{name}" installed</h1>
      <p style={{ color: 'var(--text-dim)', fontSize: 13.5, margin: '10px 0 24px' }}>
        {PLATFORMS.find(p=>p.id===platform)?.name} {version} is ready in <span className="val">~/servers/{name}</span>.
      </p>
      <div className="row" style={{ justifyContent: 'center', gap: 10 }}>
        <button className="btn ghost" onClick={onNew}><Icon name="plus" size={15} />New server</button>
        <button className="btn primary" onClick={onOpen}><Icon name="play" size={15} />Open & start</button>
      </div>
    </div>
  );
}

const __wizKeyframes = document.createElement('style');
__wizKeyframes.textContent = '@keyframes shimmer{0%{background-position:200% 0}100%{background-position:-200% 0}}@keyframes spin{to{transform:rotate(360deg)}}.spin{animation:spin .7s linear infinite}';
document.head.appendChild(__wizKeyframes);

Object.assign(window, { NewServerWizard, WIZ_STEPS });
