import { NavLink, Outlet } from 'react-router-dom'
import {
  LayoutDashboard,
  Boxes,
  Server,
  Terminal,
  Shield,
} from 'lucide-react'
import './Layout.css'

const nav = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/modules', icon: Boxes, label: 'Modules' },
  { to: '/services', icon: Server, label: 'Core Services' },
  { to: '/console', icon: Terminal, label: 'Console' },
  { to: '/Firewall', icon: Terminal, label: 'Firewall' },
]

export function Layout() {
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
              className={({ isActive }) => (isActive ? 'nav-item active' : 'nav-item')}
            >
              <Icon size={18} />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-footer">v0.1.0-alpha</div>
      </aside>
      <main className="content">
        <Outlet />
      </main>
    </div>
  )
}