import { useEffect, useState } from 'react'
import { api } from '../api/client'

export function Dashboard() {
  const [modules, setModules] = useState(0)
  const [services, setServices] = useState(0)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([api.modules(), api.coreServices()])
      .then(([m, s]) => {
        setModules(m.length)
        setServices(s.length)
        setError(null)
      })
      .catch((e) => setError(e.message))
  }, [])

  return (
    <div>
      <h1 className="page-title">Dashboard</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}
      <div className="grid grid-2">
        <div className="card">
          <h3>Modules</h3>
          <p style={{ fontSize: 28, fontWeight: 700 }}>{modules}</p>
        </div>
        <div className="card">
          <h3>Core Services</h3>
          <p style={{ fontSize: 28, fontWeight: 700 }}>{services}</p>
        </div>
      </div>
    </div>
  )
}