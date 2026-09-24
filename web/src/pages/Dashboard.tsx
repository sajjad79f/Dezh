import { useEffect, useState } from 'react'
import { getToken } from '../api/auth'
import { api } from '../api/client'

type Iface = {
  name: string
  zone: string
  up: boolean
  addresses: string[]
}

type Session = {
  id: string
  hostname: string | null
  ip_address: string | null
  live_bytes_in: number
  live_bytes_out: number
}

function headers(): HeadersInit {
  const token = getToken()
  return token ? { Authorization: `Bearer ${token}` } : {}
}

function fmtBytes(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return '0 B'
  if (n < 1024) return `${n} B`
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`
  if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`
  return `${(n / 1024 ** 3).toFixed(2)} GB`
}

export function Dashboard() {
  const [modules, setModules] = useState(0)
  const [services, setServices] = useState(0)
  const [rules, setRules] = useState(0)
  const [ifaces, setIfaces] = useState<Iface[]>([])
  const [sessions, setSessions] = useState<Session[]>([])
  const [error, setError] = useState<string | null>(null)

  async function load() {
    try {
      const [m, s, r, ifaceRes, sessRes] = await Promise.all([
        api.modules(),
        api.coreServices(),
        fetch('/api/firewall/rules', { headers: headers() }).then(async (res) => {
          if (!res.ok) return []
          return res.json()
        }),
        fetch('/api/firewall/interfaces', { headers: headers() }).then(async (res) => {
          if (!res.ok) return []
          return res.json()
        }),
        fetch('/api/accounting/sessions', { headers: headers() }).then(async (res) => {
          if (!res.ok) return []
          return res.json()
        }),
      ])
      setModules(m.length)
      setServices(s.length)
      setRules(Array.isArray(r) ? r.length : 0)
      setIfaces(ifaceRes)
      setSessions(sessRes)
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }

  useEffect(() => {
    load()
    const t = setInterval(load, 15_000)
    return () => clearInterval(t)
  }, [])

  const upCount = ifaces.filter((i) => i.up).length

  return (
    <div>
      <h1 className="page-title">Dashboard</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}

      <div className="grid grid-2" style={{ marginBottom: 16 }}>
        <div className="card">
          <h3>Modules</h3>
          <p style={{ fontSize: 28, fontWeight: 700, margin: 0 }}>{modules}</p>
        </div>
        <div className="card">
          <h3>Core Services</h3>
          <p style={{ fontSize: 28, fontWeight: 700, margin: 0 }}>{services}</p>
        </div>
        <div className="card">
          <h3>Firewall rules</h3>
          <p style={{ fontSize: 28, fontWeight: 700, margin: 0 }}>{rules}</p>
        </div>
        <div className="card">
          <h3>Active sessions</h3>
          <p style={{ fontSize: 28, fontWeight: 700, margin: 0 }}>{sessions.length}</p>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <h3 style={{ marginTop: 0 }}>
          Interfaces ({upCount}/{ifaces.length} up)
        </h3>
        {ifaces.length === 0 ? (
          <p style={{ color: 'var(--text-muted)' }}>No interfaces detected</p>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
                <th style={{ padding: 8 }}>Name</th>
                <th>Zone</th>
                <th>State</th>
                <th>Addresses</th>
              </tr>
            </thead>
            <tbody>
              {ifaces.map((i) => (
                <tr key={i.name} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: 8 }}>{i.name}</td>
                  <td>{i.zone}</td>
                  <td style={{ color: i.up ? '#7ddb8a' : '#f0b429' }}>
                    {i.up ? 'UP' : 'DOWN'}
                  </td>
                  <td>{i.addresses.length ? i.addresses.join(', ') : '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Live sessions</h3>
        {sessions.length === 0 ? (
          <p style={{ color: 'var(--text-muted)' }}>No active sessions</p>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
                <th style={{ padding: 8 }}>Host</th>
                <th>IP</th>
                <th>↓ In</th>
                <th>↑ Out</th>
              </tr>
            </thead>
            <tbody>
              {sessions.slice(0, 10).map((s) => (
                <tr key={s.id} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: 8 }}>{s.hostname ?? '—'}</td>
                  <td>{s.ip_address ?? '—'}</td>
                  <td>{fmtBytes(s.live_bytes_in)}</td>
                  <td>{fmtBytes(s.live_bytes_out)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}