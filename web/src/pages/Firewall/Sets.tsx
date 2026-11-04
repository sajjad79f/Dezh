import { useCallback, useEffect, useState, type FormEvent } from 'react'
import { firewallApi, type IpSet } from '../../api/firewall'

const KIND_LABEL: Record<string, { fa: string; color: string }> = {
  blacklist: { fa: 'لیست سیاه — همه‌چیز Drop می‌شود', color: '#ff9b9b' },
  whitelist: { fa: 'لیست سفید — همیشه Accept', color: '#7ddb8a' },
}

export function FirewallSets() {
  const [sets, setSets] = useState<IpSet[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)
  const [expanded, setExpanded] = useState<string | null>(null)

  // فرم ساخت لیست
  const [name, setName] = useState('')
  const [kind, setKind] = useState<'blacklist' | 'whitelist'>('blacklist')
  const [description, setDescription] = useState('')

  // فرم افزودن ورودی
  const [bulk, setBulk] = useState('')

  const load = useCallback(async function load() {
    try {
      setSets(await firewallApi.listSets())
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  async function onCreate(e: FormEvent) {
    e.preventDefault()
    setOk(null)
    try {
      await firewallApi.createSet({ name, kind, description })
      setName('')
      setDescription('')
      await load()
      setOk(`Address list "${name}" created`)
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onDelete(s: IpSet) {
    if (!confirm(`Delete list "${s.name}" with ${s.entries_count} entries?`)) return
    setOk(null)
    try {
      await firewallApi.deleteSet(s.name)
      await load()
      setOk(`Address list "${s.name}" deleted`)
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onToggle(s: IpSet) {
    try {
      await firewallApi.toggleSet(s.name, !s.enabled)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onAddEntries(s: IpSet) {
    const entries = bulk
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean)
    if (!entries.length) return
    setOk(null)
    try {
      const res = await firewallApi.addEntries(s.name, entries)
      setBulk('')
      await load()
      setOk(
        `${res.added} entry added to "${s.name}" (${entries.length - res.added} duplicate/empty skipped)`,
      )
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onRemoveEntry(s: IpSet, cidr: string) {
    try {
      await firewallApi.removeEntry(s.name, cidr)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  return (
    <div>
      <h1 className="page-title">Firewall · Address Lists</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        معادل nftables set — blacklist قبل از همه‌چیز Drop می‌کند (حتی اتصال زنده) و whitelist همیشه
        Accept. ورودی‌ها: IP یا CIDR، هر خط یکی. می‌توانی فایل blocklist را هم Paste کنی.
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
        <h3 style={{ marginTop: 0 }}>New address list</h3>
        <form onSubmit={onCreate} style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input
            placeholder="name (a-z 0-9 - _) *"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            style={field}
          />
          <select value={kind} onChange={(e) => setKind(e.target.value as any)} style={field}>
            <option value="blacklist">blacklist</option>
            <option value="whitelist">whitelist</option>
          </select>
          <input
            placeholder="Description (optional)"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            style={{ ...field, flex: 1 }}
          />
          <button type="submit" style={btn}>
            Create
          </button>
        </form>
      </div>

      {sets.length === 0 ? (
        <div className="card" style={{ color: 'var(--text-muted)' }}>
          هنوز لیستی نداری — مثلاً یک blacklist با نام botnet بساز و CIDRهای مخرب را درش بریز.
        </div>
      ) : (
        sets.map((s) => {
          const meta = KIND_LABEL[s.kind] ?? KIND_LABEL.blacklist
          const isOpen = expanded === s.name
          return (
            <div className="card" key={s.id} style={{ marginBottom: 12 }}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  gap: 12,
                  flexWrap: 'wrap',
                }}
              >
                <div>
                  <span style={{ fontWeight: 700, fontSize: 15 }}>{s.name}</span>{' '}
                  <span
                    style={{
                      fontSize: 11,
                      padding: '2px 8px',
                      borderRadius: 999,
                      border: `1px solid ${meta.color}`,
                      color: meta.color,
                    }}
                  >
                    {s.kind}
                  </span>{' '}
                  <span
                    style={{
                      fontSize: 11,
                      color: s.enabled ? '#7ddb8a' : 'var(--text-muted)',
                    }}
                  >
                    {s.enabled ? '● active' : '○ disabled'}
                  </span>
                  {s.description && (
                    <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>{s.description}</div>
                  )}
                </div>
                <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                  <span style={{ color: 'var(--text-muted)', fontSize: 12 }}>
                    {s.entries_count.toLocaleString()} entries
                  </span>
                  <button type="button" onClick={() => onToggle(s)} style={ghostBtn}>
                    {s.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    type="button"
                    onClick={() => setExpanded(isOpen ? null : s.name)}
                    style={ghostBtn}
                  >
                    {isOpen ? 'Hide' : 'Manage'}
                  </button>
                  <button type="button" onClick={() => onDelete(s)} style={dangerBtn}>
                    Delete
                  </button>
                </div>
              </div>

              {isOpen && (
                <div style={{ marginTop: 14 }}>
                  <div style={{ display: 'flex', gap: 8, marginBottom: 10 }}>
                    <textarea
                      placeholder={'one IP/CIDR per line:\n185.220.101.0/24\n203.0.113.7\n2001:db8::/32'}
                      value={bulk}
                      onChange={(e) => setBulk(e.target.value)}
                      rows={4}
                      style={{ ...field, flex: 1, resize: 'vertical', fontFamily: 'monospace' }}
                    />
                    <button type="button" onClick={() => onAddEntries(s)} style={btn}>
                      Add entries
                    </button>
                  </div>

                  {s.entries.length > 0 && (
                    <div
                      style={{
                        maxHeight: 220,
                        overflowY: 'auto',
                        border: '1px solid var(--border)',
                        borderRadius: 8,
                        padding: 8,
                        fontSize: 12,
                        fontFamily: 'monospace',
                      }}
                    >
                      {s.entries.slice(0, 500).map((e) => (
                        <div
                          key={e}
                          style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            padding: '2px 4px',
                          }}
                        >
                          <span>{e}</span>
                          <button
                            type="button"
                            onClick={() => onRemoveEntry(s, e)}
                            style={{ ...dangerBtn, border: 'none', color: '#ff9b9b' }}
                            title="remove entry"
                          >
                            ✕
                          </button>
                        </div>
                      ))}
                      {s.entries.length > 500 && (
                        <div style={{ color: 'var(--text-muted)', padding: 4 }}>
                          … and {s.entries.length - 500} more
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
            </div>
          )
        })
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

const ghostBtn: React.CSSProperties = {
  background: 'transparent',
  border: '1px solid var(--border)',
  color: 'var(--text)',
  borderRadius: 6,
  padding: '4px 10px',
  cursor: 'pointer',
  fontSize: 12,
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