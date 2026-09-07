import { FormEvent, useEffect, useState } from 'react'
import { api } from '../../api/client'
import type { CommandDto } from '../../types'

export function ConsolePage() {
  const [commands, setCommands] = useState<CommandDto[]>([])
  const [name, setName] = useState('')
  const [argsText, setArgsText] = useState('')
  const [output, setOutput] = useState('')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api
      .commands()
      .then((list) => {
        setCommands(list)
        if (list[0]) setName(list[0].name)
      })
      .catch((e) => setError(e.message))
  }, [])

  async function onRun(e: FormEvent) {
    e.preventDefault()
    setError(null)
    try {
      const args = argsText.trim() ? argsText.trim().split(/\s+/) : []
      const res = await api.execute(name, args)
      setOutput(res.output)
    } catch (err: any) {
      setError(err.message)
    }
  }

  return (
    <div>
      <h1 className="page-title">Diagnostics · Console</h1>
      {error && (
        <div className="card" style={{ color: 'var(--danger)', marginBottom: 16 }}>
          {error}
        </div>
      )}
      <div className="card">
        <form onSubmit={onRun} style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          <select value={name} onChange={(e) => setName(e.target.value)} style={inputStyle}>
            {commands.map((c) => (
              <option key={c.name} value={c.name}>
                {c.name} — {c.description}
              </option>
            ))}
          </select>
          <input
            placeholder="args (space separated)"
            value={argsText}
            onChange={(e) => setArgsText(e.target.value)}
            style={inputStyle}
          />
          <button type="submit" style={btnStyle}>
            Run
          </button>
        </form>
        {output && (
          <pre
            style={{
              marginTop: 16,
              padding: 12,
              background: 'var(--bg)',
              borderRadius: 8,
              fontSize: 12,
              overflow: 'auto',
            }}
          >
            {output}
          </pre>
        )}
      </div>
    </div>
  )
}

const inputStyle: React.CSSProperties = {
  background: 'var(--bg)',
  border: '1px solid var(--border)',
  borderRadius: 8,
  padding: '10px 12px',
  color: 'var(--text)',
  fontSize: 13,
}

const btnStyle: React.CSSProperties = {
  background: 'var(--accent-dim)',
  color: '#fff',
  border: 'none',
  borderRadius: 8,
  padding: '10px 16px',
  fontWeight: 600,
  cursor: 'pointer',
}