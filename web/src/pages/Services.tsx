import { useEffect, useState } from 'react'
import { api } from '../api/client'
import type { CoreServiceDto } from '../types'

const DESCRIPTIONS: Record<string, string> = {
  DEF: 'Event Fabric — communication backbone',
  DAI: 'Asset Intelligence — asset registry',
  DKG: 'Knowledge Graph — relationships between assets',
  DIE: 'Intelligence Engine — findings & analysis',
  DDE: 'Decision Engine — autonomy & approvals',
}

export function Services() {
  const [services, setServices] = useState<CoreServiceDto[]>([])

  useEffect(() => {
    api.coreServices().then(setServices)
  }, [])

  return (
    <div>
      <h1 className="page-title">Core Services</h1>
      <div className="grid grid-2">
        {services.map((s) => (
          <div className="card" key={s.name}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h3 style={{ margin: 0, fontSize: 18, color: 'var(--text)', textTransform: 'none', letterSpacing: 0 }}>
                {s.name}
              </h3>
              <span
                style={{
                  background: 'rgba(34, 197, 94, 0.15)',
                  color: 'var(--success)',
                  padding: '4px 10px',
                  borderRadius: 20,
                  fontSize: 12,
                  fontWeight: 600,
                }}
              >
                {s.status}
              </span>
            </div>
            <p style={{ marginTop: 10, color: 'var(--text-muted)', fontSize: 14 }}>
              {DESCRIPTIONS[s.name] ?? 'Core platform service'}
            </p>
          </div>
        ))}
      </div>
    </div>
  )
}