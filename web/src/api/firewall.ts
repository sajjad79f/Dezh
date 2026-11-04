import { authHeaders, readJson, readEmpty } from './http'

// ─── تایپ‌ها ────────────────────────────────────────────

export interface FirewallRule {
  id: string
  name: string
  action: string
  direction: string
  protocol: string
  source: string
  destination: string
  port: number | null
  enabled: boolean
  priority: number
  description: string
  log: boolean
  source_set: string | null
  dest_set: string | null
  rate_limit: number | null
  // ── فاز ۴: زمان‌بندی (محاسبه‌ی active با ساعت سرور) ──
  schedule_enabled: boolean
  schedule_start: string
  schedule_end: string
  schedule_days: string[]
  schedule_active: boolean
}

export interface RuleStat {
  rule_id: string
  packets: number
  bytes: number
}

export interface IpSet {
  id: string
  name: string
  kind: 'blacklist' | 'whitelist'
  description: string
  enabled: boolean
  entries_count: number
  entries: string[]
}

// ─── فاز ۵: اشیاء FQDN ─────────────────────────────────

export interface FqdnObject {
  id: string
  name: string
  fqdn: string
  description: string
  enabled: boolean
  /** کش رزولوشن — IPv4 و IPv6 مخلوط (تفکیک با وجود ':') */
  resolved: string[]
  last_resolved_at: string | null
  last_error: string | null
  created_at: string
  updated_at: string
}

export interface Connection {
  protocol: string
  state: string
  src: string
  dst: string
  sport: number | null
  dport: number | null
  timeout_secs: number | null
}

export interface FirewallSettings {
  dos_protection: boolean
  syn_rate: number
}

export interface NatRule {
  id: string
  name: string
  enabled: boolean
  kind: string
  interface: string
  source: string
  destination: string
  protocol: string
  dest_port: number | null
  target: string | null
  description: string
}

export interface FirewallExportData {
  version: number
  exported_at: string
  rules: FirewallRule[]
  nat_rules: NatRule[]
  ip_sets: IpSet[]
}

export interface ImportResult {
  rules_added: number
  rules_skipped: number
  nat_added: number
  nat_skipped: number
  sets_added: number
  sets_skipped: number
}

// ─── فاز ۳: Security Events ────────────────────────────

export interface SecurityEvent {
  id: string
  created_at: string
  kind: string
  severity: string
  source_ip: string | null
  dest_port: number | null
  rule_name: string | null
  hits: number
  message: string
}

export interface EventKindCount {
  kind: string
  count: number
}

export interface EventSourceCount {
  ip: string
  count: number
}

export interface EventPortCount {
  port: number
  count: number
}

export interface EventTimePoint {
  hour: string
  count: number
}

export interface EventSummary {
  since_hours: number
  total_hits: number
  by_kind: EventKindCount[]
  top_sources: EventSourceCount[]
  top_ports: EventPortCount[]
  timeline: EventTimePoint[]
}

// ─── helpers ────────────────────────────────────────────

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`/api${path}`, { headers: authHeaders() })
  return readJson<T>(res)
}

/** POST/PATCH/PUT/DELETE — یا JSON برمی‌گرداند یا 204 */
async function send<T>(path: string, method: string, body?: unknown): Promise<T> {
  const res = await fetch(`/api${path}`, {
    method,
    headers: authHeaders(body !== undefined),
    body: body !== undefined ? JSON.stringify(body) : undefined,
  })

  const ct = res.headers.get('content-type') || ''
  if (res.status === 204 || !ct.includes('json')) {
    await readEmpty(res) // اگر !ok باشد throw می‌کند
    return undefined as T
  }
  return readJson<T>(res)
}

// ─── API ────────────────────────────────────────────────

export const firewallApi = {
  // rules
  listRules: () => get<FirewallRule[]>('/firewall/rules'),
  createRule: (body: Record<string, unknown>) =>
    send<{ id: string }>('/firewall/rules', 'POST', body),
  updateRule: (id: string, body: Record<string, unknown>) =>
    send<void>(`/firewall/rules/${encodeURIComponent(id)}`, 'PATCH', body),
  deleteRule: (id: string) => send<void>(`/firewall/rules/${encodeURIComponent(id)}`, 'DELETE'),
  stats: () => get<RuleStat[]>('/firewall/stats'),

  // security events (فاز ۳)
  events: (limit = 200, hours = 24) =>
    get<SecurityEvent[]>(
      `/events?limit=${limit}&hours=${hours}`,
    ),
  eventSummary: (hours = 24) =>
    get<EventSummary>(`/events/summary?hours=${hours}`),

  // address lists
  listSets: () => get<IpSet[]>('/firewall/sets'),
  createSet: (body: { name: string; kind: string; description?: string; enabled?: boolean }) =>
    send<{ id: string }>('/firewall/sets', 'POST', body),
  deleteSet: (name: string) => send<void>(`/firewall/sets/${encodeURIComponent(name)}`, 'DELETE'),
  toggleSet: (name: string, enabled: boolean) =>
    send<void>(`/firewall/sets/${encodeURIComponent(name)}`, 'PATCH', { enabled }),
  addEntries: (name: string, entries: string[]) =>
    send<{ added: number }>(`/firewall/sets/${encodeURIComponent(name)}/entries`, 'POST', {
      entries,
    }),
  removeEntry: (name: string, cidr: string) =>
    send<void>(`/firewall/sets/${encodeURIComponent(name)}/entries`, 'DELETE', { cidr }),

  // fqdn objects (فاز ۵)
  listFqdn: () => get<FqdnObject[]>('/firewall/fqdn'),
  createFqdn: (body: {
    name: string
    fqdn: string
    description?: string
    enabled?: boolean
  }) => send<{ id: string }>('/firewall/fqdn', 'POST', body),
  deleteFqdn: (name: string) =>
    send<void>(`/firewall/fqdn/${encodeURIComponent(name)}`, 'DELETE'),
  toggleFqdn: (name: string, enabled: boolean) =>
    send<void>(`/firewall/fqdn/${encodeURIComponent(name)}`, 'PATCH', { enabled }),  

  // monitoring & protection
  connections: () => get<Connection[]>('/firewall/connections'),
  settings: () => get<FirewallSettings>('/firewall/settings'),
  saveSettings: (body: Partial<FirewallSettings>) =>
    send<void>('/firewall/settings', 'PUT', body),

  // import / export
  exportConfig: () => get<FirewallExportData>('/firewall/export'),
  importConfig: (data: FirewallExportData) => send<ImportResult>('/firewall/import', 'POST', data),
}

/** نمایش خوانا برای تعداد بایت */
export function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`
}