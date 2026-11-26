import React from 'react';
import { Layers, Plus, Shield, Cpu, Server, Globe, CheckCircle } from 'lucide-react';
import { OperatingMode } from '../types';

interface ProfilesProps {
  currentMode: OperatingMode;
  isConnected: boolean;
  onActivateProfile: (mode: OperatingMode) => void;
  onDisconnect: () => void;
}

export const Profiles: React.FC<ProfilesProps> = ({
  currentMode,
  isConnected,
  onActivateProfile,
  onDisconnect,
}) => {
  const profiles = [
    {
      id: 'stealth',
      name: 'Campus / Enterprise Wi-Fi Stealth (TCP 443)',
      modeKey: 'stealth_tunnel' as OperatingMode,
      modeLabel: 'Stealth Tunnel',
      desc: 'Encrypted TLS 1.3 tunnel over TCP Port 443 to bypass strict Fortinet/Cisco category blocks and UDP drop policies.',
      icon: Globe,
      color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
    },
    {
      id: 'byedpi',
      name: 'Campus Wi-Fi ByeDPI (Stateless Filter)',
      modeKey: 'dpi_desync' as OperatingMode,
      modeLabel: 'ByeDPI Desync',
      desc: 'Split-SNI (2+s) + Low TTL (3) to bypass basic ISP DPI with zero latency penalty.',
      icon: Cpu,
      color: 'text-teal-400 bg-teal-500/10 border-teal-500/20',
    },
    {
      id: 'autopilot',
      name: 'Smart Auto-Pilot (Automated Bypass)',
      modeKey: 'auto_pilot' as OperatingMode,
      modeLabel: 'Auto-Pilot',
      desc: 'Probes gateway latency, captive portals, and automatically selects optimal desync parameters.',
      icon: Globe,
      color: 'text-amber-400 bg-amber-500/10 border-amber-500/20',
    },
    {
      id: 'warp',
      name: 'Cloudflare WARP Privacy Shield',
      modeKey: 'warp_shield' as OperatingMode,
      modeLabel: 'WireGuard',
      desc: '1-Click full WireGuard encryption via Anycast port 2408 directly to Madras edge.',
      icon: Shield,
      color: 'text-blue-400 bg-blue-500/10 border-blue-500/20',
    },
    {
      id: 'hybrid',
      name: 'Hybrid Stealth Shield',
      modeKey: 'hybrid_stealth' as OperatingMode,
      modeLabel: 'Hybrid Mode',
      desc: 'Combined packet desynchronization with failover proxy interception.',
      icon: Server,
      color: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
    },
  ];

  return (
    <div className="flex-1 p-6 overflow-y-auto max-w-4xl mx-auto w-full">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-bold text-white flex items-center space-x-2">
            <Layers className="w-5 h-5 text-emerald-400" />
            <span>Connection Profiles</span>
          </h2>
          <p className="text-xs text-gray-400 mt-1">
            Manage, customize, and switch between saved connectivity profiles.
          </p>
        </div>

        <button className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-emerald-500 hover:bg-emerald-600 text-white text-xs font-semibold transition">
          <Plus className="w-3.5 h-3.5" />
          <span>New Profile</span>
        </button>
      </div>

      <div className="space-y-3">
        {profiles.map((p) => {
          const Icon = p.icon;
          const isActive = currentMode === p.modeKey && isConnected;

          return (
            <div
              key={p.id}
              className={`border rounded-xl p-4 flex items-center justify-between transition ${
                isActive
                  ? 'bg-emerald-950/20 border-emerald-500/40 shadow-[0_0_15px_rgba(16,185,129,0.15)]'
                  : 'bg-surface/60 border-gray-800 hover:bg-surface/80'
              }`}
            >
              <div className="flex items-start space-x-3.5">
                <div className={`p-2.5 rounded-lg border ${p.color}`}>
                  <Icon className="w-5 h-5" />
                </div>
                <div>
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-semibold text-white">{p.name}</span>
                    <span className="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
                      {p.modeLabel}
                    </span>
                    {isActive && (
                      <span className="flex items-center space-x-1 text-[10px] font-semibold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/30">
                        <CheckCircle className="w-3 h-3" />
                        <span>ACTIVE</span>
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-gray-400 mt-1">{p.desc}</p>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                {isActive ? (
                  <button
                    onClick={onDisconnect}
                    className="px-3.5 py-1.5 rounded-lg bg-red-500/10 border border-red-500/30 text-xs font-medium text-red-400 hover:bg-red-500/20 transition"
                  >
                    Disconnect
                  </button>
                ) : (
                  <button
                    onClick={() => onActivateProfile(p.modeKey)}
                    className="px-3.5 py-1.5 rounded-lg bg-emerald-500 hover:bg-emerald-600 text-white text-xs font-semibold transition"
                  >
                    Activate
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
