import { useState, useEffect } from 'react';
import { Navbar } from './components/Navbar';
import { Dashboard } from './components/Dashboard';
import { DpiLab } from './components/DpiLab';
import { Diagnostics } from './components/Diagnostics';
import { Settings } from './components/Settings';
import { Profiles } from './components/Profiles';
import { ConnectionStatus, DpiSettings, HealthMetrics, OperatingMode } from './types';
import { invoke } from '@tauri-apps/api/core';
import { AlertTriangle, X } from 'lucide-react';

const isTauriEnv = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export function App() {
  const [currentTab, setCurrentTab] = useState('dashboard');
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [mode, setMode] = useState<OperatingMode>(() => {
    return (localStorage.getItem('netfreedom_mode') as OperatingMode) || 'auto_pilot';
  });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const [dpiSettings, setDpiSettings] = useState<DpiSettings>({
    split_pos: '2+s',
    fake_ttl: 3,
    disorder: false,
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

  // Persist mode choice
  useEffect(() => {
    localStorage.setItem('netfreedom_mode', mode);
  }, [mode]);

  // Initial status check & periodic sync
  useEffect(() => {
    const checkStatus = async () => {
      if (isTauriEnv) {
        try {
          const s = await invoke<string>('get_connection_status');
          setStatus(s.toLowerCase() === 'connected' ? 'connected' : 'disconnected');
          const m = await invoke<HealthMetrics>('get_health_metrics');
          if (m) setMetrics(m);
        } catch (e) {
          console.error('Tauri status check failed:', e);
        }
      } else {
        try {
          const res = await fetch('/api/status');
          if (res.ok) {
            const data = await res.json();
            setStatus(data.status === 'connected' ? 'connected' : 'disconnected');
          }
        } catch {
          // Fallback or running standalone
        }
      }
    };
    checkStatus();
  }, []);

  // Live throughput & latency sync when connected
  useEffect(() => {
    if (status !== 'connected') {
      setMetrics((prev) => ({
        ...prev,
        download_bytes_sec: 0,
        upload_bytes_sec: 0,
      }));
      return;
    }

    const interval = setInterval(async () => {
      if (isTauriEnv) {
        try {
          const m = await invoke<HealthMetrics>('get_health_metrics');
          if (m) setMetrics(m);
        } catch (e) {
          console.error('Failed to poll live metrics from backend:', e);
        }
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [status]);

  const handleToggleConnect = async () => {
    if (status === 'connected') {
      await handleDisconnect();
    } else {
      await handleConnectWithMode(mode);
    }
  };

  const handleConnectWithMode = async (targetMode: OperatingMode) => {
    setErrorMessage(null);
    setStatus('connecting');
    if (isTauriEnv) {
      try {
        await invoke('connect_profile', { mode: targetMode, dpiSettings });
        setStatus('connected');
        const m = await invoke<HealthMetrics>('get_health_metrics');
        if (m) setMetrics(m);
      } catch (err: any) {
        console.error('Failed to connect via Tauri', err);
        const errMsg = typeof err === 'string' ? err : err?.message || 'Connection failed';
        setErrorMessage(errMsg);
        setStatus('disconnected');
      }
    } else {
      try {
        const res = await fetch('/api/connect', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mode: targetMode, dpiSettings }),
        });
        if (res.ok) {
          setStatus('connected');
        } else {
          setErrorMessage('API connect failed with status: ' + res.status);
          setStatus('disconnected');
        }
      } catch (err: any) {
        console.error('Failed to connect via API', err);
        setErrorMessage('Cannot reach backend daemon.');
        setStatus('disconnected');
      }
    }
  };

  const handleDisconnect = async () => {
    setErrorMessage(null);
    if (isTauriEnv) {
      try {
        await invoke('disconnect_profile');
      } catch (err) {
        console.error('Failed to disconnect via Tauri', err);
      }
    } else {
      try {
        await fetch('/api/disconnect', { method: 'POST' });
      } catch (err) {
        console.error('Failed to disconnect via API', err);
      }
    }
    setStatus('disconnected');
  };

  const handleActivateProfile = (targetMode: OperatingMode) => {
    setMode(targetMode);
    handleConnectWithMode(targetMode);
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-background overflow-hidden">
      <Navbar
        currentTab={currentTab}
        setCurrentTab={setCurrentTab}
        status={status}
        latency={status === 'connected' ? metrics.tcp_latency_ms : null}
      />

      {/* User-Facing Error Alert Toast Banner */}
      {errorMessage && (
        <div className="bg-red-500/10 border-b border-red-500/30 px-4 py-2.5 flex items-center justify-between text-xs text-red-300">
          <div className="flex items-center space-x-2 overflow-hidden">
            <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
            <span className="font-semibold text-red-200">Connection Error:</span>
            <span className="truncate">{errorMessage}</span>
          </div>
          <button
            onClick={() => setErrorMessage(null)}
            className="p-1 rounded hover:bg-red-500/20 text-red-400 shrink-0 ml-2"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

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

        {currentTab === 'profiles' && (
          <Profiles
            currentMode={mode}
            isConnected={status === 'connected'}
            onActivateProfile={handleActivateProfile}
            onDisconnect={handleDisconnect}
          />
        )}

        {currentTab === 'dpi_lab' && (
          <DpiLab dpiSettings={dpiSettings} setDpiSettings={setDpiSettings} />
        )}

        {currentTab === 'diagnostics' && (
          <Diagnostics
            metrics={metrics}
            onRefreshMetrics={async () => {
              if (isTauriEnv) {
                try {
                  const m = await invoke<HealthMetrics>('get_health_metrics');
                  if (m) setMetrics(m);
                } catch (e) {
                  console.error('Diagnostic metric refresh failed:', e);
                }
              }
            }}
          />
        )}

        {currentTab === 'settings' && <Settings />}
      </main>
    </div>
  );
}

export default App;
