import type {
  ModuleDto,
  CommandDto,
  CoreServiceDto,
  ExecuteCommandResponse,
} from '../types'

const BASE = '/api'

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(err.error ?? `HTTP ${res.status}`)
  }
  return res.json()
}

export const api = {
  modules: () => get<ModuleDto[]>('/modules'),
  commands: () => get<CommandDto[]>('/commands'),
  coreServices: () => get<CoreServiceDto[]>('/core-services'),
  execute: (name: string, args: string[]) =>
    post<ExecuteCommandResponse>('/commands/execute', { name, args }),
}