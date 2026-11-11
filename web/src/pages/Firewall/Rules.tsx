import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  Shield,
  ShieldCheck,
  Ban,
  Power,
  Search,
  Plus,
  Download,
  Upload,
  Pencil,
  Trash2,
  X,
  Server,
  Globe,
  Gauge,
  Clock,
  CheckCircle2,
  AlertTriangle,
} from 'lucide-react'
import {
  firewallApi,
  fmtBytes,
  type FirewallRule,
  type IpSet,
  type FqdnObject,
  type RuleStat,
  type Connection,
} from '../../api/firewall'
import { RuleModal } from '../../components/RuleModal'
import '../../styles/firewall.css'

type ChangeType = 'added' | 'removed' | 'modified' | 'enabled' | 'disabled'

type ChangeEntry = {
  key: number
  type: ChangeType
  name: string
  at: Date
}

type Toast = { kind: 'ok' | 'err'; text: string }

const DENY_ACTIONS = ['deny', 'drop', 'reject']

/* ── فاز ۴: Time-based Rules ── */
const DAY_ORDER = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat']

/** لیست روزها برای tooltip — خالی یا ۷ روز = همه‌روزها */
function fmtDays(days: string[]): string {
  if (!days || days.length === 0 || days.length === 7) return 'Every day'
  const set = new Set(days)
  return DAY_ORDER.filter((d) => set.has(d))
    .map((d) => d[0].toUpperCase() + d.slice(1))
    .join(', ')
}

function fmtNum(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return n.toLocaleString()
}

function fmtTime(d: Date): string {
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

/* ── اسپارک‌لاین فعال‌دیتا (SVG — بدون وابستگی) ── */
function Sparkline({ points, width = 220, height = 46 }: { points: number[]; width?: number; height?: number }) {
  if (points.length < 2) {
    return (
      <svg width={width} height={height}>
        <line
          x1={0}
          y1={height / 2}
          x2={width}
          y2={height / 2}
          stroke="var(--border)"
          strokeDasharray="4 4"
        />
      </svg>
    )
  }
  const max = Math.max(...points, 1)
  const step = width / (points.length - 1)
  const pts = points.map((p, i) => {
    const x = i * step
    const y = height - 4 - (p / max) * (height - 10)
    return [x, y] as const
  })
  const line = pts.map(([x, y], i) => `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`).join(' ')
  const area = `${line} L${width},${height} L0,${height} Z`
  return (
    <svg width={width} height={height} style={{ display: 'block', maxWidth: '100%' }}>
      <path d={area} fill="var(--accent-soft)" />
      <path d={line} fill="none" stroke="var(--accent)" strokeWidth="1.8" strokeLinejoin="round" />
      <circle cx={pts[pts.length - 1][0]} cy={pts[pts.length - 1][1]} r="2.6" fill="var(--accent)" />
    </svg>
  )
}

/* ── دونات توزیع پالیسی (SVG — بدون وابستگی) ── */
function Donut({ segments, total }: { segments: { label: string; value: number; color: string }[]; total: number }) {
  const R = 44
  const C = 2 * Math.PI * R
  let offset = 0
  return (
    <svg width={124} height={124} viewBox="0 0 124 124">
      <g transform="rotate(-90 62 62)">
        <circle cx="62" cy="62" r={R} fill="none" stroke="var(--border-soft)" strokeWidth="15" />
        {total > 0 &&
          segments
            .filter((s) => s.value > 0)
            .map((s) => {
              const frac = s.value / total
              const el = (
                <circle
                  key={s.label}
                  cx="62"
                  cy="62"
                  r={R}
                  fill="none"
                  stroke={s.color}
                  strokeWidth="15"
                  strokeDasharray={`${(frac * C).toFixed(2)} ${C.toFixed(2)}`}
                  strokeDashoffset={(-offset * C).toFixed(2)}
                  strokeLinecap="butt"
                />
              )
              offset += frac
              return el
            })}
      </g>
      <text x="62" y="58" textAnchor="middle" fill="var(--text)" fontSize="19" fontWeight="700">
        {total}
      </text>
      <text x="62" y="75" textAnchor="middle" fill="var(--text-muted)" fontSize="9.5">
        Total Rules
      </text>
    </svg>
  )
}

export function FirewallRules() {
  const [rules, setRules] = useState<FirewallRule[]>([])
  const [sets, setSets] = useState<IpSet[]>([])
  const [fqdns, setFqdns] = useState<FqdnObject[]>([])
  const [stats, setStats] = useState<Record<string, RuleStat>>({})
  const [conns, setConns] = useState<Connection[]>([])
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [q, setQ] = useState('')
  const [fAction, setFAction] = useState('all')
  const [fStatus, setFStatus] = useState('all')
  const [modal, setModal] = useState<{ open: boolean; rule: FirewallRule | null }>({
    open: false,
    rule: null,
  })
  const [toast, setToast] = useState<Toast | null>(null)
  const [busyId, setBusyId] = useState<string | null>(null)
  const [hitHistory, setHitHistory] = useState<Record<string, number[]>>({})
  const [changes, setChanges] = useState<ChangeEntry[]>([])

  const prevRulesRef = useRef<Map<string, FirewallRule>>(new Map())
  const prevPacketsRef = useRef<Map<string, number>>(new Map())
  const changeKeyRef = useRef(0)
  const importRef = useRef<HTMLInputElement>(null)

  const showToast = useCallback((kind: 'ok' | 'err', text: string) => {
    setToast({ kind, text })
  }, [])

  // toast خودکار محو شود
  useEffect(() => {
    if (!toast) return
    const t = setTimeout(() => setToast(null), 4000)
    return () => clearTimeout(t)
  }, [toast])

  /* ── تشخیص تغییرات پالیسی (در همین نشست) ── */
  const diffChanges = useCallback((next: FirewallRule[]) => {
    const prev = prevRulesRef.current
    const now = new Date()
    const entries: Omit<ChangeEntry, 'key'>[] = []
    const nextMap = new Map(next.map((r) => [r.id, r]))

    if (prev.size > 0) {
      for (const rule of next) {
        const p = prev.get(rule.id)
        if (!p) {
          entries.push({ type: 'added', name: rule.name, at: now })
        } else if (
          p.name !== rule.name ||
          p.action !== rule.action ||
          p.priority !== rule.priority ||
          p.source !== rule.source ||
          p.destination !== rule.destination ||
          p.port !== rule.port ||
          p.protocol !== rule.protocol ||
          p.source_set !== rule.source_set ||
          p.dest_set !== rule.dest_set ||
          p.rate_limit !== rule.rate_limit ||
          p.log !== rule.log ||
          p.schedule_enabled !== rule.schedule_enabled ||
          p.schedule_start !== rule.schedule_start ||
          p.schedule_end !== rule.schedule_end ||
          p.schedule_days.join(',') !== rule.schedule_days.join(',')
        ) {
          entries.push({ type: 'modified', name: rule.name, at: now })
        }
      }
      for (const [id, p] of prev) {
        if (!nextMap.has(id)) entries.push({ type: 'removed', name: p.name, at: now })
      }
    }

    prevRulesRef.current = nextMap
    if (entries.length) {
      setChanges((c) => {
        const withKeys = entries.map((e) => ({ ...e, key: ++changeKeyRef.current }))
        return [...withKeys.reverse(), ...c].slice(0, 8)
      })
    }
  }, [])

  /* ── لود قوانین + مجموعه‌ها ── */
  const load = useCallback(async function load() {
    try {
      const [r, s, f] = await Promise.all([
        firewallApi.listRules(),
        firewallApi.listSets().catch(() => [] as IpSet[]),
        firewallApi.listFqdn().catch(() => [] as FqdnObject[]),
      ])
      setRules(r)
      setSets(s)
      setFqdns(f)
      diffChanges(r)
    } catch (e: any) {
      showToast('err', e.message)
    }
  }, [diffChanges, showToast])

  /* ── آمار per-rule + اتصالات زنده — poll هر ۵ ثانیه ── */
  const loadStats = useCallback(async function loadStats() {
    try {
      const list = await firewallApi.stats()
      const map: Record<string, RuleStat> = {}
      const prevPackets = prevPacketsRef.current
      const deltas: Record<string, number> = {}

      for (const s of list) {
        map[s.rule_id] = s
        const prev = prevPackets.get(s.rule_id)
        if (prev != null && s.packets > prev) deltas[s.rule_id] = s.packets - prev
      }
      prevPacketsRef.current = new Map(list.map((s) => [s.rule_id, s.packets]))

      setStats(map)
      if (Object.keys(deltas).length) {
        setHitHistory((h) => {
          const next = { ...h }
          for (const [id, d] of Object.entries(deltas)) {
            next[id] = [...(next[id] ?? []), d].slice(-40)
          }
          return next
        })
      }
    } catch {
      /* بی‌صدا — آمار حیاتی نیست */
    }
  }, [])

  const loadConns = useCallback(async function loadConns() {
    try {
      setConns(await firewallApi.connections())
    } catch {
      /* بی‌صدا */
    }
  }, [])

  useEffect(() => {
    load()
    loadStats()
    loadConns()
    const t = setInterval(() => {
      loadStats()
      loadConns()
    }, 5000)
    // فاز ۴: بازخوانی قوانین هر ۳۰ ثانیه — تا نشانگر Active/Outside window (که
    // سمت سرور با ساعتِ سرور محاسبه می‌شود) پس از عبور از مرز پنجره تازه بماند
    const t2 = setInterval(load, 30000)
    return () => {
      clearInterval(t)
      clearInterval(t2)
    }
  }, [load, loadStats, loadConns])

  /* ── عملیات ── */
  async function removeRule(r: FirewallRule) {
    if (!window.confirm(`Delete rule "${r.name}"?`)) return
    setBusyId(r.id)
    try {
      await firewallApi.deleteRule(r.id)
      if (selectedId === r.id) setSelectedId(null)
      await load()
      showToast('ok', `Rule "${r.name}" deleted`)
    } catch (e: any) {
      showToast('err', e.message)
    } finally {
      setBusyId(null)
    }
  }

  async function toggleRule(r: FirewallRule) {
    setBusyId(r.id)
    try {
      await firewallApi.updateRule(r.id, { enabled: !r.enabled })
      await load()
    } catch (e: any) {
      showToast('err', e.message)
    } finally {
      setBusyId(null)
    }
  }

  async function changePriority(r: FirewallRule, value: string) {
    const p = Number(value)
    if (!Number.isFinite(p) || p === r.priority || p < 1) return
    setBusyId(r.id)
    try {
      await firewallApi.updateRule(r.id, { priority: p })
      await load()
      showToast('ok', `Priority of "${r.name}" changed to ${p}`)
    } catch (e: any) {
      showToast('err', e.message)
    } finally {
      setBusyId(null)
    }
  }

  async function onExport() {
    try {
      const data = await firewallApi.exportConfig()
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `dezh-firewall-${new Date().toISOString().slice(0, 10)}.json`
      a.click()
      URL.revokeObjectURL(url)
      showToast('ok', 'Configuration exported')
    } catch (e: any) {
      showToast('err', e.message)
    }
  }

  async function onImport(file: File) {
    try {
      const data = JSON.parse(await file.text())
      const res = await firewallApi.importConfig(data)
      await load()
      showToast(
        'ok',
        `Imported: ${res.rules_added} rule(s), ${res.nat_added} NAT, ${res.sets_added} list(s) — ` +
          `${res.rules_skipped + res.nat_skipped + res.sets_skipped} duplicate(s) skipped`,
      )
    } catch (e: any) {
      showToast('err', `Import failed: ${e.message}`)
    }
  }

  /* ── محاسبات ── */
  const counts = useMemo(() => {
    const allow = rules.filter((r) => r.action === 'allow').length
    const deny = rules.filter((r) => DENY_ACTIONS.includes(r.action)).length
    const inactive = rules.filter((r) => !r.enabled).length
    const total = rules.length
    const pct = (n: number) => (total ? `${((n / total) * 100).toFixed(1)}%` : '0%')
    return { total, allow, deny, inactive, pct }
  }, [rules])

  const filtered = useMemo(() => {
    const term = q.trim().toLowerCase()
    return rules
      .filter((r) => {
        if (fAction !== 'all') {
          if (fAction === 'deny' ? !DENY_ACTIONS.includes(r.action) : r.action !== fAction)
            return false
        }
        if (fStatus === 'active' && !r.enabled) return false
        if (fStatus === 'inactive' && r.enabled) return false
        if (!term) return true
        const hay = [
          r.name,
          r.description,
          r.source,
          r.destination,
          r.source_set ?? '',
          r.dest_set ?? '',
          r.protocol,
          r.port?.toString() ?? '',
          // فاز ۴: جستجو در پنجره‌ی زمانی (مثلاً "09:00" یا "18:00")
          r.schedule_enabled ? `${r.schedule_start}-${r.schedule_end}` : '',
        ]
          .join(' ')
          .toLowerCase()
        return hay.includes(term)
      })
      .sort((a, b) => a.priority - b.priority || a.name.localeCompare(b.name))
  }, [rules, q, fAction, fStatus])

  const donutSegments = useMemo(() => {
    const deny = rules.filter((r) => r.action === 'deny').length
    const drop = rules.filter((r) => r.action === 'drop').length
    const reject = rules.filter((r) => r.action === 'reject').length
    return [
      { label: 'Allow', value: counts.allow, color: '#22c55e' },
      { label: 'Deny', value: deny, color: '#ef4444' },
      { label: 'Drop', value: drop, color: '#f59e0b' },
      { label: 'Reject', value: reject, color: '#a78bfa' },
    ]
  }, [rules, counts.allow])

  const topIps = useMemo(() => {
    function top(getter: (c: Connection) => string) {
      const m = new Map<string, number>()
      for (const c of conns) {
        const k = getter(c)
        m.set(k, (m.get(k) ?? 0) + 1)
      }
      const total = conns.length || 1
      return [...m.entries()]
        .sort((a, b) => b[1] - a[1])
        .slice(0, 5)
        .map(([ip, n]) => ({ ip, n, pct: `${((n / total) * 100).toFixed(1)}%` }))
    }
    return { src: top((c) => c.src), dst: top((c) => c.dst) }
  }, [conns])

  const selected = useMemo(
    () => rules.find((r) => r.id === selectedId) ?? null,
    [rules, selectedId],
  )
  const selStat = selected ? stats[selected.id] : undefined
  const selHistory = selected ? hitHistory[selected.id] ?? [] : []

  const CHANGE_META: Record<ChangeType, { label: string; color: string; Icon: typeof Plus }> = {
    added: { label: 'Rule Added', color: 'green', Icon: Plus },
    removed: { label: 'Rule Deleted', color: 'red', Icon: Trash2 },
    modified: { label: 'Rule Modified', color: 'blue', Icon: Pencil },
    enabled: { label: 'Rule Enabled', color: 'green', Icon: Power },
    disabled: { label: 'Rule Disabled', color: 'amber', Icon: Power },
  }

  return (
    <div>
      {/* ── Header ── */}
      <div className="fw-header">
        <h1>Firewall Policies</h1>
        <p>Manage and monitor your next-generation firewall policies.</p>
      </div>

      {/* ── Stat cards ── */}
      <div className="stat-cards">
        <div className="stat-card">
          <span className="stat-icon blue">
            <Shield size={20} />
          </span>
          <div className="stat-meta">
            <div className="stat-label">Total Rules</div>
            <div className="stat-value fw-num">{counts.total}</div>
            <div className="stat-extra blue">{counts.pct(counts.total)}</div>
          </div>
        </div>
        <div className="stat-card">
          <span className="stat-icon green">
            <ShieldCheck size={20} />
          </span>
          <div className="stat-meta">
            <div className="stat-label">Allow Rules</div>
            <div className="stat-value fw-num">{counts.allow}</div>
            <div className="stat-extra green">{counts.pct(counts.allow)}</div>
          </div>
        </div>
        <div className="stat-card">
          <span className="stat-icon red">
            <Ban size={20} />
          </span>
          <div className="stat-meta">
            <div className="stat-label">Deny Rules</div>
            <div className="stat-value fw-num">{counts.deny}</div>
            <div className="stat-extra red">{counts.pct(counts.deny)}</div>
          </div>
        </div>
        <div className="stat-card">
          <span className="stat-icon amber">
            <Power size={20} />
          </span>
          <div className="stat-meta">
            <div className="stat-label">Inactive Rules</div>
            <div className="stat-value fw-num">{counts.inactive}</div>
            <div className="stat-extra amber">{counts.pct(counts.inactive)}</div>
          </div>
        </div>
      </div>

      {/* ── Main: table + detail ── */}
      <div className="fw-main">
        <div className="fw-card">
          <div className="fw-card-head">
            <h2>Firewall Policies</h2>
            <div className="fw-card-actions">
              <button type="button" className="btn btn-ghost" onClick={onExport} title="Export config as JSON">
                <Download size={14} />
                Export
              </button>
              <button type="button" className="btn btn-ghost" onClick={() => importRef.current?.click()} title="Import config JSON">
                <Upload size={14} />
                Import
              </button>
              <input
                ref={importRef}
                type="file"
                accept=".json,application/json"
                style={{ display: 'none' }}
                onChange={(e) => {
                  const f = e.target.files?.[0]
                  e.target.value = ''
                  if (f) onImport(f)
                }}
              />
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => setModal({ open: true, rule: null })}
              >
                <Plus size={15} />
                Add Rule
              </button>
            </div>
          </div>

          <div className="fw-toolbar">
            <div className="search-box">
              <Search size={14} />
              <input
                placeholder="Search rules by name, source, destination, port, schedule…"
                value={q}
                onChange={(e) => setQ(e.target.value)}
              />
              <select value={fAction} onChange={(e) => setFAction(e.target.value)}>
                <option value="all">All Actions</option>
                <option value="allow">Allow</option>
                <option value="deny">Deny</option>
                <option value="drop">Drop</option>
                <option value="reject">Reject</option>
              </select>
            </div>
            <select className="select" value={fStatus} onChange={(e) => setFStatus(e.target.value)}>
              <option value="all">All Status</option>
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
            </select>
          </div>

          <div className="fw-table-wrap">
            <table className="fw-table">
              <thead>
                <tr>
                  <th style={{ width: 34 }}>#</th>
                  <th>Name</th>
                  <th>Source</th>
                  <th>Destination</th>
                  <th>Service</th>
                  <th>Action</th>
                  <th>Priority</th>
                  <th>Hits</th>
                  <th>Bytes</th>
                  <th>Status</th>
                  <th style={{ width: 70 }}></th>
                </tr>
              </thead>
              <tbody>
                {filtered.length === 0 ? (
                  <tr>
                    <td colSpan={11}>
                      <div className="fw-empty">
                        {rules.length === 0
                          ? 'No rules yet — click "Add Rule" or import a config.'
                          : 'No rules match your filters.'}
                      </div>
                    </td>
                  </tr>
                ) : (
                  filtered.map((r, i) => {
                    const st = stats[r.id]
                    return (
                      <tr
                        key={r.id}
                        className={`fw-row ${selectedId === r.id ? 'selected' : ''}`}
                        onClick={() => setSelectedId(r.id)}
                      >
                        <td className="fw-num" style={{ color: 'var(--text-muted)' }}>
                          {i + 1}
                        </td>
                        <td>
                          <div className="rule-name">
                            {r.name}
                            {r.log && (
                              <span className="log-tag" title="Hits are logged to kernel log">
                                LOG
                              </span>
                            )}
                            {/* فاز ۴: نشان زمان‌بندی — سبز = داخل پنجره، خاکستری = بیرون پنجره */}
                            {r.schedule_enabled && (
                              <span
                                className={`sched-badge ${r.schedule_active ? 'on' : 'off'}`}
                                title={`Time window ${r.schedule_start}–${r.schedule_end} · ${fmtDays(r.schedule_days)} — ${
                                  r.schedule_active ? 'inside window now' : 'outside window now'
                                }`}
                              >
                                <Clock size={10} />
                                {r.schedule_start}–{r.schedule_end}
                              </span>
                            )}
                          </div>
                          {r.description && <div className="rule-desc">{r.description}</div>}
                        </td>
                        <td>
                          <div>{r.source_set ? `List: ${r.source_set}` : r.source || 'any'}</div>
                          {r.source_set && <div className="cell-sub">{r.source || 'any'}</div>}
                        </td>
                        <td>
                          <div>{r.dest_set ? `List: ${r.dest_set}` : r.destination || 'any'}</div>
                          {r.dest_set && <div className="cell-sub">{r.destination || 'any'}</div>}
                        </td>
                        <td>
                          <div>{r.protocol.toUpperCase()}</div>
                          {r.port != null && <div className="cell-sub">port {r.port}</div>}
                        </td>
                        <td>
                          <span className={`pill pill-${r.action}`}>
                            <span className="dot" />
                            {r.action}
                          </span>
                        </td>
                        <td onClick={(e) => e.stopPropagation()}>
                          <input
                            className="prio-input fw-num"
                            defaultValue={r.priority}
                            onBlur={(e) => changePriority(r, e.target.value)}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter') (e.target as HTMLInputElement).blur()
                            }}
                            title="Priority — Enter to apply"
                          />
                        </td>
                        <td className="fw-num">{st ? fmtNum(st.packets) : '—'}</td>
                        <td className="fw-num">{st ? fmtBytes(st.bytes) : '—'}</td>
                        <td onClick={(e) => e.stopPropagation()}>
                          <label className="switch" title={r.enabled ? 'Disable rule' : 'Enable rule'}>
                            <input
                              type="checkbox"
                              checked={r.enabled}
                              disabled={busyId === r.id}
                              onChange={() => toggleRule(r)}
                            />
                            <span className="track" />
                          </label>
                        </td>
                        <td onClick={(e) => e.stopPropagation()}>
                          <div className="row-actions">
                            <button
                              type="button"
                              className="icon-btn"
                              title="Edit rule"
                              onClick={() => setModal({ open: true, rule: r })}
                            >
                              <Pencil size={13} />
                            </button>
                            <button
                              type="button"
                              className="icon-btn danger"
                              title="Delete rule"
                              disabled={busyId === r.id}
                              onClick={() => removeRule(r)}
                            >
                              <Trash2 size={13} />
                            </button>
                          </div>
                        </td>
                      </tr>
                    )
                  })
                )}
              </tbody>
            </table>
          </div>
        </div>

        {/* ── Detail panel ── */}
        {selected && (
          <div className="detail-panel">
            <div className="detail-head">
              <h3>Policy Details</h3>
              <button type="button" className="icon-btn" onClick={() => setSelectedId(null)} title="Close">
                <X size={14} />
              </button>
            </div>

            <div className="detail-title-row">
              <strong>{selected.name}</strong>
              <span className={`pill ${selected.enabled ? 'pill-allow' : 'pill-neutral'}`}>
                <span className="dot" />
                {selected.enabled ? 'Active' : 'Inactive'}
              </span>
            </div>
            {selected.description && <div className="detail-desc">{selected.description}</div>}

            <div className="kv">
              <span className="k">Rule ID</span>
              <span className="v fw-num" title={selected.id}>
                {selected.id.slice(0, 8)}…
              </span>
            </div>
            <div className="kv">
              <span className="k">Priority</span>
              <span className="v fw-num">{selected.priority}</span>
            </div>
            <div className="kv">
              <span className="k">Action</span>
              <span className={`v pill pill-${selected.action}`} style={{ width: 'fit-content' }}>
                <span className="dot" />
                {selected.action}
              </span>
            </div>
            <div className="kv">
              <span className="k">Direction</span>
              <span className="v">{selected.direction}</span>
            </div>
            <div className="kv">
              <span className="k">Rate limit</span>
              <span className="v fw-num">{selected.rate_limit ? `${selected.rate_limit}/s` : 'Unlimited'}</span>
            </div>
            <div className="kv">
              <span className="k">Logging</span>
              <span className="v">{selected.log ? 'Enabled' : 'Disabled'}</span>
            </div>

            <div className="detail-sec">
              <div className="detail-sec-title">Match</div>
              <div className="detail-block">
                <span className="blk-icon">
                  <Server size={15} />
                </span>
                <div>
                  <div className="blk-label">Source</div>
                  <div className="blk-value">
                    {selected.source_set ? `List: ${selected.source_set}` : selected.source || 'any'}
                  </div>
                </div>
              </div>
              <div className="detail-block">
                <span className="blk-icon">
                  <Globe size={15} />
                </span>
                <div>
                  <div className="blk-label">Destination</div>
                  <div className="blk-value">
                    {selected.dest_set ? `List: ${selected.dest_set}` : selected.destination || 'any'}
                  </div>
                </div>
              </div>
              <div className="detail-block">
                <span className="blk-icon">
                  <Gauge size={15} />
                </span>
                <div>
                  <div className="blk-label">Service</div>
                  <div className="blk-value">
                    {selected.protocol.toUpperCase()}
                    {selected.port != null ? ` · port ${selected.port}` : ' · any port'}
                  </div>
                </div>
              </div>
            </div>

            {/* فاز ۴: زمان‌بندی — فقط برای قوانین زمان‌دار نمایش داده می‌شود */}
            {selected.schedule_enabled && (
              <div className="detail-sec">
                <div className="detail-sec-title">Schedule</div>
                <div className="detail-block">
                  <span className="blk-icon sched-icon">
                    <Clock size={15} />
                  </span>
                  <div>
                    <div className="blk-label">Time window</div>
                    <div className="blk-value">
                      {selected.schedule_start} – {selected.schedule_end}
                      <span className={`sched-state ${selected.schedule_active ? 'on' : 'off'}`}>
                        {selected.schedule_active ? 'Active now' : 'Outside window'}
                      </span>
                    </div>
                  </div>
                </div>
                <div className="sched-days-row">
                  {DAY_ORDER.map((d) => {
                    const on =
                      selected.schedule_days.length === 0 || selected.schedule_days.includes(d)
                    return (
                      <span key={d} className={`sched-day-chip ${on ? 'on' : ''}`} title={d}>
                        {d}
                      </span>
                    )
                  })}
                </div>
              </div>
            )}

            <div className="detail-sec">
              <div className="detail-sec-title">Usage</div>
              <div className="usage-head">
                <span className="big fw-num">{selStat ? selStat.packets.toLocaleString() : 0}</span>
                <span className="lbl">Hits (packets)</span>
              </div>
              <Sparkline points={selHistory} />
              <div className="kv" style={{ marginTop: 6 }}>
                <span className="k">Bytes</span>
                <span className="v fw-num">{selStat ? fmtBytes(selStat.bytes) : '—'}</span>
              </div>
              <div className="kv">
                <span className="k">Sampled</span>
                <span className="v fw-num">{selHistory.length} × 5s</span>
              </div>
            </div>

            <div className="detail-actions">
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setModal({ open: true, rule: selected })}
              >
                <Pencil size={14} />
                Edit
              </button>
              <button type="button" className="btn btn-danger" onClick={() => removeRule(selected)}>
                <Trash2 size={14} />
                Delete
              </button>
            </div>
          </div>
        )}
      </div>

      {/* ── Bottom charts ── */}
      <div className="fw-charts">
        <div className="chart-card">
          <h4>Policy Distribution</h4>
          <div className="donut-wrap">
            <Donut segments={donutSegments} total={counts.total} />
            <div className="legend">
              {donutSegments.map((s) => (
                <div className="legend-row" key={s.label}>
                  <span className="swatch" style={{ background: s.color }} />
                  <span className="l-name">{s.label}</span>
                  <span className="l-val fw-num">{s.value}</span>
                  <span className="l-pct">{counts.total ? `${((s.value / counts.total) * 100).toFixed(1)}%` : '0%'}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="chart-card">
          <h4>Top Source IPs</h4>
          {topIps.src.length === 0 ? (
            <div className="mini-empty">No live connections right now.</div>
          ) : (
            <table className="mini-table">
              <thead>
                <tr>
                  <th>Source IP</th>
                  <th>Conns</th>
                  <th>%</th>
                </tr>
              </thead>
              <tbody>
                {topIps.src.map((x) => (
                  <tr key={x.ip}>
                    <td className="mono">{x.ip}</td>
                    <td>{x.n}</td>
                    <td>{x.pct}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div className="chart-card">
          <h4>Top Destination IPs</h4>
          {topIps.dst.length === 0 ? (
            <div className="mini-empty">No live connections right now.</div>
          ) : (
            <table className="mini-table">
              <thead>
                <tr>
                  <th>Destination IP</th>
                  <th>Conns</th>
                  <th>%</th>
                </tr>
              </thead>
              <tbody>
                {topIps.dst.map((x) => (
                  <tr key={x.ip}>
                    <td className="mono">{x.ip}</td>
                    <td>{x.n}</td>
                    <td>{x.pct}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div className="chart-card">
          <h4>Recent Policy Changes</h4>
          {changes.length === 0 ? (
            <div className="mini-empty">
              No policy changes in this session yet — changes you make appear here live.
            </div>
          ) : (
            <div className="change-list">
              {changes.map((c) => {
                const meta = CHANGE_META[c.type]
                const Icon = meta.Icon
                return (
                  <div className="change-row" key={c.key}>
                    <span className={`change-ico ${meta.color}`}>
                      <Icon size={12} />
                    </span>
                    <div className="change-txt">
                      <div className="t">
                        <b>{meta.label}</b> · {c.name}
                      </div>
                      <small>by you · this session</small>
                    </div>
                    <span className="change-time">{fmtTime(c.at)}</span>
                  </div>
                )
              })}
            </div>
          )}
        </div>
      </div>

      {/* ── Modal ── */}
      {modal.open && (
        <RuleModal
          rule={modal.rule}
          sets={sets}
          fqdns={fqdns}
          onClose={() => setModal({ open: false, rule: null })}
          onSaved={async (msg) => {
            setModal({ open: false, rule: null })
            await load()
            showToast('ok', msg)
          }}
        />
      )}

      {/* ── Toast ── */}
      {toast && (
        <div className={`toast ${toast.kind}`}>
          {toast.kind === 'ok' ? <CheckCircle2 size={16} /> : <AlertTriangle size={16} />}
          <span>{toast.text}</span>
        </div>
      )}
    </div>
  )
}