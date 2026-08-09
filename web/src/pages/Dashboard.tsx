import { useEffect, useState } from 'react'
import { api } from '../api/client'
import type { ModuleDto, CoreServiceDto, CommandDto } from '../types'

export function Dashboard() {
  const [modules, setModules] = useState<ModuleDto[]>([])
  const [services, setServices] = useState<CoreServiceDto[]>([])
  const [commands, setCommands] = useState<CommandDto[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    Promise.all([api.modules(), api.coreServices(), api.commands()])
      .then(([m, s, c]) => {
        setModules(m)
        setServices(s)
        setCommands(c)
      })
      .catch((e) => setError(e.message))
  }, [])

  if (error) {
    return (
      <div>
        <h1 className="page-title">Dashboard</h1>
        <div className="card" style={{ color: 'var(--danger)' }}>
          Cannot reach API: {error}
          <br />
          <small style={{ color: 'var(--text-muted)' }}>
            Make sure DCM is running on port 7878
          </small>
        </div>
      </div>
    )
  }

  return (
    <div>
      <h1 className="page-title">Dashboard</h1>

      <div className="grid grid-3" style={{ marginBottom: 20 }}>
        <div className="card">
          <h3>Modules</h3>
          <div style={{ fontSize: 32, fontWeight: 700, color: 'var(--accent)' }}>
            {modules.length}
          </div>
        </div>
        <div className="card">
          <h3>Core Services</h3>
          <div style={{ fontSize: 32, fontWeight: 700, color: 'var(--success)' }}>
            {services.length}
          </div>
        </div>
        <div className="card">
          <h3>Commands</h3>
          <div style={{ fontSize: 32, fontWeight: 700, color: 'var(--warning)' }}>
            {commands.length}
          </div>
        </div>
      </div>

      <div className="grid grid-2">
        <div className="card">
          <h3>Loaded Modules</h3>
          {modules.length === 0 ? (
            <p style={{ color: 'var(--text-muted)' }}>No modules</p>
          ) : (
            <ul style={{ listStyle: 'none' }}>
              {modules.map((m) => (
                <li
                  key={m.id}
                  style={{
                    padding: '8px 0',
                    borderBottom: '1px solid var(--border)',
                    display: 'flex',
                    justifyContent: 'space-between',
                  }}
                >
                  <span>{m.name}</span>
                  <span style={{ color: 'var(--text-muted)', fontSize: 13 }}>
                    {m.version}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="card">
          <h3>Core Services</h3>
          <ul style={{ listStyle: 'none' }}>
            {services.map((s) => (
              <li
                key={s.name}
                style={{
                  padding: '8px 0',
                  borderBottom: '1px solid var(--border)',
                  display: 'flex',
                  justifyContent: 'space-between',
                }}
              >
                <span>{s.name}</span>
                <span
                  style={{
                    color: 'var(--success)',
                    fontSize: 12,
                    fontWeight: 600,
                    textTransform: 'uppercase',
                  }}
                >
                  {s.status}
                </span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}