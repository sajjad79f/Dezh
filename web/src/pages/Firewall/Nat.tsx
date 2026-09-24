import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'

interface NatRule {
  id: string
  name: string
  enabled: boolean
  kind: string
  interface: string
  source: string
  destination: string
  protocol: string
  dest_port: number | null
  target: string | null
  description: string
}

interface Iface {
  name: string
  zone: string
  up: boolean
  addresses: string[]
}

export function FirewallNat() {
  const [rules, setRules] = useState<NatRule[]>([])
  const [ifaces, setIfaces] = useState<Iface[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)

  const [kind, setKind] = useState<'masquerade' | 'dnat'>('masquerade')
  const [name, setName] = useState('')
  const [iface, setIface] = useState('')
  const [source, setSource] = useState('any')
  const [protocol, setProtocol] = useState('tcp')
  const [destPort, setDestPort] = useState('')
  const [target, setTarget] = useState('')
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
      const [n, i] = await Promise.all([
        fetch('/api/firewall/nat', { headers: headers() }),
        fetch('/api/firewall/interfaces', { headers: headers() }),
      ])
      if (!n.ok) throw new Error((await n.json()).error || n.statusText)
      if (!i.ok) throw new Error((await i.json()).error || i.statusText)
      setRules(await n.json())
      setIfaces(await i.json())
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
    const body: Record<string, unknown> = {
      name: name || (kind === 'masquerade' ? 'Outbound NAT' : 'Port forward'),
      kind,
      interface: iface,
      source: source || 'any',
      destination: 'any',
      protocol: kind === 'dnat' ? protocol : 'any',
      dest_port: kind === 'dnat' && destPort ? Number(destPort) : null,
      target: kind === 'dnat' ? target : null,
      description,
    }
    const res = await fetch('/api/firewall/nat', {
      method: 'POST',
      headers: headers(true),
      body: JSON.stringify(body),
    })
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error || res.statusText)
      return
    }
    setName('')
    setTarget('')
    setDestPort('')
    setDescription('')
    setOk('NAT rule applied')
    setError(null)
    await load()
  }

  async function onDelete(id: string) {
    if (!confirm('Delete this NAT rule?')) return
    const res = await fetch(`/api/firewall/nat/${id}`, {
      method: 'DELETE',
      headers: headers(),
    })
    if (!res.ok && res.status !== 204) {
      const err = await res.json().catch(() => ({ error: res.statusText }))
      setError(err.error || res.statusText)
      return
    }
    setOk('Deleted')
    await load()
  }

  const wanIfaces = ifaces.filter((i) => i.zone === 'wan')
  const ifaceOptions = ifaces.length ? ifaces : []

  return (
    <div>
      <h1 className="page-title">NAT</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        Outbound NAT (masquerade) برای اینترنت‌دادن به LAN — Port Forward برای باز کردن سرویس از WAN.
        نیاز به <code>ip_forward</code> و اجرای Dezh با root.
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
        <h3 style={{ marginTop: 0 }}>Add NAT rule</h3>
        <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
          <button
            type="button"
            onClick={() => setKind('masquerade')}
            style={{
              ...btn,
              background: kind === 'masquerade' ? 'var(--accent-dim)' : 'transparent',
              border: '1px solid var(--border)',
            }}
          >
            Outbound (Masquerade)
          </button>
          <button
            type="button"
            onClick={() => setKind('dnat')}
            style={{
              ...btn,
              background: kind === 'dnat' ? 'var(--accent-dim)' : 'transparent',
              border: '1px solid var(--border)',
            }}
          >
            Port Forward (DNAT)
          </button>
        </div>

        <form onSubmit={onCreate} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
            <input
              placeholder="Name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              style={field}
            />
            <select
              value={iface}
              onChange={(e) => setIface(e.target.value)}
              required
              style={field}
            >
              <option value="">Interface *</option>
              {ifaceOptions.map((i) => (
                <option key={i.name} value={i.name}>
                  {i.name} [{i.zone}] {i.up ? '↑' : '↓'}
                  {i.addresses[0] ? ` — ${i.addresses[0]}` : ''}
                </option>
              ))}
            </select>
            {kind === 'masquerade' && (
              <input
                placeholder="Source (LAN CIDR or any)"
                value={source}
                onChange={(e) => setSource(e.target.value)}
                style={field}
              />
            )}
            {kind === 'dnat' && (
              <>
                <select value={protocol} onChange={(e) => setProtocol(e.target.value)} style={field}>
                  <option value="tcp">TCP</option>
                  <option value="udp">UDP</option>
                </select>
                <input
                  placeholder="External port"
                  value={destPort}
                  onChange={(e) => setDestPort(e.target.value)}
                  required
                  style={field}
                />
                <input
                  placeholder="Target IP:port (e.g. 192.168.1.10:80)"
                  value={target}
                  onChange={(e) => setTarget(e.target.value)}
                  required
                  style={{ ...field, minWidth: 220 }}
                />
              </>
            )}
            <input
              placeholder="Description (optional)"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              style={field}
            />
            <button type="submit" style={btn}>
              Add
            </button>
          </div>
          {kind === 'masquerade' && wanIfaces.length === 0 && (
            <p style={{ fontSize: 12, color: '#f0b429', margin: 0 }}>
              هنوز اینترفیس WAN تعریف نشده — از Firewall → Interfaces یک اینترفیس را zone=wan کن.
            </p>
          )}
        </form>
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Rules ({rules.length})</h3>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
              <th style={{ padding: 8 }}>Name</th>
              <th>Kind</th>
              <th>Interface</th>
              <th>Source</th>
              <th>Port / Target</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rules.length === 0 ? (
              <tr>
                <td colSpan={6} style={{ padding: 12, color: 'var(--text-muted)' }}>
                  No NAT rules — add Outbound masquerade on WAN to share internet with LAN.
                </td>
              </tr>
            ) : (
              rules.map((r) => (
                <tr key={r.id} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: 8 }}>{r.name}</td>
                  <td>{r.kind}</td>
                  <td>{r.interface}</td>
                  <td>{r.source}</td>
                  <td>
                    {r.kind === 'dnat'
                      ? `${r.protocol}/${r.dest_port ?? '—'} → ${r.target ?? '—'}`
                      : 'masquerade'}
                  </td>
                  <td style={{ padding: 8 }}>
                    <button
                      type="button"
                      onClick={() => onDelete(r.id)}
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