import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import {
  Network,
  Shield,
  Link2,
  Layers,
  Boxes,
  Monitor,
  RefreshCw,
  Plus,
  Search,
  CheckCircle2,
  AlertTriangle,
  Gauge,
  Settings2,
  Cloud,
  Server,
  BarChart3,
  Activity,
  Trash2,
  Pencil,
  X,
  ExternalLink,
  ArrowUpRight,
} from 'lucide-react'
import {
  networkApi,
  type NetworkInterface,
  type ZoneInfo,
  type NetworkSummary,
  type PppoeProfile,
  type PppoeSession,
  type PppoeState,
  type VpnConnection,
  type VpnWanState,
} from '../../api/network'
import '../../styles/network.css'

/* ───────────────────────── helpers ───────────────────────── */

function fmtBytes(n: number): string {
  if (n >= 1 << 30) return `${(n / (1 << 30)).toFixed(2)} GB`
  if (n >= 1 << 20) return `${(n / (1 << 20)).toFixed(1)} MB`
  if (n >= 1 << 10) return `${(n / (1 << 10)).toFixed(1)} KB`
  return `${n} B`
}

function fmtPct(p: number): string {
  const v = p >= 0 ? `+${p.toFixed(0)}%` : `${p.toFixed(0)}%`
  return v
}

/** رنگ زون‌ها مثل مدآپ: Internal آبی، WAN بنفش، DMZ کهربایی، Guest نارنجی */
const ZONE_COLORS: Record<string, string> = {
  lan: '#3b82f6',
  internal: '#3b82f6',
  wan: '#a78bfa',
  dmz: '#f59e0b',
  guest: '#f97316',
  vpn: '#2dd4bf',
  mgmt: '#22d3ee',
  management: '#22d3ee',
}
const FALLBACK_COLOR = '#7d93b2'

function zoneColor(name: string): string {
  return ZONE_COLORS[name.toLowerCase()] ?? FALLBACK_COLOR
}

function hexAlpha(hex: string, alpha: number): string {
  const h = hex.replace('#', '')
  const r = parseInt(h.slice(0, 2), 16)
  const g = parseInt(h.slice(2, 4), 16)
  const b = parseInt(h.slice(4, 6), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

function ZonePill({ name, zones }: { name: string; zones: ZoneInfo[] }) {
  const c = zoneColor(name)
  const label = zones.find((z) => z.name === name)?.display_name ?? name
  return (
    <span className="zone-pill" style={{ color: c, background: hexAlpha(c, 0.14) }}>
      {label}
    </span>
  )
}

function ifaceType(name: string): string {
  const n = name.toLowerCase()
  if (/^ppp/.test(n)) return 'PPP'
  if (/^(tun|tap|wg)/.test(n)) return 'Tunnel'
  if (/^br/.test(n)) return 'Bridge'
  if (/^bond/.test(n)) return 'Bond'
  if (n.includes('.')) return 'VLAN'
  if (/^(wl|wlp)/.test(n)) return 'WiFi'
  if (/^(en|eth|ens|enp)/.test(n)) return 'Physical'
  if (/^(dummy|veth|docker|virbr)/.test(n)) return 'Virtual'
  return 'Other'
}

type Toast = { kind: 'ok' | 'err'; text: string }

/* ───────────────────────── donut (SVG) ───────────────────────── */

function Donut({
  slices,
  centerValue,
  centerLabel,
}: {
  slices: { label: string; value: number; color: string }[]
  centerValue: string
  centerLabel: string
}) {
  const total = slices.reduce((s, x) => s + x.value, 0)
  const R = 34
  const C = 2 * Math.PI * R
  let offset = 0

  return (
    <div className="donut-wrap">
      <svg width="96" height="96" viewBox="0 0 96 96">
        <circle cx="48" cy="48" r={R} fill="none" stroke="var(--bg)" strokeWidth="12" />
        {total > 0 &&
          slices.map((s, i) => {
            const frac = s.value / total
            const dash = frac * C
            const el = (
              <circle
                key={i}
                cx="48"
                cy="48"
                r={R}
                fill="none"
                stroke={s.color}
                strokeWidth="12"
                strokeDasharray={`${dash} ${C - dash}`}
                strokeDashoffset={-offset}
                transform="rotate(-90 48 48)"
                strokeLinecap="butt"
              />
            )
            offset += dash
            return el
          })}
        <text x="48" y="45" textAnchor="middle" fontSize="17" fontWeight="700" fill="var(--text)">
          {centerValue}
        </text>
        <text x="48" y="58" textAnchor="middle" fontSize="7.5" fill="var(--text-muted)">
          {centerLabel}
        </text>
      </svg>
      <div className="legend">
        {slices.map((s, i) => (
          <div className="legend-row" key={i}>
            <span className="swatch" style={{ background: s.color }} />
            <span className="l-name">{s.label}</span>
            <span className="l-val">{s.value}</span>
            <span className="l-pct">{total ? `${Math.round((s.value / total) * 100)}%` : '0%'}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

/* ───────────────────────── topology panel ───────────────────────── */

function TopologyPanel({
  ifaces,
  zones,
}: {
  ifaces: NetworkInterface[]
  zones: ZoneInfo[]
}) {
  const up = ifaces.filter((i) => i.up).length
  const down = ifaces.filter((i) => !i.up && i.enabled).length
  const off = ifaces.filter((i) => !i.enabled).length

  const byZone = useMemo(() => {
    const m = new Map<string, NetworkInterface[]>()
    for (const i of ifaces) {
      const arr = m.get(i.zone) ?? []
      arr.push(i)
      m.set(i.zone, arr)
    }
    return [...m.entries()].sort((a, b) => a[0].localeCompare(b[0]))
  }, [ifaces])

  return (
    <div className="topo-panel">
      <div className="topo-head">
        <h3>Network Topology</h3>
      </div>

      <div className="topo-flow">
        <div className="topo-node">
          <span className="t-icon">
            <Cloud size={20} />
          </span>
          <span className="t-label">Internet</span>
          <span className="t-sub">uplink</span>
        </div>
        <div className="topo-link" />
        <div className="topo-node">
          <span className="t-icon">
            <Server size={20} />
          </span>
          <span className="t-label">DEZH</span>
          <span className="t-sub">
            {ifaces.length} interface{ifaces.length === 1 ? '' : 's'}
          </span>
        </div>

        {byZone.map(([zone, list]) => (
          <div className="topo-zone" key={zone}>
            <div className="z-head">
              <ZonePill name={zone} zones={zones} />
              <span className="cell-muted" style={{ fontSize: 11 }}>
                {list.length} iface{list.length === 1 ? '' : 's'}
              </span>
            </div>
            <div className="z-ifaces">
              {list.map((i) => (
                <span className="iface-chip" key={i.name} title={`${i.name} — ${i.ipv4_mode}`}>
                  <span
                    className={`dot ${i.up ? 'up' : i.enabled ? 'down' : 'off'}`}
                  />
                  {i.name}
                </span>
              ))}
            </div>
          </div>
        ))}
        {ifaces.length === 0 && <div className="net-empty">No interfaces detected</div>}
      </div>

      <div className="topo-legend">
        <div className="legend-row">
          <span className="dot ok" /> Healthy (up) <span className="cnt">{up}</span>
        </div>
        <div className="legend-row">
          <span className="dot bad" /> Critical (down) <span className="cnt">{down}</span>
        </div>
        <div className="legend-row">
          <span className="dot off" /> Offline (disabled) <span className="cnt">{off}</span>
        </div>
      </div>
    </div>
  )
}

/* ───────────────────────── configure modal ───────────────────────── */

function ConfigureModal({
  iface,
  ifaces,
  zones,
  onClose,
  onDone,
}: {
  iface: NetworkInterface | null
  ifaces: NetworkInterface[]
  zones: ZoneInfo[]
  onClose: () => void
  onDone: (msg: string) => void
}) {
  const [name, setName] = useState(iface?.name ?? ifaces[0]?.name ?? '')
  const [zone, setZone] = useState(iface?.zone ?? 'lan')
  const [enabled, setEnabled] = useState(iface?.enabled ?? true)
  const [mode, setMode] = useState(iface?.ipv4_mode || 'none')
  const [cidr, setCidr] = useState(iface?.address_cidr ?? '')
  const [gateway, setGateway] = useState(iface?.gateway ?? '')
  const [description, setDescription] = useState(iface?.description ?? '')
  const [err, setErr] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [newName, setNewName] = useState(iface?.name ?? '')

  const isNew = iface === null

  async function submit() {
    setErr(null)

    // [N1b] تغییر نام اختیاری — فقط در حالت ویرایش اینترفیس موجود
    let target = name
    if (!isNew && newName.trim() !== name) {
      target = newName.trim()
      if (!/^[a-zA-Z0-9._@:-]{1,15}$/.test(target)) {
        setErr('Interface name: max 15 chars, allowed: a-z 0-9 . - _ @ :')
        return
      }
      if (ifaces.some((i) => i.name === target)) {
        setErr(`Interface '${target}' already exists`)
        return
      }
    }

    // [N1b-fix] rename confirm — لینک چند ثانیه down میشود؛ اگر اینترفیس
    // مدیریت را عوض کنی مرورگر NetworkError میدهد حتی اگر rename موفق بوده.
    if (!isNew && target !== name) {
      const go = window.confirm(
        `Rename ${name} → ${target}?\n\n` +
          'The link goes down for a few seconds (kernel requirement).\n' +
          'If this is your management interface, the browser will show a\n' +
          'network error even though the rename succeeds. Continue?',
      )
      if (!go) return
    }

    setBusy(true)
    try {
      if (target !== name) {
        try {
          await networkApi.renameInterface(name, target)
        } catch (re: unknown) {
          // [N1b-fix] link-blink resilience — NetworkError بعد از rename
          // معمولاً هشدار کاذب است: لینک چند ثانیه down بوده.
          const rmsg = re instanceof Error ? re.message : String(re)
          if (/networkerror|failed to fetch/i.test(rmsg)) {
            await new Promise((r) => setTimeout(r, 2500))
            const fresh = await networkApi.interfaces().catch(() => [])
            if (!fresh.some((x) => x.name === target)) {
              throw new Error(
                'Connection dropped during rename and the new name is not ' +
                  'visible yet. Refresh the page or check the VM console ' +
                  'before retrying.',
              )
            }
          } else {
            throw re
          }
        }
        // اگر apply بعدی شکست بخورد، retry فقط apply را تکرار میکند
        setName(target)
      }
      await networkApi.applyConfig({
        name: target,
        zone,
        enabled,
        ipv4_mode: mode,
        address_cidr: mode === 'static' ? cidr || null : null,
        gateway: mode === 'static' ? gateway || null : null,
        description,
      })
      onDone(target !== name ? `Renamed ${name} → ${target} — applied` : `Applied ${target}`)
    } catch (e: unknown) {
      // [N1b-fix] friendly network error
      const msg = e instanceof Error ? e.message : String(e)
      setErr(
        /networkerror|failed to fetch/i.test(msg)
          ? msg +
              ' — if you were renaming, the link blink dropped this request. ' +
              'Refresh the page to verify; the rename usually succeeded.'
          : msg,
      )
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="net-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <span className="head-icon">
            <Settings2 size={20} />
          </span>
          <div>
            <h2>{isNew ? 'Configure Interface' : `Configure: ${iface?.name}`}</h2>
            <p>Zone, IP mode and link state — applied live and persisted.</p>
          </div>
          <button className="icon-btn close-btn" onClick={onClose} aria-label="close">
            <X size={16} />
          </button>
        </div>

        <div className="modal-body-flat">
          {err && <div className="form-error">{err}</div>}

          <div className="form-field">
            <label>Interface</label>
            {isNew ? (
              <select value={name} onChange={(e) => setName(e.target.value)}>
                {ifaces.map((i) => (
                  <option key={i.name} value={i.name}>
                    {i.name}
                    {i.addresses.length ? ` — ${i.addresses.join(', ')}` : ''}
                  </option>
                ))}
              </select>
            ) : (
              <>
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  spellCheck={false}
                />
                <div className="field-hint">
                  Rename here if needed — max 15 chars (a-z 0-9 . - _ @ :).
                  Renaming brings the link down for a moment (kernel
                  requirement) and keeps zone, IP config and description.
                </div>
              </>
            )}
          </div>

          <div className="form-field">
            <label>Zone</label>
            <select value={zone} onChange={(e) => setZone(e.target.value)}>
              {zones.map((z) => (
                <option key={z.name} value={z.name}>
                  {z.display_name} ({z.name}){z.accounting ? ' · accounting' : ''}
                </option>
              ))}
            </select>
            <div className="field-hint">
              Accounting counts traffic only on zones with accounting=ON.
            </div>
          </div>

          <div className="form-field">
            <label>IPv4 Mode</label>
            <div className="segmented">
              {['none', 'static', 'dhcp'].map((m) => (
                <button key={m} type="button" className={mode === m ? 'on' : ''} onClick={() => setMode(m)}>
                  {m.toUpperCase()}
                </button>
              ))}
            </div>
          </div>

          {mode === 'static' && (
            <div className="form-row">
              <div className="form-field">
                <label>Address (CIDR)</label>
                <input
                  type="text"
                  placeholder="192.168.1.1/24"
                  value={cidr}
                  onChange={(e) => setCidr(e.target.value)}
                />
              </div>
              <div className="form-field">
                <label>Gateway (optional)</label>
                <input
                  type="text"
                  placeholder="192.168.1.254"
                  value={gateway}
                  onChange={(e) => setGateway(e.target.value)}
                />
                <div className="field-hint">Default route is managed by the Routing module.</div>
              </div>
            </div>
          )}

          <div className="form-field">
            <label>Description</label>
            <input
              type="text"
              placeholder="optional"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          <label className="toggle-row">
            <input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />
            Enabled (link up)
          </label>
        </div>

        <div className="modal-foot">
          <button className="btn-ghost" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button className="btn-primary" onClick={submit} disabled={busy || !(isNew ? name : newName.trim())}>
            {busy ? 'Applying…' : 'Apply'}
          </button>
        </div>
      </div>
    </div>
  )
}

/* ───────────────────────── zone modal ───────────────────────── */

function ZoneModal({
  zone,
  onClose,
  onDone,
}: {
  zone: ZoneInfo | null
  onClose: () => void
  onDone: (msg: string) => void
}) {
  const isEdit = zone !== null
  const [name, setName] = useState(zone?.name ?? '')
  const [displayName, setDisplayName] = useState(zone?.display_name ?? '')
  const [accounting, setAccounting] = useState(zone?.accounting ?? false)
  const [description, setDescription] = useState(zone?.description ?? '')
  const [err, setErr] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  async function submit() {
    setErr(null)
    setBusy(true)
    try {
      if (isEdit && zone) {
        await networkApi.updateZone(zone.name, {
          display_name: displayName,
          accounting,
          description,
        })
        onDone(`Zone ${zone.name} updated`)
      } else {
        await networkApi.createZone({ name, display_name: displayName, accounting, description })
        onDone(`Zone ${name} created`)
      }
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="net-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <span className="head-icon">
            <Shield size={20} />
          </span>
          <div>
            <h2>{isEdit ? `Edit Zone: ${zone?.name}` : 'New Zone'}</h2>
            <p>Zones group interfaces; accounting zones feed usage stats.</p>
          </div>
          <button className="icon-btn close-btn" onClick={onClose} aria-label="close">
            <X size={16} />
          </button>
        </div>

        <div className="modal-body-flat">
          {err && <div className="form-error">{err}</div>}

          <div className="form-row">
            <div className="form-field">
              <label>Name</label>
              <input
                type="text"
                placeholder="lan"
                value={name}
                onChange={(e) => setName(e.target.value.toLowerCase())}
                disabled={isEdit}
              />
            </div>
            <div className="form-field">
              <label>Display Name</label>
              <input
                type="text"
                placeholder="Internal"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
              />
            </div>
          </div>

          <div className="form-field">
            <label>Description</label>
            <input
              type="text"
              placeholder="optional"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={accounting}
              onChange={(e) => setAccounting(e.target.checked)}
            />
            Accounting ON — count traffic through this zone
          </label>
        </div>

        <div className="modal-foot">
          <button className="btn-ghost" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button
            className="btn-primary"
            onClick={submit}
            disabled={busy || (!isEdit && (!name || !displayName)) || (isEdit && !displayName)}
          >
            {busy ? 'Saving…' : isEdit ? 'Save' : 'Create'}
          </button>
        </div>
      </div>
    </div>
  )
}

/* ───────────────────────── zones tab ───────────────────────── */

function ZonesTab({
  zones,
  ifaces,
  onToast,
  onEdit,
  onCreate,
  reload,
}: {
  zones: ZoneInfo[]
  ifaces: NetworkInterface[]
  onToast: (t: Toast) => void
  onEdit: (z: ZoneInfo) => void
  onCreate: () => void
  reload: () => void
}) {
  const [query, setQuery] = useState('')
  const [busy, setBusy] = useState<string | null>(null)

  const list = useMemo(
    () =>
      zones.filter(
        (z) =>
          z.name.toLowerCase().includes(query.toLowerCase()) ||
          z.display_name.toLowerCase().includes(query.toLowerCase()),
      ),
    [zones, query],
  )

  async function toggleAccounting(z: ZoneInfo) {
    setBusy(z.name)
    try {
      await networkApi.updateZone(z.name, {
        display_name: z.display_name,
        accounting: !z.accounting,
        description: z.description,
      })
      onToast({ kind: 'ok', text: `Accounting ${!z.accounting ? 'ON' : 'OFF'} for ${z.name}` })
      reload()
    } catch (e: unknown) {
      onToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setBusy(null)
    }
  }

  async function onDelete(z: ZoneInfo) {
    if (!window.confirm(`Delete zone "${z.name}"?`)) return
    setBusy(z.name)
    try {
      await networkApi.deleteZone(z.name)
      onToast({ kind: 'ok', text: `Zone ${z.name} deleted` })
      reload()
    } catch (e: unknown) {
      onToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setBusy(null)
    }
  }

  return (
    <div>
      <div className="net-card">
        <div className="net-card-head">
          <h2>Zones ({zones.length})</h2>
          <div className="net-card-actions">
            <button className="btn-primary" onClick={onCreate}>
              <Plus size={15} /> New Zone
            </button>
          </div>
        </div>
        <div className="net-toolbar">
          <div className="search-box">
            <Search size={15} />
            <input
              placeholder="Search zones…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
        </div>
        <div className="net-table-wrap">
          <table className="net-table">
            <thead>
              <tr>
                <th>Zone</th>
                <th>Display Name</th>
                <th>Accounting</th>
                <th>Interfaces</th>
                <th>Description</th>
                <th style={{ width: 90 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {list.map((z) => {
                const count = ifaces.filter((i) => i.zone === z.name).length
                return (
                  <tr className="net-row" key={z.name}>
                    <td>
                      <span className="iface-name">{z.name}</span>
                    </td>
                    <td>
                      <ZonePill name={z.name} zones={zones} />
                    </td>
                    <td>
                      <button
                        className={`zone-acc-toggle ${z.accounting ? 'on' : ''}`}
                        onClick={() => toggleAccounting(z)}
                        disabled={busy === z.name}
                        title="toggle accounting"
                      >
                        {z.accounting ? '● ON' : '○ OFF'}
                      </button>
                    </td>
                    <td className="mono-cell">{count}</td>
                    <td className="cell-muted">{z.description || '—'}</td>
                    <td>
                      <div className="row-actions">
                        <button className="icon-btn" title="edit" onClick={() => onEdit(z)}>
                          <Pencil size={14} />
                        </button>
                        <button
                          className="icon-btn danger"
                          title="delete"
                          onClick={() => onDelete(z)}
                          disabled={busy === z.name}
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
          {list.length === 0 && <div className="net-empty">No zones match your search</div>}
        </div>
      </div>
    </div>
  )
}

/* ───────────────────────── connections tab (N2) ───────────────────────── */

function PppoeModal({
  profile,
  ifaces,
  onClose,
  onDone,
}: {
  profile: PppoeProfile | null
  ifaces: NetworkInterface[]
  onClose: () => void
  onDone: (msg: string) => void
}) {
  const isEdit = profile !== null
  const [name, setName] = useState(profile?.name ?? '')
  const [parent, setParent] = useState(profile?.parent_iface ?? ifaces[0]?.name ?? '')
  const [username, setUsername] = useState(profile?.username ?? '')
  const [password, setPassword] = useState('')
  const [mtu, setMtu] = useState(String(profile?.mtu ?? 1492))
  const [defaultRoute, setDefaultRoute] = useState(profile?.default_route ?? true)
  const [keepalive, setKeepalive] = useState(String(profile?.keepalive ?? 10))
  const [enabled, setEnabled] = useState(profile?.enabled ?? true)
  const [description, setDescription] = useState(profile?.description ?? '')
  const [err, setErr] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  async function submit() {
    setErr(null)
    if (!/^[a-z0-9][a-z0-9-]{0,31}$/.test(name.trim())) {
      setErr('Profile name: 1-32 chars, a-z 0-9 - (starts with a letter/digit)')
      return
    }
    if (!parent) {
      setErr('Choose the parent (ethernet) interface')
      return
    }
    if (!username.trim()) {
      setErr('Username is required')
      return
    }
    if (!isEdit && !password) {
      setErr('Password is required')
      return
    }
    const m = Number(mtu)
    if (!Number.isInteger(m) || m < 576 || m > 1500) {
      setErr('MTU must be 576–1500')
      return
    }
    setBusy(true)
    try {
      await networkApi.pppoeSave({
        name: name.trim(),
        parent_iface: parent,
        username: username.trim(),
        password,
        mtu: m,
        default_route: defaultRoute,
        keepalive: Number(keepalive) || 10,
        enabled,
        description,
      })
      onDone(isEdit ? `Profile ${name.trim()} updated` : `Profile ${name.trim()} created`)
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="net-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <span className="head-icon">
            <Link2 size={20} />
          </span>
          <div>
            <h2>{isEdit ? `Edit PPPoE: ${profile?.name}` : 'New PPPoE Profile'}</h2>
            <p>Username, password and MTU — the session dials on the parent interface.</p>
          </div>
          <button className="icon-btn close-btn" onClick={onClose} aria-label="close">
            <X size={16} />
          </button>
        </div>

        <div className="modal-body-flat">
          {err && <div className="form-error">{err}</div>}

          <div className="form-row">
            <div className="form-field">
              <label>Profile Name</label>
              <input
                type="text"
                placeholder="isp-wan"
                value={name}
                onChange={(e) => setName(e.target.value.toLowerCase())}
                disabled={isEdit}
              />
            </div>
            <div className="form-field">
              <label>Parent Interface</label>
              <select value={parent} onChange={(e) => setParent(e.target.value)}>
                {ifaces.map((i) => (
                  <option key={i.name} value={i.name}>
                    {i.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="form-row">
            <div className="form-field">
              <label>Username</label>
              <input
                type="text"
                placeholder="user@isp"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                spellCheck={false}
              />
            </div>
            <div className="form-field">
              <label>Password {isEdit ? '(blank = keep current)' : ''}</label>
              <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
            </div>
          </div>

          <div className="form-row">
            <div className="form-field">
              <label>MTU / MRU</label>
              <input type="number" min={576} max={1500} value={mtu} onChange={(e) => setMtu(e.target.value)} />
            </div>
            <div className="form-field">
              <label>LCP Keepalive (s)</label>
              <input
                type="number"
                min={1}
                max={600}
                value={keepalive}
                onChange={(e) => setKeepalive(e.target.value)}
              />
            </div>
          </div>

          <div className="form-field">
            <label>Description</label>
            <input
              type="text"
              placeholder="optional"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={defaultRoute}
              onChange={(e) => setDefaultRoute(e.target.checked)}
            />
            Default route via this session
          </label>
          <label className="toggle-row">
            <input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />
            Enabled
          </label>
        </div>

        <div className="modal-foot">
          <button className="btn-ghost" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button className="btn-primary" onClick={submit} disabled={busy}>
            {busy ? 'Saving…' : isEdit ? 'Save' : 'Create'}
          </button>
        </div>
      </div>
    </div>
  )
}

/* ───────────────────── VPN WAN modal (N3) ───────────────────── */

function VpnModal({
  conn,
  onClose,
  onDone,
}: {
  conn: VpnConnection | null
  onClose: () => void
  onDone: (msg: string) => void
}) {
  const isEdit = conn !== null
  const [name, setName] = useState(conn?.name ?? '')
  const [kind, setKind] = useState<'pptp' | 'l2tp'>(conn?.kind ?? 'pptp')
  const [server, setServer] = useState(conn?.server ?? '')
  const [username, setUsername] = useState(conn?.username ?? '')
  const [password, setPassword] = useState('')
  const [mtu, setMtu] = useState(String(conn?.mtu ?? 1400))
  const [defaultRoute, setDefaultRoute] = useState(conn?.default_route ?? false)
  const [keepalive, setKeepalive] = useState(String(conn?.keepalive ?? 10))
  const [enabled, setEnabled] = useState(conn?.enabled ?? true)
  const [description, setDescription] = useState(conn?.description ?? '')
  const [err, setErr] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  async function submit() {
    setErr(null)
    if (!/^[a-z0-9][a-z0-9-]{0,31}$/.test(name.trim())) {
      setErr('Connection name: 1-32 chars, a-z 0-9 - (starts with a letter/digit)')
      return
    }
    const srv = server.trim()
    if (!/^[a-zA-Z0-9._:-]{1,253}$/.test(srv)) {
      setErr('Server: host or IP (chars a-z 0-9 . - _ :)')
      return
    }
    if (!username.trim()) {
      setErr('Username is required (no spaces or quotes)')
      return
    }
    if (!isEdit && !password) {
      setErr('Password is required (no spaces or quotes)')
      return
    }
    const m = Number(mtu)
    if (!Number.isInteger(m) || m < 576 || m > 1500) {
      setErr('MTU must be 576–1500')
      return
    }
    setBusy(true)
    try {
      await networkApi.vpnSave({
        name: name.trim(),
        kind,
        server: srv,
        username: username.trim(),
        password,
        mtu: m,
        default_route: defaultRoute,
        keepalive: Number(keepalive) || 10,
        enabled,
        description,
      })
      onDone(isEdit ? `Connection ${name.trim()} updated` : `Connection ${name.trim()} created`)
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="net-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <span className="head-icon">
            <Shield size={20} />
          </span>
          <div>
            <h2>{isEdit ? `Edit VPN: ${conn?.name}` : 'New VPN Connection'}</h2>
            <p>PPTP and L2TP WAN uplinks — needs pptp-linux / xl2tpd on the node.</p>
          </div>
          <button className="icon-btn close-btn" onClick={onClose} aria-label="close">
            <X size={16} />
          </button>
        </div>

        <div className="modal-body-flat">
          {err && <div className="form-error">{err}</div>}

          <div className="form-row">
            <div className="form-field">
              <label>Connection Name</label>
              <input
                type="text"
                placeholder="office-vpn"
                value={name}
                onChange={(e) => setName(e.target.value.toLowerCase())}
                disabled={isEdit}
              />
            </div>
            <div className="form-field">
              <label>Type</label>
              <select
                value={kind}
                onChange={(e) => setKind(e.target.value as 'pptp' | 'l2tp')}
                disabled={isEdit}
              >
                <option value="pptp">PPTP</option>
                <option value="l2tp">L2TP</option>
              </select>
            </div>
          </div>

          <div className="form-row">
            <div className="form-field">
              <label>Server (host or IP)</label>
              <input
                type="text"
                placeholder="vpn.example.com"
                value={server}
                onChange={(e) => setServer(e.target.value)}
                spellCheck={false}
              />
            </div>
            <div className="form-field">
              <label>Username</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                spellCheck={false}
              />
            </div>
          </div>

          <div className="form-row">
            <div className="form-field">
              <label>Password {isEdit ? '(blank = keep current)' : ''}</label>
              <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
            </div>
            <div className="form-field">
              <label>MTU / MRU</label>
              <input type="number" min={576} max={1500} value={mtu} onChange={(e) => setMtu(e.target.value)} />
            </div>
          </div>

          <div className="form-row">
            <div className="form-field">
              <label>LCP Keepalive (s)</label>
              <input
                type="number"
                min={1}
                max={600}
                value={keepalive}
                onChange={(e) => setKeepalive(e.target.value)}
              />
            </div>
            <div className="form-field">
              <label>Description</label>
              <input
                type="text"
                placeholder="optional"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </div>
          </div>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={defaultRoute}
              onChange={(e) => setDefaultRoute(e.target.checked)}
            />
            Default route via this connection
          </label>
          <label className="toggle-row">
            <input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />
            Enabled
          </label>
        </div>

        <div className="modal-foot">
          <button className="btn-ghost" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button className="btn-primary" onClick={submit} disabled={busy}>
            {busy ? 'Saving…' : isEdit ? 'Save' : 'Create'}
          </button>
        </div>
      </div>
    </div>
  )
}

function ConnectionsTab({
  profiles,
  vpn,
  sessions,
  ifaces,
  onToast,
  reload,
}: {
  profiles: PppoeProfile[]
  vpn: VpnConnection[]
  sessions: PppoeSession[]
  ifaces: NetworkInterface[]
  onToast: (t: Toast) => void
  reload: () => void
}) {
  const [vpnModal, setVpnModal] = useState<{ conn: VpnConnection | null } | null>(null)
  const [modal, setModal] = useState<{ profile: PppoeProfile | null } | null>(null)
  const [busy, setBusy] = useState<string | null>(null)
  const [query, setQuery] = useState('')

  const list = useMemo(
    () =>
      profiles.filter(
        (p) =>
          p.name.toLowerCase().includes(query.toLowerCase()) ||
          p.username.toLowerCase().includes(query.toLowerCase()),
      ),
    [profiles, query],
  )

  async function act(p: PppoeProfile, action: 'start' | 'stop') {
    setBusy(p.name)
    try {
      if (action === 'start') {
        await networkApi.pppoeStart(p.name)
        onToast({ kind: 'ok', text: `Dialing ${p.name} on ${p.parent_iface}…` })
      } else {
        await networkApi.pppoeStop(p.name)
        onToast({ kind: 'ok', text: `Session ${p.name} stopped` })
      }
      reload()
    } catch (e: unknown) {
      onToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setBusy(null)
    }
  }

  async function onDelete(p: PppoeProfile) {
    if (!window.confirm(`Delete profile "${p.name}"? The session (if any) is stopped first.`)) return
    setBusy(p.name)
    try {
      await networkApi.pppoeDelete(p.name)
      onToast({ kind: 'ok', text: `Profile ${p.name} deleted` })
      reload()
    } catch (e: unknown) {
      onToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setBusy(null)
    }
  }

  async function vpnAct(c: VpnConnection, action: 'start' | 'stop') {
    setBusy(c.name)
    try {
      if (action === 'start') {
        await networkApi.vpnStart(c.name)
        onToast({ kind: 'ok', text: `Connecting ${c.kind.toUpperCase()} ${c.name} → ${c.server}…` })
      } else {
        await networkApi.vpnStop(c.name)
        onToast({ kind: 'ok', text: `Connection ${c.name} stopped` })
      }
      reload()
    } catch (e: unknown) {
      onToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setBusy(null)
    }
  }

  async function onVpnDelete(c: VpnConnection) {
    if (!window.confirm(`Delete VPN connection "${c.name}"? The session (if any) is stopped first.`)) return
    setBusy(c.name)
    try {
      await networkApi.vpnDelete(c.name)
      onToast({ kind: 'ok', text: `VPN connection ${c.name} deleted` })
      reload()
    } catch (e: unknown) {
      onToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setBusy(null)
    }
  }

  return (
    <div>
      <div className="net-card">
        <div className="net-card-head">
          <h2>PPPoE Profiles ({profiles.length})</h2>
          <div className="net-card-actions">
            <button className="btn-primary" onClick={() => setModal({ profile: null })}>
              <Plus size={15} /> New Profile
            </button>
          </div>
        </div>
        <div className="net-toolbar">
          <div className="search-box">
            <Search size={15} />
            <input
              placeholder="Search profiles…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
        </div>
        <div className="net-table-wrap">
          <table className="net-table">
            <thead>
              <tr>
                <th>Profile</th>
                <th>Parent</th>
                <th>Username</th>
                <th>Session</th>
                <th>MTU</th>
                <th>Description</th>
                <th style={{ width: 130 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {list.map((p) => (
                <tr className="net-row" key={p.name}>
                  <td>
                    <span className="iface-name">{p.name}</span>
                  </td>
                  <td className="mono-cell">{p.parent_iface}</td>
                  <td className="cell-muted">{p.username}</td>
                  <td>
                    {p.session ? (
                      <span className="status-cell up">
                        <span className="dot" /> {p.session.iface}
                        {p.session.local_ip ? ` · ${p.session.local_ip}` : ''}
                      </span>
                    ) : (
                      <span className="status-cell down">
                        <span className="dot" /> idle
                      </span>
                    )}
                  </td>
                  <td className="mono-cell">{p.mtu}</td>
                  <td className="cell-muted">{p.description || '—'}</td>
                  <td>
                    <div className="row-actions">
                      {p.session ? (
                        <button
                          className="icon-btn danger"
                          title="disconnect"
                          onClick={() => act(p, 'stop')}
                          disabled={busy === p.name}
                        >
                          <X size={14} />
                        </button>
                      ) : (
                        <button
                          className="icon-btn"
                          title="connect"
                          onClick={() => act(p, 'start')}
                          disabled={busy === p.name}
                        >
                          <Link2 size={14} />
                        </button>
                      )}
                      <button className="icon-btn" title="edit" onClick={() => setModal({ profile: p })}>
                        <Pencil size={14} />
                      </button>
                      <button
                        className="icon-btn danger"
                        title="delete"
                        onClick={() => onDelete(p)}
                        disabled={busy === p.name}
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {list.length === 0 && (
            <div className="net-empty">No profiles yet — create a PPPoE profile to dial your ISP</div>
          )}
        </div>
      </div>

      <div className="net-card">
        <div className="net-card-head">
          <h2>VPN WAN — PPTP / L2TP ({vpn.length})</h2>
          <div className="net-card-actions">
            <button className="btn-primary" onClick={() => setVpnModal({ conn: null })}>
              <Plus size={15} /> New VPN Connection</button>
          </div>
        </div>
        <div className="net-table-wrap">
          <table className="net-table">
            <thead>
              <tr>
                <th>Connection</th>
                <th>Type</th>
                <th>Server</th>
                <th>Username</th>
                <th>Session</th>
                <th style={{ width: 130 }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {vpn.map((c) => (
                <tr className="net-row" key={c.name}>
                  <td>
                    <span className="iface-name">{c.name}</span>
                  </td>
                  <td className="mono-cell">{c.kind.toUpperCase()}</td>
                  <td className="mono-cell">{c.server}</td>
                  <td className="cell-muted">{c.username}</td>
                  <td>
                    {c.session ? (
                      <span className="status-cell up">
                        <span className="dot" /> {c.session.iface}
                        {c.session.local_ip ? ` · ${c.session.local_ip}` : ''}
                      </span>
                    ) : (
                      <span className="status-cell down">
                        <span className="dot" /> idle
                      </span>
                    )}
                  </td>
                  <td>
                    <div className="row-actions">
                      {c.session ? (
                        <button
                          className="icon-btn danger"
                          title="disconnect"
                          onClick={() => vpnAct(c, 'stop')}
                          disabled={busy === c.name}
                        >
                          <X size={14} />
                        </button>
                      ) : (
                        <button
                          className="icon-btn"
                          title="connect"
                          onClick={() => vpnAct(c, 'start')}
                          disabled={busy === c.name}
                        >
                          <Link2 size={14} />
                        </button>
                      )}
                      <button className="icon-btn" title="edit" onClick={() => setVpnModal({ conn: c })}>
                        <Pencil size={14} />
                      </button>
                      <button
                        className="icon-btn danger"
                        title="delete"
                        onClick={() => onVpnDelete(c)}
                        disabled={busy === c.name}
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {vpn.length === 0 && (
            <div className="net-empty">No VPN connections yet — add a PPTP or L2TP WAN uplink</div>
          )}
        </div>
      </div>

      <div className="net-card">
        <div className="net-card-head">
          <h2>Live Sessions ({sessions.length})</h2>
          <div className="net-card-actions">
            <span className="cell-muted" style={{ fontSize: 12 }}>
              auto-refresh · 5s
            </span>
          </div>
        </div>
        <div className="net-table-wrap">
          <table className="net-table">
            <thead>
              <tr>
                <th>Interface</th>
                <th>Owner</th>
                <th>Local IP</th>
                <th>Peer IP</th>
              </tr>
            </thead>
            <tbody>
              {sessions.map((s) => (
                <tr className="net-row" key={s.iface}>
                  <td>
                    <span className="iface-name">{s.iface}</span>
                  </td>
                  <td>
                    {s.profile ? (
                      <span className="iface-name">{s.profile}</span>
                    ) : (
                      <span className="cell-muted">external</span>
                    )}
                  </td>
                  <td className="mono-cell">{s.local_ip ?? '—'}</td>
                  <td className="mono-cell">{s.remote_ip ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {sessions.length === 0 && <div className="net-empty">No live ppp sessions on this node</div>}
        </div>
      </div>

      {modal && (
        <PppoeModal
          profile={modal.profile}
          ifaces={ifaces}
          onClose={() => setModal(null)}
          onDone={(msg) => {
            setModal(null)
            onToast({ kind: 'ok', text: msg })
            reload()
          }}
        />
      )}

      {vpnModal && (
        <VpnModal
          conn={vpnModal.conn}
          onClose={() => setVpnModal(null)}
          onDone={(msg) => {
            setVpnModal(null)
            onToast({ kind: 'ok', text: msg })
            reload()
          }}
        />
      )}
    </div>
  )
}

/* ───────────────────────── main dashboard ───────────────────────── */

type TabKey = 'interfaces' | 'zones' | 'connections'

// [N2] Connections tab is live — placeholder removed
const PLACEHOLDER_TABS = [
  { key: 'vlans', label: 'VLANs', icon: Layers, soon: 'N3' },
  { key: 'segments', label: 'Network Segments', icon: Boxes, soon: 'N6' },
  { key: 'devices', label: 'Devices', icon: Monitor, soon: 'N6' },
]

export function NetworkDashboard() {
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const tabParam = searchParams.get('tab')
  const tab: TabKey =
    tabParam === 'connections'
      ? 'connections'
      : tabParam === 'zones'
        ? 'zones'
        : 'interfaces'

  const [ifaces, setIfaces] = useState<NetworkInterface[]>([])
  const [zones, setZones] = useState<ZoneInfo[]>([])
  const [summary, setSummary] = useState<NetworkSummary | null>(null)
  const [pppoe, setPppoe] = useState<PppoeState | null>(null)
  const [vpn, setVpn] = useState<VpnWanState | null>(null)
  const [loading, setLoading] = useState(false)
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)

  const [query, setQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [typeFilter, setTypeFilter] = useState('all')
  const [zoneFilter, setZoneFilter] = useState('all')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)

  const [configure, setConfigure] = useState<NetworkInterface | null | undefined>(undefined)
  const [zoneModal, setZoneModal] = useState<{ zone: ZoneInfo | null } | null>(null)
  const [toast, setToast] = useState<Toast | null>(null)

  const prevTraffic = useRef<number | null>(null)
  const trafficDelta = useRef<number | null>(null)
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  function showToast(t: Toast) {
    setToast(t)
    if (toastTimer.current) clearTimeout(toastTimer.current)
    toastTimer.current = setTimeout(() => setToast(null), 3200)
  }

  const load = useCallback(async (silent = false) => {
    if (!silent) setLoading(true)
    try {
      const [s, i, z, p, v] = await Promise.all([
        networkApi.summary(),
        networkApi.interfaces(),
        networkApi.zones(),
        // N2: PPPoE — fail-soft
        networkApi.pppoeState().catch(() => null),
        // N3: VPN WAN — fail-soft
        networkApi.vpnState().catch(() => null),
      ])
      const total = s.rx_bytes + s.tx_bytes
      /* delta٪ ترافیک بین دو refresh محاسبه میشود (مثل مدآپ +12%) */
      if (prevTraffic.current !== null && prevTraffic.current > 0) {
        trafficDelta.current = ((total - prevTraffic.current) / prevTraffic.current) * 100
      }
      prevTraffic.current = total
      setSummary(s)
      setIfaces(i)
      setZones(z)
      setPppoe(p)
      setVpn(v)
      setLastUpdate(new Date())
    } catch (e: unknown) {
      if (!silent) showToast({ kind: 'err', text: e instanceof Error ? e.message : String(e) })
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    const t = setInterval(() => load(true), 30000)
    return () => clearInterval(t)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // [N2] روی تب Connections رفرش سریع‌تر (5s)
  useEffect(() => {
    if (tab !== 'connections') return
    const t = setInterval(() => load(true), 5000)
    return () => clearInterval(t)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab])

  useEffect(() => {
    setPage(1)
  }, [query, statusFilter, typeFilter, zoneFilter, pageSize])

  function switchTab(next: TabKey) {
    setSearchParams(next === 'interfaces' ? {} : { tab: next })
  }

  /* ── derived: filters, types, paging ── */
  const typeOptions = useMemo(() => {
    const s = new Set(ifaces.map((i) => ifaceType(i.name)))
    return [...s].sort()
  }, [ifaces])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    return ifaces.filter((i) => {
      if (q && !i.name.toLowerCase().includes(q) && !i.addresses.join(' ').toLowerCase().includes(q))
        return false
      if (statusFilter === 'up' && !i.up) return false
      if (statusFilter === 'down' && (i.up || !i.enabled)) return false
      if (statusFilter === 'disabled' && i.enabled) return false
      if (typeFilter !== 'all' && ifaceType(i.name) !== typeFilter) return false
      if (zoneFilter !== 'all' && i.zone !== zoneFilter) return false
      return true
    })
  }, [ifaces, query, statusFilter, typeFilter, zoneFilter])

  const pages = Math.max(1, Math.ceil(filtered.length / pageSize))
  const curPage = Math.min(page, pages)
  const rows = filtered.slice((curPage - 1) * pageSize, curPage * pageSize)

  /* ── derived: stats ── */
  const total = summary?.total_interfaces ?? ifaces.length
  const upN = summary?.up_interfaces ?? ifaces.filter((i) => i.up).length
  const downN = summary?.down_interfaces ?? Math.max(0, total - upN)
  const traffic = (summary?.rx_bytes ?? 0) + (summary?.tx_bytes ?? 0)
  const upPct = total ? Math.round((upN / total) * 100) : 0
  const downPct = total ? Math.round((downN / total) * 100) : 0
  const delta = trafficDelta.current

  const health =
    total === 0 ? 'unknown' : downN === 0 ? 'ok' : upN >= downN ? 'warn' : 'crit'
  const healthLabel =
    health === 'ok' ? 'Healthy' : health === 'warn' ? 'Degraded' : health === 'crit' ? 'Critical' : 'Unknown'

  /* ── derived: zones donut ── */
  const zoneSlices = useMemo(() => {
    const counts = new Map<string, number>()
    for (const i of ifaces) counts.set(i.zone, (counts.get(i.zone) ?? 0) + 1)
    const named = zones
      .filter((z) => counts.has(z.name))
      .map((z) => ({
        label: z.display_name,
        value: counts.get(z.name) ?? 0,
        color: zoneColor(z.name),
      }))
    const extra = [...counts.keys()]
      .filter((name) => !zones.some((z) => z.name === name))
      .map((name) => ({ label: name, value: counts.get(name) ?? 0, color: zoneColor(name) }))
    return [...named, ...extra]
  }, [ifaces, zones])

  function statusCell(i: NetworkInterface) {
    if (!i.enabled)
      return (
        <span className="status-cell disabled">
          <span className="dot" /> Offline
        </span>
      )
    return i.up ? (
      <span className="status-cell up">
        <span className="dot" /> Up
      </span>
    ) : (
      <span className="status-cell down">
        <span className="dot" /> Down
      </span>
    )
  }

  return (
    <div>
      {/* header */}
      <div className="net-header">
        <span className="title-icon">
          <Network size={24} />
        </span>
        <div className="titles">
          <h1>Network</h1>
          <div className="subtitle">Manage and monitor your network infrastructure.</div>
        </div>
        <div className="head-actions">
          <span className={`net-status ${health === 'ok' ? '' : health === 'warn' ? 'warn' : 'crit'}`}>
            <span className="dot" />
            Network Status&nbsp;: <b>{healthLabel}</b>
          </span>
          <span className="net-last-update">
            Last Update: <b>{lastUpdate ? lastUpdate.toLocaleTimeString() : '—'}</b>
          </span>
          <button
            className={`btn-refresh ${loading ? 'spin' : ''}`}
            title="refresh"
            onClick={() => load()}
          >
            <RefreshCw size={15} />
          </button>
          <button
            className="btn-ghost"
            title="Routing is an independent module"
            onClick={() => navigate('/routing')}
          >
            <ExternalLink size={14} /> Routing
          </button>
          <button
            className="btn-primary"
            onClick={() => {
              if (tab === 'zones') {
                setZoneModal({ zone: null })
              } else if (ifaces.length) {
                setConfigure(null)
              } else {
                showToast({ kind: 'err', text: 'No interfaces detected on this node' })
              }
            }}
          >
            <Plus size={15} /> Add
          </button>
        </div>
      </div>

      {/* tabs */}
      <div className="net-tabs">
        <button
          className={`net-tab ${tab === 'interfaces' ? 'active' : ''}`}
          onClick={() => switchTab('interfaces')}
        >
          <Network size={15} /> Interfaces
        </button>
        <button
          className={`net-tab ${tab === 'zones' ? 'active' : ''}`}
          onClick={() => switchTab('zones')}
        >
          <Shield size={15} /> Zones
        </button>
        <button
          className={`net-tab ${tab === 'connections' ? 'active' : ''}`}
          onClick={() => switchTab('connections')}
        >
          <Link2 size={15} /> Connections
        </button>
        {PLACEHOLDER_TABS.map((t) => (
          <button key={t.key} className="net-tab" disabled title={`coming in ${t.soon}`}>
            <t.icon size={15} /> {t.label} <span className="soon">{t.soon}</span>
          </button>
        ))}
      </div>

      {tab === 'interfaces' && (
        <>
          {/* stat cards */}
          <div className="stat-cards">
            <div className="stat-card">
              <span className="stat-icon blue">
                <Network size={20} />
              </span>
              <div className="stat-meta">
                <div className="stat-label">Total Interfaces</div>
                <div className="stat-value">{total}</div>
                <div className="stat-extra blue">
                  {zones.length} zone{zones.length === 1 ? '' : 's'}
                </div>
              </div>
            </div>
            <div className="stat-card">
              <span className="stat-icon green">
                <CheckCircle2 size={20} />
              </span>
              <div className="stat-meta">
                <div className="stat-label">Up Interfaces</div>
                <div className="stat-value">{upN}</div>
                <div className="stat-extra green">{upPct}% up</div>
              </div>
            </div>
            <div className="stat-card">
              <span className="stat-icon red">
                <AlertTriangle size={20} />
              </span>
              <div className="stat-meta">
                <div className="stat-label">Down Interfaces</div>
                <div className="stat-value">{downN}</div>
                <div className="stat-extra red">{total ? `${downPct}% of total` : '—'}</div>
              </div>
            </div>
            <div className="stat-card">
              <span className="stat-icon purple">
                <Gauge size={20} />
              </span>
              <div className="stat-meta">
                <div className="stat-label">Traffic (RX + TX)</div>
                <div className="stat-value">{fmtBytes(traffic)}</div>
                <div className="stat-extra purple">
                  {delta !== null ? fmtPct(delta) : 'cumulative'}
                  {delta === null ? ' · since boot' : ''}
                </div>
              </div>
            </div>
          </div>

          {/* main: table + topology */}
          <div className="net-main">
            <div className="net-card">
              <div className="net-card-head">
                <h2>Interfaces</h2>
                <div className="net-card-actions">
                  <span className="cell-muted" style={{ fontSize: 12 }}>
                    Showing {filtered.length} of {ifaces.length}
                  </span>
                </div>
              </div>
              <div className="net-toolbar">
                <div className="search-box">
                  <Search size={15} />
                  <input
                    placeholder="Search interfaces…"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                  />
                  <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
                    <option value="all">All Status</option>
                    <option value="up">Up</option>
                    <option value="down">Down</option>
                    <option value="disabled">Disabled</option>
                  </select>
                  <select value={typeFilter} onChange={(e) => setTypeFilter(e.target.value)}>
                    <option value="all">All Types</option>
                    {typeOptions.map((t) => (
                      <option key={t} value={t}>
                        {t}
                      </option>
                    ))}
                  </select>
                  <select value={zoneFilter} onChange={(e) => setZoneFilter(e.target.value)}>
                    <option value="all">All Zones</option>
                    {zones.map((z) => (
                      <option key={z.name} value={z.name}>
                        {z.display_name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="net-table-wrap">
                <table className="net-table">
                  <thead>
                    <tr>
                      <th>Interface</th>
                      <th>Status</th>
                      <th>IP Address</th>
                      <th>Zone</th>
                      <th>Type</th>
                      <th>Mode</th>
                      <th style={{ width: 60 }}>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {rows.map((i) => (
                      <tr className="net-row" key={i.name}>
                        <td>
                          <div className="iface-name">{i.name}</div>
                          {i.description ? <div className="iface-desc">{i.description}</div> : null}
                        </td>
                        <td>{statusCell(i)}</td>
                        <td className="mono-cell">
                          {i.address_cidr
                            ? i.address_cidr
                            : i.addresses.length
                              ? i.addresses.join(', ')
                              : '—'}
                        </td>
                        <td>
                          <ZonePill name={i.zone} zones={zones} />
                        </td>
                        <td>{ifaceType(i.name)}</td>
                        <td className="cell-muted">{i.ipv4_mode}</td>
                        <td>
                          <div className="row-actions">
                            <button
                              className="icon-btn"
                              title="configure"
                              onClick={() => setConfigure(i)}
                            >
                              <Settings2 size={14} />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {rows.length === 0 && (
                  <div className="net-empty">No interfaces match your filters</div>
                )}
              </div>
              <div className="net-pager">
                <span className="showing">
                  Showing{' '}
                  {filtered.length === 0 ? 0 : (curPage - 1) * pageSize + 1}–
                  {Math.min(curPage * pageSize, filtered.length)} of {filtered.length} items
                </span>
                <div className="pages">
                  <button
                    className="page-btn"
                    disabled={curPage <= 1}
                    onClick={() => setPage(curPage - 1)}
                  >
                    ‹
                  </button>
                  {Array.from({ length: pages }, (_, idx) => idx + 1).map((p) => (
                    <button
                      key={p}
                      className={`page-btn ${p === curPage ? 'on' : ''}`}
                      onClick={() => setPage(p)}
                    >
                      {p}
                    </button>
                  ))}
                  <button
                    className="page-btn"
                    disabled={curPage >= pages}
                    onClick={() => setPage(curPage + 1)}
                  >
                    ›
                  </button>
                  <select
                    className="page-size"
                    value={pageSize}
                    onChange={(e) => setPageSize(Number(e.target.value))}
                  >
                    {[8, 10, 16, 32].map((n) => (
                      <option key={n} value={n}>
                        {n} / page
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>

            <TopologyPanel ifaces={ifaces} zones={zones} />
          </div>

          {/* bottom row */}
          <div className="net-charts">
            <div className="chart-card">
              <h4>
                Zones <span className="cell-muted" style={{ fontSize: 11 }}>{zones.length} total</span>
              </h4>
              {zoneSlices.length ? (
                <Donut
                  slices={zoneSlices}
                  centerValue={String(zones.length)}
                  centerLabel="Total Zones"
                />
              ) : (
                <div className="chart-placeholder">No zones yet</div>
              )}
            </div>
            <div className="chart-card">
              <h4>
                Connection Profiles
                <button className="link-btn" onClick={() => switchTab('connections')}>
                  Manage <ArrowUpRight size={13} />
                </button>
              </h4>
              {pppoe && (pppoe.profiles.length > 0 || pppoe.sessions.length > 0) ? (
                <div className="pppoe-mini">
                  {pppoe.profiles.map((p) => (
                    <div className="pppoe-mini-row" key={p.name}>
                      <span className="swatch" style={{ background: p.session ? '#22c55e' : '#7d93b2' }} />
                      <span className="l-name">{p.name}</span>
                      <span className="l-val">{p.parent_iface}</span>
                      <span className={p.session ? 'pppoe-live' : 'pppoe-idle'}>
                        {p.session ? `${p.session.iface} · ${p.session.local_ip ?? 'connecting…'}` : 'idle'}
                      </span>
                    </div>
                  ))}
                  {pppoe.sessions
                    .filter((s) => !s.profile)
                    .map((s) => (
                      <div className="pppoe-mini-row" key={s.iface}>
                        <span className="swatch" style={{ background: '#22d3ee' }} />
                        <span className="l-name">{s.iface}</span>
                        <span className="l-val">external</span>
                        <span className="pppoe-live">{s.local_ip ?? '—'}</span>
                      </div>
                    ))}
                </div>
              ) : (
                <div className="chart-placeholder">
                  <Link2 size={22} />
                  <span>No PPPoE profiles yet — create one in Connections</span>
                </div>
              )}
            </div>
            <div className="chart-card">
              <h4>
                Recent Events
                <button className="link-btn" onClick={() => navigate('/firewall/events')}>
                  View All <ArrowUpRight size={13} />
                </button>
              </h4>
              <div className="chart-placeholder">
                <Activity size={22} />
                <span>Interface and security events</span>
                <span className="ph-tag">N5 · feed</span>
              </div>
            </div>
            <div className="chart-card">
              <h4>Interface Statistics</h4>
              <div className="chart-placeholder">
                <BarChart3 size={22} />
                <span>Per-interface RX/TX history chart</span>
                <span className="ph-tag">N5 · stats</span>
              </div>
            </div>
          </div>
        </>
      )}

      {tab === 'zones' && (
        <ZonesTab
          zones={zones}
          ifaces={ifaces}
          onToast={showToast}
          onEdit={(z) => setZoneModal({ zone: z })}
          onCreate={() => setZoneModal({ zone: null })}
          reload={() => load(true)}
        />
      )}

      {tab === 'connections' && (
        <ConnectionsTab
          profiles={pppoe?.profiles ?? []}
          vpn={vpn?.connections ?? []}
          sessions={pppoe?.sessions ?? []}
          ifaces={ifaces}
          onToast={showToast}
          reload={() => load(true)}
        />
      )}

      {/* modals */}
      {configure !== undefined && ifaces.length > 0 && (
        <ConfigureModal
          iface={configure}
          ifaces={ifaces}
          zones={zones}
          onClose={() => setConfigure(undefined)}
          onDone={(msg) => {
            setConfigure(undefined)
            showToast({ kind: 'ok', text: msg })
            load(true)
          }}
        />
      )}

      {zoneModal && (
        <ZoneModal
          zone={zoneModal.zone}
          onClose={() => setZoneModal(null)}
          onDone={(msg) => {
            setZoneModal(null)
            showToast({ kind: 'ok', text: msg })
            load(true)
          }}
        />
      )}

      {toast && (
        <div className={`toast ${toast.kind}`}>
          {toast.kind === 'ok' ? <CheckCircle2 size={16} /> : <AlertTriangle size={16} />}
          {toast.text}
        </div>
      )}
    </div>
  )
}
