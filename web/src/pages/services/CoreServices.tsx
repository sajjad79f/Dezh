import { useEffect, useState } from 'react'
import { api } from '../../api/client'
import type { CoreServiceDto } from '../../types'

export function CoreServicesPage() {
  const [items, setItems] = useState<CoreServiceDto[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .coreServices()
      .then(setItems)
      .catch((e) => setError(e.message))
  }, [])

  return (
    <div>
      <h1 className="page-title">Services · Core</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)' }}>
          {error}
        </div>
      )}
      <div className="card">
        <ul style={{ listStyle: 'none' }}>
          {items.map((s) => (
            <li
              key={s.name}
              style={{
                padding: '10px 0',
                borderBottom: '1px solid var(--border)',
                display: 'flex',
                justifyContent: 'space-between',
              }}
            >
              <span>{s.name}</span>
              <span style={{ color: 'var(--text-muted)', fontSize: 13 }}>{s.status}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  )
}