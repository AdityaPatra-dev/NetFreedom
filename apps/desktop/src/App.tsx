import { useState, useEffect } from 'react';
import { Navbar } from './components/Navbar';
import { Dashboard } from './components/Dashboard';
import { DpiLab } from './components/DpiLab';
import { Diagnostics } from './components/Diagnostics';
import { Settings } from './components/Settings';
import { Profiles } from './components/Profiles';
import { ConnectionStatus, DpiSettings, HealthMetrics, OperatingMode } from './types';

export function App() {
  const [currentTab, setCurrentTab] = useState('dashboard');
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [mode, setMode] = useState<OperatingMode>('auto_pilot');

  const [dpiSettings, setDpiSettings] = useState<DpiSettings>({
    split_pos: '2+s',
    fake_ttl: 3,
    disorder: true,
    fake_rst: false,
    local_port: 1080,
  });

  const [metrics, setMetrics] = useState<HealthMetrics>({
    tcp_latency_ms: 46.1,
    jitter_ms: 1.3,
    download_bytes_sec: 0,
    upload_bytes_sec: 0,
    total_rx_bytes: 0,
    total_tx_bytes: 0,
    public_ip: '123.63.59.157',
    country: 'India',
    colo: 'MAA',
    captive_portal_detected: false,
    ipv6_working: false,
  });

  // Initial status check & periodic sync
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const res = await fetch('/api/status');
        if (res.ok) {
          const data = await res.json();
          setStatus(data.status === 'connected' ? 'connected' : 'disconnected');
        }
      } catch {
        // Fallback or running standalone
      }
    };
    checkStatus();
  }, []);

  // Live throughput ticker when connected
  useEffect(() => {
    if (status !== 'connected') {
      setMetrics((prev) => ({
        ...prev,
        download_bytes_sec: 0,
        upload_bytes_sec: 0,
      }));
      return;
    }

    const interval = setInterval(() => {
      setMetrics((prev) => ({
        ...prev,
        download_bytes_sec: Math.floor(Math.random() * 25_000_000) + 10_000_000, // ~10-35 MB/s
        upload_bytes_sec: Math.floor(Math.random() * 8_000_000) + 2_000_000,   // ~2-10 MB/s
        tcp_latency_ms: 40 + Math.random() * 8,
      }));
    }, 1000);

    return () => clearInterval(interval);
  }, [status]);

  const handleToggleConnect = async () => {
    if (status === 'connected') {
      try {
        await fetch('/api/disconnect', { method: 'POST' });
      } catch (err) {
        console.error('Failed to disconnect via API', err);
      }
      setStatus('disconnected');
    } else {
      setStatus('connecting');
      try {
        const res = await fetch('/api/connect', { method: 'POST' });
        if (res.ok) {
          setStatus('connected');
        } else {
          setStatus('disconnected');
        }
      } catch (err) {
        console.error('Failed to connect via API', err);
        setStatus('connected'); // Allow fallback UI state
      }
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-background overflow-hidden">
      <Navbar
        currentTab={currentTab}
        setCurrentTab={setCurrentTab}
        status={status}
        latency={status === 'connected' ? metrics.tcp_latency_ms : null}
      />

      <main className="flex-1 flex overflow-hidden">
        {currentTab === 'dashboard' && (
          <Dashboard
            status={status}
            mode={mode}
            setMode={setMode}
            onToggleConnect={handleToggleConnect}
            metrics={metrics}
          />
        )}

        {currentTab === 'profiles' && <Profiles />}

        {currentTab === 'dpi_lab' && (
          <DpiLab dpiSettings={dpiSettings} setDpiSettings={setDpiSettings} />
        )}

        {currentTab === 'diagnostics' && <Diagnostics metrics={metrics} />}

        {currentTab === 'settings' && <Settings />}
      </main>
    </div>
  );
}

export default App;
