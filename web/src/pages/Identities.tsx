import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../api/auth'

interface Identity {
  id: string
  username: string
  display_name: string | null
  source: string
  enabled: boolean
}

interface Session {
  id: string
  identity_id: string | null
  protocol: string
  ip_address: string | null
  started_at: string
  bytes_in: number
  bytes_out: number
}

export function Identities() {
  const [identities, setIdentities] = useState<Identity[]>([])
  const [sessions, setSessions] = useState<Session[]>([])
  const [error, setError] = useState<string | null>(null)
  const [username, setUsername] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [loading, setLoading] = useState(false)
  const [protocol, setProtocol] = useState('vpn')
  const [ip, setIp] = useState('10.0.0.10')
  const [selectedIdentity, setSelectedIdentity] = useState('')

  function headers(): HeadersInit {
    const token = getToken()
    return token ? { Authorization: `Bearer ${token}` } : {}
  }

  async function load() {
    try {
      const [iRes, sRes] = await Promise.all([
        fetch('/api/identities', { headers: headers() }),
        fetch('/api/accounting/sessions', { headers: headers() }),
      ])
      if (!iRes.ok) throw new Error((await iRes.json()).error || iRes.statusText)
      if (!sRes.ok) throw new Error((await sRes.json()).error || sRes.statusText)
      setIdentities(await iRes.json())
      setSessions(await sRes.json())
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
        headers: {
          'Content-Type': 'application/json',
          ...headers(),
        },
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

  async function startSession() {
    setLoading(true)
    try {
      const res = await fetch('/api/accounting/sessions/start', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers(),
        },
        body: JSON.stringify({
          identity_id: selectedIdentity || null,
          protocol,
          ip_address: ip || null,
        }),
      })
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.statusText }))
        throw new Error(err.error || 'start failed')
      }
      await load()
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  async function endSession(sessionId: string) {
    setLoading(true)
    try {
      const res = await fetch('/api/accounting/sessions/end', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers(),
        },
        body: JSON.stringify({
          session_id: sessionId,
          bytes_in: 1024,
          bytes_out: 2048,
          terminate_cause: 'manual-test',
        }),
      })
      if (!res.ok && res.status !== 204) {
        const err = await res.json().catch(() => ({ error: res.statusText }))
        throw new Error(err.error || 'end failed')
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
      <h1 className="page-title">Network Identities</h1>
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

          <h3 style={{ marginTop: 24 }}>Identities ({identities.length})</h3>
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

        <div className="card">
          <h3>Active sessions ({sessions.length})</h3>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 8, marginBottom: 16 }}>
            <select
              value={selectedIdentity}
              onChange={(e) => setSelectedIdentity(e.target.value)}
              style={inputStyle}
            >
              <option value="">(no identity)</option>
              {identities.map((i) => (
                <option key={i.id} value={i.id}>
                  {i.username}
                </option>
              ))}
            </select>
            <select
              value={protocol}
              onChange={(e) => setProtocol(e.target.value)}
              style={inputStyle}
            >
              <option value="vpn">vpn</option>
              <option value="nac">nac</option>
              <option value="portal">portal</option>
              <option value="other">other</option>
            </select>
            <input
              placeholder="IP address"
              value={ip}
              onChange={(e) => setIp(e.target.value)}
              style={inputStyle}
            />
            <button type="button" onClick={startSession} disabled={loading} style={btnStyle}>
              Start test session
            </button>
          </div>

          {sessions.length === 0 ? (
            <p style={{ color: 'var(--text-muted)' }}>No active network sessions</p>
          ) : (
            <ul style={{ listStyle: 'none' }}>
              {sessions.map((s) => (
                <li
                  key={s.id}
                  style={{
                    padding: '8px 0',
                    borderBottom: '1px solid var(--border)',
                    fontSize: 13,
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    gap: 8,
                  }}
                >
                  <div>
                    <div>
                      {s.protocol} · {s.ip_address ?? '—'}
                    </div>
                    <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
                      {s.id.slice(0, 8)}… · in={s.bytes_in} out={s.bytes_out}
                    </div>
                  </div>
                  <button
                    type="button"
                    onClick={() => endSession(s.id)}
                    style={{
                      background: 'transparent',
                      border: '1px solid var(--danger)',
                      color: 'var(--danger)',
                      borderRadius: 6,
                      padding: '4px 10px',
                      fontSize: 12,
                      cursor: 'pointer',
                    }}
                  >
                    End
                  </button>
                </li>
              ))}
            </ul>
          )}
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