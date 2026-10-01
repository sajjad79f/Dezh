import { FormEvent, useEffect, useState } from 'react'
import { getToken } from '../../api/auth'
import { authHeaders, readJson, readEmpty } from '../../api/http'

interface Iface {
  name: string
  zone: string
  up: boolean
  addresses: string[]
  enabled: boolean
  ipv4_mode: string
  address_cidr: string | null
  gateway: string | null
  description: string
}

interface Zone {
  name: string
  display_name: string
  accounting: boolean
}

export function NetworkInterfaces() {
  const [ifaces, setIfaces] = useState<Iface[]>([])
  const [zones, setZones] = useState<Zone[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)
  const [edit, setEdit] = useState<string | null>(null)

  const [zone, setZone] = useState('lan')
  const [enabled, setEnabled] = useState(true)
  const [mode, setMode] = useState('none')
  const [cidr, setCidr] = useState('')
  const [gateway, setGateway] = useState('')

  function headers(json = false): HeadersInit {
    const token = getToken()
    return {
      ...(json ? { 'Content-Type': 'application/json' } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    }
  }

  async function load() {
    try {
      const [i, z] = await Promise.all([
        fetch('/api/network/interfaces', { headers: authHeaders() }),
        fetch('/api/network/zones', { headers: authHeaders() }),
      ])
      setIfaces(await readJson(i))
      setZones(await readJson(z))
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }

  useEffect(() => {
    load()
  }, [])

  function openEdit(iface: Iface) {
    setEdit(iface.name)
    setZone(iface.zone)
    setEnabled(iface.enabled)
    setMode(iface.ipv4_mode || 'none')
    setCidr(iface.address_cidr || '')
    setGateway(iface.gateway || '')
    setOk(null)
  }

  async function onApply(e: FormEvent) {
    e.preventDefault()
    if (!edit) return
    setOk(null)
    try {
      const res = await fetch('/api/network/interfaces/config', {
        method: 'POST',
        headers: authHeaders(true),
        body: JSON.stringify({
          name: edit,
          zone,
          enabled,
          ipv4_mode: mode,
          address_cidr: mode === 'static' ? cidr || null : null,
          gateway: mode === 'static' ? gateway || null : null,
          description: '',
        }),
      })
      await readEmpty(res)
      setOk(`Applied ${edit}`)
      setEdit(null)
      setError(null)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  return (
    <div>
      <h1 className="page-title">Network · Interfaces</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        زون را از لیست Zones انتخاب کن. برای شمارش ترافیک، زون باید Accounting=ON داشته باشد.
        اعمال IP نیاز به اجرای Dezh با root دارد.
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
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
              <th style={{ padding: 8 }}>Name</th>
              <th>State</th>
              <th>Zone</th>
              <th>Mode</th>
              <th>Addresses</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {ifaces.map((i) => {
              const z = zones.find((x) => x.name === i.zone)
              return (
                <tr key={i.name} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: 8 }}>
                    <code>{i.name}</code>
                  </td>
                  <td style={{ color: i.up ? '#7ddb8a' : '#f0b429' }}>
                    {i.up ? 'UP' : 'DOWN'}
                    {!i.enabled ? ' (disabled)' : ''}
                  </td>
                  <td>
                    {i.zone}
                    {z?.accounting ? (
                      <span style={{ marginLeft: 6, color: '#7ddb8a', fontSize: 11 }}>acc</span>
                    ) : null}
                  </td>
                  <td>{i.ipv4_mode}</td>
                  <td>{i.addresses.length ? i.addresses.join(', ') : '—'}</td>
                  <td style={{ padding: 8 }}>
                    <button type="button" onClick={() => openEdit(i)} style={btn}>
                      Configure
                    </button>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {edit && (
        <div className="card">
          <h3 style={{ marginTop: 0 }}>Configure: {edit}</h3>
          <form onSubmit={onApply} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
              <select value={zone} onChange={(e) => setZone(e.target.value)} style={field}>
                {zones.map((z) => (
                  <option key={z.name} value={z.name}>
                    {z.display_name} ({z.name})
                    {z.accounting ? ' · accounting' : ''}
                  </option>
                ))}
              </select>
              <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 13 }}>
                <input
                  type="checkbox"
                  checked={enabled}
                  onChange={(e) => setEnabled(e.target.checked)}
                />
                Enabled (link up)
              </label>
              <select value={mode} onChange={(e) => setMode(e.target.value)} style={field}>
                <option value="none">IP: none</option>
                <option value="static">IP: static</option>
                <option value="dhcp">IP: DHCP</option>
              </select>
              {mode === 'static' && (
                <>
                  <input
                    placeholder="CIDR e.g. 192.168.1.1/24"
                    value={cidr}
                    onChange={(e) => setCidr(e.target.value)}
                    required
                    style={field}
                  />
                  <input
                    placeholder="Gateway (optional)"
                    value={gateway}
                    onChange={(e) => setGateway(e.target.value)}
                    style={field}
                  />
                </>
              )}
            </div>
            <div style={{ display: 'flex', gap: 8 }}>
              <button type="submit" style={btn}>
                Apply
              </button>
              <button type="button" onClick={() => setEdit(null)} style={{ ...btn, background: 'transparent', border: '1px solid var(--border)', color: 'var(--text-muted)' }}>
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}
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