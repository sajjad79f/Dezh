import { useEffect, useState } from 'react'
import { Navigate, Outlet } from 'react-router-dom'
import { getToken, me } from '../api/auth'

/**
 * محافظ مسیرهای احراز هویت‌شده.
 * اگر کاربر لاگین باشد ولی رمز عبورش موقت (پیش‌فرض) باشد،
 * قبل از هر چیز به صفحه تغییر رمز هدایت می‌شود.
 */
export function RequireAuth() {
  const [status, setStatus] = useState<'loading' | 'ok' | 'must_change' | 'no'>(
    'loading',
  )

  useEffect(() => {
    if (!getToken()) {
      setStatus('no')
      return
    }
    me()
      .then((user) => setStatus(user.must_change_password ? 'must_change' : 'ok'))
      .catch(() => setStatus('no'))
  }, [])

  if (status === 'loading') {
    return (
      <div style={{ padding: 40, color: 'var(--text-muted)' }}>
        Checking session…
      </div>
    )
  }

  if (status === 'no') {
    return <Navigate to="/login" replace />
  }

  if (status === 'must_change') {
    return <Navigate to="/change-password" replace />
  }

  return <Outlet />
}