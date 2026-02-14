import { Routes, Route, Navigate } from 'react-router-dom';
import { AppLayout } from '@/components/layout/AppLayout';
import { ClaudeConfigPage } from '@/pages/ClaudeConfigPage';
import { ZenMuxQuotaPage } from '@/pages/ZenMuxQuotaPage';

function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/claude-config" element={<ClaudeConfigPage />} />
        <Route path="/zenmux-quota" element={<ZenMuxQuotaPage />} />
        <Route path="*" element={<Navigate to="/claude-config" replace />} />
      </Route>
    </Routes>
  );
}

export default App;
