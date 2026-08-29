import { useEffect, useState } from 'react'
import { Navigate, Outlet } from 'react-router-dom'
import { getToken, me } from '../api/auth'

export function RequireAuth() {
  const [status, setStatus] = useState<'loading' | 'ok' | 'no'>('loading')

  useEffect(() => {
    if (!getToken()) {
      setStatus('no')
      return
    }
    me()
      .then(() => setStatus('ok'))
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

  return <Outlet />
}