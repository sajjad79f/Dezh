import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'

interface Identity {
  id: string
  username: string
  display_name: string | null
  source: string
  enabled: boolean
}

export function IdentitiesPage() {
  const [identities, setIdentities] = useState<Identity[]>([])
  const [error, setError] = useState<string | null>(null)
  const [username, setUsername] = useState('')
  const [displayName, setDisplayName] = useState('')
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
      const res = await fetch('/api/identities', { headers: headers() })
      if (!res.ok) throw new Error((await res.json()).error || res.statusText)
      setIdentities(await res.json())
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
    setLoading(true)
    try {
      const res = await fetch('/api/identities', {
        method: 'POST',
        headers: headers(true),
        body: JSON.stringify({
          username,
          display_name: displayName || null,
          source: 'local',
        }),
      })
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.statusText }))
        throw new Error(err.error || 'failed')
      }
      setUsername('')
      setDisplayName('')
      await load()
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div>
      <h1 className="page-title">Identities</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}
      <div className="grid grid-2">
        <div className="card">
          <h3>Add identity</h3>
          <form onSubmit={onCreate} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            <input
              placeholder="Username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
              style={inputStyle}
            />
            <input
              placeholder="Display name"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              style={inputStyle}
            />
            <button type="submit" disabled={loading} style={btnStyle}>
              {loading ? 'Creating…' : 'Create'}
            </button>
          </form>
        </div>
        <div className="card">
          <h3>All ({identities.length})</h3>
          <ul style={{ listStyle: 'none' }}>
            {identities.map((i) => (
              <li
                key={i.id}
                style={{
                  padding: '8px 0',
                  borderBottom: '1px solid var(--border)',
                  display: 'flex',
                  justifyContent: 'space-between',
                }}
              >
                <span>{i.username}</span>
                <span style={{ color: 'var(--text-muted)', fontSize: 12 }}>{i.source}</span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}

const inputStyle: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 8,
  padding: '10px 12px',
  color: 'var(--text)',
  fontSize: 13,
  outline: 'none',
}

const btnStyle: React.CSSProperties = {
  background: 'var(--accent-dim)',
  color: '#fff',
  border: 'none',
  borderRadius: 8,
  padding: '10px 16px',
  fontWeight: 600,
  cursor: 'pointer',
}