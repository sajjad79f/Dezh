import { getToken } from './auth'

export function authHeaders(json = false): HeadersInit {
  const token = getToken()
  return {
    ...(json ? { 'Content-Type': 'application/json' } : {}),
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  }
}

/** برای جواب‌های JSON (GET / لیست‌ها) */
export async function readJson<T>(res: Response): Promise<T> {
  const text = await res.text()
  if (!res.ok) {
    let msg = res.statusText || `HTTP ${res.status}`
    if (text) {
      try {
        const j = JSON.parse(text)
        msg = j.error || j.message || msg
      } catch {
        msg = text.slice(0, 200)
      }
    }
    throw new Error(msg)
  }
  if (!text.trim()) {
    // 204 یا body خالی
    return undefined as T
  }
  return JSON.parse(text) as T
}

/** برای POST/PATCH/DELETE که اغلب 204 هستند */
export async function readEmpty(res: Response): Promise<void> {
  const text = await res.text()
  if (res.ok || res.status === 204) return
  let msg = res.statusText || `HTTP ${res.status}`
  if (text) {
    try {
      const j = JSON.parse(text)
      msg = j.error || j.message || msg
    } catch {
      msg = text.slice(0, 200)
    }
  }
  throw new Error(msg)
}