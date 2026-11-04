import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { Layout } from './components/Layout'
import { RequireAuth } from './components/RequireAuth'
import { Login } from './pages/Login'
import { ChangePassword } from './pages/ChangePassword'
import { Dashboard } from './pages/Dashboard'

import { NetworkInterfaces } from './pages/network/Interfaces'
import { NetworkZones } from './pages/network/Zones'
import { FirewallRules } from './pages/Firewall/Rules'
import { FirewallSets } from './pages/Firewall/Sets'
import { FirewallFqdn } from './pages/Firewall/Fqdn'
import { FirewallConnections } from './pages/Firewall/Connections'
import { FirewallProtection } from './pages/Firewall/Protection'
import { FirewallEvents } from './pages/Firewall/Events'
import { RoutingPage } from './pages/routing/Index'
import { IdentitiesPage } from './pages/Identity/Identities'
import { SessionsPage } from './pages/Identity/Sessions'
import { UsersPage } from './pages/system/Users'
import { ModulesPage } from './pages/services/Modules'
import { CoreServicesPage } from './pages/services/CoreServices'
import { ConsolePage } from './pages/diagnostics/Console'
import Accounting from './pages/Identity/Accounting'
import { FirewallNat } from './pages/Firewall/Nat'

import './styles/global.css'
import './components/Layout.css'

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/change-password" element={<ChangePassword />} />

        <Route element={<RequireAuth />}>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />

            <Route path="/network/interfaces" element={<NetworkInterfaces />} />
            <Route path="/network/zones" element={<NetworkZones />} />

            <Route path="/firewall" element={<Navigate to="/firewall/rules" replace />} />
            <Route path="/firewall/rules" element={<FirewallRules />} />
            <Route path="/firewall/nat" element={<FirewallNat />} />
            <Route path="/firewall/sets" element={<FirewallSets />} />
            <Route path="/firewall/fqdn" element={<FirewallFqdn />} />
            <Route path="/firewall/connections" element={<FirewallConnections />} />
            <Route path="/firewall/protection" element={<FirewallProtection />} />
            <Route path="/firewall/events" element={<FirewallEvents />} />

            <Route path="/routing" element={<RoutingPage />} />

            <Route path="/identities" element={<IdentitiesPage />} />
            <Route path="/identities/sessions" element={<SessionsPage />} />
            <Route path="/identity/accounting" element={<Accounting />} />

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