import { FC } from 'react'
import { HashRouter as Router, Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { ToastProvider } from './context/ToastContext'
import { WelcomeScreen } from './screens/WelcomeScreen'
import { ScanScreen } from './screens/ScanScreen'
import { ReportScreen } from './screens/ReportScreen'
import { SettingsScreen } from './screens/SettingsScreen'
import { ThinktankScreen } from './screens/ThinktankScreen'

const App: FC = () => {
  return (
    <Router>
      <ToastProvider>
        <Layout>
          <Routes>
            <Route path="/" element={<WelcomeScreen />} />
            <Route path="/scan" element={<ScanScreen />} />
            <Route path="/thinktank" element={<ThinktankScreen />} />
            <Route path="/report" element={<ReportScreen />} />
            <Route path="/settings" element={<SettingsScreen />} />
          </Routes>
        </Layout>
      </ToastProvider>
    </Router>
  )
}

export default App
