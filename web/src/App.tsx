import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { Layout } from './components/Layout'
import { RequireAuth } from './components/RequireAuth'
import { Dashboard } from './pages/Dashboard'
import { Modules } from './pages/Modules'
import { Services } from './pages/Services'
import { Console } from './pages/Console'
import { Firewall } from './pages/Firewall'
import { Login } from './pages/Login'
import { Users } from './pages/Users'
import { Identities } from './pages/Identities'
import Accounting from './pages/Accounting'

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
            <Route path="/users" element={<Users />} />
            <Route path="/identities" element={<Identities />} />
            <Route path="/modules" element={<Modules />} />
            <Route path="/services" element={<Services />} />
            <Route path="/console" element={<Console />} />
            <Route path="/accounting" element={<RequireAuth><Accounting /></RequireAuth>} />
            <Route path="/firewall" element={<Firewall />} />
          </Route>
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}