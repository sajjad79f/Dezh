import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import {
  LayoutDashboard,
  Boxes,
  Server,
  Terminal,
  Shield,
  Users,
} from 'lucide-react'
import { logout } from '../api/auth'
import './Layout.css'

const nav = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/modules', icon: Boxes, label: 'Modules' },
  { to: '/services', icon: Server, label: 'Core Services' },
  { to: '/console', icon: Terminal, label: 'Console' },
  { to: '/firewall', icon: Shield, label: 'Firewall' },
  { to: '/users', icon: Users, label: 'Users' },
  { to: '/identities', icon: Users, label: 'Identities' },
]

export function Layout() {
  const navigate = useNavigate()

  async function onLogout() {
    await logout()
    navigate('/login', { replace: true })
  }

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="brand">
          <Shield size={22} />
          <span>DEZH</span>
        </div>

        <nav>
          {nav.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive }) =>
                isActive ? 'nav-item active' : 'nav-item'
              }
            >
              <Icon size={18} />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>

        <div className="sidebar-footer">
          <div style={{ marginBottom: 10, fontSize: 12 }}>v0.1.0-alpha</div>
          <button
            type="button"
            onClick={onLogout}
            style={{
              background: 'transparent',
              border: '1px solid var(--border)',
              color: 'var(--text-muted)',
              borderRadius: 8,
              padding: '8px 12px',
              width: '100%',
              cursor: 'pointer',
              fontSize: 13,
            }}
          >
            Sign out
          </button>
        </div>
      </aside>

      <main className="content">
        <Outlet />
      </main>
    </div>
  )
}