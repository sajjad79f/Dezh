import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { Layout } from './components/Layout'
import { RequireAuth } from './components/RequireAuth'
import { Login } from './pages/Login'
import { Dashboard } from './pages/Dashboard'

import { FirewallRules } from './pages/firewall/Rules'
import { FirewallInterfaces } from './pages/firewall/Interfaces'
import { RoutingPage } from './pages/routing/Index'
import { IdentitiesPage } from './pages/identity/Identities'
import { SessionsPage } from './pages/identity/Sessions'
import { UsersPage } from './pages/system/Users'
import { ModulesPage } from './pages/services/Modules'
import { CoreServicesPage } from './pages/services/CoreServices'
import { ConsolePage } from './pages/diagnostics/Console'
import Accounting from './identity/Accounting.tsx'

import './styles/global.css'
import './components/Layout.css'

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<Login />} />

        <Route element={<RequireAuth />}>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />

            <Route path="/firewall" element={<Navigate to="/firewall/rules" replace />} />
            <Route path="/firewall/rules" element={<FirewallRules />} />
            <Route path="/firewall/interfaces" element={<FirewallInterfaces />} />

            <Route path="/routing" element={<RoutingPage />} />

            <Route path="/identities" element={<IdentitiesPage />} />
            <Route path="/identities/sessions" element={<SessionsPage />} />
            <Route path="/identity/Accounting" element={<Accounting />} />

            <Route path="/users" element={<UsersPage />} />

            <Route path="/modules" element={<ModulesPage />} />
            <Route path="/services" element={<CoreServicesPage />} />

            <Route path="/console" element={<ConsolePage />} />
          </Route>
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}