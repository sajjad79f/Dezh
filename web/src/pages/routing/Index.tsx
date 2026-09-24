import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'

interface RouteRow {
  destination: string
  gateway: string | null
  device: string | null
  proto: string | null
  metric: number | null
}

interface IfaceRow {
  name: string
  zone: string
  up: boolean
  addresses: string[]
}

export function RoutingPage() {
  const [routes, setRoutes] = useState<RouteRow[]>([])
  const [ifaces, setIfaces] = useState<IfaceRow[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)

  const [gateway, setGateway] = useState('')
  const [device, setDevice] = useState('')

  const [dest, setDest] = useState('')
  const [via, setVia] = useState('')
  const [dev, setDev] = useState('')

  function headers(json = false): HeadersInit {
    const token = getToken()
    return {
      ...(json ? { 'Content-Type': 'application/json' } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    }
  }

  async function loadRoutes() {
    const res = await fetch('/api/routing/routes', { headers: headers() })
    if (!res.ok) throw new Error((await res.json()).error || res.statusText)
    setRoutes(await res.json())
  }

  async function loadIfaces() {
    const res = await fetch('/api/firewall/interfaces', { headers: headers() })
    if (!res.ok) throw new Error((await res.json()).error || res.statusText)
    setIfaces(await res.json())
  }

  async function load() {
    try {
      await Promise.all([loadRoutes(), loadIfaces()])
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }

  useEffect(() => {
    load()
  }, [])

  async function setDefault(e: FormEvent) {
    e.preventDefault()
    setOk(null)
    const res = await fetch('/api/routing/default', {
      method: 'POST',
      headers: headers(true),
      body: JSON.stringify({ gateway, device: device || null }),
    })
    if (!res.ok && res.status !== 204) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error || res.statusText)
      return
    }
    setOk('Default gateway applied')
    setError(null)
    await loadRoutes()
  }

  async function addStatic(e: FormEvent) {
    e.preventDefault()
    setOk(null)
    if (!via.trim() && !dev.trim()) {
      setError('Gateway یا Device را مشخص کنید')
      return
    }
    const res = await fetch('/api/routing/routes', {
      method: 'POST',
      headers: headers(true),
      body: JSON.stringify({
        destination: dest.trim(),
        gateway: via.trim() || null,
        device: dev.trim() || null,
      }),
    })
    if (!res.ok && res.status !== 204) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error || res.statusText)
      return
    }
    setDest('')
    setVia('')
    setDev('')
    setOk('Route added')
    setError(null)
    await loadRoutes()
  }

  async function removeRoute(r: RouteRow) {
    if (!confirm(`Delete route ${r.destination}?`)) return
    setOk(null)
    const res = await fetch('/api/routing/routes', {
      method: 'DELETE',
      headers: headers(true),
      body: JSON.stringify({
        destination: r.destination,
        gateway: r.gateway,
        device: r.device,
      }),
    })
    if (!res.ok && res.status !== 204) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error || res.statusText)
      return
    }
    setOk('Route deleted')
    setError(null)
    await loadRoutes()
  }

  const selectStyle: React.CSSProperties = {
    ...field,
    minWidth: 160,
  }

  return (
    <div>
      <h1 className="page-title">Routing</h1>

      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}
      {ok && (
        <div className="card" style={{ color: '#7ddb8a', marginBottom: 16 }}>
          {ok}
        </div>
      )}

      <div className="card" style={{ marginBottom: 16 }}>
        <h3 style={{ marginTop: 0 }}>Default gateway</h3>
        <form onSubmit={setDefault} style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input
            placeholder="Gateway IP (e.g. 192.168.1.1)"
            value={gateway}
            onChange={(e) => setGateway(e.target.value)}
            required
            style={field}
          />
          <select
            value={device}
            onChange={(e) => setDevice(e.target.value)}
            style={selectStyle}
          >
            <option value="">Device (optional)</option>
            {ifaces.map((i) => (
              <option key={i.name} value={i.name}>
                {i.name}
                {i.up ? ' ↑' : ' ↓'} [{i.zone}]
                {i.addresses[0] ? ` — ${i.addresses[0]}` : ''}
              </option>
            ))}
          </select>
          <button type="submit" style={btn}>
            Apply
          </button>
        </form>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <h3 style={{ marginTop: 0 }}>Add static route</h3>
        <p style={{ fontSize: 12, color: 'var(--text-muted)', marginTop: 0 }}>
          Destination مثل <code>10.0.0.0/8</code> — حداقل یکی از Gateway یا Device لازم است.
        </p>
        <form onSubmit={addStatic} style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input
            placeholder="Destination (e.g. 10.0.0.0/8)"
            value={dest}
            onChange={(e) => setDest(e.target.value)}
            required
            style={field}
          />
          <input
            placeholder="Via gateway (optional)"
            value={via}
            onChange={(e) => setVia(e.target.value)}
            style={field}
          />
          <select value={dev} onChange={(e) => setDev(e.target.value)} style={selectStyle}>
            <option value="">Device (optional)</option>
            {ifaces.map((i) => (
              <option key={i.name} value={i.name}>
                {i.name}
                {i.up ? ' ↑' : ' ↓'} [{i.zone}]
                {i.addresses[0] ? ` — ${i.addresses[0]}` : ''}
              </option>
            ))}
          </select>
          <button type="submit" style={btn}>
            Add route
          </button>
        </form>
      </div>

      <div className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h3 style={{ margin: 0 }}>Routes ({routes.length})</h3>
          <button
            type="button"
            onClick={() => load()}
            style={{
              ...btn,
              background: 'transparent',
              border: '1px solid var(--border)',
              color: 'var(--text-muted)',
            }}
          >
            Refresh
          </button>
        </div>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13, marginTop: 12 }}>
          <thead>
            <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
              <th style={{ padding: 8 }}>Destination</th>
              <th>Gateway</th>
              <th>Device</th>
              <th>Proto</th>
              <th>Metric</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {routes.length === 0 ? (
              <tr>
                <td colSpan={6} style={{ padding: 12, color: 'var(--text-muted)' }}>
                  No routes
                </td>
              </tr>
            ) : (
              routes.map((r, i) => (
                <tr key={i} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: 8 }}>{r.destination}</td>
                  <td>{r.gateway ?? '—'}</td>
                  <td>{r.device ?? '—'}</td>
                  <td>{r.proto ?? '—'}</td>
                  <td>{r.metric ?? '—'}</td>
                  <td style={{ padding: 8 }}>
                    <button
                      type="button"
                      onClick={() => removeRoute(r)}
                      style={{
                        padding: '4px 10px',
                        fontSize: 12,
                        cursor: 'pointer',
                        borderRadius: 6,
                        border: '1px solid rgba(255,123,123,0.4)',
                        background: 'rgba(255,123,123,0.08)',
                        color: '#ff9b9b',
                      }}
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}

const field: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 8,
  padding: '8px 12px',
  color: 'var(--text)',
  fontSize: 13,
}

const btn: React.CSSProperties = {
  background: 'var(--accent-dim)',
  color: '#fff',
  border: 'none',
  borderRadius: 8,
  padding: '8px 16px',
  fontWeight: 600,
  cursor: 'pointer',
}