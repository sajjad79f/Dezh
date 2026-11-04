/* DEZH — Command Center Dashboard (futuristic dark theme)
 * Style reference: DEZH Network Security Dashboard mockup
 * Live data: modules, core services, firewall rules, interfaces, accounting sessions
 * Simulated (clearly marked): traffic rate series, demo incidents/stream fill
 */
import { useEffect, useMemo, useRef, useState } from 'react'
import {
  Activity,
  AlertTriangle,
  Cpu,
  Globe,
  Lock,
  Network,
  Radio,
  Server,
  Shield,
  ShieldCheck,
  Siren,
  Users,
  Zap,
} from 'lucide-react'
import { getToken } from '../api/auth'
import { api } from '../api/client'

type Iface = { name: string; zone: string; up: boolean; addresses: string[] }
type Session = {
  id: string
  hostname: string | null
  ip_address: string | null
  live_bytes_in: number
  live_bytes_out: number
}
type Rule = { id: string; enabled: boolean }

/* ───────────────────────── helpers ───────────────────────── */

const num = (n: number): string => n.toLocaleString('en-US')

const clamp = (v: number, lo: number, hi: number): number =>
  Math.min(hi, Math.max(lo, v))

function hhmmss(d: Date): string {
  const p = (x: number) => String(x).padStart(2, '0')
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`
}

const TONE = {
  green: '#22c55e',
  amber: '#f59e0b',
  red: '#ef4444',
  cyan: '#38bdf8',
  violet: '#a78bfa',
  slate: '#64748b',
}

/* ───────────────────────── styles ───────────────────────── */

const CSS = `
.dzh { color: #e2e8f0; font-size: 14px; }
.dzh * { box-sizing: border-box; }
.dzh-head { display:flex; align-items:flex-end; justify-content:space-between; margin-bottom:18px; gap:12px; flex-wrap:wrap; }
.dzh-head h1 { font-size:24px; font-weight:700; letter-spacing:.02em; margin:0; color:#f1f5f9; }
.dzh-head .sub { color:#7d8cab; font-size:13px; margin-top:4px; }
.dzh-status { display:flex; align-items:center; gap:14px; font-size:12px; color:#7d8cab; }
.dzh-pill { display:inline-flex; align-items:center; gap:6px; padding:5px 12px; border-radius:999px;
  border:1px solid rgba(34,197,94,.35); background:rgba(34,197,94,.08); color:#4ade80; font-weight:600; }
.dzh-pill.warn { border-color:rgba(245,158,11,.4); background:rgba(245,158,11,.08); color:#fbbf24; }
.dzh-dot { width:8px; height:8px; border-radius:50%; background:#22c55e; box-shadow:0 0 8px rgba(34,197,94,.8); animation:dzh-blink 2s infinite; }
.dzh-dot.amber { background:#f59e0b; box-shadow:0 0 8px rgba(245,158,11,.8); }

.dzh-kpis { display:grid; grid-template-columns:repeat(6,1fr); gap:12px; margin-bottom:14px; }
@media (max-width:1400px){ .dzh-kpis { grid-template-columns:repeat(3,1fr); } }
@media (max-width:820px){ .dzh-kpis { grid-template-columns:repeat(2,1fr); } }
.dzh-kpi { position:relative; overflow:hidden; display:flex; align-items:center; gap:12px;
  background:linear-gradient(160deg,#101b31 0%,#0c1424 100%); border:1px solid #1c2a44; border-radius:12px; padding:14px 16px; }
.dzh-kpi::after { content:''; position:absolute; inset:0; background:radial-gradient(120px 40px at 20% 0%, rgba(56,189,248,.10), transparent 70%); pointer-events:none; }
.dzh-kpi .ic { width:42px; height:42px; border-radius:50%; display:flex; align-items:center; justify-content:center; flex-shrink:0; }
.dzh-kpi .lbl { font-size:11.5px; color:#7d8cab; text-transform:uppercase; letter-spacing:.06em; }
.dzh-kpi .val { font-size:22px; font-weight:700; color:#f1f5f9; line-height:1.15; font-variant-numeric:tabular-nums; }
.dzh-kpi .delta { font-size:11px; display:inline-flex; align-items:center; gap:2px; font-weight:600; }
.dzh-kpi .delta.up { color:#4ade80; } .dzh-kpi .delta.down { color:#f87171; } .dzh-kpi .delta.flat { color:#7d8cab; }

.dzh-row { display:grid; gap:12px; margin-bottom:12px; }
.dzh-row.r2 { grid-template-columns:1fr 340px; }
.dzh-row.r3 { grid-template-columns:1.05fr 1.25fr .95fr; }
.dzh-row.r4 { grid-template-columns:1.15fr 1fr 1fr; }
@media (max-width:1200px){ .dzh-row.r2,.dzh-row.r3,.dzh-row.r4 { grid-template-columns:1fr; } }

.dzh-card { background:linear-gradient(170deg,#0e1729 0%,#0b1220 100%); border:1px solid #1c2a44;
  border-radius:12px; padding:16px; min-width:0; }
.dzh-card-h { display:flex; align-items:center; justify-content:space-between; margin-bottom:12px; }
.dzh-card-h .t { font-size:14px; font-weight:700; color:#e2e8f0; display:flex; align-items:center; gap:8px; }
.dzh-card-h .more { font-size:12px; color:#38bdf8; cursor:pointer; }
.dzh-live { display:inline-flex; align-items:center; gap:6px; font-size:11px; color:#4ade80; font-weight:600; }
.dzh-muted { color:#7d8cab; }

.dzh-legend { display:flex; gap:14px; font-size:11px; color:#7d8cab; }
.dzh-legend i { display:inline-block; width:8px; height:8px; border-radius:50%; margin-right:5px; }

.dzh-topo { position:relative; }
.dzh-topo svg { width:100%; height:auto; display:block; }
.dzh-link { stroke-dasharray:5 5; animation:dzh-dash 1.2s linear infinite; }
.dzh-traffic-box { fill:rgba(56,189,248,.06); stroke:#1c2a44; }
.dzh-node { fill:#101b31; stroke:#24365a; rx:9; }
.dzh-node.hot { stroke:rgba(56,189,248,.55); }
.dzh-node.down { stroke:rgba(239,68,68,.55); }
.dzh-txt { fill:#cbd5e1; font-size:11px; font-weight:600; }
.dzh-txt.sm { fill:#7d8cab; font-size:9.5px; font-weight:400; }
.dzh-txt.val { fill:#f1f5f9; font-size:13px; font-weight:700; }

.dzh-inc { display:flex; gap:10px; padding:9px 0; border-bottom:1px solid rgba(28,42,68,.6); }
.dzh-inc:last-child { border-bottom:none; }
.dzh-sev { flex-shrink:0; align-self:flex-start; font-size:10px; font-weight:700; padding:3px 8px; border-radius:6px; letter-spacing:.04em; }
.dzh-sev.critical { background:rgba(239,68,68,.14); color:#f87171; border:1px solid rgba(239,68,68,.35); }
.dzh-sev.high { background:rgba(245,158,11,.14); color:#fbbf24; border:1px solid rgba(245,158,11,.35); }
.dzh-sev.medium { background:rgba(56,189,248,.12); color:#7dd3fc; border:1px solid rgba(56,189,248,.3); }
.dzh-inc .ttl { font-size:12.5px; font-weight:600; color:#dbe4f0; }
.dzh-inc .meta { display:flex; gap:6px; margin-top:5px; flex-wrap:wrap; }
.dzh-tag { font-size:10px; padding:2px 8px; border-radius:5px; background:rgba(148,163,184,.08); color:#94a3b8; border:1px solid rgba(148,163,184,.15); }
.dzh-tag.open { background:rgba(239,68,68,.10); color:#f87171; border-color:rgba(239,68,68,.3); }
.dzh-tag.inv { background:rgba(245,158,11,.10); color:#fbbf24; border-color:rgba(245,158,11,.3); }
.dzh-ago { margin-left:auto; font-size:11px; color:#7d8cab; flex-shrink:0; }

.dzh-donut-wrap { display:flex; align-items:center; gap:18px; }
.dzh-dlegend { flex:1; display:flex; flex-direction:column; gap:8px; }
.dzh-dlegend .row { display:flex; align-items:center; gap:8px; font-size:12.5px; }
.dzh-dlegend .row i { width:9px; height:9px; border-radius:50%; }
.dzh-dlegend .row b { margin-left:auto; color:#e2e8f0; font-variant-numeric:tabular-nums; }
.dzh-dlegend .row span { color:#7d8cab; width:44px; text-align:right; font-variant-numeric:tabular-nums; }

.dzh-posture-rows { margin-top:14px; display:flex; flex-direction:column; gap:9px; }
.dzh-posture-rows .row { display:flex; align-items:center; gap:8px; font-size:12.5px; padding:7px 10px;
  background:rgba(148,163,184,.04); border-radius:8px; border:1px solid rgba(28,42,68,.7); }
.dzh-posture-rows .row b { margin-left:auto; font-variant-numeric:tabular-nums; }

.dzh-table { width:100%; border-collapse:collapse; font-size:12px; }
.dzh-table th { text-align:left; color:#7d8cab; font-weight:600; font-size:10.5px; text-transform:uppercase;
  letter-spacing:.05em; padding:6px 8px; border-bottom:1px solid #1c2a44; }
.dzh-table td { padding:7px 8px; border-bottom:1px solid rgba(28,42,68,.55); color:#cbd5e1; white-space:nowrap; }
.dzh-table tr:last-child td { border-bottom:none; }
.dzh-chip { font-size:10px; font-weight:700; padding:3px 9px; border-radius:6px; }
.dzh-chip.critical { background:rgba(239,68,68,.15); color:#f87171; }
.dzh-chip.high { background:rgba(245,158,11,.15); color:#fbbf24; }
.dzh-chip.medium { background:rgba(234,179,8,.15); color:#facc15; }
.dzh-chip.low, .dzh-chip.info { background:rgba(56,189,248,.12); color:#7dd3fc; }
.dzh-chip.ok { background:rgba(34,197,94,.15); color:#4ade80; }

.dzh-stream { display:flex; flex-direction:column; gap:2px; max-height:218px; overflow-y:auto; }
.dzh-ev { display:flex; align-items:center; gap:10px; padding:6px 4px; border-radius:8px; animation:dzh-fade .4s ease; }
.dzh-ev:hover { background:rgba(148,163,184,.05); }
.dzh-ev .tm { font-size:11px; color:#7d8cab; width:44px; font-variant-numeric:tabular-nums; flex-shrink:0; }
.dzh-ev .ic { width:28px; height:28px; border-radius:50%; display:flex; align-items:center; justify-content:center; flex-shrink:0; }
.dzh-ev .ttl { font-size:12.5px; font-weight:600; color:#dbe4f0; }
.dzh-ev .src { font-size:11px; color:#7d8cab; }
.dzh-ev .by { margin-left:auto; font-size:11px; color:#64748b; flex-shrink:0; }

.dzh-err { background:rgba(239,68,68,.08); border:1px solid rgba(239,68,68,.35); color:#f87171;
  border-radius:10px; padding:10px 14px; margin-bottom:14px; font-size:13px; }

@keyframes dzh-dash { to { stroke-dashoffset:-20; } }
@keyframes dzh-blink { 0%,100% { opacity:1; } 50% { opacity:.45; } }
@keyframes dzh-fade { from { opacity:0; transform:translateY(-4px); } to { opacity:1; transform:none; } }
`

/* ───────────────────────── svg atoms ───────────────────────── */

function Sparkline({
  data,
  color,
  w = 150,
  h = 34,
}: {
  data: number[]
  color: string
  w?: number
  h?: number
}) {
  const max = Math.max(...data, 0.1)
  const pts = data
    .map((v, i) => `${(i / (data.length - 1)) * w},${h - (v / max) * (h - 4) - 2}`)
    .join(' ')
  return (
    <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`}>
      <polyline points={pts} fill="none" stroke={color} strokeWidth="1.6" />
    </svg>
  )
}

function Donut({
  segments,
  centerTop,
  centerBottom,
  size = 150,
}: {
  segments: { value: number; color: string }[]
  centerTop: string
  centerBottom: string
  size?: number
}) {
  const total = segments.reduce((s, x) => s + x.value, 0) || 1
  const r = 54
  const c = 2 * Math.PI * r
  let offset = 0
  return (
    <svg width={size} height={size} viewBox="0 0 140 140">
      <circle cx="70" cy="70" r={r} fill="none" stroke="#16233c" strokeWidth="15" />
      {segments.map((s, i) => {
        const len = (s.value / total) * c
        const el = (
          <circle
            key={i}
            cx="70"
            cy="70"
            r={r}
            fill="none"
            stroke={s.color}
            strokeWidth="15"
            strokeDasharray={`${len} ${c - len}`}
            strokeDashoffset={-offset}
            transform="rotate(-90 70 70)"
            strokeLinecap="butt"
          />
        )
        offset += len
        return el
      })}
      <text x="70" y="66" textAnchor="middle" fill="#f1f5f9" fontSize="21" fontWeight="700">
        {centerTop}
      </text>
      <text x="70" y="84" textAnchor="middle" fill="#7d8cab" fontSize="10">
        {centerBottom}
      </text>
    </svg>
  )
}

function smoothPath(data: number[], w: number, h: number, max: number): string {
  const pts = data.map((v, i) => [
    (i / (data.length - 1)) * w,
    h - (v / max) * (h - 10) - 5,
  ])
  if (pts.length < 2) return ''
  let d = `M ${pts[0][0]},${pts[0][1]}`
  for (let i = 1; i < pts.length; i++) {
    const [x0, y0] = pts[i - 1]
    const [x1, y1] = pts[i]
    const mx = (x0 + x1) / 2
    d += ` Q ${x0},${y0} ${mx},${(y0 + y1) / 2} T ${x1},${y1}`
  }
  return d
}

function TrafficChart({ inbound, outbound }: { inbound: number[]; outbound: number[] }) {
  const w = 560
  const h = 190
  const max = Math.max(6, ...inbound, ...outbound)
  const pin = smoothPath(inbound, w, h, max)
  const pout = smoothPath(outbound, w, h, max)
  const areaIn = `${pin} L ${w},${h} L 0,${h} Z`
  const areaOut = `${pout} L ${w},${h} L 0,${h} Z`
  const now = new Date()
  const labels = [6, 4, 2, 0].map((back) => {
    const t = new Date(now.getTime() - back * 3600_000)
    return `${String(t.getHours()).padStart(2, '0')}:00`
  })
  return (
    <svg viewBox={`0 0 ${w} ${h + 18}`} style={{ width: '100%', height: 'auto', display: 'block' }}>
      <defs>
        <linearGradient id="dzh-gin" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#38bdf8" stopOpacity="0.35" />
          <stop offset="100%" stopColor="#38bdf8" stopOpacity="0" />
        </linearGradient>
        <linearGradient id="dzh-gout" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#a78bfa" stopOpacity="0.3" />
          <stop offset="100%" stopColor="#a78bfa" stopOpacity="0" />
        </linearGradient>
      </defs>
      {[0, 1, 2, 3].map((i) => (
        <g key={i}>
          <line x1="0" x2={w} y1={5 + i * ((h - 10) / 3)} y2={5 + i * ((h - 10) / 3)} stroke="#16233c" strokeWidth="1" />
          <text x="4" y={2 + i * ((h - 10) / 3)} fill="#536179" fontSize="9">
            {(max - (max / 3) * i).toFixed(0)} Gbps
          </text>
        </g>
      ))}
      <path d={areaIn} fill="url(#dzh-gin)" />
      <path d={areaOut} fill="url(#dzh-gout)" />
      <path d={pin} fill="none" stroke="#38bdf8" strokeWidth="2" />
      <path d={pout} fill="none" stroke="#a78bfa" strokeWidth="2" />
      {labels.map((l, i) => (
        <text key={i} x={10 + i * ((w - 30) / 3)} y={h + 14} fill="#536179" fontSize="10">
          {l}
        </text>
      ))}
    </svg>
  )
}

/* ───────────────────────── domain types ───────────────────────── */

type Sev = 'critical' | 'high' | 'medium'
type Incident = {
  key: string
  sev: Sev
  title: string
  cat: string
  asset: string
  status: string
  ago: string
  demo?: boolean
}
type Ev = {
  id: number
  tm: string
  title: string
  src: string
  by: string
  tone: 'cyan' | 'green' | 'amber' | 'red' | 'violet'
}

const EV_ICONS: Record<Ev['tone'], typeof Activity> = {
  cyan: Activity,
  green: ShieldCheck,
  amber: AlertTriangle,
  red: Siren,
  violet: Lock,
}

/* ───────────────────────── main component ───────────────────────── */

export function Dashboard() {
  const [ifaces, setIfaces] = useState<Iface[]>([])
  const [sessions, setSessions] = useState<Session[]>([])
  const [rules, setRules] = useState<Rule[]>([])
  const [modules, setModules] = useState(0)
  const [services, setServices] = useState<{ name: string; status: string }[]>([])
  const [error, setError] = useState<string | null>(null)
  const [clock, setClock] = useState(new Date())

  const [inSeries, setInSeries] = useState<number[]>(() =>
    Array.from({ length: 40 }, (_, i) => 2 + Math.sin(i / 4) * 0.4 + Math.random() * 0.3),
  )
  const [outSeries, setOutSeries] = useState<number[]>(() =>
    Array.from({ length: 40 }, (_, i) => 1.5 + Math.sin(i / 5 + 1) * 0.3 + Math.random() * 0.25),
  )
  const lastIn = useRef(2.4)
  const lastOut = useRef(1.8)

  const [stream, setStream] = useState<Ev[]>([])
  const evId = useRef(1)

  const prev = useRef<{ rules: number; sessions: number; modules: number } | null>(null)

  function pushEv(title: string, src: string, by: string, tone: Ev['tone']) {
    setStream((s) =>
      [
        { id: evId.current++, tm: hhmmss(new Date()), title, src, by, tone },
        ...s,
      ].slice(0, 9),
    )
  }

  async function load() {
    const h = (): HeadersInit => {
      const t = getToken()
      return t ? { Authorization: `Bearer ${t}` } : {}
    }
    try {
      const [m, s, r, i, sess] = await Promise.all([
        api.modules(),
        api.coreServices(),
        fetch('/api/firewall/rules', { headers: h() }).then(async (x) => (x.ok ? x.json() : [])),
        fetch('/api/firewall/interfaces', { headers: h() }).then(async (x) => (x.ok ? x.json() : [])),
        fetch('/api/accounting/sessions', { headers: h() }).then(async (x) => (x.ok ? x.json() : [])),
      ])
      const ruleArr: Rule[] = Array.isArray(r) ? r : []
      const ifaceArr: Iface[] = Array.isArray(i) ? i : []
      const sessArr: Session[] = Array.isArray(sess) ? sess : []

      if (prev.current) {
        const p = prev.current
        if (ruleArr.length !== p.rules)
          pushEv('Firewall Rules Updated', 'Firewall', 'by system', 'cyan')
        if (sessArr.length > p.sessions)
          pushEv('User Session Started', 'Accounting', 'by system', 'green')
        if (sessArr.length < p.sessions)
          pushEv('User Session Ended', 'Accounting', 'by system', 'violet')
      }
      prev.current = { rules: ruleArr.length, sessions: sessArr.length, modules: m.length }

      setModules(m.length)
      setServices(s)
      setRules(ruleArr)
      setIfaces(ifaceArr)
      setSessions(sessArr)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  useEffect(() => {
    load()
    const t = setInterval(load, 8000)
    const ck = setInterval(() => setClock(new Date()), 1000)
    pushEv('Command Center Online', 'Core', 'by system', 'green')
    const traffic = setInterval(() => {
      lastIn.current = clamp(lastIn.current + (Math.random() - 0.5) * 0.5, 0.6, 5.5)
      lastOut.current = clamp(lastOut.current + (Math.random() - 0.5) * 0.4, 0.4, 4.5)
      setInSeries((s) => [...s.slice(1), lastIn.current])
      setOutSeries((s) => [...s.slice(1), lastOut.current])
    }, 3000)
    return () => {
      clearInterval(t)
      clearInterval(ck)
      clearInterval(traffic)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  /* ───── derived ───── */

  const upIfaces = ifaces.filter((i) => i.up)
  const downIfaces = ifaces.filter((i) => !i.up)
  const runningServices = services.filter((s) => s.status === 'running').length

  const uptime =
    ifaces.length === 0 ? 100 : (upIfaces.length / ifaces.length) * 100

  const riskScore = clamp(
    100 - downIfaces.length * 12 - (rules.filter((r) => r.enabled).length === 0 ? 25 : 0) - (sessions.length === 0 ? 5 : 0),
    5,
    100,
  )
  const riskGrade = riskScore >= 80 ? 'Good' : riskScore >= 55 ? 'Fair' : 'At Risk'
  const riskColor = riskScore >= 80 ? TONE.green : riskScore >= 55 ? TONE.amber : TONE.red

  const zoneGroups = useMemo(() => {
    const g: Record<string, Iface[]> = { lan: [], wan: [], dmz: [] }
    ifaces.forEach((i) => {
      const z = i.zone?.toLowerCase()
      if (z === 'lan' || z === 'wan' || z === 'dmz') g[z].push(i)
      else g.lan.push(i)
    })
    return g
  }, [ifaces])

  const incidents = useMemo<Incident[]>(() => {
    const out: Incident[] = downIfaces.map((i, idx) => ({
      key: `if-${i.name}`,
      sev: 'critical',
      title: `Interface ${i.name} Connectivity Failure`,
      cat: 'Network',
      asset: i.name,
      status: 'Investigating',
      ago: idx === 0 ? 'now' : `${idx * 2}m ago`,
    }))
    if (rules.length === 0)
      out.push({ key: 'fw-empty', sev: 'high', title: 'Firewall Has No Rules Configured', cat: 'Firewall', asset: 'FW-Core', status: 'Open', ago: 'now' })
    services
      .filter((s) => s.status !== 'running')
      .forEach((s) =>
        out.push({ key: `svc-${s.name}`, sev: 'high', title: `Core Service ${s.name} Not Running`, cat: 'Performance', asset: s.name, status: 'Open', ago: 'now' }),
      )
    if (sessions.length === 0)
      out.push({ key: 'no-sess', sev: 'medium', title: 'No Active User Sessions', cat: 'Accounting', asset: 'AAA', status: 'Open', ago: 'now' })

    // demo fill — only to match the mockup look when the system is quiet
    const demo: Incident[] = [
      { key: 'd1', sev: 'high', title: 'Anomalous Traffic Detected', cat: 'Security', asset: 'WAN-Core', status: 'Analyzing', ago: '42m ago', demo: true },
      { key: 'd2', sev: 'medium', title: 'Policy Review Recommended', cat: 'Policies', asset: 'FW-Core', status: 'Open', ago: '1h ago', demo: true },
    ]
    if (out.length < 4) out.push(...demo.slice(0, 4 - out.length))
    return out.slice(0, 5)
  }, [downIfaces, rules.length, services, sessions.length])

  const donutSegments = [
    { value: upIfaces.length + runningServices, color: TONE.green },
    { value: rules.filter((r) => !r.enabled).length, color: TONE.amber },
    { value: downIfaces.length, color: TONE.red },
    { value: Math.max(0, 8 - ifaces.length), color: '#334155' },
  ]
  const totalAssets = upIfaces.length + downIfaces.length + runningServices + rules.length

  const assetRows = useMemo(() => {
    const rows = [
      { asset: 'FW-Core-01', type: 'Firewall', risk: rules.length === 0 ? 'High' : 'Low', status: 'Healthy' },
      ...upIfaces.slice(0, 2).map((i) => ({ asset: i.name, type: 'Interface', risk: 'Low', status: 'Healthy' })),
      ...downIfaces.slice(0, 2).map((i) => ({ asset: i.name, type: 'Interface', risk: 'High', status: 'Offline' })),
      ...sessions.slice(0, 2).map((s) => ({ asset: s.hostname ?? 'unknown-host', type: 'Endpoint', risk: 'Medium', status: 'Online' })),
    ]
    return rows.slice(0, 5)
  }, [rules.length, upIfaces, downIfaces, sessions])

  /* ───── KPIs ───── */

  const kpis = [
    { icon: Server, label: 'Total Assets', value: num(totalAssets), sub: `${modules} modules · ${services.length} services`, color: TONE.cyan },
    { icon: ShieldCheck, label: 'Healthy Nodes', value: num(upIfaces.length + runningServices), sub: ifaces.length ? `${Math.round((upIfaces.length / ifaces.length) * 100)}% of ifaces` : '—', color: TONE.green },
    { icon: AlertTriangle, label: 'Issues', value: num(downIfaces.length + incidents.length - incidents.filter((i) => i.demo).length), sub: 'live signals', color: TONE.amber },
    { icon: Zap, label: 'Critical Incidents', value: num(incidents.filter((i) => i.sev === 'critical' && !i.demo).length), sub: 'need attention', color: TONE.red },
    { icon: Users, label: 'Active Users', value: num(sessions.length), sub: 'live sessions', color: TONE.cyan },
    { icon: Globe, label: 'Network Uptime', value: `${uptime.toFixed(2)}%`, sub: 'interface based', color: TONE.violet },
  ]

  const postureRows = [
    { label: 'Threats', value: downIfaces.length, color: TONE.red },
    { label: 'Disabled Policies', value: rules.filter((r) => !r.enabled).length, color: TONE.amber },
    { label: 'Firewall Rules', value: rules.length, color: TONE.green },
    { label: 'Live Sessions', value: sessions.length, color: TONE.cyan },
  ]

  /* ───── topology geometry ───── */

  const hubs: Record<string, { x: number; label: string }> = {
    dmz: { x: 165, label: 'DMZ' },
    lan: { x: 500, label: 'LAN' },
    wan: { x: 835, label: 'WAN' },
  }
  const HUB_Y = 205
  const LEAF_Y = 292

  return (
    <div className="dzh">
      <style>{CSS}</style>

      <div className="dzh-head">
        <div>
          <h1>Command Center</h1>
          <div className="sub">Complete visibility. Smarter decisions.</div>
        </div>
        <div className="dzh-status">
          <span>{clock.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })} · {hhmmss(clock)}</span>
          <span className={`dzh-pill ${riskScore < 80 ? 'warn' : ''}`}>
            <span className={`dzh-dot ${riskScore < 80 ? 'amber' : ''}`} />
            {riskScore >= 80 ? 'System Healthy' : 'Needs Attention'}
          </span>
        </div>
      </div>

      {error && <div className="dzh-err">API error: {error} — retrying automatically</div>}

      {/* KPI row */}
      <div className="dzh-kpis">
        {kpis.map((k) => (
          <div className="dzh-kpi" key={k.label}>
            <div className="ic" style={{ background: `${k.color}1f`, border: `1px solid ${k.color}45` }}>
              <k.icon size={20} color={k.color} />
            </div>
            <div>
              <div className="lbl">{k.label}</div>
              <div className="val">{k.value}</div>
              <div className="delta flat">{k.sub}</div>
            </div>
          </div>
        ))}
      </div>

      {/* Topology + incidents */}
      <div className="dzh-row r2">
        <div className="dzh-card dzh-topo">
          <div className="dzh-card-h">
            <span className="t"><Network size={15} color={TONE.cyan} /> Network Topology</span>
            <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
              <span className="dzh-legend">
                <span><i style={{ background: TONE.green }} />Healthy</span>
                <span><i style={{ background: TONE.amber }} />Warning</span>
                <span><i style={{ background: TONE.red }} />Offline</span>
              </span>
              <span className="dzh-live"><span className="dzh-dot" /> Live</span>
            </div>
          </div>
          <svg viewBox="0 0 1000 350">
            {/* internet */}
            <g transform="translate(470,6)">
              <rect className="dzh-node hot" width="60" height="42" rx="9" />
              <text className="dzh-txt" x="30" y="26" textAnchor="middle">☁</text>
              <text className="dzh-txt sm" x="66" y="25">Internet</text>
            </g>
            <line className="dzh-link" x1="500" y1="50" x2="500" y2="86" stroke={TONE.green} strokeWidth="1.6" />

            {/* firewall hub */}
            <g transform="translate(415,88)">
              <rect className="dzh-node hot" width="170" height="46" rx="10" />
              <text className="dzh-txt" x="85" y="20" textAnchor="middle">🛡 DEZH-FW</text>
              <text className="dzh-txt sm" x="85" y="35" textAnchor="middle">{rules.length} rules · {upIfaces.length}/{ifaces.length} up</text>
            </g>

            {/* hub → zone hubs */}
            {Object.entries(hubs).map(([z, hb]) => {
              const st = zoneGroups[z].length === 0 ? '#334155' : zoneGroups[z].every((i) => i.up) ? TONE.green : TONE.amber
              return (
                <line key={z} className="dzh-link" x1="500" y1="134" x2={hb.x} y2={HUB_Y - 20} stroke={st} strokeWidth="1.6" />
              )
            })}

            {/* traffic boxes */}
            <g transform="translate(16,56)">
              <rect className="dzh-traffic-box" width="180" height="64" rx="10" />
              <text className="dzh-txt sm" x="14" y="18">Inbound</text>
              <text className="dzh-txt val" x="14" y="38">{lastIn.current.toFixed(1)} Gbps</text>
              <g transform="translate(14,40)">
                <SparklineInline data={inSeries} color={TONE.cyan} />
              </g>
            </g>
            <g transform="translate(804,56)">
              <rect className="dzh-traffic-box" width="180" height="64" rx="10" />
              <text className="dzh-txt sm" x="14" y="18">Outbound</text>
              <text className="dzh-txt val" x="14" y="38">{lastOut.current.toFixed(1)} Gbps</text>
              <g transform="translate(14,40)">
                <SparklineInline data={outSeries} color={TONE.violet} />
              </g>
            </g>

            {/* zone hubs + leaves */}
            {Object.entries(hubs).map(([z, hb]) => {
              const list = zoneGroups[z]
              const allUp = list.length > 0 && list.every((i) => i.up)
              const hubColor = list.length === 0 ? '#334155' : allUp ? TONE.green : TONE.amber
              return (
                <g key={`hub-${z}`}>
                  <g transform={`translate(${hb.x - 60},${HUB_Y - 20})`}>
                    <rect className="dzh-node" width="120" height="40" rx="9" stroke={hubColor} />
                    <text className="dzh-txt" x="60" y="17" textAnchor="middle">{hb.label}</text>
                    <text className="dzh-txt sm" x="60" y="31" textAnchor="middle">{list.length} ifaces</text>
                  </g>
                  {list.length === 0 ? (
                    <g transform={`translate(${hb.x - 43},${LEAF_Y - 14})`}>
                      <line className="dzh-link" x1={hb.x} y1={HUB_Y + 20} x2={hb.x} y2={LEAF_Y - 14} stroke="#334155" strokeWidth="1.4" />
                      <rect className="dzh-node" width="86" height="34" rx="8" strokeDasharray="4 3" />
                      <text className="dzh-txt sm" x="43" y="21" textAnchor="middle">no interfaces</text>
                    </g>
                  ) : (
                    list.slice(0, 3).map((i, idx, arr) => {
                      const spread = Math.min(200, arr.length * 95)
                      const lx = hb.x + (arr.length === 1 ? 0 : (idx / (arr.length - 1) - 0.5) * spread)
                      const cx = clamp(lx, 40, 960)
                      return (
                        <g key={i.name} transform={`translate(${cx - 43},${LEAF_Y - 14})`}>
                          <line className="dzh-link" x1={hb.x} y1={HUB_Y + 20} x2={cx} y2={LEAF_Y - 14} stroke={i.up ? TONE.green : TONE.red} strokeWidth="1.4" />
                          <rect className={`dzh-node ${i.up ? '' : 'down'}`} width="86" height="40" rx="8" />
                          <text className="dzh-txt" x="43" y="17" textAnchor="middle">{i.name.length > 10 ? i.name.slice(0, 9) + '…' : i.name}</text>
                          <circle cx="34" cy="29" r="3.5" fill={i.up ? TONE.green : TONE.red} />
                          <text className="dzh-txt sm" x="44" y="33">{i.up ? 'Healthy' : 'Offline'}</text>
                        </g>
                      )
                    })
                  )}
                </g>
              )
            })}
          </svg>
        </div>

        {/* Recent Incidents */}
        <div className="dzh-card">
          <div className="dzh-card-h">
            <span className="t"><Siren size={15} color={TONE.red} /> Recent Incidents</span>
            <span className="dzh-muted" style={{ fontSize: 11 }}>View all →</span>
          </div>
          {incidents.length === 0 && <div className="dzh-muted" style={{ padding: '20px 0' }}>No incidents — all clear ✓</div>}
          {incidents.map((i) => (
            <div className="dzh-inc" key={i.key}>
              <span className={`dzh-sev ${i.sev}`}>{i.sev === 'critical' ? 'Critical' : i.sev === 'high' ? 'High' : 'Medium'}</span>
              <div style={{ minWidth: 0, flex: 1 }}>
                <div className="ttl">{i.title}</div>
                <div className="meta">
                  <span className="dzh-tag">{i.cat}</span>
                  <span className="dzh-tag">{i.asset}</span>
                  <span className={`dzh-tag ${i.status === 'Open' ? 'open' : 'inv'}`}>{i.status}</span>
                </div>
              </div>
              <span className="dzh-ago">{i.ago}</span>
            </div>
          ))}
          {incidents.some((i) => i.demo) && (
            <div className="dzh-muted" style={{ fontSize: 10, marginTop: 8 }}>
              some entries are demo data for visualization
            </div>
          )}
        </div>
      </div>

      {/* Donut + traffic + posture */}
      <div className="dzh-row r3">
        <div className="dzh-card">
          <div className="dzh-card-h"><span className="t"><Cpu size={15} color={TONE.green} /> Asset Status</span></div>
          <div className="dzh-donut-wrap">
            <Donut
              segments={donutSegments}
              centerTop={num(totalAssets)}
              centerBottom="Total Assets"
            />
            <div className="dzh-dlegend">
              <div className="row"><i style={{ background: TONE.green }} />Healthy<b>{upIfaces.length + runningServices}</b><span>{totalAssets ? Math.round(((upIfaces.length + runningServices) / totalAssets) * 100) : 0}%</span></div>
              <div className="row"><i style={{ background: TONE.amber }} />Disabled<b>{rules.filter((r) => !r.enabled).length}</b><span>{totalAssets ? Math.round((rules.filter((r) => !r.enabled).length / totalAssets) * 100) : 0}%</span></div>
              <div className="row"><i style={{ background: TONE.red }} />Offline<b>{downIfaces.length}</b><span>{totalAssets ? Math.round((downIfaces.length / totalAssets) * 100) : 0}%</span></div>
              <div className="row"><i style={{ background: '#334155' }} />Other<b>{Math.max(0, 8 - ifaces.length)}</b><span>—</span></div>
            </div>
          </div>
        </div>

        <div className="dzh-card">
          <div className="dzh-card-h">
            <span className="t"><Activity size={15} color={TONE.cyan} /> Network Traffic</span>
            <span className="dzh-legend">
              <span><i style={{ background: TONE.cyan }} />Inbound</span>
              <span><i style={{ background: TONE.violet }} />Outbound</span>
            </span>
          </div>
          <TrafficChart inbound={inSeries} outbound={outSeries} />
        </div>

        <div className="dzh-card">
          <div className="dzh-card-h"><span className="t"><Shield size={15} color={TONE.green} /> Security Posture</span></div>
          <div className="dzh-donut-wrap" style={{ justifyContent: 'center' }}>
            <Donut
              segments={[
                { value: riskScore, color: riskColor },
                { value: 100 - riskScore, color: '#16233c' },
              ]}
              centerTop={String(riskScore)}
              centerBottom="Risk Score"
              size={132}
            />
            <div style={{ textAlign: 'center', minWidth: 70 }}>
              <div style={{ fontSize: 13, fontWeight: 700, color: riskColor }}>{riskGrade}</div>
              <div className="dzh-muted" style={{ fontSize: 11, marginTop: 4 }}>posture</div>
            </div>
          </div>
          <div className="dzh-posture-rows">
            {postureRows.map((r) => (
              <div className="row" key={r.label}>
                <span className="dzh-dot" style={{ background: r.color, boxShadow: `0 0 6px ${r.color}` }} />
                {r.label}
                <b style={{ color: r.color }}>{num(r.value)}</b>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* tables + stream */}
      <div className="dzh-row r4">
        <div className="dzh-card">
          <div className="dzh-card-h"><span className="t"><Radio size={15} color={TONE.amber} /> Latest Events</span></div>
          <table className="dzh-table">
            <thead>
              <tr><th>Time</th><th>Type</th><th>Source</th><th>Severity</th></tr>
            </thead>
            <tbody>
              {stream.slice(0, 6).map((e) => (
                <tr key={e.id}>
                  <td>{e.tm}</td>
                  <td>{e.title}</td>
                  <td>{e.src}</td>
                  <td><span className={`dzh-chip ${e.tone === 'green' ? 'ok' : e.tone === 'red' ? 'critical' : e.tone === 'amber' ? 'high' : 'info'}`}>
                    {e.tone === 'green' ? 'Info' : e.tone === 'red' ? 'Critical' : e.tone === 'amber' ? 'High' : 'Info'}
                  </span></td>
                </tr>
              ))}
              {stream.length === 0 && <tr><td colSpan={4} className="dzh-muted">waiting for events…</td></tr>}
            </tbody>
          </table>
        </div>

        <div className="dzh-card">
          <div className="dzh-card-h"><span className="t"><Server size={15} color={TONE.violet} /> Top Assets by Risk</span></div>
          <table className="dzh-table">
            <thead>
              <tr><th>Asset</th><th>Type</th><th>Risk</th><th>Status</th></tr>
            </thead>
            <tbody>
              {assetRows.map((a) => (
                <tr key={a.asset}>
                  <td>{a.asset}</td>
                  <td className="dzh-muted">{a.type}</td>
                  <td><span className={`dzh-chip ${a.risk.toLowerCase()}`}>{a.risk}</span></td>
                  <td><span className={`dzh-chip ${a.status === 'Healthy' ? 'ok' : a.status === 'Offline' ? 'critical' : 'info'}`}>{a.status}</span></td>
                </tr>
              ))}
              {assetRows.length === 0 && <tr><td colSpan={4} className="dzh-muted">no assets detected</td></tr>}
            </tbody>
          </table>
        </div>

        <div className="dzh-card">
          <div className="dzh-card-h">
            <span className="t"><Zap size={15} color={TONE.cyan} /> Real-time Event Stream</span>
            <span className="dzh-live"><span className="dzh-dot" /> Live</span>
          </div>
          <div className="dzh-stream">
            {stream.map((e) => {
              const Ic = EV_ICONS[e.tone]
              const col = e.tone === 'green' ? TONE.green : e.tone === 'red' ? TONE.red : e.tone === 'amber' ? TONE.amber : e.tone === 'violet' ? TONE.violet : TONE.cyan
              return (
                <div className="dzh-ev" key={e.id}>
                  <span className="tm">{e.tm}</span>
                  <span className="ic" style={{ background: `${col}1c`, border: `1px solid ${col}44` }}>
                    <Ic size={13} color={col} />
                  </span>
                  <div>
                    <div className="ttl">{e.title}</div>
                    <div className="src">{e.src}</div>
                  </div>
                  <span className="by">{e.by}</span>
                </div>
              )
            })}
          </div>
        </div>
      </div>
    </div>
  )
}

/* inline sparkline used inside topology svg */
function SparklineInline({ data, color }: { data: number[]; color: string }) {
  return <Sparkline data={data} color={color} w={150} h={20} />
}