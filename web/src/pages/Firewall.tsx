import { useEffect, useState } from 'react'
function headers(): Record<string, string> {
  const token = localStorage.getItem('dezh_token') // ⚠️ کلید را از web/src/api/client.ts چک کن — همان که توکن آنجا ذخیره می‌شود
  return token ? { Authorization: `Bearer ${token}` } : {}
}

interface Rule {
  id: string
  name: string
  action: string
  direction: string
  protocol: string
  source: string
  destination: string
  port: number | null
  enabled: boolean
  priority: number
}

const emptyForm = {
  name: '',
  action: 'drop',
  direction: 'in',
  protocol: 'any',
  source: '',
  destination: 'any',
  port: '',
  priority: '100',
}

export function Firewall() {
  const [rules, setRules] = useState<Rule[]>([])
  const [form, setForm] = useState(emptyForm)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function load() {
    try {
      const res = await fetch('/api/firewall/rules')
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      setRules(await res.json())
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }

  useEffect(() => {
    load()
  }, [])

  async function createRule(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    try {
      const body = {
        name: form.name,
        action: form.action,
        direction: form.direction,
        protocol: form.protocol,
        source: form.source || 'any',
        destination: form.destination || 'any',
        port: form.port ? Number(form.port) : null,
        priority: Number(form.priority) || 100,
      }
      const res = await fetch('/api/firewall/rules', {
        method: 'POST',
        headers: { ...headers(), 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || res.statusText)
      }
      setForm(emptyForm)
      await load()
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  // قبل:  const res = await fetch('/api/firewall/rules')

  async function removeRule(id: string) {
    const res = await fetch(`/api/firewall/rules/${id}`, { method: 'DELETE', headers: headers() })
    if (!res.ok && res.status !== 204) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error)
      return
    }
    await load()
  }

  return (
    <div>
      <h1 className="page-title">Firewall Rules</h1>

      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}

      <div className="grid grid-2">
        <div className="card">
          <h3>Add Rule</h3>
          <form onSubmit={createRule} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            <input
              placeholder="Rule name"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              required
              style={inputStyle}
            />
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              <select value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })} style={inputStyle}>
                <option value="allow">Allow</option>
                <option value="deny">Deny</option>
                <option value="drop">Drop</option>
                <option value="reject">Reject</option>
              </select>
              <select value={form.direction} onChange={(e) => setForm({ ...form, direction: e.target.value })} style={inputStyle}>
                <option value="in">Inbound</option>
                <option value="out">Outbound</option>
                <option value="both">Both</option>
              </select>
              <select value={form.protocol} onChange={(e) => setForm({ ...form, protocol: e.target.value })} style={inputStyle}>
                <option value="any">Any</option>
                <option value="tcp">TCP</option>
                <option value="udp">UDP</option>
                <option value="icmp">ICMP</option>
              </select>
              <input
                placeholder="Port (optional)"
                value={form.port}
                onChange={(e) => setForm({ ...form, port: e.target.value })}
                style={inputStyle}
              />
            </div>
            <input
              placeholder="Source (IP / CIDR / any)"
              value={form.source}
              onChange={(e) => setForm({ ...form, source: e.target.value })}
              style={inputStyle}
            />
            <input
              placeholder="Destination (IP / CIDR / any)"
              value={form.destination}
              onChange={(e) => setForm({ ...form, destination: e.target.value })}
              style={inputStyle}
            />
            <button
              type="submit"
              disabled={loading}
              style={{
                background: 'var(--accent-dim)',
                color: '#fff',
                border: 'none',
                borderRadius: 8,
                padding: '10px 16px',
                fontWeight: 600,
              }}
            >
              {loading ? 'Adding…' : 'Add Rule'}
            </button>
          </form>
        </div>

        <div className="card">
          <h3>Active Rules ({rules.length})</h3>
          {rules.length === 0 ? (
            <p style={{ color: 'var(--text-muted)' }}>No rules yet</p>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              {rules.map((r) => (
                <div
                  key={r.id}
                  style={{
                    border: '1px solid var(--border)',
                    borderRadius: 8,
                    padding: 12,
                    fontSize: 13,
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6 }}>
                    <strong>{r.name}</strong>
                    <button
                      onClick={() => removeRule(r.id)}
                      style={{
                        background: 'transparent',
                        border: '1px solid var(--danger)',
                        color: 'var(--danger)',
                        borderRadius: 6,
                        padding: '2px 8px',
                        fontSize: 12,
                      }}
                    >
                      Remove
                    </button>
                  </div>
                  <div style={{ color: 'var(--text-muted)', fontFamily: 'var(--mono)', fontSize: 12 }}>
                    {r.action.toUpperCase()} · {r.direction} · {r.protocol}
                    <br />
                    {r.source}:{r.port ?? '*'} → {r.destination}
                    <br />
                    pri={r.priority} · {r.enabled ? 'enabled' : 'disabled'}
                  </div>
                </div>
              ))}
            </div>
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