import { useEffect, useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { changePassword, clearToken, me } from '../api/auth'
import '../styles/login.css'
import logo from '../assets/logo.png'

/**
 * صفحه اجباری تغییر رمز عبور.
 * وقتی پرچم must_change_password فعال باشد (مثلاً اولین ورود admin با
 * رمز پیش‌فرض)، تا تغییر رمز از این صفحه رد نمی‌شوی.
 */
export function ChangePassword() {
  const navigate = useNavigate()
  const [checking, setChecking] = useState(true)
  const [current, setCurrent] = useState('')
  const [next, setNext] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    me()
      .then((user) => {
        if (!user.must_change_password) {
          navigate('/', { replace: true })
          return
        }
        setChecking(false)
      })
      .catch(() => {
        clearToken()
        navigate('/login', { replace: true })
      })
  }, [navigate])

  function validate(): string | null {
    if (!current) return 'رمز عبور فعلی را وارد کنید'
    if (next.length < 8) return 'رمز جدید باید حداقل ۸ کاراکتر باشد'
    if (!/[A-Za-z]/.test(next)) return 'رمز جدید باید حداقل یک حرف داشته باشد'
    if (!/[0-9]/.test(next)) return 'رمز جدید باید حداقل یک عدد داشته باشد'
    if (next === current) return 'رمز جدید نباید با رمز فعلی یکسان باشد'
    if (next !== confirm) return 'تکرار رمز جدید مطابقت ندارد'
    return null
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    const v = validate()
    if (v) {
      setError(v)
      return
    }
    setError(null)
    setLoading(true)
    try {
      await changePassword(current, next)
      // رمز عوض شد؛ session فعلی معتبر می‌ماند — برو به داشبورد
      navigate('/', { replace: true })
    } catch (err: any) {
      setError(err.message || 'Password change failed')
    } finally {
      setLoading(false)
    }
  }

  if (checking) {
    return (
      <div id="loginBG">
        <div className="login">
          <div className="login-side">
            <div style={{ padding: 40, textAlign: 'center' }}>Checking session…</div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div id="loginBG">
      <div className="login">
        {error && (
          <div className="alert">
            <button type="button" onClick={() => setError(null)}>
              <span aria-hidden="true">&times;</span>
            </button>
            {error}
          </div>
        )}

        <div className="brand-side">
          <img
            src={logo}
            alt="Logo"
            onError={(e) => {
              e.currentTarget.style.display = 'none'
            }}
          />
          <p>© Dezh 2026</p>
        </div>

        <div className="login-side">
          <div className="login-header">
            <h1>تغییر رمز عبور</h1>
            <p>برای ادامه باید رمز عبور خود را تغییر دهید</p>
          </div>

          <form onSubmit={onSubmit}>
            <div className="form-group">
              <input
                type="password"
                value={current}
                onChange={(e) => setCurrent(e.target.value)}
                autoComplete="current-password"
                required
              />
              <label>رمز عبور فعلی</label>
            </div>

            <div className="form-group">
              <input
                type="password"
                value={next}
                onChange={(e) => setNext(e.target.value)}
                autoComplete="new-password"
                required
              />
              <label>رمز عبور جدید</label>
            </div>

            <div className="form-group">
              <input
                type="password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                autoComplete="new-password"
                required
              />
              <label>تکرار رمز عبور جدید</label>
            </div>

            <p style={{ fontSize: 13, color: 'rgba(255,255,255,0.55)', margin: '0 0 16px' }}>
              حداقل ۸ کاراکتر، شامل حداقل یک حرف و یک عدد
            </p>

            <button type="submit" disabled={loading} className="login-btn">
              {loading ? 'در حال تغییر…' : 'تغییر رمز و ادامه'}
            </button>
          </form>
        </div>
      </div>
    </div>
  )
}