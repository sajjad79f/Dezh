const TOKEN_KEY = 'dezh_token'

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token)
}

export function clearToken() {
  localStorage.removeItem(TOKEN_KEY)
}

export interface LoginResult {
  token: string
  username: string
  role: string
  display_name: string | null
}

export interface MeResult {
  id: string
  username: string
  role: string
  display_name: string | null
}

export async function login(username: string, password: string): Promise<LoginResult> {
  const res = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(err.error || 'Login failed')
  }
  return res.json()
}

export async function me(): Promise<MeResult> {
  const token = getToken()
  if (!token) throw new Error('not authenticated')

  const res = await fetch('/api/auth/me', {
    headers: { Authorization: `Bearer ${token}` },
  })
  if (!res.ok) {
    clearToken()
    throw new Error('session expired')
  }
  return res.json()
}

export async function logout(): Promise<void> {
  const token = getToken()
  if (token) {
    await fetch('/api/auth/logout', {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    }).catch(() => {})
  }
  clearToken()
}