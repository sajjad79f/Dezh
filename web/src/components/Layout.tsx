import { useState } from 'react'
import { NavLink, Outlet, useNavigate, useLocation } from 'react-router-dom'
import {
  LayoutDashboard,
  Shield,
  Route,
  Network,
  Users,
  Boxes,
  Server,
  Terminal,
  ChevronDown,
  LogOut,
} from 'lucide-react'
import { logout } from '../api/auth'
import './Layout.css'

type NavItem = {
  to?: string
  icon: React.ComponentType<{ size?: number }>
  label: string
  children?: { to: string; label: string }[]
}

const nav: NavItem[] = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  {
    icon: Shield,
    label: 'Firewall',
    children: [
      { to: '/firewall/rules', label: 'Rules' },
      { to: '/firewall/interfaces', label: 'Interfaces / Zones' },
    ],
  },
  { to: '/routing', icon: Route, label: 'Routing' },
  {
    icon: Network,
    label: 'Identity & Accounting',
    children: [
      { to: '/identities', label: 'Identities' },
      { to: '/identities/sessions', label: 'Sessions' },
    ],
  },
  {
    icon: Users,
    label: 'System',
    children: [{ to: '/users', label: 'Users' }],
  },
  {
    icon: Boxes,
    label: 'Services',
    children: [
      { to: '/modules', label: 'Modules' },
      { to: '/services', label: 'Core Services' },
    ],
  },
  { to: '/console', icon: Terminal, label: 'Console' },
]

function isGroupActive(item: NavItem, pathname: string): boolean {
  if (item.to && (pathname === item.to || (item.to !== '/' && pathname.startsWith(item.to)))) {
    return true
  }
  return !!item.children?.some(
    (c) => pathname === c.to || pathname.startsWith(c.to + '/'),
  )
}

export function Layout() {
  const navigate = useNavigate()
  const location = useLocation()
  const [open, setOpen] = useState<Record<string, boolean>>({})

  function toggle(label: string) {
    setOpen((prev) => ({ ...prev, [label]: !prev[label] }))
  }

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

        <nav className="side-nav">
          {nav.map((item) => {
            const Icon = item.icon
            const hasChildren = !!item.children?.length
            const active = isGroupActive(item, location.pathname)
            const expanded = open[item.label] ?? active

            if (!hasChildren && item.to) {
              return (
                <NavLink
                  key={item.label}
                  to={item.to}
                  end={item.to === '/'}
                  className={({ isActive }) =>
                    isActive ? 'nav-item active' : 'nav-item'
                  }
                >
                  <Icon size={18} />
                  <span>{item.label}</span>
                </NavLink>
              )
            }

            return (
              <div key={item.label} className={`nav-group ${active ? 'active' : ''}`}>
                <button
                  type="button"
                  className={`nav-item nav-parent ${expanded ? 'open' : ''}`}
                  onClick={() => toggle(item.label)}
                >
                  <Icon size={18} />
                  <span>{item.label}</span>
                  <ChevronDown
                    size={16}
                    className={`chevron ${expanded ? 'rot' : ''}`}
                  />
                </button>
                {expanded && (
                  <div className="nav-sub">
                    {item.children!.map((c) => (
                      <NavLink
                        key={c.to}
                        to={c.to}
                        className={({ isActive }) =>
                          isActive ? 'nav-sub-item active' : 'nav-sub-item'
                        }
                      >
                        {c.label}
                      </NavLink>
                    ))}
                  </div>
                )}
              </div>
            )
          })}
        </nav>

        <div className="sidebar-footer">
          <div className="ver">v0.1.0-alpha</div>
          <button type="button" className="logout-btn" onClick={onLogout}>
            <LogOut size={16} />
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