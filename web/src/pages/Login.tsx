import { FormEvent, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { login, setToken } from '../api/auth'
import { Shield } from 'lucide-react'

export function Login() {
  const navigate = useNavigate()
  const [username, setUsername] = useState('admin')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      const result = await login(username.trim(), password)
      setToken(result.token)
      navigate('/', { replace: true })
    } catch (err: any) {
      setError(err.message || 'Login failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'var(--bg, #0b0f14)',
        padding: 24,
      }}
    >
      <div
        style={{
          width: '100%',
          maxWidth: 380,
          background: 'var(--bg-panel, #101010)',
          border: '1px solid var(--border, #101010)',
          borderRadius: 16,
          padding: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            marginBottom: 28,
            color: 'var(--accent, #02606D)',
          }}
        >
          <Shield size={28} />
          <div>
            <div style={{ fontWeight: 700, fontSize: 20, letterSpacing: '0.04em' }}>DEZH</div>
            <div style={{ fontSize: 12, color: 'var(--text-muted, #94a3b8)' }}>
              Sign in to continue
            </div>
          </div>
        </div>

        <form onSubmit={onSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
          <label style={{ fontSize: 13, color: 'var(--text-muted, #94a3b8)' }}>
            Username
            <input
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
              required
              style={inputStyle}
            />
          </label>

          <label style={{ fontSize: 13, color: 'var(--text-muted, #94a3b8)' }}>
            Password
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
              required
              style={inputStyle}
            />
          </label>

          {error && (
            <div
              style={{
                color: 'var(--danger, #ef4444)',
                fontSize: 13,
                background: 'rgba(239,68,68,0.08)',
                border: '1px solid rgba(239,68,68,0.25)',
                borderRadius: 8,
                padding: '10px 12px',
              }}
            >
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={loading}
            style={{
              marginTop: 6,
              background: 'var(--accent-dim, #02606D)',
              color: '#fff',
              border: 'none',
              borderRadius: 10,
              padding: '12px 16px',
              fontWeight: 600,
              fontSize: 14,
              opacity: loading ? 0.7 : 1,
              cursor: loading ? 'wait' : 'pointer',
            }}
          >
            {loading ? 'Signing in…' : 'Sign in'}
          </button>
        </form>
      </div>
    </div>
  )
}

const inputStyle: React.CSSProperties = {
  display: 'block',
  width: '100%',
  marginTop: 6,
  background: 'var(--bg, #0b0f14)',
  border: '1px solid var(--border, #1e293b)',
  borderRadius: 8,
  padding: '10px 12px',
  color: 'var(--text, #e2e8f0)',
  fontSize: 14,
  outline: 'none',
}