import { FormEvent, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { login, setToken } from '../api/auth'
import '../styles/login.css'
import logo from '../assets/logo.png'

export function Login() {
  const navigate = useNavigate()
  const [username, setUsername] = useState('admin')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      const result = await login(username.trim(), password)
      setToken(result.token)
      navigate('/', { replace: true })
    } catch (err: any) {
      setError(err.message || 'Login failed')
    } finally {
      setLoading(false)
    }
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
            <h1>دژ</h1>
            <p>سیستم مدیریت یکپارچه شبکه</p>
          </div>

          <form onSubmit={onSubmit}>
            <div className="form-group">
              <input
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                autoComplete="username"
                required
              />
              <label>نام کاربری</label>
            </div>

            <div className="form-group">
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoComplete="current-password"
                required
              />
              <label>رمز عبور</label>
            </div>

            <button type="submit" disabled={loading} className="login-btn">
              {loading ? 'درحال ورود' : 'ورود'}
            </button>
          </form>

          <div className="security-notice">
            <h4>اطلاعیه امنیتی</h4>
            <p>تمام فعالیت‌ها به دلایل امنیتی رصد و ثبت می‌شوند.</p>
          </div>
        </div>
      </div>
    </div>
  )
}