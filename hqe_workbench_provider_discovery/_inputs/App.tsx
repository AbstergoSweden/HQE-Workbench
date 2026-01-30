import { FC } from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { WelcomeScreen } from './screens/WelcomeScreen'
import { ScanScreen } from './screens/ScanScreen'
import { ReportScreen } from './screens/ReportScreen'
import { SettingsScreen } from './screens/SettingsScreen'

const App: FC = () => {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<WelcomeScreen />} />
          <Route path="/scan" element={<ScanScreen />} />
          <Route path="/report" element={<ReportScreen />} />
          <Route path="/settings" element={<SettingsScreen />} />
        </Routes>
      </Layout>
    </Router>
  )
}

export default App
