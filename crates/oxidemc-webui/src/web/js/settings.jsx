// ============================================================================
// settings.jsx -- settings form used by the Configure tab AND the install wizard.
//   treatment: 'inline' | 'drawer' | 'grouped'   (a variation dimension)
// ============================================================================
const { useState: useStateS, useRef: useRefS, useEffect: useEffectS } = React;

function validateField(f, v, values) {
  if (f.kind === 'number') {
    if (v === '' || isNaN(+v)) return 'Enter a number';
    if (+v < f.min || +v > f.max) return `Must be ${f.min}-${f.max}`;
  }
  if (f.kind === 'ram') {
    if (!/^\d+[GMgm]$/.test(v)) return 'Format: digits + G or M (e.g. 2G, 512M)';
  }
  if (f.key === 'rcon_password' && values.rcon_enabled && !v) return 'Required while RCON is enabled';
  return null;
}

// raw control for a single field (shared by grouped mode + editors)
function FieldControl({ f, value, values, onChange, autoFocus }) {
  const disabled = f.dep && !values[f.dep];
  if (f.kind === 'toggle') {
    return <Toggle on={!!value} onChange={onChange} disabled={disabled} />;
  }
  if (f.kind === 'choice') {
    const segmented = f.options.length <= 4 && f.options.every(o => o.length <= 10);
    if (segmented) {
      return (
        <div className="seg" style={{ display: 'flex', background: 'var(--inset)', border: '1px solid var(--border)', borderRadius: 6, padding: 2, gap: 2, flexWrap: 'wrap' }}>
          {f.options.map(o => (
            <button key={o} onClick={() => onChange(o)}
              style={{
                flex: '1 1 auto', minWidth: 64, height: 28, border: 0, borderRadius: 4, cursor: 'pointer',
                fontFamily: 'var(--mono)', fontSize: 12, textTransform: 'capitalize',
                background: value === o ? 'var(--accent)' : 'transparent',
                color: value === o ? '#1a0f08' : 'var(--text-mid)', fontWeight: value === o ? 600 : 400,
                transition: 'background .12s, color .12s',
              }}>{o}</button>
          ))}
        </div>
      );
    }
    return (
      <select className="select" value={value} disabled={disabled} onChange={e => onChange(e.target.value)}>
        {f.options.map(o => <option key={o} value={o}>{o}</option>)}
      </select>
    );
  }
  const type = f.kind === 'password' ? 'text' : f.kind === 'number' ? 'text' : 'text';
  return (
    <input className="input" type={type} value={value ?? ''} disabled={disabled} autoFocus={autoFocus}
      placeholder={f.kind === 'ram' ? '2G' : ''}
      onChange={e => onChange(f.kind === 'number' ? e.target.value.replace(/[^\d-]/g, '') : e.target.value)} />
  );
}

// editor body (used in popover + drawer): label, control, hint, validation, actions
function FieldEditor({ f, values, draft, setDraft, onSave, onCancel }) {
  const err = validateField(f, draft, values);
  const rangeHint = f.kind === 'number' ? `${f.min}-${f.max}` : f.kind === 'ram' ? 'digits + G/M' : f.unit || '';
  return (
    <div className="col" style={{ gap: 12 }}>
      <div className="row" style={{ justifyContent: 'space-between' }}>
        <span style={{ fontSize: 12, letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--border-title)' }}>{f.label}</span>
        {rangeHint && <span className="mono" style={{ fontSize: 11, color: 'var(--text-faint)' }}>{rangeHint}</span>}
      </div>
      <FieldControl f={f} value={draft} values={values} autoFocus
        onChange={(v) => { if (f.kind === 'toggle' || f.kind === 'choice') { setDraft(v); } else setDraft(v); }} />
      {f.note && !err && <span style={{ fontSize: 11.5, color: 'var(--text-dim)', lineHeight: 1.45 }}>{f.note}</span>}
      {err && <span className="row" style={{ gap: 6, fontSize: 11.5, color: 'var(--red)' }}><Icon name="alert" size={13} />{err}</span>}
      <div className="row" style={{ justifyContent: 'flex-end', gap: 8, marginTop: 2 }}>
        <button className="btn sm ghost" onClick={onCancel}>Cancel</button>
        <button className="btn sm primary" disabled={!!err} onClick={() => onSave(coerce(f, draft))}>Save</button>
      </div>
    </div>
  );
}

function coerce(f, v) {
  if (f.kind === 'number') return Math.max(f.min, Math.min(f.max, parseInt(v, 10) || f.min));
  return v;
}

// read-only row used in inline + drawer treatments
function SettingRow({ f, values, active, onClick }) {
  const disabled = f.dep && !values[f.dep];
  return (
    <button className="set-row" onClick={onClick} disabled={disabled}
      style={{
        display: 'grid', gridTemplateColumns: '180px 1fr auto', alignItems: 'center', gap: 14,
        width: '100%', textAlign: 'left', padding: '11px 14px', cursor: disabled ? 'not-allowed' : 'pointer',
        background: active ? 'var(--accent-faint)' : 'transparent', border: 0,
        borderLeft: '2px solid ' + (active ? 'var(--accent)' : 'transparent'),
        color: 'var(--text)', fontFamily: 'var(--mono)', opacity: disabled ? .45 : 1,
        transition: 'background .12s',
      }}>
      <span style={{ fontSize: 13, color: active ? 'var(--accent-bright)' : 'var(--text-mid)' }}>{f.label}</span>
      <span className="val truncate" style={{ fontSize: 13 }}>{displayVal(f, values[f.key])}</span>
      <span style={{ color: 'var(--text-faint)' }}><Icon name="chevR" size={14} /></span>
    </button>
  );
}

function SettingsForm({ values, onChange, treatment = 'inline' }) {
  const [editing, setEditing] = useStateS(null); // field key
  const [draft, setDraft] = useStateS(null);
  const [anchor, setAnchor] = useStateS(null);

  const openEditor = (f, e) => {
    if (f.dep && !values[f.dep]) return;
    setEditing(f.key); setDraft(values[f.key]);
    if (treatment === 'inline' && e) {
      const r = e.currentTarget.getBoundingClientRect();
      const host = e.currentTarget.closest('.content').getBoundingClientRect();
      setAnchor({ top: r.bottom - host.top + 6, left: r.left - host.left + 16 });
    }
  };
  const save = (val) => { onChange(editing, val); setEditing(null); };
  const fieldByKey = (k) => SETTINGS_SCHEMA.flatMap(g => g.fields).find(f => f.key === k);

  return (
    <div className="col" style={{ gap: 18, position: 'relative' }}>
      {SETTINGS_SCHEMA.map(group => (
        <div className="panel" key={group.group}>
          <div className="panel-h">
            <span style={{ color: 'var(--accent-bright)' }}><Icon name={group.icon} size={15} /></span>
            <span className="t">{group.group}</span>
          </div>
          <div style={{ padding: treatment === 'grouped' ? 16 : 0 }}>
            {treatment === 'grouped' ? (
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 16 }}>
                {group.fields.map(f => {
                  const err = validateField(f, values[f.key], values);
                  const disabled = f.dep && !values[f.dep];
                  return (
                    <div className="col" key={f.key} style={{ gap: 7, opacity: disabled ? .5 : 1 }}>
                      <div className="row" style={{ justifyContent: 'space-between' }}>
                        <label style={{ fontSize: 12.5, color: 'var(--text-mid)' }}>{f.label}</label>
                        {(f.kind === 'number' || f.kind === 'ram') &&
                          <span className="mono" style={{ fontSize: 10.5, color: 'var(--text-faint)' }}>
                            {f.kind === 'number' ? `${f.min}-${f.max}` : 'G/M'}</span>}
                      </div>
                      <FieldControl f={f} value={values[f.key]} values={values}
                        onChange={(v) => onChange(f.key, coerce(f, v))} />
                      {err ? <span className="row" style={{ gap: 5, fontSize: 11, color: 'var(--red)' }}><Icon name="alert" size={12} />{err}</span>
                           : f.note ? <span style={{ fontSize: 11, color: 'var(--text-faint)', lineHeight: 1.4 }}>{f.note}</span> : null}
                    </div>
                  );
                })}
              </div>
            ) : (
              <div>
                {group.fields.map((f, i) => (
                  <div key={f.key}>
                    {i > 0 && <div className="divider" style={{ margin: '0 14px' }} />}
                    <SettingRow f={f} values={values} active={editing === f.key}
                      onClick={(e) => openEditor(f, e)} />
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      ))}

      {/* inline popover editor */}
      {treatment === 'inline' && editing && anchor && (
        <>
          <div onClick={() => setEditing(null)} style={{ position: 'fixed', inset: 0, zIndex: 40 }} />
          <div className="panel fade-in" style={{
            position: 'absolute', top: anchor.top, left: Math.min(anchor.left, 520), width: 320, zIndex: 41,
            background: 'var(--panel-2)', boxShadow: '0 18px 50px #000a', padding: 16,
          }}>
            <FieldEditor f={fieldByKey(editing)} values={values} draft={draft} setDraft={setDraft}
              onSave={save} onCancel={() => setEditing(null)} />
          </div>
        </>
      )}

      {/* drawer editor */}
      {treatment === 'drawer' && editing && (
        <>
          <div onClick={() => setEditing(null)} className="fade-in"
            style={{ position: 'fixed', inset: 0, background: '#0008', zIndex: 60 }} />
          <div style={{
            position: 'fixed', top: 0, right: 0, bottom: 'var(--footer-h)', width: 360, zIndex: 61,
            background: 'var(--panel)', borderLeft: '1px solid var(--border)', boxShadow: '-18px 0 50px #0007',
            padding: 22, animation: 'drawerIn .22s ease both',
          }}>
            <div className="row" style={{ justifyContent: 'space-between', marginBottom: 18 }}>
              <span className="kicker">Edit field</span>
              <button className="btn icon sm ghost" onClick={() => setEditing(null)}><Icon name="arrowR" size={15} /></button>
            </div>
            <FieldEditor f={fieldByKey(editing)} values={values} draft={draft} setDraft={setDraft}
              onSave={save} onCancel={() => setEditing(null)} />
          </div>
        </>
      )}
    </div>
  );
}

const __drawerKeyframes = document.createElement('style');
__drawerKeyframes.textContent = '@keyframes drawerIn{from{transform:translateX(100%)}to{transform:none}}';
document.head.appendChild(__drawerKeyframes);

Object.assign(window, { SettingsForm, FieldControl, FieldEditor, validateField, coerce });
