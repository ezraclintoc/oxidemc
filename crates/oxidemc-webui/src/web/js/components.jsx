// ============================================================================
// components.jsx — shared UI primitives for OxideMC web
// ============================================================================
const { useState, useEffect, useRef, useMemo } = React;

// ── keyboard footer hint bar ────────────────────────────────────────────────
function Kbd({ children }) { return <span className="kbd">{children}</span>; }

function Footer({ hints = [], right = null }) {
  return (
    <div className="footer">
      {hints.map((h, i) => (
        <div className="hint" key={i}>
          {h.keys.map((k, j) => <Kbd key={j}>{k}</Kbd>)}
          <span>{h.label}</span>
        </div>
      ))}
      <div className="spacer" />
      {right}
    </div>
  );
}

// ── status helpers ──────────────────────────────────────────────────────────
const STATUS_LABEL = { running:'running', stopped:'stopped', booting:'starting…', error:'error' };
const STATUS_CLASS = { running:'run', stopped:'stop', booting:'boot', error:'err' };

function StatusPill({ status }) {
  const c = STATUS_CLASS[status] || 'stop';
  return (
    <span className={`pill ${c}`}>
      <span className={`dot ${c}`} />
      {STATUS_LABEL[status] || status}
    </span>
  );
}

// ── platform logo placeholder slot (user fills later) ───────────────────────
function PlatformLogo({ id, size = 44, tone }) {
  return (
    <div className="ph" style={{
      width: size, height: size, borderRadius: 10, flex: 'none',
      flexDirection: 'column', gap: 2, position: 'relative', overflow: 'hidden',
    }}>
      <div style={{ width: size*0.34, height: size*0.34, borderRadius: 6, border: `1.5px solid ${tone || 'var(--text-faint)'}`, opacity: .55 }} />
      <span style={{ fontSize: 8, color: 'var(--text-faint)', letterSpacing: '.02em' }}>{id}.png</span>
    </div>
  );
}

// ── toggle switch ───────────────────────────────────────────────────────────
function Toggle({ on, onChange, disabled }) {
  return (
    <button
      className="ox-toggle"
      role="switch" aria-checked={on} disabled={disabled}
      onClick={() => !disabled && onChange(!on)}
      style={{
        width: 40, height: 22, borderRadius: 999, position: 'relative', flex: 'none',
        border: '1px solid ' + (on ? 'color-mix(in oklch, var(--accent) 60%, #000)' : 'var(--border)'),
        background: on ? 'var(--accent)' : 'var(--inset)',
        cursor: disabled ? 'not-allowed' : 'pointer', opacity: disabled ? .45 : 1,
        transition: 'background .15s, border-color .15s', padding: 0,
      }}>
      <span style={{
        position: 'absolute', top: 2, left: on ? 19 : 2, width: 16, height: 16, borderRadius: '50%',
        background: on ? '#1a0f08' : 'var(--text-dim)', transition: 'left .15s, background .15s',
      }} />
    </button>
  );
}

// ── radial gauge (cpu / ram) ────────────────────────────────────────────────
function Gauge({ value = 0, label, sub, size = 96, accentByLoad = false }) {
  const r = size / 2 - 8, c = 2 * Math.PI * r;
  const pct = Math.max(0, Math.min(100, value));
  const off = c * (1 - pct / 100);
  let stroke = 'var(--accent)';
  if (accentByLoad) stroke = pct > 85 ? 'var(--red)' : pct > 65 ? 'var(--amber)' : 'var(--green)';
  return (
    <div className="col" style={{ alignItems: 'center', gap: 8 }}>
      <div style={{ position: 'relative', width: size, height: size }}>
        <svg width={size} height={size} style={{ transform: 'rotate(-90deg)' }}>
          <circle cx={size/2} cy={size/2} r={r} fill="none" stroke="var(--inset)" strokeWidth="7" />
          <circle cx={size/2} cy={size/2} r={r} fill="none" stroke={stroke} strokeWidth="7"
            strokeLinecap="round" strokeDasharray={c} strokeDashoffset={off}
            style={{ transition: 'stroke-dashoffset .5s ease, stroke .3s' }} />
        </svg>
        <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
          <span className="val mono-num" style={{ fontSize: size*0.26, fontWeight: 600, lineHeight: 1 }}>{Math.round(pct)}</span>
          <span style={{ fontSize: 10, color: 'var(--text-faint)' }}>{sub || '%'}</span>
        </div>
      </div>
      {label && <span className="kicker" style={{ fontSize: 10 }}>{label}</span>}
    </div>
  );
}

// ── linear meter ────────────────────────────────────────────────────────────
function BarMeter({ value = 0, max = 100, label, valueLabel, tone }) {
  const pct = Math.max(0, Math.min(100, (value / max) * 100));
  return (
    <div className="col" style={{ gap: 6 }}>
      <div className="row" style={{ justifyContent: 'space-between' }}>
        <span style={{ fontSize: 11.5, color: 'var(--text-dim)' }}>{label}</span>
        <span className="val mono-num" style={{ fontSize: 12 }}>{valueLabel}</span>
      </div>
      <div style={{ height: 7, borderRadius: 999, background: 'var(--inset)', overflow: 'hidden', border: '1px solid var(--border-soft)' }}>
        <div style={{ width: pct + '%', height: '100%', borderRadius: 999, background: tone || 'var(--accent)', transition: 'width .5s ease' }} />
      </div>
    </div>
  );
}

// ── area / line chart (TPS, load history) ───────────────────────────────────
function AreaChart({ data = [], min = 0, max = 20, height = 120, tone = 'var(--accent)', unit = '', target }) {
  const w = 600;
  const pad = 6;
  const n = data.length;
  const pts = data.map((v, i) => {
    const x = n <= 1 ? 0 : (i / (n - 1)) * (w - pad * 2) + pad;
    const y = height - pad - ((v - min) / (max - min)) * (height - pad * 2);
    return [x, Math.max(pad, Math.min(height - pad, y))];
  });
  const line = pts.map((p, i) => (i ? 'L' : 'M') + p[0].toFixed(1) + ' ' + p[1].toFixed(1)).join(' ');
  const area = pts.length ? `${line} L ${pts[pts.length-1][0].toFixed(1)} ${height} L ${pts[0][0].toFixed(1)} ${height} Z` : '';
  const gid = 'g' + Math.round(min*100) + '_' + Math.round(max*100) + '_' + height;
  const targetY = target != null ? height - pad - ((target - min) / (max - min)) * (height - pad * 2) : null;
  return (
    <svg viewBox={`0 0 ${w} ${height}`} preserveAspectRatio="none" style={{ width: '100%', height, display: 'block' }}>
      <defs>
        <linearGradient id={gid} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={tone} stopOpacity="0.32" />
          <stop offset="100%" stopColor={tone} stopOpacity="0" />
        </linearGradient>
      </defs>
      {targetY != null && <line x1="0" y1={targetY} x2={w} y2={targetY} stroke="var(--border)" strokeWidth="1" strokeDasharray="4 4" />}
      {area && <path d={area} fill={`url(#${gid})`} />}
      {line && <path d={line} fill="none" stroke={tone} strokeWidth="2" strokeLinejoin="round" vectorEffect="non-scaling-stroke" />}
    </svg>
  );
}

// ── stat tile ───────────────────────────────────────────────────────────────
function Stat({ icon, label, value, accent }) {
  return (
    <div className="row" style={{ gap: 11, alignItems: 'center' }}>
      <div style={{ width: 34, height: 34, borderRadius: 8, display: 'grid', placeItems: 'center',
        background: 'var(--accent-faint)', border: '1px solid var(--border-soft)', color: 'var(--accent-bright)', flex: 'none' }}>
        <Icon name={icon} size={16} />
      </div>
      <div className="col" style={{ gap: 1, minWidth: 0 }}>
        <span style={{ fontSize: 10, letterSpacing: '.14em', textTransform: 'uppercase', color: 'var(--text-faint)' }}>{label}</span>
        <span className="mono-num truncate" style={{ fontSize: 14.5, fontWeight: 600, color: accent ? 'var(--amber)' : 'var(--text)' }}>{value}</span>
      </div>
    </div>
  );
}

// ── player avatar (blocky placeholder head) ─────────────────────────────────
function PlayerHead({ name, size = 26 }) {
  // deterministic warm hue per name
  let h = 0; for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) % 360;
  const c1 = `oklch(0.62 0.12 ${h})`, c2 = `oklch(0.42 0.08 ${h})`;
  return (
    <div title={name} style={{
      width: size, height: size, borderRadius: 5, flex: 'none', overflow: 'hidden',
      background: `linear-gradient(135deg, ${c1}, ${c2})`, border: '1px solid #0007',
      display: 'grid', placeItems: 'center', color: '#fff', fontSize: size*0.42, fontWeight: 700,
      fontFamily: 'var(--mono)', imageRendering: 'pixelated',
    }}>{name[0].toUpperCase()}</div>
  );
}

Object.assign(window, {
  Kbd, Footer, StatusPill, STATUS_LABEL, STATUS_CLASS, PlatformLogo,
  Toggle, Gauge, BarMeter, AreaChart, Stat, PlayerHead,
});
