import { useCallback, useEffect, useState } from 'react'
import { firewallApi, type FirewallSettings } from '../../api/firewall'

export function FirewallProtection() {
  const [settings, setSettings] = useState<FirewallSettings | null>(null)
  const [dos, setDos] = useState(false)
  const [rate, setRate] = useState('200')
  const [error, setError] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const load = useCallback(async function load() {
    try {
      const s = await firewallApi.settings()
      setSettings(s)
      setDos(s.dos_protection)
      setRate(String(s.syn_rate))
      setError(null)
    } catch (e: any) {
      setError(e.message)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  async function onSave() {
    setSaving(true)
    setOk(null)
    try {
      const r = Number(rate)
      await firewallApi.saveSettings({
        dos_protection: dos,
        syn_rate: Number.isFinite(r) && r > 0 ? r : undefined,
      })
      await load()
      setOk('Settings saved and applied to nftables')
    } catch (e: any) {
      setError(e.message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div>
      <h1 className="page-title">Firewall · Protection</h1>
      <p style={{ color: 'var(--text-muted)', fontSize: 13, marginTop: -8 }}>
        حفاظت‌های stateful خودکار — بدون نیاز به ساخت قانون دستی.
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
        <h3 style={{ marginTop: 0 }}>SYN-Flood Protection (DoS)</h3>
        <p style={{ fontSize: 13, color: 'var(--text-muted)', lineHeight: 1.8 }}>
          روی زنجیره <code>input</code>: اتصال‌های SYN جدید تا سقف مشخصی در ثانیه پذیرفته می‌شوند و
          بیش از سقف <b style={{ color: '#ff9b9b' }}>Drop</b> می‌شوند. برای سرور‌های رو اینترنت
          پیشنهاد می‌شود روشن باشد.
        </p>

        <div style={{ display: 'flex', gap: 16, alignItems: 'center', flexWrap: 'wrap', marginTop: 12 }}>
          <label style={{ display: 'flex', gap: 8, alignItems: 'center', fontSize: 14 }}>
            <input type="checkbox" checked={dos} onChange={(e) => setDos(e.target.checked)} />
            {dos ? 'Enabled' : 'Disabled'}
          </label>

          <label style={{ display: 'flex', gap: 8, alignItems: 'center', fontSize: 13 }}>
            SYN limit / second:
            <input
              value={rate}
              onChange={(e) => setRate(e.target.value.replace(/[^0-9]/g, ''))}
              disabled={!dos}
              style={{ ...field, width: 100 }}
            />
          </label>

          <button type="button" onClick={onSave} disabled={saving} style={btn}>
            {saving ? 'Saving…' : 'Save & Apply'}
          </button>
        </div>

        {settings && dos !== settings.dos_protection && (
          <p style={{ fontSize: 12, color: '#f0b429', marginBottom: 0 }}>
            تغییرات ذخیره نشده — روی «Save & Apply» بزن.
          </p>
        )}
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Always-on hardening (بدون تنظیم)</h3>
        <ul style={{ fontSize: 13, lineHeight: 2, color: 'var(--text-muted)', margin: 0, paddingLeft: 20 }}>
          <li>
            <code style={{ color: 'var(--text)' }}>ct state invalid drop</code> — بسته‌های بی‌رضایت
            state ماشین (بخشی از اسکن/جعل) در هر سه زنجیره Drop می‌شوند.
          </li>
          <li>
            <code style={{ color: 'var(--text)' }}>ct state established,related accept</code> —
            اتصال‌های برقرار از روی قوانین رد می‌شوند (پس adding a rule doesn't kill live sessions —
            به‌جز blacklist).
          </li>
          <li>
            blacklist/whitelist بالاتر از همه‌ی قوانین اعمال می‌شوند — لیست سیاه حتی اتصال زنده را
            همان لحظه قطع می‌کند.
          </li>
          <li>
            لاگ hit هر قانون (اگر LOG فعال باشد): <code>journalctl -k | grep "dezh: "</code>
          </li>
          <li>
            لاگ hit هر قانون (اگر LOG فعال باشد) در صفحه‌ی{' '}
            <b style={{ color: 'var(--text)' }}>Events</b> زنده نمایش داده می‌شود
            — یا مستقیم: <code>journalctl -k | grep dezh:</code>
          </li>
        </ul>
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