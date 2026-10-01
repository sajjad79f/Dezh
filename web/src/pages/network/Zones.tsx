import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'
import { authHeaders, readJson, readEmpty } from '../../api/http'

interface Zone {
  id: string
  name: string
  display_name: string
  accounting: boolean
  description: string
}

export function NetworkZones() {
  const [zones, setZones] = useState<Zone[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)

  const [name, setName] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [accounting, setAccounting] = useState(false)
  const [description, setDescription] = useState('')

  function headers(json = false): HeadersInit {
    const token = getToken()
    return {
      ...(json ? { 'Content-Type': 'application/json' } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    }
  }

  async function load() {
    try {
      const res = await fetch('/api/network/zones', { headers: authHeaders() })
      setZones(await readJson(res))
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }

  useEffect(() => {
    load()
  }, [])

  async function onCreate(e: FormEvent) {
    e.preventDefault()
    setOk(null)
    try {
      const res = await fetch('/api/network/zones', {
        method: 'POST',
        headers: authHeaders(true),
        body: JSON.stringify({
          name,
          display_name: displayName || name,
          accounting,
          description,
        }),
      })
      await readJson(res) // 201 با { id } یا اگر 204 شد هم امن است
      setName('')
      setDisplayName('')
      setAccounting(false)
      setDescription('')
      setOk('Zone created')
      setError(null)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function toggleAccounting(z: Zone) {
    setOk(null)
    try {
      const res = await fetch(`/api/network/zones/${encodeURIComponent(z.name)}`, {
        method: 'PATCH',
        headers: authHeaders(true),
        body: JSON.stringify({
          display_name: z.display_name,
          accounting: !z.accounting,
          description: z.description,
        }),
      })
      await readEmpty(res)
      setOk(`Accounting ${!z.accounting ? 'ON' : 'OFF'} for ${z.name}`)
      setError(null)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onDelete(z: Zone) {
    if (!confirm(`Delete zone "${z.name}"?`)) return
    try {
      const res = await fetch(`/api/network/zones/${encodeURIComponent(z.name)}`, {
        method: 'DELETE',
        headers: authHeaders(),
      })
      await readEmpty(res)
      setOk('Deleted')
      setError(null)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  return (
    <div>
      <h1 className="page-title">Network · Zones</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        زون دلخواه بساز. اگر <strong>Accounting</strong> روشن باشد، ترافیک اینترفیس‌های آن زون شمارش می‌شود.
      </p>

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
        <h3 style={{ marginTop: 0 }}>Create zone</h3>
        <form onSubmit={onCreate} style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
          <input
            placeholder="name (e.g. guest)"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            style={field}
          />
          <input
            placeholder="Display name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            style={field}
          />
          <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 13 }}>
            <input
              type="checkbox"
              checked={accounting}
              onChange={(e) => setAccounting(e.target.checked)}
            />
            Accounting
          </label>
          <input
            placeholder="Description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            style={field}
          />
          <button type="submit" style={btn}>
            Create
          </button>
        </form>
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Zones ({zones.length})</h3>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
              <th style={{ padding: 8 }}>Name</th>
              <th>Display</th>
              <th>Accounting</th>
              <th>Description</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {zones.map((z) => (
              <tr key={z.id} style={{ borderTop: '1px solid var(--border)' }}>
                <td style={{ padding: 8 }}>
                  <code>{z.name}</code>
                </td>
                <td>{z.display_name}</td>
                <td>
                  <button type="button" onClick={() => toggleAccounting(z)} style={chip(z.accounting)}>
                    {z.accounting ? 'ON' : 'OFF'}
                  </button>
                </td>
                <td>{z.description || '—'}</td>
                <td style={{ padding: 8 }}>
                  <button type="button" onClick={() => onDelete(z)} style={delBtn}>
                    Delete
                  </button>
                </td>
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

const delBtn: React.CSSProperties = {
  padding: '4px 10px',
  fontSize: 12,
  cursor: 'pointer',
  borderRadius: 6,
  border: '1px solid rgba(255,123,123,0.4)',
  background: 'rgba(255,123,123,0.08)',
  color: '#ff9b9b',
}

function chip(on: boolean): React.CSSProperties {
  return {
    padding: '4px 10px',
    fontSize: 12,
    borderRadius: 6,
    cursor: 'pointer',
    border: on ? '1px solid rgba(125,219,138,0.5)' : '1px solid var(--border)',
    background: on ? 'rgba(125,219,138,0.12)' : 'transparent',
    color: on ? '#7ddb8a' : 'var(--text-muted)',
  }
}