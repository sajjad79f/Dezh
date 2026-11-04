import { useCallback, useEffect, useRef, useState } from 'react'
import {
  firewallApi,
  type EventSummary,
  type SecurityEvent,
} from '../../api/firewall'

const KIND_COLORS: Record<string, string> = {
  rule_hit: '#ff9b9b',
  login_failed: '#f0b429',
  login_rate_limited: '#ff5f5f',
  login: '#7ddb8a',
  firewall_rule_created: '#c792ea',
  firewall_rule_deleted: '#c792ea',
  firewall_rule_updated: '#c792ea',
}

const SEVERITY_COLORS: Record<string, string> = {
  info: 'var(--text-muted)',
  warning: '#f0b429',
  critical: '#ff5f5f',
}

function timeAgo(iso: string): string {
  const s = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000)
  if (s < 60) return `${Math.floor(s)}s ago`
  if (s < 3600) return `${Math.floor(s / 60)}m ago`
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`
  return `${Math.floor(s / 86400)}d ago`
}

export function FirewallEvents() {
  const [summary, setSummary] = useState<EventSummary | null>(null)
  const [events, setEvents] = useState<SecurityEvent[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)
  const [live, setLive] = useState(true)
  const [kindFilter, setKindFilter] = useState('')
  const [busyIp, setBusyIp] = useState<string | null>(null)
  const timer = useRef<ReturnType<typeof setInterval> | null>(null)

  const load = useCallback(
    async function load() {
      try {
        const [s, e] = await Promise.all([
          firewallApi.eventSummary(24),
          firewallApi.events(300, 24),
        ])
        setSummary(s)
        setEvents(e)
        setError(null)
      } catch (err: any) {
        setError(err.message)
      }
    },
    [],
  )

  useEffect(() => {
    load()
    if (live) {
      timer.current = setInterval(load, 5000)
    }
    return () => {
      if (timer.current) clearInterval(timer.current)
    }
  }, [live, load])

  /** افزودن سریع IP به لیست سیاه — لیست را در صورت نبود می‌سازد */
  async function quickBlacklist(ip: string) {
    if (!confirm(`Block ${ip} via "blacklist" address list?`)) return
    setBusyIp(ip)
    setOk(null)
    try {
      const sets = await firewallApi.listSets().catch(() => [])
      if (!sets.some((s) => s.name === 'blacklist')) {
        await firewallApi.createSet({
          name: 'blacklist',
          kind: 'blacklist',
          description: 'quick-block from Events page',
        })
      }
      await firewallApi.addEntries('blacklist', [ip])
      setOk(`${ip} added to "blacklist" — blocked immediately (even live sessions)`)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setBusyIp(null)
    }
  }

  const shown = kindFilter
    ? events.filter((e) => e.kind === kindFilter)
    : events

  // ── نمودار ۲۴ ساعته (بدون کتابخانه — CSS bars)
  const timeline = summary?.timeline ?? []
  const maxCount = Math.max(1, ...timeline.map((t) => t.count))

  return (
    <div>
      <h1 className="page-title">Firewall · Security Events</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        استریم زنده‌ی رخدادهای امنیتی — hit قوانین (log=true) از kernel log +
        تلاش‌های ورود ناموفق. نگهداری ۷ روز، تجمیع هر ۵ ثانیه.
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

      {/* ── کارت‌های خلاصه ── */}
      <div className="grid grid-2" style={{ marginBottom: 16 }}>
        <div className="card" style={{ display: 'flex', gap: 24, alignItems: 'center' }}>
          <div>
            <div style={{ fontSize: 28, fontWeight: 700, color: '#ff9b9b' }}>
              {(summary?.total_hits ?? 0).toLocaleString()}
            </div>
            <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
              rule hits (24h)
            </div>
          </div>
          <div>
            <div style={{ fontSize: 28, fontWeight: 700, color: '#f0b429' }}>
              {(summary?.by_kind.find((k) => k.kind === 'login_failed')?.count ?? 0).toLocaleString()}
            </div>
            <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
              failed logins (24h)
            </div>
          </div>
          <div>
            <div style={{ fontSize: 28, fontWeight: 700 }}>
              {(summary?.top_sources.length ?? 0).toLocaleString()}
            </div>
            <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
              distinct sources
            </div>
          </div>
        </div>

        {/* ── تایم‌لاین ۲۴ ساعته ── */}
        <div className="card">
          <h3 style={{ marginTop: 0, marginBottom: 10 }}>Hits · last 24h</h3>
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 3, height: 64 }}>
            {timeline.length === 0 && (
              <span style={{ color: 'var(--text-muted)', fontSize: 12 }}>
                no hits yet — enable Log on a rule and send traffic
              </span>
            )}
            {timeline.map((t) => (
              <div
                key={t.hour}
                title={`${new Date(t.hour).toLocaleString()} — ${t.count.toLocaleString()} hits`}
                style={{
                  flex: 1,
                  minWidth: 6,
                  height: `${Math.max(4, (t.count / maxCount) * 100)}%`,
                  background:
                    t.count > maxCount * 0.66
                      ? '#ff9b9b'
                      : t.count > maxCount * 0.33
                        ? '#f0b429'
                        : '#7ddb8a',
                  borderRadius: 2,
                }}
              />
            ))}
          </div>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              color: 'var(--text-muted)',
              fontSize: 11,
              marginTop: 4,
            }}
          >
            <span>-24h</span>
            <span>now</span>
          </div>
        </div>
      </div>

      {/* ── Top attackers / ports ── */}
      <div className="grid grid-2" style={{ marginBottom: 16 }}>
        <div className="card">
          <h3 style={{ marginTop: 0 }}>Top sources (24h)</h3>
          {(summary?.top_sources.length ?? 0) === 0 ? (
            <div style={{ color: 'var(--text-muted)', fontSize: 13 }}>no data yet</div>
          ) : (
            summary!.top_sources.map((s) => (
              <div
                key={s.ip}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '4px 0',
                  borderBottom: '1px solid var(--border)',
                  fontSize: 13,
                }}
              >
                <code style={{ fontSize: 12 }}>{s.ip}</code>
                <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
                  <span style={{ fontVariantNumeric: 'tabular-nums', color: '#ff9b9b' }}>
                    {s.count.toLocaleString()}
                  </span>
                  <button
                    type="button"
                    onClick={() => quickBlacklist(s.ip)}
                    disabled={busyIp === s.ip}
                    style={{
                      background: 'transparent',
                      border: '1px solid var(--danger)',
                      color: 'var(--danger)',
                      borderRadius: 6,
                      padding: '2px 8px',
                      cursor: 'pointer',
                      fontSize: 11,
                    }}
                    title="add to blacklist address list"
                  >
                    {busyIp === s.ip ? '…' : '⛔ Block'}
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="card">
          <h3 style={{ marginTop: 0 }}>Top target ports (24h)</h3>
          {(summary?.top_ports.length ?? 0) === 0 ? (
            <div style={{ color: 'var(--text-muted)', fontSize: 13 }}>no data yet</div>
          ) : (
            summary!.top_ports.map((p) => (
              <div
                key={p.port}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  padding: '4px 0',
                  borderBottom: '1px solid var(--border)',
                  fontSize: 13,
                }}
              >
                <code style={{ fontSize: 12 }}>port {p.port}</code>
                <span style={{ fontVariantNumeric: 'tabular-nums', color: '#f0b429' }}>
                  {p.count.toLocaleString()}
                </span>
              </div>
            ))
          )}
        </div>
      </div>

      {/* ── جدول رخدادها ── */}
      <div className="card">
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: 8,
            flexWrap: 'wrap',
            gap: 8,
          }}
        >
          <h3 style={{ margin: 0 }}>Event stream ({shown.length})</h3>
          <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
            <label style={{ display: 'flex', gap: 6, alignItems: 'center', fontSize: 12 }}>
              <input
                type="checkbox"
                checked={live}
                onChange={(e) => setLive(e.target.checked)}
              />
              live (5s)
            </label>
            <select
              value={kindFilter}
              onChange={(e) => setKindFilter(e.target.value)}
              style={{
                background: 'var(--bg)',
                border: '1px solid var(--border)',
                borderRadius: 6,
                color: 'var(--text)',
                fontSize: 12,
                padding: '4px 8px',
              }}
            >
              <option value="">all kinds</option>
              <option value="rule_hit">rule hits</option>
              <option value="login_failed">failed logins</option>
              <option value="login_rate_limited">blocked IPs</option>
              <option value="login">successful logins</option>
            </select>
          </div>
        </div>

        <div style={{ overflowX: 'auto', maxHeight: 480, overflowY: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
            <thead>
              <tr style={{ color: 'var(--text-muted)', textAlign: 'left' }}>
                <th style={{ padding: '6px 8px', position: 'sticky', top: 0, background: 'var(--card-bg, inherit)' }}>Time</th>
                <th style={{ position: 'sticky', top: 0, background: 'var(--card-bg, inherit)' }}>Kind</th>
                <th style={{ position: 'sticky', top: 0, background: 'var(--card-bg, inherit)' }}>Source</th>
                <th style={{ position: 'sticky', top: 0, background: 'var(--card-bg, inherit)' }}>Port</th>
                <th style={{ position: 'sticky', top: 0, background: 'var(--card-bg, inherit)' }}>Rule</th>
                <th style={{ position: 'sticky', top: 0, background: 'var(--card-bg, inherit)' }}>Message</th>
              </tr>
            </thead>
            <tbody>
              {shown.length === 0 ? (
                <tr>
                  <td colSpan={6} style={{ padding: 12, color: 'var(--text-muted)' }}>
                    هیچ رخدادی نیست — یک قانون با Log فعال بساز و ترافیک عبور بده، یا
                    چند بار با پسورد غلط login کن.
                  </td>
                </tr>
              ) : (
                shown.map((e) => (
                  <tr key={e.id} style={{ borderTop: '1px solid var(--border)' }}>
                    <td
                      style={{ padding: '5px 8px', color: 'var(--text-muted)', whiteSpace: 'nowrap' }}
                      title={new Date(e.created_at).toLocaleString()}
                    >
                      {timeAgo(e.created_at)}
                    </td>
                    <td style={{ whiteSpace: 'nowrap' }}>
                      <span
                        style={{
                          fontSize: 11,
                          padding: '1px 8px',
                          borderRadius: 999,
                          border: `1px solid ${KIND_COLORS[e.kind] ?? 'var(--border)'}`,
                          color: KIND_COLORS[e.kind] ?? 'var(--text)',
                        }}
                      >
                        {e.kind}
                      </span>
                      {e.severity !== 'info' && (
                        <span
                          style={{
                            fontSize: 10,
                            marginLeft: 6,
                            color: SEVERITY_COLORS[e.severity] ?? 'var(--text)',
                          }}
                        >
                          ●{e.severity}
                        </span>
                      )}
                    </td>
                    <td style={{ fontFamily: 'monospace', fontSize: 11 }}>
                      {e.source_ip ?? '—'}
                    </td>
                    <td style={{ fontVariantNumeric: 'tabular-nums' }}>
                      {e.dest_port ?? '—'}
                    </td>
                    <td style={{ fontSize: 11, color: 'var(--accent)' }}>
                      {e.rule_name ?? '—'}
                      {e.hits > 1 && (
                        <span style={{ color: 'var(--text-muted)' }}> ×{e.hits}</span>
                      )}
                    </td>
                    <td style={{ color: 'var(--text-muted)' }}>{e.message}</td>
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