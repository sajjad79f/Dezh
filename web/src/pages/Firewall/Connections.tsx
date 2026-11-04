import { useCallback, useEffect, useRef, useState } from 'react'
import { firewallApi, type Connection } from '../../api/firewall'

const STATE_COLORS: Record<string, string> = {
  ESTABLISHED: '#7ddb8a',
  TIME_WAIT: '#f0b429',
  CLOSE_WAIT: '#f0b429',
  SYN_SENT: '#c792ea',
  SYN_RECV: '#c792ea',
  FIN_WAIT: '#f0b429',
  '—': 'var(--text-muted)',
}

export function FirewallConnections() {
  const [conns, setConns] = useState<Connection[]>([])
  const [error, setError] = useState<string | null>(null)
  const [live, setLive] = useState(true)
  const [filter, setFilter] = useState('')
  const timer = useRef<ReturnType<typeof setInterval> | null>(null)

  const load = useCallback(async function load() {
    try {
      setConns(await firewallApi.connections())
      setError(null)
    } catch (e: any) {
      setError(e.message)
      setLive(false) // جلوی حلقه‌ی خطا را بگیر
    }
  }, [])

  useEffect(() => {
    load()
    if (live) {
      timer.current = setInterval(load, 3000)
    }
    return () => {
      if (timer.current) clearInterval(timer.current)
    }
  }, [live, load])

  const filtered = filter
    ? conns.filter(
        (c) =>
          c.src.includes(filter) ||
          c.dst.includes(filter) ||
          c.protocol.includes(filter.toLowerCase()) ||
          c.state.includes(filter.toUpperCase()),
      )
    : conns

  const established = conns.filter((c) => c.state === 'ESTABLISHED').length

  return (
    <div>
      <h1 className="page-title">Firewall · Live Connections</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        جدول conntrack هسته — همه‌ی اتصال‌های عبوری از فایروال. منبع:{' '}
        <code>/proc/net/nf_conntrack</code> یا <code>conntrack -L</code>.
      </p>

      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}

      <div
        className="card"
        style={{ marginBottom: 16, display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap' }}
      >
        <label style={{ display: 'flex', gap: 8, alignItems: 'center', fontSize: 13 }}>
          <input type="checkbox" checked={live} onChange={(e) => setLive(e.target.checked)} />
          Auto-refresh (3s)
        </label>
        <input
          placeholder="filter: IP / proto / state…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          style={field}
        />
        <span style={{ color: 'var(--text-muted)', fontSize: 12 }}>
          total <b style={{ color: 'var(--text)' }}>{conns.length}</b> · established{' '}
          <b style={{ color: '#7ddb8a' }}>{established}</b>
          {conns.length >= 500 && ' · (capped at 500)'}
        </span>
      </div>

      <div className="card">
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead>
              <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
                <th style={{ padding: '6px 8px' }}>Proto</th>
                <th>Source</th>
                <th>Destination</th>
                <th>State</th>
                <th>Timeout</th>
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={5} style={{ padding: 12, color: 'var(--text-muted)' }}>
                    {conns.length === 0
                      ? 'هیچ اتصالی دیده نشد — اگر Dezh با root اجرا می‌شود و conntrack نصب است، ترافیک بگذار تا پر شود.'
                      : 'no match for filter'}
                  </td>
                </tr>
              ) : (
                filtered.map((c, i) => (
                  <tr key={`${c.src}-${c.dst}-${c.sport}-${c.dport}-${i}`} style={{ borderTop: '1px solid var(--border)' }}>
                    <td style={{ padding: '5px 8px', textTransform: 'uppercase' }}>{c.protocol}</td>
                    <td style={{ fontFamily: 'monospace', fontSize: 12 }}>
                      {c.src}
                      {c.sport != null ? `:${c.sport}` : ''}
                    </td>
                    <td style={{ fontFamily: 'monospace', fontSize: 12 }}>
                      {c.dst}
                      {c.dport != null ? `:${c.dport}` : ''}
                    </td>
                    <td>
                      <span style={{ color: STATE_COLORS[c.state] ?? 'var(--text)' }}>
                        {c.state}
                      </span>
                    </td>
                    <td style={{ color: 'var(--text-muted)', fontVariantNumeric: 'tabular-nums' }}>
                      {c.timeout_secs != null ? `${c.timeout_secs}s` : '—'}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
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
  minWidth: 220,
}