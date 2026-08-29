import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../api/auth'

interface User {
  id: string
  username: string
  role: string
  display_name: string | null
  enabled: boolean
}

export function Users() {
  const [users, setUsers] = useState<User[]>([])
  const [error, setError] = useState<string | null>(null)
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [role, setRole] = useState('operator')
  const [loading, setLoading] = useState(false)

  async function load() {
    const token = getToken()
    const res = await fetch('/api/users', {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error || `HTTP ${res.status}`)
      return
    }
    setUsers(await res.json())
    setError(null)
  }

  useEffect(() => {
    load()
  }, [])

  async function onCreate(e: FormEvent) {
    e.preventDefault()
    setLoading(true)
    try {
      const token = getToken()
      const res = await fetch('/api/users', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({ username, password, role }),
      })
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.statusText }))
        throw new Error(err.error || 'failed')
      }
      setUsername('')
      setPassword('')
      await load()
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div>
      <h1 className="page-title">Users</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}

      <div className="grid grid-2">
        <div className="card">
          <h3>Create user</h3>
          <form onSubmit={onCreate} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            <input
              placeholder="Username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
              style={inputStyle}
            />
            <input
              type="password"
              placeholder="Password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              style={inputStyle}
            />
            <select value={role} onChange={(e) => setRole(e.target.value)} style={inputStyle}>
              <option value="admin">admin</option>
              <option value="operator">operator</option>
              <option value="viewer">viewer</option>
            </select>
            <button type="submit" disabled={loading} style={btnStyle}>
              {loading ? 'Creating…' : 'Create'}
            </button>
          </form>
        </div>

        <div className="card">
          <h3>All users ({users.length})</h3>
          <ul style={{ listStyle: 'none' }}>
            {users.map((u) => (
              <li
                key={u.id}
                style={{
                  padding: '10px 0',
                  borderBottom: '1px solid var(--border)',
                  display: 'flex',
                  justifyContent: 'space-between',
                }}
              >
                <span>
                  {u.username}
                  {u.display_name ? ` (${u.display_name})` : ''}
                </span>
                <span style={{ color: 'var(--text-muted)', fontSize: 13 }}>{u.role}</span>
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