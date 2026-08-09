import { useEffect, useRef, useState } from 'react'
import { api } from '../api/client'
import type { CommandDto } from '../types'

export function Console() {
  const [commands, setCommands] = useState<CommandDto[]>([])
  const [input, setInput] = useState('')
  const [lines, setLines] = useState<string[]>([
    'DEZH Web Console ready.',
    'Type a command (e.g. system, asset list, help) and press Enter.',
  ])
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    api.commands().then(setCommands)
  }, [])

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [lines])

  async function run() {
    const raw = input.trim()
    if (!raw) return

    const parts = raw.split(/\s+/)
    const name = parts[0]
    const args = parts.slice(1)

    setLines((prev) => [...prev, `DEZH> ${raw}`])
    setInput('')

    try {
      const res = await api.execute(name, args)
      setLines((prev) => [...prev, res.output || '(no output)', ''])
    } catch (e: any) {
      setLines((prev) => [...prev, `Error: ${e.message}`, ''])
    }
  }

  return (
    <div>
      <h1 className="page-title">Console</h1>

      <div className="grid grid-2">
        <div className="card" style={{ display: 'flex', flexDirection: 'column', minHeight: 420 }}>
          <h3>Terminal</h3>
          <div
            style={{
              flex: 1,
              background: '#010409',
              borderRadius: 8,
              padding: 14,
              fontFamily: 'var(--mono)',
              fontSize: 13,
              overflowY: 'auto',
              whiteSpace: 'pre-wrap',
              marginBottom: 12,
              lineHeight: 1.5,
            }}
          >
            {lines.map((l, i) => (
              <div key={i}>{l}</div>
            ))}
            <div ref={bottomRef} />
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && run()}
              placeholder="e.g. asset list"
              style={{
                flex: 1,
                background: 'var(--bg)',
                border: '1px solid var(--border)',
                borderRadius: 8,
                padding: '10px 14px',
                color: 'var(--text)',
                fontFamily: 'var(--mono)',
                fontSize: 13,
                outline: 'none',
              }}
            />
            <button
              onClick={run}
              style={{
                background: 'var(--accent-dim)',
                color: '#fff',
                border: 'none',
                borderRadius: 8,
                padding: '10px 18px',
                fontWeight: 600,
              }}
            >
              Run
            </button>
          </div>
        </div>

        <div className="card">
          <h3>Available Commands</h3>
          <ul style={{ listStyle: 'none' }}>
            {commands.map((c) => (
              <li
                key={c.name}
                onClick={() => setInput(c.name + ' ')}
                style={{
                  padding: '10px 0',
                  borderBottom: '1px solid var(--border)',
                  cursor: 'pointer',
                }}
              >
                <div style={{ fontFamily: 'var(--mono)', color: 'var(--accent)', fontSize: 13 }}>
                  {c.name}
                </div>
                <div style={{ fontSize: 12, color: 'var(--text-muted)', marginTop: 2 }}>
                  {c.description}
                </div>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}