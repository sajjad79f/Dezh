import { useEffect, useState } from 'react'
import { api } from '../../api/client'
import type { ModuleDto } from '../../types'

export function ModulesPage() {
  const [items, setItems] = useState<ModuleDto[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .modules()
      .then(setItems)
      .catch((e) => setError(e.message))
  }, [])

  return (
    <div>
      <h1 className="page-title">Services · Modules</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)' }}>
          {error}
        </div>
      )}
      <div className="card">
        <ul style={{ listStyle: 'none' }}>
          {items.map((m) => (
            <li
              key={m.id}
              style={{
                padding: '10px 0',
                borderBottom: '1px solid var(--border)',
                display: 'flex',
                justifyContent: 'space-between',
              }}
            >
              <span>{m.name}</span>
              <span style={{ color: 'var(--text-muted)', fontSize: 13 }}>{m.version}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  )
}