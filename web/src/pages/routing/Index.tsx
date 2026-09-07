import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'

interface RouteRow {
  destination: string
  gateway: string | null
  device: string | null
  proto: string | null
  metric: number | null
}

export function RoutingPage() {
  const [routes, setRoutes] = useState<RouteRow[]>([])
  const [error, setError] = useState<string | null>(null)
  const [gateway, setGateway] = useState('')
  const [device, setDevice] = useState('')

  function headers(json = false): HeadersInit {
    const token = getToken()
    return {
      ...(json ? { 'Content-Type': 'application/json' } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    }
  }

  async function load() {
    try {
      const res = await fetch('/api/routing/routes', { headers: headers() })
      if (!res.ok) throw new Error((await res.json()).error || res.statusText)
      setRoutes(await res.json())
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
    await load()
  }

  return (
    <div>
      <h1 className="page-title">Routing</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}
      <div className="card" style={{ marginBottom: 16 }}>
        <h3>Default gateway</h3>
        <form onSubmit={setDefault} style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input
            placeholder="Gateway IP"
            value={gateway}
            onChange={(e) => setGateway(e.target.value)}
            required
            style={field}
          />
          <input
            placeholder="Device (optional)"
            value={device}
            onChange={(e) => setDevice(e.target.value)}
            style={field}
          />
          <button type="submit" style={btn}>
            Apply
          </button>
        </form>
      </div>
      <div className="card">
        <h3>Routes ({routes.length})</h3>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
              <th style={{ padding: 8 }}>Destination</th>
              <th>Gateway</th>
              <th>Device</th>
              <th>Proto</th>
              <th>Metric</th>
            </tr>
          </thead>
          <tbody>
            {routes.map((r, i) => (
              <tr key={i} style={{ borderTop: '1px solid var(--border)' }}>
                <td style={{ padding: 8 }}>{r.destination}</td>
                <td>{r.gateway ?? '—'}</td>
                <td>{r.device ?? '—'}</td>
                <td>{r.proto ?? '—'}</td>
                <td>{r.metric ?? '—'}</td>
              </tr>
            ))}
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