import { useCallback, useEffect, useState, type FormEvent } from 'react'
import { firewallApi, type FqdnObject } from '../../api/firewall'

// فقط example.com یا *.example.com — همان قاعده‌ای که سرور هم اعمال می‌کند
const FQDN_RE =
  /^(\*\.)?[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$/

function resolveSummary(o: FqdnObject) {
  const list = o.resolved ?? []
  const v6 = list.filter((ip) => ip.includes(':')).length
  return { v4: list.length - v6, v6, total: list.length }
}

function timeAgo(iso: string | null): string {
  if (!iso) return '—'
  const mins = Math.floor((Date.now() - new Date(iso).getTime()) / 60000)
  if (mins < 1) return 'همین حالا'
  if (mins < 60) return `${mins} دقیقه پیش`
  const h = Math.floor(mins / 60)
  if (h < 24) return `${h} ساعت پیش`
  return `${Math.floor(h / 24)} روز پیش`
}

export function FirewallFqdn() {
  const [objs, setObjs] = useState<FqdnObject[]>([])
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)
  const [expanded, setExpanded] = useState<string | null>(null)

  // فرم ساخت شیء
  const [name, setName] = useState('')
  const [fqdn, setFqdn] = useState('')
  const [description, setDescription] = useState('')

  const load = useCallback(async function load() {
    try {
      setObjs(await firewallApi.listFqdn())
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
    const domain = fqdn.trim().toLowerCase()
    if (!FQDN_RE.test(domain)) {
      setError(
        'قالب دامنه معتبر نیست — فقط example.com یا *.example.com (حروف کوچک، حداقل یک نقطه)',
      )
      return
    }
    try {
      await firewallApi.createFqdn({ name, fqdn: domain, description, enabled: true })
      setName('')
      setFqdn('')
      setDescription('')
      await load()
      setOk(
        `FQDN object "${name}" created — اولین رزولوشن در چرخه بعدی ترد resolver انجام می‌شود`,
      )
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onDelete(o: FqdnObject) {
    if (!confirm(`Delete FQDN object "${o.name}" (${o.fqdn})?`)) return
    setOk(null)
    try {
      await firewallApi.deleteFqdn(o.name)
      await load()
      setOk(`FQDN object "${o.name}" deleted`)
    } catch (e: any) {
      setError(e.message)
    }
  }

  async function onToggle(o: FqdnObject) {
    try {
      await firewallApi.toggleFqdn(o.name, !o.enabled)
      await load()
    } catch (e: any) {
      setError(e.message)
    }
  }

  return (
    <div>
      <h1 className="page-title">Firewall · FQDN Objects</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        دامنه به‌جای IP ثابت. ترد resolver هرچند وقت‌یک (کلید fqdn_resolve_interval) دامنه پایه را
        resolve می‌کند و IPها در nft sets به نام dezh_fqdn_&lt;name&gt;_4/_6 می‌روند. اگر رزولوشن
        موقتاً شکست بخورد، کش قبلی حفظ می‌شود (fail-open) و فقط خطا ثبت می‌شود.
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
        <h3 style={{ marginTop: 0 }}>New FQDN object</h3>
        <form onSubmit={onCreate} style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input
            placeholder="name (a-z 0-9 - _) *"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            style={field}
          />
          <input
            placeholder="*.example.com"
            value={fqdn}
            onChange={(e) => setFqdn(e.target.value)}
            required
            dir="ltr"
            style={{ ...field, fontFamily: 'monospace' }}
          />
          <input
            placeholder="توضیح (اختیاری)"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            style={{ ...field, flex: 1 }}
          />
          <button type="submit" style={btn}>
            Create
          </button>
        </form>
      </div>

      {objs.length === 0 ? (
        <div className="card" style={{ color: 'var(--text-muted)' }}>
          هنوز شیء FQDN نداری — مثلاً نام cdn با دامنه *.cloudflare.com بساز و بعد در قوانین به‌عنوان
          مقصد استفاده کن؛ قانون همیشه IPهای فعلی همان سرویس را می‌گیرد.
        </div>
      ) : (
        objs.map((o) => {
          const sum = resolveSummary(o)
          const isOpen = expanded === o.name
          const isWildcard = o.fqdn.startsWith('*.')
          return (
            <div className="card" key={o.id} style={{ marginBottom: 12 }}>
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
                  <span style={{ fontWeight: 700, fontSize: 15 }}>{o.name}</span>{' '}
                  {isWildcard && (
                    <span
                      style={{
                        fontSize: 11,
                        padding: '2px 8px',
                        borderRadius: 999,
                        border: '1px solid #e8c46b',
                        color: '#e8c46b',
                      }}
                    >
                      wildcard
                    </span>
                  )}{' '}
                  <span
                    style={{
                      fontSize: 11,
                      color: o.enabled ? '#7ddb8a' : 'var(--text-muted)',
                    }}
                  >
                    {o.enabled ? '● active' : '○ disabled'}
                  </span>
                  <div dir="ltr" style={{ fontFamily: 'monospace', fontSize: 12 }}>
                    {o.fqdn}
                  </div>
                  {o.description && (
                    <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>{o.description}</div>
                  )}
                </div>
                <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                  <span style={{ color: 'var(--text-muted)', fontSize: 12 }}>
                    {sum.total === 0
                      ? 'کش خالی'
                      : `${sum.v4} IPv4 · ${sum.v6} IPv6`}{' '}
                    — {timeAgo(o.last_resolved_at)}
                  </span>
                  <button type="button" onClick={() => onToggle(o)} style={ghostBtn}>
                    {o.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    type="button"
                    onClick={() => setExpanded(isOpen ? null : o.name)}
                    style={ghostBtn}
                  >
                    {isOpen ? 'Hide' : 'Details'}
                  </button>
                  <button type="button" onClick={() => onDelete(o)} style={dangerBtn}>
                    Delete
                  </button>
                </div>
              </div>

              {o.last_error && (
                <div style={{ color: 'var(--danger)', fontSize: 12, marginTop: 8 }}>
                  خطای آخرین resolve: {o.last_error} — کش قبلی همچنان در setها فعال است (fail-open)
                </div>
              )}

              {isOpen && (
                <div style={{ marginTop: 14 }}>
                  {sum.total === 0 ? (
                    <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
                      هنوز IP ای در کش نیست — یا دامنه رزولوشن نشده یا شیء غیرفعال است. شیء
                      غیرفعال = set خالی، یعنی قانون ارجاع‌شده عملاً بی‌اثر است.
                    </div>
                  ) : (
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
                      {(o.resolved ?? []).map((ip) => (
                        <div key={ip} style={{ padding: '2px 4px' }}>
                          {ip}
                        </div>
                      ))}
                    </div>
                  )}
                  <div style={{ color: 'var(--text-muted)', fontSize: 11, marginTop: 6 }}>
                    رزولوشن بعدی خودکار در چرخه بعدی ترد dezh-fqdn انجام می‌شود.
                  </div>
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