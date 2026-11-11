import { authHeaders, readJson, readEmpty } from './http'

/* ── Types ── */
export interface NetworkInterface {
  name: string
  zone: string
  up: boolean
  addresses: string[]
  enabled: boolean
  ipv4_mode: string
  address_cidr: string | null
  gateway: string | null
  description: string
}

export interface ZoneInfo {
  id: string
  name: string
  display_name: string
  accounting: boolean
  description: string
}

export interface NetworkSummary {
  total_interfaces: number
  up_interfaces: number
  down_interfaces: number
  zones: number
  rx_bytes: number
  tx_bytes: number
  generated_at: number
}

/* ── API ── */
const BASE = '/api'

export const networkApi = {
  summary: () =>
    fetch(`${BASE}/network/summary`, { headers: authHeaders() }).then(readJson<NetworkSummary>),

  interfaces: () =>
    fetch(`${BASE}/network/interfaces`, { headers: authHeaders() }).then(readJson<NetworkInterface[]>),

  zones: () =>
    fetch(`${BASE}/network/zones`, { headers: authHeaders() }).then(readJson<ZoneInfo[]>),

  createZone: (body: { name: string; display_name: string; accounting: boolean; description: string }) =>
    fetch(`${BASE}/network/zones`, {
      method: 'POST',
      headers: authHeaders(true),
      body: JSON.stringify(body),
    }).then(readEmpty),

  updateZone: (name: string, body: { display_name: string; accounting: boolean; description: string }) =>
    fetch(`${BASE}/network/zones/${encodeURIComponent(name)}`, {
      method: 'PATCH',
      headers: authHeaders(true),
      body: JSON.stringify(body),
    }).then(readEmpty),

  deleteZone: (name: string) =>
    fetch(`${BASE}/network/zones/${encodeURIComponent(name)}`, {
      method: 'DELETE',
      headers: authHeaders(),
    }).then(readEmpty),

  setZone: (name: string, zone: string) =>
    fetch(`${BASE}/network/interfaces/zone`, {
      method: 'POST',
      headers: authHeaders(true),
      body: JSON.stringify({ name, zone }),
    }).then(readEmpty),

  applyConfig: (body: {
    name: string
    zone: string
    enabled: boolean
    ipv4_mode: string
    address_cidr: string | null
    gateway: string | null
    description: string
  }) =>
    fetch(`${BASE}/network/interfaces/config`, {
      method: 'POST',
      headers: authHeaders(true),
      body: JSON.stringify(body),
    }).then(readEmpty),

  renameInterface: (name: string, new_name: string) =>
    fetch(`${BASE}/network/interfaces/rename`, {
      method: 'POST',
      headers: authHeaders(true),
      body: JSON.stringify({ name, new_name }),
    }).then(readEmpty),
  pppoeState: () =>
    fetch(`${BASE}/network/pppoe`, { headers: authHeaders() }).then(readJson<PppoeState>),

  pppoeSave: (body: {
    name: string
    parent_iface: string
    username: string
    password: string
    mtu: number
    default_route: boolean
    keepalive: number
    enabled: boolean
    description: string
  }) =>
    fetch(`${BASE}/network/pppoe`, {
      method: 'POST',
      headers: authHeaders(true),
      body: JSON.stringify(body),
    }).then(readEmpty),

  pppoeDelete: (name: string) =>
    fetch(`${BASE}/network/pppoe/${encodeURIComponent(name)}`, {
      method: 'DELETE',
      headers: authHeaders(),
    }).then(readEmpty),

  pppoeStart: (name: string) =>
    fetch(`${BASE}/network/pppoe/${encodeURIComponent(name)}/start`, {
      method: 'POST',
      headers: authHeaders(),
    }).then(readEmpty),

  pppoeStop: (name: string) =>
    fetch(`${BASE}/network/pppoe/${encodeURIComponent(name)}/stop`, {
      method: 'POST',
      headers: authHeaders(),
    }).then(readEmpty),

  vpnState: () =>
    fetch(`${BASE}/network/vpn`, { headers: authHeaders() }).then(readJson<VpnWanState>),

  vpnSave: (body: {
    name: string
    kind: 'pptp' | 'l2tp'
    server: string
    username: string
    password: string
    mtu: number
    default_route: boolean
    keepalive: number
    enabled: boolean
    description: string
  }) =>
    fetch(`${BASE}/network/vpn`, {
      method: 'POST',
      headers: authHeaders(true),
      body: JSON.stringify(body),
    }).then(readEmpty),

  vpnDelete: (name: string) =>
    fetch(`${BASE}/network/vpn/${encodeURIComponent(name)}`, {
      method: 'DELETE',
      headers: authHeaders(),
    }).then(readEmpty),

  vpnStart: (name: string) =>
    fetch(`${BASE}/network/vpn/${encodeURIComponent(name)}/start`, {
      method: 'POST',
      headers: authHeaders(),
    }).then(readEmpty),

  vpnStop: (name: string) =>
    fetch(`${BASE}/network/vpn/${encodeURIComponent(name)}/stop`, {
      method: 'POST',
      headers: authHeaders(),
    }).then(readEmpty),
}


/* ── N2: PPPoE types ── */
export interface PppoeSession {
  iface: string
  profile: string | null
  local_ip: string | null
  remote_ip: string | null
  up: boolean
}

export interface PppoeProfile {
  name: string
  parent_iface: string
  username: string
  mtu: number
  default_route: boolean
  keepalive: number
  enabled: boolean
  description: string
  session: PppoeSession | null
}

export interface PppoeState {
  profiles: PppoeProfile[]
  sessions: PppoeSession[]
}


/* ── N3: VPN WAN (PPTP / L2TP) types ── */
export interface VpnConnection {
  name: string
  kind: 'pptp' | 'l2tp'
  server: string
  username: string
  mtu: number
  default_route: boolean
  keepalive: number
  enabled: boolean
  description: string
  session: PppoeSession | null
}

export interface VpnWanState {
  connections: VpnConnection[]
  sessions: PppoeSession[]
}
