import { useEffect, useState } from 'react'
import { api } from '../api/client'
import type { ModuleDto } from '../types'

export function Modules() {
  const [modules, setModules] = useState<ModuleDto[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.modules()
      .then(setModules)
      .finally(() => setLoading(false))
  }, [])

  return (
    <div>
      <h1 className="page-title">Modules</h1>
      <div className="card">
        {loading ? (
          <p style={{ color: 'var(--text-muted)' }}>Loading…</p>
        ) : modules.length === 0 ? (
          <p style={{ color: 'var(--text-muted)' }}>No modules loaded</p>
        ) : (
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ textAlign: 'left', color: 'var(--text-muted)', fontSize: 13 }}>
                <th style={{ padding: '8px 0' }}>ID</th>
                <th>Name</th>
                <th>Version</th>
              </tr>
            </thead>
            <tbody>
              {modules.map((m) => (
                <tr key={m.id} style={{ borderTop: '1px solid var(--border)' }}>
                  <td style={{ padding: '12px 0', fontFamily: 'var(--mono)', fontSize: 13 }}>
                    {m.id}
                  </td>
                  <td>{m.name}</td>
                  <td style={{ color: 'var(--text-muted)' }}>{m.version}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  )
}