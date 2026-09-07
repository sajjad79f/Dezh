import { useEffect, useState } from 'react'
import { getToken } from '../../api/auth'

interface Iface {
  name: string
  zone: string
  up: boolean
  addresses: string[]
}

export function FirewallInterfaces() {
  const [ifaces, setIfaces] = useState<Iface[]>([])
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  function headers(json = false): HeadersInit {
    const token = getToken()
    return {
      ...(json ? { 'Content-Type': 'application/json' } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    }
  }

  async function load() {
    try {
      const res = await fetch('/api/firewall/interfaces', { headers: headers() })
      if (!res.ok) throw new Error((await res.json()).error || res.statusText)
      setIfaces(await res.json())
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }

  useEffect(() => {
    load()
  }, [])

  async function setZone(name: string, zone: string) {
    setLoading(true)
    try {
      const res = await fetch('/api/firewall/interfaces/zone', {
        method: 'POST',
        headers: headers(true),
        body: JSON.stringify({ name, zone }),
      })
      if (!res.ok && res.status !== 204) {
        const err = await res.json().catch(() => ({ error: res.statusText }))
        throw new Error(err.error || res.statusText)
      }
      await load()
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div>
      <h1 className="page-title">Firewall · Interfaces</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}
      <div className="card">
        <p style={{ color: 'var(--text-muted)', fontSize: 13, marginBottom: 12 }}>
          برای شمارش ترافیک اینترنت، یک اینترفیس را <strong>wan</strong> کنید.
        </p>
        {ifaces.length === 0 ? (
          <p style={{ color: 'var(--text-muted)' }}>No interfaces</p>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ textAlign: 'left', color: 'var(--text-muted)' }}>
                <th style={{ padding: '8px 4px' }}>Name</th>
                <th>Status</th>
                <th>Addresses</th>
                <th>Zone</th>
              </tr>
            </thead>
            <tbody>
              {ifaces.map((i) => (
                <tr key={i.name} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: '10px 4px' }}>{i.name}</td>
                  <td>{i.up ? 'UP' : 'DOWN'}</td>
                  <td>{i.addresses.join(', ') || '—'}</td>
                  <td>
                    <select
                      value={i.zone}
                      disabled={loading}
                      onChange={(e) => setZone(i.name, e.target.value)}
                      style={{
                        background: 'var(--bg)',
                        color: 'var(--text)',
                        border: '1px solid var(--border)',
                        borderRadius: 6,
                        padding: '4px 8px',
                      }}
                    >
                      <option value="lan">lan</option>
                      <option value="wan">wan</option>
                      <option value="dmz">dmz</option>
                    </select>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}