import { useCallback, useEffect, useState } from 'react'

type LiveSession = {
  id: string
  identity_id: string | null
  protocol: string
  ip_address: string | null
  hostname: string | null
  started_at: string
  bytes_in: number
  bytes_out: number
  live_bytes_in: number
  live_bytes_out: number
  has_live_counters: boolean
}

type HistorySession = {
  id: string
  identity_id: string | null
  protocol: string
  ip_address: string | null
  hostname: string | null
  started_at: string
  ended_at: string | null
  bytes_in: number
  bytes_out: number
}

type UsageRow = {
  identity_id: string | null
  username: string | null
  sessions: number
  bytes_in: number
  bytes_out: number
}

function headers(): Record<string, string> {
  // ⚠️ کلید را از web/src/api/client.ts چک کن — همان که توکن آنجا ذخیره می‌شود
  const token = localStorage.getItem('dezh_token')
  return token ? { Authorization: `Bearer ${token}` } : {}
}

function fmtBytes(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return '0 B'
  if (n < 1024) return `${n} B`
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`
  if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`
  return `${(n / 1024 ** 3).toFixed(2)} GB`
}

function fmtTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}

const th: React.CSSProperties = {
  textAlign: 'left',
  padding: '8px 10px',
  borderBottom: '1px solid rgba(255,255,255,0.12)',
  fontSize: 12,
  color: 'var(--text-muted, #8b8b9a)',
}
const td: React.CSSProperties = {
  padding: '8px 10px',
  borderBottom: '1px solid rgba(255,255,255,0.06)',
  fontSize: 13,
}

export default function Accounting() {
  const [tab, setTab] = useState<'live' | 'history' | 'usage'>('live')
  const [sessions, setSessions] = useState<LiveSession[]>([])
  const [history, setHistory] = useState<HistorySession[]>([])
  const [usage, setUsage] = useState<UsageRow[]>([])
  const [error, setError] = useState('')

  const loadLive = useCallback(async () => {
    try {
      const res = await fetch('/api/accounting/sessions', { headers: headers() })
      if (!res.ok) throw new Error(`sessions: HTTP ${res.status}`)
      setSessions(await res.json())
      setError('')
    } catch (e) {
      setError(String(e))
    }
  }, [])

  const loadHistory = useCallback(async () => {
    try {
      const res = await fetch('/api/accounting/sessions/history?limit=100', {
        headers: headers(),
      })
      if (!res.ok) throw new Error(`history: HTTP ${res.status}`)
      setHistory(await res.json())
      setError('')
    } catch (e) {
      setError(String(e))
    }
  }, [])

  const loadUsage = useCallback(async () => {
    try {
      const res = await fetch('/api/accounting/usage', { headers: headers() })
      if (!res.ok) throw new Error(`usage: HTTP ${res.status}`)
      setUsage(await res.json())
      setError('')
    } catch (e) {
      setError(String(e))
    }
  }, [])

  useEffect(() => {
    loadLive()
    const t = setInterval(loadLive, 10_000) // رفرش زنده هر ۱۰ ثانیه
    return () => clearInterval(t)
  }, [loadLive])

  useEffect(() => {
    if (tab === 'history') loadHistory()
    if (tab === 'usage') loadUsage()
  }, [tab, loadHistory, loadUsage])

  async function endSession(id: string) {
    try {
      // بایت‌ها سمت سرور از counterهای nft محاسبه می‌شوند؛ صفر می‌فرستیم
      const res = await fetch('/api/accounting/sessions/end', {
        method: 'POST',
        headers: { ...headers(), 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: id,
          bytes_in: 0,
          bytes_out: 0,
          terminate_cause: 'manual-ui',
        }),
      })
      if (!res.ok) throw new Error(`end: HTTP ${res.status}`)
      loadLive()
    } catch (e) {
      setError(String(e))
    }
  }

  const tabBtn = (key: typeof tab, label: string): React.CSSProperties => ({
    padding: '6px 14px',
    borderRadius: 8,
    cursor: 'pointer',
    fontSize: 13,
    border: tab === key ? '1px solid #5b8cff' : '1px solid transparent',
    background: tab === key ? 'rgba(91,140,255,0.15)' : 'transparent',
    color: tab === key ? '#9db9ff' : 'var(--text-muted, #8b8b9a)',
  })

  return (
    <div style={{ padding: 20, maxWidth: 1100, margin: '0 auto' }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 16,
        }}
      >
        <h2 style={{ margin: 0 }}>Accounting</h2>
        <div style={{ display: 'flex', gap: 8 }}>
          <button style={tabBtn('live', 'Live')} onClick={() => setTab('live')}>
            Live sessions ({sessions.length})
          </button>
          <button style={tabBtn('history', 'History')} onClick={() => setTab('history')}>
            History
          </button>
          <button style={tabBtn('usage', 'Usage')} onClick={() => setTab('usage')}>
            Usage summary
          </button>
        </div>
      </div>

      {error && (
        <p style={{ color: '#ff7b7b', fontSize: 13 }}>
          {error} — اگر 401 گرفتی، کلید token در headers() را با client.ts چک کن
        </p>
      )}

      {tab === 'live' && (
        <>
          {sessions.length === 0 ? (
            <p style={{ color: 'var(--text-muted, #8b8b9a)' }}>No active sessions</p>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={th}>Identity</th>
                  <th style={th}>Hostname</th>
                  <th style={th}>IP</th>
                  <th style={th}>Started</th>
                  <th style={th}>↓ Live in</th>
                  <th style={th}>↑ Live out</th>
                  <th style={th}>Counters</th>
                  <th style={th}></th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((s) => (
                  <tr key={s.id}>
                    <td style={td}>{s.identity_id?.slice(0, 8) ?? '—'}</td>
                    <td style={td}>{s.hostname ?? '—'}</td>
                    <td style={td}>{s.ip_address ?? '—'}</td>
                    <td style={td}>{fmtTime(s.started_at)}</td>
                    <td style={td}>{fmtBytes(s.live_bytes_in)}</td>
                    <td style={td}>{fmtBytes(s.live_bytes_out)}</td>
                    <td style={{ ...td, color: s.has_live_counters ? '#7ddb8a' : '#f0b429' }}>
                      {s.has_live_counters ? 'nft ✓' : 'no nft counter'}
                    </td>
                    <td style={td}>
                      <button
                        onClick={() => endSession(s.id)}
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
                        End
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}

      {tab === 'history' && (
        <>
          {history.length === 0 ? (
            <p style={{ color: 'var(--text-muted, #8b8b9a)' }}>No closed sessions yet</p>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={th}>Identity</th>
                  <th style={th}>Hostname</th>
                  <th style={th}>IP</th>
                  <th style={th}>Start</th>
                  <th style={th}>End</th>
                  <th style={th}>↓ In</th>
                  <th style={th}>↑ Out</th>
                  <th style={th}>Total</th>
                </tr>
              </thead>
              <tbody>
                {history.map((s) => (
                  <tr key={s.id}>
                    <td style={td}>{s.identity_id?.slice(0, 8) ?? '—'}</td>
                    <td style={td}>{s.hostname ?? '—'}</td>
                    <td style={td}>{s.ip_address ?? '—'}</td>
                    <td style={td}>{fmtTime(s.started_at)}</td>
                    <td style={td}>{s.ended_at ? fmtTime(s.ended_at) : '—'}</td>
                    <td style={td}>{fmtBytes(s.bytes_in)}</td>
                    <td style={td}>{fmtBytes(s.bytes_out)}</td>
                    <td style={td}>{fmtBytes(s.bytes_in + s.bytes_out)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}

      {tab === 'usage' && (
        <>
          {usage.length === 0 ? (
            <p style={{ color: 'var(--text-muted, #8b8b9a)' }}>No usage data yet</p>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr>
                  <th style={th}>Username</th>
                  <th style={th}>Sessions</th>
                  <th style={th}>↓ Total in</th>
                  <th style={th}>↑ Total out</th>
                  <th style={th}>Total</th>
                </tr>
              </thead>
              <tbody>
                {usage.map((u, i) => (
                  <tr key={u.identity_id ?? `row-${i}`}>
                    <td style={td}>{u.username ?? u.identity_id?.slice(0, 8) ?? '—'}</td>
                    <td style={td}>{u.sessions}</td>
                    <td style={td}>{fmtBytes(u.bytes_in)}</td>
                    <td style={td}>{fmtBytes(u.bytes_out)}</td>
                    <td style={td}>{fmtBytes(u.bytes_in + u.bytes_out)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}
    </div>
  )
}