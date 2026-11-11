import { useEffect, useRef, useState } from 'react'
import { NavLink, Outlet, useNavigate, useLocation } from 'react-router-dom'
import {
  LayoutDashboard,
  Cable,
  Shield,
  Route,
  Users,
  Boxes,
  Terminal,
  ChevronDown,
  LogOut,
  Search,
  Bell,
  Sun,
  Moon,
  HelpCircle,
  KeyRound,
  Monitor,
} from 'lucide-react'
import { logout, me, type MeResult } from '../api/auth'
import { firewallApi } from '../api/firewall'
import './Layout.css'

type NavItem = {
  to?: string
  icon: React.ComponentType<{ size?: number }>
  label: string
  children?: { to: string; label: string }[]
}

type NavSection = {
  title?: string
  items: NavItem[]
}

const navSections: NavSection[] = [
  {
    items: [{ to: '/', icon: LayoutDashboard, label: 'Command Center' }],
  },
  {
    title: 'Operations',
    items: [
      { to: '/network', icon: Cable, label: 'Network' },
      { to: '/routing', icon: Route, label: 'Routing' },
      { to: '/console', icon: Terminal, label: 'Console' },
    ],
  },
  {
    title: 'Security',
    items: [
      {
        icon: Shield,
        label: 'Firewall',
        children: [
          { to: '/firewall/rules', label: 'Rules' },
          { to: '/firewall/nat', label: 'NAT' },
          { to: '/firewall/sets', label: 'Address Lists' },
          { to: '/firewall/connections', label: 'Connections' },
          { to: '/firewall/protection', label: 'Protection' },
          { to: '/firewall/events', label: 'Events' },
        ],
      },
    ],
  },
  {
    title: 'Identity',
    items: [
      {
        icon: Users,
        label: 'Identity & Accounting',
        children: [
          { to: '/identities', label: 'Identities' },
          { to: '/identities/sessions', label: 'Sessions' },
          { to: '/identity/accounting', label: 'Accounting' },
        ],
      },
      { to: '/users', icon: Monitor, label: 'Users' },
    ],
  },
  {
    title: 'Services',
    items: [
      {
        icon: Boxes,
        label: 'Services',
        children: [
          { to: '/modules', label: 'Modules' },
          { to: '/services', label: 'Core Services' },
        ],
      },
    ],
  },
]

const firewallTabs = [
  { to: '/firewall/rules', label: 'Rules' },
  { to: '/firewall/nat', label: 'NAT' },
  { to: '/firewall/sets', label: 'Address Lists' },
  { to: '/firewall/fqdn', label: 'FQDN' },
  { to: '/firewall/connections', label: 'Connections' },
  { to: '/firewall/protection', label: 'Protection' },
  { to: '/firewall/events', label: 'Events' },
]

function isGroupActive(item: NavItem, pathname: string): boolean {
  if (item.to && (pathname === item.to || (item.to !== '/' && pathname.startsWith(item.to)))) {
    return true
  }
  return !!item.children?.some((c) => pathname === c.to || pathname.startsWith(c.to + '/'))
}

export function Layout() {
  const navigate = useNavigate()
  const location = useLocation()
  const [open, setOpen] = useState<Record<string, boolean>>({})
  const [user, setUser] = useState<MeResult | null>(null)
  const [menuOpen, setMenuOpen] = useState(false)
  const [alerts, setAlerts] = useState(0)
  const [theme, setTheme] = useState<'dark' | 'light'>(() => {
    const t = localStorage.getItem('dezh_theme')
    return t === 'light' ? 'light' : 'dark'
  })
  const searchRef = useRef<HTMLInputElement>(null)
  const isFirewall = location.pathname.startsWith('/firewall')

  // ── تم ──
  useEffect(() => {
    document.documentElement.dataset.theme = theme
    localStorage.setItem('dezh_theme', theme)
  }, [theme])

  // ── کاربر + شمارنده‌ی هشدارها ──
  useEffect(() => {
    me().then(setUser).catch(() => {})

    async function pollAlerts() {
      try {
        const list = await firewallApi.events(50, 1)
        setAlerts(list.length)
      } catch {
        /* بی‌صدا */
      }
    }
    pollAlerts()
    const t = setInterval(pollAlerts, 30_000)
    return () => clearInterval(t)
  }, [])

  // ── Ctrl+K برای جستجو ──
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        searchRef.current?.focus()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  function onSearchEnter(e: React.KeyboardEvent<HTMLInputElement>) {
    const q = e.currentTarget.value.trim().toLowerCase()
    if (!q) return
    const flat = navSections.flatMap((s) =>
      s.items.flatMap((i) =>
        i.to
          ? [{ label: i.label, to: i.to }]
          : (i.children ?? []).map((c) => ({ label: c.label, to: c.to })),
      ),
    )
    const hit = flat.find((x) => x.label.toLowerCase().includes(q))
    if (hit) {
      navigate(hit.to)
      e.currentTarget.value = ''
      e.currentTarget.blur()
    }
  }

  function toggle(label: string) {
    setOpen((prev) => ({ ...prev, [label]: !prev[label] }))
  }

  async function onLogout() {
    await logout()
    navigate('/login', { replace: true })
  }

  const displayName = user?.display_name || user?.username || 'Admin'
  const initial = displayName.charAt(0).toUpperCase()

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">
            <Shield size={16} />
          </span>
          <span>DEZH</span>
        </div>

        <nav className="side-nav">
          {navSections.map((section, si) => (
            <div key={section.title ?? `sec-${si}`} className="nav-section">
              {section.title && <div className="nav-section-title">{section.title}</div>}
              {section.items.map((item) => {
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
                      <Icon size={17} />
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
                      <Icon size={17} />
                      <span>{item.label}</span>
                      <ChevronDown
                        size={15}
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
            </div>
          ))}
        </nav>

        <div className="sidebar-footer">
          <div className="ver">DEZH · v0.1.0-alpha</div>
          <button type="button" className="logout-btn" onClick={onLogout}>
            <LogOut size={15} />
            Sign out
          </button>
        </div>
      </aside>

      <div className="main-col">
        <header className="topbar">
          <div className="topbar-search">
            <Search size={15} />
            <input
              ref={searchRef}
              placeholder="Search pages, rules, sets…"
              onKeyDown={(e) => {
                if (e.key === 'Enter') onSearchEnter(e)
              }}
            />
            <kbd>Ctrl + K</kbd>
          </div>

          <div className="topbar-actions">
            <button
              type="button"
              className="topbar-icon"
              title="Security events (last hour)"
              onClick={() => navigate('/firewall/events')}
            >
              <Bell size={17} />
              {alerts > 0 && (
                <span className="topbar-badge">{alerts > 99 ? '99+' : alerts}</span>
              )}
            </button>

            <button
              type="button"
              className="topbar-icon"
              title={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
              onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
            >
              {theme === 'dark' ? <Sun size={17} /> : <Moon size={17} />}
            </button>

            <button
              type="button"
              className="topbar-icon"
              title="Search shortcut: Ctrl+K"
            >
              <HelpCircle size={17} />
            </button>

            <div className="topbar-user-wrap">
              <button
                type="button"
                className="topbar-user"
                onClick={() => setMenuOpen((v) => !v)}
              >
                <span className="avatar">{initial}</span>
                <span className="user-meta">
                  <strong>{displayName}</strong>
                  <small>{user?.role ?? '—'}</small>
                </span>
                <ChevronDown size={14} className={menuOpen ? 'rot' : ''} />
              </button>

              {menuOpen && (
                <>
                  <div className="menu-backdrop" onClick={() => setMenuOpen(false)} />
                  <div className="user-menu">
                    <div className="user-menu-head">
                      <span className="avatar">{initial}</span>
                      <div>
                        <strong>{displayName}</strong>
                        <small>{user?.username}</small>
                      </div>
                    </div>
                    <button
                      type="button"
                      onClick={() => {
                        setMenuOpen(false)
                        navigate('/change-password')
                      }}
                    >
                      <KeyRound size={15} />
                      Change password
                    </button>
                    <button type="button" className="danger" onClick={onLogout}>
                      <LogOut size={15} />
                      Sign out
                    </button>
                  </div>
                </>
              )}
            </div>
          </div>
        </header>

        <main className="content">
          {isFirewall && (
            <div className="fw-tabs">
              {firewallTabs.map((t) => (
                <NavLink
                  key={t.to}
                  to={t.to}
                  className={({ isActive }) => (isActive ? 'fw-tab active' : 'fw-tab')}
                >
                  {t.label}
                </NavLink>
              ))}
            </div>
          )}
          <Outlet />
        </main>
      </div>
    </div>
  )
}