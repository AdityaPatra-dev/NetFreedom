import React from 'react';
import {
  Shield,
  Zap,
  Globe,
  Wifi,
  Server,
  ArrowDown,
  ArrowUp,
  Cpu,
} from 'lucide-react';
import { ConnectionStatus, HealthMetrics, OperatingMode } from '../types';

interface DashboardProps {
  status: ConnectionStatus;
  mode: OperatingMode;
  setMode: (mode: OperatingMode) => void;
  onToggleConnect: () => void;
  metrics: HealthMetrics;
}

export const Dashboard: React.FC<DashboardProps> = ({
  status,
  mode,
  setMode,
  onToggleConnect,
  metrics,
}) => {
  const isConnected = status === 'connected';
  const isConnecting = status === 'connecting';

  return (
    <div className="flex-1 p-6 flex flex-col justify-between overflow-y-auto max-w-5xl mx-auto w-full">
      {/* Mode Selector Carousel */}
      <div className="grid grid-cols-5 gap-2.5 mb-6">
        {[
          { id: 'auto_pilot', label: 'Auto-Pilot', desc: '1-Click Adaptive', icon: Zap },
          { id: 'warp_shield', label: 'WARP Shield', desc: 'WireGuard (Port 2408)', icon: Shield },
          { id: 'dpi_desync', label: 'ByeDPI Desync', desc: 'Serverless 0ms Latency', icon: Cpu },
          { id: 'custom_vpn', label: 'Custom VPN', desc: 'WireGuard / .conf', icon: Server },
          { id: 'modular_proxy', label: 'SOCKS5 Proxy', desc: 'Modular Proxy', icon: Globe },
        ].map((item) => {
          const Icon = item.icon;
          const active = mode === item.id;
          return (
            <button
              key={item.id}
              onClick={() => setMode(item.id as OperatingMode)}
              className={`p-3 rounded-xl border text-left transition ${
                active
                  ? 'bg-emerald-500/10 border-emerald-500/50 text-white shadow-lg shadow-emerald-500/5'
                  : 'bg-surface/50 border-gray-800 text-gray-400 hover:bg-surface hover:text-gray-200'
              }`}
            >
              <div className="flex items-center space-x-2 mb-1">
                <Icon className={`w-4 h-4 ${active ? 'text-emerald-400' : 'text-gray-400'}`} />
                <span className="font-semibold text-xs text-white">{item.label}</span>
              </div>
              <p className="text-[10px] text-gray-400 leading-tight">{item.desc}</p>
            </button>
          );
        })}
      </div>

      {/* Giant Connect Button & State Display */}
      <div className="flex flex-col items-center justify-center my-6">
        <div className="relative">
          {isConnected && (
            <div className="absolute inset-0 rounded-full bg-emerald-500/20 blur-xl animate-pulse" />
          )}

          <button
            onClick={onToggleConnect}
            disabled={isConnecting}
            className={`relative w-44 h-44 rounded-full flex flex-col items-center justify-center transition-all duration-300 border-2 ${
              isConnected
                ? 'bg-emerald-950/40 border-emerald-500 text-emerald-400 hover:bg-emerald-900/40 shadow-[0_0_40px_rgba(16,185,129,0.3)]'
                : isConnecting
                ? 'bg-amber-950/40 border-amber-500 text-amber-400 animate-pulse'
                : 'bg-surface border-gray-700 text-gray-400 hover:border-gray-500 hover:text-white'
            }`}
          >
            <Shield className={`w-14 h-14 mb-2 ${isConnected ? 'text-emerald-400' : 'text-gray-400'}`} />
            <span className="text-sm font-bold tracking-wider uppercase">
              {isConnected ? 'Disconnect' : isConnecting ? 'Connecting' : 'Connect'}
            </span>
            <span className="text-[11px] text-gray-400 mt-1 capitalize">
              {isConnected ? 'Protected' : 'Unprotected'}
            </span>
          </button>
        </div>

        <div className="mt-4 text-center">
          <div className="text-xs font-mono text-gray-400">
            {isConnected ? 'Active Protocol: WireGuard ChaCha20-Poly1305' : 'Select a mode and click connect'}
          </div>
        </div>
      </div>

      {/* Metrics & Gateway Telemetry */}
      <div className="grid grid-cols-4 gap-4 mt-6">
        {/* Download Speed */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-3.5 flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400">
            <ArrowDown className="w-5 h-5" />
          </div>
          <div>
            <div className="text-[11px] text-gray-400">Download</div>
            <div className="text-base font-bold font-mono text-white">
              {(metrics.download_bytes_sec / (1024 * 1024)).toFixed(1)}{' '}
              <span className="text-xs font-normal text-gray-400">MB/s</span>
            </div>
          </div>
        </div>

        {/* Upload Speed */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-3.5 flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-blue-500/10 text-blue-400">
            <ArrowUp className="w-5 h-5" />
          </div>
          <div>
            <div className="text-[11px] text-gray-400">Upload</div>
            <div className="text-base font-bold font-mono text-white">
              {(metrics.upload_bytes_sec / (1024 * 1024)).toFixed(1)}{' '}
              <span className="text-xs font-normal text-gray-400">MB/s</span>
            </div>
          </div>
        </div>

        {/* Public IP */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-3.5 flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-purple-500/10 text-purple-400">
            <Globe className="w-5 h-5" />
          </div>
          <div className="overflow-hidden">
            <div className="text-[11px] text-gray-400">Public IP</div>
            <div className="text-sm font-bold font-mono text-white truncate">
              {metrics.public_ip || '123.63.59.157'}
            </div>
          </div>
        </div>

        {/* Datacenter / Location */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-3.5 flex items-center space-x-3">
          <div className="p-2 rounded-lg bg-amber-500/10 text-amber-400">
            <Wifi className="w-5 h-5" />
          </div>
          <div>
            <div className="text-[11px] text-gray-400">Edge Gateway</div>
            <div className="text-sm font-bold text-white flex items-center space-x-1.5">
              <span>{metrics.country || 'India'}</span>
              <span className="text-xs font-mono text-emerald-400">
                ({metrics.colo || 'MAA'})
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
