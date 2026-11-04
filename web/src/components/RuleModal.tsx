import { useEffect, useState } from 'react'
import {
  X,
  ShieldCheck,
  ShieldAlert,
  XCircle,
  Undo2,
  Server,
  Globe,
  ArrowRight,
  ArrowLeftRight,
  SlidersHorizontal,
  Gauge,
  Info,
  Ban,
  Clock,
} from 'lucide-react'
import { firewallApi, type FirewallRule, type IpSet } from '../api/firewall'
import '../styles/firewall.css'

type Draft = {
  name: string
  description: string
  priority: string
  enabled: boolean
  log: boolean
  direction: string
  source_set: string
  source: string
  dest_set: string
  destination: string
  protocol: string
  port: string
  action: string
  rate: string
  // فاز ۴: زمان‌بندی
  sched_on: boolean
  sched_start: string
  sched_end: string
  sched_days: string[]
}

const STEPS = [
  { id: 0, label: 'General', icon: SlidersHorizontal },
  { id: 1, label: 'Source & Destination', icon: ArrowLeftRight },
  { id: 2, label: 'Service & Action', icon: ShieldCheck },
  { id: 3, label: 'Advanced', icon: Gauge },
]

const ACTIONS: { value: string; label: string; icon: typeof ShieldCheck }[] = [
  { value: 'allow', label: 'Allow', icon: ShieldCheck },
  { value: 'deny', label: 'Deny', icon: Ban },
  { value: 'drop', label: 'Drop', icon: XCircle },
  { value: 'reject', label: 'Reject', icon: Undo2 },
]

// فاز ۴: روزهای هفته — مطابق قرارداد بک‌اند (sun..sat)
const DAYS = [
  { value: 'sun', label: 'Sun' },
  { value: 'mon', label: 'Mon' },
  { value: 'tue', label: 'Tue' },
  { value: 'wed', label: 'Wed' },
  { value: 'thu', label: 'Thu' },
  { value: 'fri', label: 'Fri' },
  { value: 'sat', label: 'Sat' },
]

function draftFromRule(r: FirewallRule | null): Draft {
  return {
    name: r?.name ?? '',
    description: r?.description ?? '',
    priority: String(r?.priority ?? 100),
    enabled: r?.enabled ?? true,
    log: r?.log ?? false,
    direction: r?.direction ?? 'inbound',
    source_set: r?.source_set ?? '',
    source: r?.source && r.source !== 'any' ? r.source : '',
    dest_set: r?.dest_set ?? '',
    destination: r?.destination && r.destination !== 'any' ? r.destination : '',
    protocol: r?.protocol ?? 'any',
    port: r?.port != null ? String(r.port) : '',
    action: r?.action ?? 'allow',
    rate: r?.rate_limit != null ? String(r.rate_limit) : '',
    sched_on: r?.schedule_enabled ?? false,
    sched_start: r?.schedule_start || '09:00',
    sched_end: r?.schedule_end || '18:00',
    sched_days: r?.schedule_days ?? [],
  }
}

function srcLabel(d: Draft): string {
  return d.source_set ? `List: ${d.source_set}` : d.source || 'Any'
}
function dstLabel(d: Draft): string {
  return d.dest_set ? `List: ${d.dest_set}` : d.destination || 'Any'
}
function svcLabel(d: Draft): string {
  const p = d.protocol.toUpperCase()
  return d.port ? `${p} · ${d.port}` : `${p} · any`
}

export function RuleModal({
  rule,
  sets,
  onClose,
  onSaved,
}: {
  rule: FirewallRule | null
  sets: IpSet[]
  onClose: () => void
  onSaved: (msg: string) => void
}) {
  const [step, setStep] = useState(0)
  const [d, setD] = useState<Draft>(() => draftFromRule(rule))
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const isEdit = !!rule

  // Escape = close
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  function set<K extends keyof Draft>(k: K, v: Draft[K]) {
    setD((prev) => ({ ...prev, [k]: v }))
    setError(null)
  }

  // فاز ۴: روشن/خاموش کردن یک روز — هیچ روزی = همه‌روزها
  function toggleDay(v: string) {
    setD((prev) => ({
      ...prev,
      sched_days: prev.sched_days.includes(v)
        ? prev.sched_days.filter((x) => x !== v)
        : [...prev.sched_days, v],
    }))
    setError(null)
  }

  function validate(): string | null {
    if (!d.name.trim()) return 'Rule name is required.'
    const p = Number(d.priority)
    if (!Number.isFinite(p) || p < 1) return 'Priority must be a number ≥ 1.'
    if ((d.protocol === 'tcp' || d.protocol === 'udp') && d.port) {
      const port = Number(d.port)
      if (!Number.isInteger(port) || port < 1 || port > 65535)
        return 'Port must be an integer between 1 and 65535.'
    }
    if (d.rate) {
      const r = Number(d.rate)
      if (!Number.isFinite(r) || r < 1 || r > 100000)
        return 'Rate limit must be between 1 and 100000 (or empty for unlimited).'
    }
    // فاز ۴: اعتبارسنجی زمان‌بندی
    if (d.sched_on) {
      const tt = /^([01]?\d|2[0-3]):[0-5]\d$/
      if (!tt.test(d.sched_start.trim()) || !tt.test(d.sched_end.trim()))
        return 'Schedule times must be valid HH:MM values.'
    }
    return null
  }

  function next() {
    if (step === 0) {
      if (!d.name.trim()) {
        setError('Rule name is required.')
        return
      }
    }
    setError(null)
    setStep((s) => Math.min(s + 1, STEPS.length - 1))
  }

  async function save() {
    const v = validate()
    if (v) {
      setError(v)
      return
    }
    setSaving(true)
    setError(null)
    const isTcpUdp = d.protocol === 'tcp' || d.protocol === 'udp'
    // در ویرایش: 0 یعنی «پورت را پاک کن»؛ در ساخت: null یعنی بدون پورت
    const portBody = isEdit
      ? isTcpUdp
        ? d.port
          ? Number(d.port)
          : 0
        : 0
      : isTcpUdp && d.port
        ? Number(d.port)
        : null
    const body = {
      name: d.name.trim(),
      action: d.action,
      direction: d.direction,
      protocol: d.protocol,
      source: d.source_set ? 'any' : d.source || 'any',
      destination: d.dest_set ? 'any' : d.destination || 'any',
      port: portBody,
      priority: Number(d.priority) || 100,
      description: d.description,
      log: d.log,
      enabled: d.enabled,
      source_set: d.source_set || null,
      dest_set: d.dest_set || null,
      rate_limit: d.rate ? Number(d.rate) : null,
      // فاز ۴: زمان‌بندی — قرارداد PATCH: هر ۴ فیلد با هم
      schedule_enabled: d.sched_on,
      schedule_start: d.sched_start.trim(),
      schedule_end: d.sched_end.trim(),
      schedule_days: d.sched_days,
    }
    try {
      if (isEdit && rule) {
        await firewallApi.updateRule(rule.id, body)
        onSaved(`Rule "${body.name}" updated`)
      } else {
        await firewallApi.createRule(body)
        onSaved(`Rule "${body.name}" created`)
      }
    } catch (e: any) {
      setError(e.message)
      setSaving(false)
    }
  }

  const infoText =
    `This rule will ${d.action.toUpperCase()} ` +
    `${d.protocol === 'any' ? 'all' : d.protocol.toUpperCase()} traffic ` +
    `from ${srcLabel(d)} to ${dstLabel(d)}` +
    (d.port && (d.protocol === 'tcp' || d.protocol === 'udp') ? ` on port ${d.port}` : '') +
    ` (${d.direction}).`

  return (
    <div className="modal-overlay" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="rule-modal">
        {/* ── Header ── */}
        <div className="modal-head">
          <span className="head-icon">
            <ShieldCheck size={20} />
          </span>
          <div>
            <h2>{isEdit ? 'Edit Firewall Rule' : 'Add Firewall Rule'}</h2>
            <p>Configure a new rule or modify an existing firewall policy.</p>
          </div>
          <button type="button" className="icon-btn close-btn" onClick={onClose} title="Close (Esc)">
            <X size={16} />
          </button>
        </div>

        {/* ── Body ── */}
        <div className="modal-body">
          {/* steps */}
          <div className="rule-steps">
            {STEPS.map((s) => {
              const Icon = s.icon
              return (
                <button
                  key={s.id}
                  type="button"
                  className={`rule-step ${step === s.id ? 'active' : ''} ${step > s.id ? 'done' : ''}`}
                  onClick={() => setStep(s.id)}
                >
                  <Icon size={16} />
                  <span>{s.label}</span>
                </button>
              )
            })}
          </div>

          {/* form */}
          <div className="rule-form-col">
            {error && <div className="form-error">{error}</div>}

            {step === 0 && (
              <>
                <div className="form-title">General Information</div>

                <div className="form-field">
                  <label>
                    Rule Name <span className="req">*</span>
                  </label>
                  <input
                    className="input"
                    maxLength={64}
                    placeholder="e.g. Allow-Web"
                    value={d.name}
                    onChange={(e) => set('name', e.target.value)}
                  />
                  <div className="char-hint">{d.name.length}/64</div>
                </div>

                <div className="form-field">
                  <label>Description</label>
                  <textarea
                    className="textarea"
                    maxLength={256}
                    placeholder="What does this rule do?"
                    value={d.description}
                    onChange={(e) => set('description', e.target.value)}
                  />
                  <div className="char-hint">{d.description.length}/256</div>
                </div>

                <div className="form-row">
                  <div className="form-field">
                    <label>
                      Priority <span className="req">*</span>
                    </label>
                    <input
                      className="input"
                      type="number"
                      min={1}
                      value={d.priority}
                      onChange={(e) => set('priority', e.target.value.replace(/[^0-9]/g, ''))}
                    />
                    <div className="field-hint">Lower number means higher priority</div>
                  </div>
                  <div className="form-field">
                    <label>Direction</label>
                    <div className="segmented">
                      {['inbound', 'outbound', 'both'].map((dir) => (
                        <button
                          key={dir}
                          type="button"
                          className={d.direction === dir ? 'on' : ''}
                          onClick={() => set('direction', dir)}
                        >
                          {dir[0].toUpperCase() + dir.slice(1)}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>

                <div className="form-field">
                  <label>Status</label>
                  <div className="toggle-row">
                    <label className="switch">
                      <input
                        type="checkbox"
                        checked={d.enabled}
                        onChange={(e) => set('enabled', e.target.checked)}
                      />
                      <span className="track" />
                    </label>
                    <span style={{ color: d.enabled ? 'var(--success)' : 'var(--text-muted)' }}>
                      {d.enabled ? 'Active' : 'Inactive'}
                    </span>
                    <span className="toggle-sep" />
                    <label className="switch">
                      <input
                        type="checkbox"
                        checked={d.log}
                        onChange={(e) => set('log', e.target.checked)}
                      />
                      <span className="track" />
                    </label>
                    <span className="toggle-label">Log hits to kernel log</span>
                  </div>
                </div>
              </>
            )}

            {step === 1 && (
              <>
                <div className="form-title">Source &amp; Destination</div>

                <div className="form-field">
                  <label>Source — Address List</label>
                  <select
                    className="select"
                    value={d.source_set}
                    onChange={(e) => set('source_set', e.target.value)}
                  >
                    <option value="">— none (use IP/CIDR below) —</option>
                    {sets.map((s) => (
                      <option key={s.id} value={s.name}>
                        {s.name} ({s.kind}) · {s.entries_count} entries
                      </option>
                    ))}
                  </select>
                </div>
                {!d.source_set && (
                  <div className="form-field">
                    <label>Source — IP / CIDR</label>
                    <input
                      className="input"
                      placeholder="any, 10.10.0.0/16, 203.0.113.5"
                      value={d.source}
                      onChange={(e) => set('source', e.target.value)}
                    />
                  </div>
                )}

                <div className="form-field">
                  <label>Destination — Address List</label>
                  <select
                    className="select"
                    value={d.dest_set}
                    onChange={(e) => set('dest_set', e.target.value)}
                  >
                    <option value="">— none (use IP/CIDR below) —</option>
                    {sets.map((s) => (
                      <option key={s.id} value={s.name}>
                        {s.name} ({s.kind}) · {s.entries_count} entries
                      </option>
                    ))}
                  </select>
                </div>
                {!d.dest_set && (
                  <div className="form-field">
                    <label>Destination — IP / CIDR</label>
                    <input
                      className="input"
                      placeholder="any, 10.10.20.0/24"
                      value={d.destination}
                      onChange={(e) => set('destination', e.target.value)}
                    />
                  </div>
                )}

                <div className="info-box">
                  <Info size={15} />
                  <span>
                    Tip: choose an Address List for dynamic groups (blacklist / whitelist), or
                    type a static IP/CIDR. Leave both empty to match <b>any</b>.
                  </span>
                </div>
              </>
            )}

            {step === 2 && (
              <>
                <div className="form-title">Service &amp; Action</div>

                <div className="form-row">
                  <div className="form-field">
                    <label>Protocol</label>
                    <select
                      className="select"
                      value={d.protocol}
                      onChange={(e) => set('protocol', e.target.value)}
                    >
                      <option value="any">any protocol</option>
                      <option value="tcp">TCP</option>
                      <option value="udp">UDP</option>
                      <option value="icmp">ICMP</option>
                    </select>
                  </div>
                  <div className="form-field">
                    <label>Port (tcp/udp only)</label>
                    <input
                      className="input"
                      placeholder="e.g. 443"
                      value={d.port}
                      onChange={(e) => set('port', e.target.value.replace(/[^0-9]/g, ''))}
                      disabled={d.protocol !== 'tcp' && d.protocol !== 'udp'}
                    />
                    <div className="field-hint">Single port — empty = any port</div>
                  </div>
                </div>

                <div className="form-field">
                  <label>Action</label>
                  <div className="action-big-grid">
                    {ACTIONS.map((a) => {
                      const Icon = a.icon
                      return (
                        <button
                          key={a.value}
                          type="button"
                          className={`action-big ${d.action === a.value ? `on-${a.value}` : ''}`}
                          onClick={() => set('action', a.value)}
                        >
                          <Icon size={16} />
                          {a.label}
                        </button>
                      )
                    })}
                  </div>
                </div>

                <div className="info-box">
                  <Info size={15} />
                  <span>{infoText}</span>
                </div>
              </>
            )}

            {step === 3 && (
              <>
                <div className="form-title">Advanced</div>

                <div className="form-field">
                  <label>Rate Limit (connections / second)</label>
                  <input
                    className="input"
                    placeholder="unlimited — e.g. 10"
                    value={d.rate}
                    onChange={(e) => set('rate', e.target.value.replace(/[^0-9]/g, ''))}
                  />
                  <div className="field-hint">
                    Caps new connections matched by this rule per second. Empty = unlimited.
                  </div>
                </div>

                {/* ── فاز ۴: زمان‌بندی قانون ── */}
                <div className="form-field">
                  <label>Schedule — Time-based Activation</label>
                  <div className="toggle-row">
                    <label className="switch">
                      <input
                        type="checkbox"
                        checked={d.sched_on}
                        onChange={(e) => set('sched_on', e.target.checked)}
                      />
                      <span className="track" />
                    </label>
                    <span className="toggle-label">
                      {d.sched_on
                        ? 'Rule applies only inside the time window'
                        : 'Always active (no time window)'}
                    </span>
                  </div>
                </div>

                {d.sched_on && (
                  <>
                    <div className="form-row">
                      <div className="form-field">
                        <label>Start Time</label>
                        <input
                          className="input"
                          type="time"
                          value={d.sched_start}
                          onChange={(e) => set('sched_start', e.target.value)}
                        />
                      </div>
                      <div className="form-field">
                        <label>End Time</label>
                        <input
                          className="input"
                          type="time"
                          value={d.sched_end}
                          onChange={(e) => set('sched_end', e.target.value)}
                        />
                      </div>
                    </div>

                    <div className="form-field">
                      <label>Days of Week</label>
                      <div className="day-chips">
                        {DAYS.map((day) => (
                          <button
                            key={day.value}
                            type="button"
                            className={`day-chip ${d.sched_days.includes(day.value) ? 'on' : ''}`}
                            onClick={() => toggleDay(day.value)}
                          >
                            {day.label}
                          </button>
                        ))}
                      </div>
                      <div className="field-hint">
                        No day selected = every day. A window like 22:00 → 06:00 crosses midnight.
                        Times follow the server clock.
                      </div>
                    </div>
                  </>
                )}

                <div className="form-field">
                  <label>Rule Flow</label>
                  <div className="flow-row">
                    <div className="flow-chip">
                      <span className="f-icon">
                        <Server size={15} />
                      </span>
                      <div>
                        <div className="f-label">Source</div>
                        <div className="f-value">{srcLabel(d)}</div>
                      </div>
                    </div>
                    <ArrowRight size={15} className="flow-arrow" />
                    <div className="flow-chip">
                      <span className="f-icon">
                        <Gauge size={15} />
                      </span>
                      <div>
                        <div className="f-label">Service</div>
                        <div className="f-value">{svcLabel(d)}</div>
                      </div>
                    </div>
                    <ArrowRight size={15} className="flow-arrow" />
                    <div className="flow-chip">
                      <span className="f-icon">
                        <Globe size={15} />
                      </span>
                      <div>
                        <div className="f-label">Destination</div>
                        <div className="f-value">{dstLabel(d)}</div>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="info-box">
                  <ShieldAlert size={15} />
                  <span>{infoText}</span>
                </div>
              </>
            )}
          </div>

          {/* live summary */}
          <div className="summary-panel">
            <div className="summary-head">
              <h3>Rule Summary</h3>
              <span
                className={`pill ${d.enabled ? 'pill-allow' : 'pill-neutral'}`}
                style={{ textTransform: 'none' }}
              >
                <span className="dot" />
                {d.enabled ? 'Active' : 'Inactive'}
              </span>
            </div>

            <div className="kv">
              <span className="k">Name</span>
              <span className="v" title={d.name}>
                {d.name || '—'}
              </span>
            </div>
            <div className="kv">
              <span className="k">Priority</span>
              <span className="v fw-num">{d.priority || '—'}</span>
            </div>
            <div className="kv">
              <span className="k">Action</span>
              <span className={`v pill pill-${d.action}`} style={{ width: 'fit-content' }}>
                <span className="dot" />
                {d.action}
              </span>
            </div>
            <div className="kv">
              <span className="k">Direction</span>
              <span className="v">{d.direction}</span>
            </div>

            <div className="summary-sec">
              <div className="summary-sec-title">Match</div>
              <div className="flow-mini">
                <div className="flow-mini-row">
                  <Server size={13} />
                  <span>
                    Source: <b>{srcLabel(d)}</b>
                  </span>
                </div>
                <div className="flow-mini-row">
                  <Gauge size={13} />
                  <span>
                    Service: <b>{svcLabel(d)}</b>
                  </span>
                </div>
                <div className="flow-mini-row">
                  <Globe size={13} />
                  <span>
                    Destination: <b>{dstLabel(d)}</b>
                  </span>
                </div>
              </div>
            </div>

            <div className="summary-sec">
              <div className="summary-sec-title">Options</div>
              <div className="kv">
                <span className="k">Logging</span>
                <span className="v">{d.log ? 'Enabled' : 'Disabled'}</span>
              </div>
              <div className="kv">
                <span className="k">Rate limit</span>
                <span className="v fw-num">{d.rate ? `${d.rate}/s` : 'Unlimited'}</span>
              </div>
              <div className="kv">
                <span className="k">Schedule</span>
                <span className="v">
                  {d.sched_on ? (
                    <span className="sched-mini">
                      <Clock size={12} />
                      {d.sched_start || '??:??'}–{d.sched_end || '??:??'}
                      {d.sched_days.length > 0 ? ` · ${d.sched_days.length} day(s)` : ' · every day'}
                    </span>
                  ) : (
                    'Always'
                  )}
                </span>
              </div>
              <div className="kv">
                <span className="k">Rule ID</span>
                <span className="v" title={rule?.id}>
                  {isEdit ? `${rule!.id.slice(0, 8)}…` : 'new'}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* ── Footer ── */}
        <div className="modal-foot">
          <span className="step-info">
            Step {step + 1} of {STEPS.length}
          </span>
          {step > 0 && (
            <button type="button" className="btn btn-ghost" onClick={() => setStep((s) => s - 1)}>
              <ArrowRight size={15} style={{ transform: 'rotate(180deg)' }} />
              Back
            </button>
          )}
          <button type="button" className="btn btn-ghost" onClick={onClose}>
            Cancel
          </button>
          {step < STEPS.length - 1 ? (
            <button type="button" className="btn btn-primary" onClick={next}>
              Next
              <ArrowRight size={15} />
            </button>
          ) : (
            <button type="button" className="btn btn-primary" onClick={save} disabled={saving}>
              <ShieldCheck size={15} />
              {saving ? 'Saving…' : isEdit ? 'Save changes' : 'Create rule'}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}