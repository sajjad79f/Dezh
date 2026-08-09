import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { Dashboard } from './pages/Dashboard'
import { Modules } from './pages/Modules'
import { Services } from './pages/Services'
import { Console } from './pages/Console'
import { Firewall } from './pages/Firewall'
import './styles/global.css'
import './components/Layout.css'

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/modules" element={<Modules />} />
          <Route path="/services" element={<Services />} />
          <Route path="/console" element={<Console />} />
          <Route path="/Firewall" element={<Firewall />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}