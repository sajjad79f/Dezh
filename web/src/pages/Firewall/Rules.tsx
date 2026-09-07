import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'

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
  direction: 'inbound',
  protocol: 'any',
  source: 'any',
  destination: 'any',
  port: '',
  priority: '100',
}

export function FirewallRules() {
  const [rules, setRules] = useState<Rule[]>([])
  const [form, setForm] = useState(emptyForm)
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
      const res = await fetch('/api/firewall/rules', { headers: headers() })
      if (!res.ok) throw new Error((await res.json()).error || res.statusText)
      setRules(await res.json())
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
        headers: headers(true),
        body: JSON.stringify(body),
      })
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.statusText }))
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

  async function removeRule(id: string) {
    const res = await fetch(`/api/firewall/rules/${id}`, {
      method: 'DELETE',
      headers: headers(),
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
      <h1 className="page-title">Firewall · Rules</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}

      <div className="grid grid-2">
        <div className="card">
          <h3>Add rule</h3>
          <form onSubmit={onCreate} style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <input
              placeholder="Name"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              required
              style={inputStyle}
            />
            <select
              value={form.action}
              onChange={(e) => setForm({ ...form, action: e.target.value })}
              style={inputStyle}
            >
              <option value="allow">allow</option>
              <option value="deny">deny</option>
              <option value="drop">drop</option>
              <option value="reject">reject</option>
            </select>
            <select
              value={form.direction}
              onChange={(e) => setForm({ ...form, direction: e.target.value })}
              style={inputStyle}
            >
              <option value="inbound">inbound</option>
              <option value="outbound">outbound</option>
              <option value="both">both</option>
            </select>
            <select
              value={form.protocol}
              onChange={(e) => setForm({ ...form, protocol: e.target.value })}
              style={inputStyle}
            >
              <option value="any">any</option>
              <option value="tcp">tcp</option>
              <option value="udp">udp</option>
              <option value="icmp">icmp</option>
            </select>
            <input
              placeholder="Source"
              value={form.source}
              onChange={(e) => setForm({ ...form, source: e.target.value })}
              style={inputStyle}
            />
            <input
              placeholder="Destination"
              value={form.destination}
              onChange={(e) => setForm({ ...form, destination: e.target.value })}
              style={inputStyle}
            />
            <input
              placeholder="Port"
              value={form.port}
              onChange={(e) => setForm({ ...form, port: e.target.value })}
              style={inputStyle}
            />
            <input
              placeholder="Priority"
              value={form.priority}
              onChange={(e) => setForm({ ...form, priority: e.target.value })}
              style={inputStyle}
            />
            <button type="submit" disabled={loading} style={btnStyle}>
              {loading ? 'Saving…' : 'Add rule'}
            </button>
          </form>
        </div>

        <div className="card">
          <h3>Rules ({rules.length})</h3>
          <ul style={{ listStyle: 'none' }}>
            {rules.map((r) => (
              <li
                key={r.id}
                style={{
                  padding: '10px 0',
                  borderBottom: '1px solid var(--border)',
                  display: 'flex',
                  justifyContent: 'space-between',
                  gap: 8,
                  fontSize: 13,
                }}
              >
                <div>
                  <div style={{ fontWeight: 600 }}>{r.name}</div>
                  <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
                    {r.action} · {r.direction} · {r.protocol} · {r.source} → {r.destination}
                    {r.port != null ? ` :${r.port}` : ''}
                  </div>
                </div>
                <button type="button" onClick={() => removeRule(r.id)} style={dangerBtn}>
                  Delete
                </button>
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

const dangerBtn: React.CSSProperties = {
  background: 'transparent',
  border: '1px solid var(--danger)',
  color: 'var(--danger)',
  borderRadius: 6,
  padding: '4px 10px',
  cursor: 'pointer',
  fontSize: 12,
}