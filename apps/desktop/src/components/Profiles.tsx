import React from 'react';
import { Layers, Plus, Shield, Cpu, Server, Globe } from 'lucide-react';

export const Profiles: React.FC = () => {
  const profiles = [
    {
      id: '1',
      name: 'Campus Wi-Fi Stealth (Recommended)',
      mode: 'ByeDPI Desync',
      desc: 'Split-SNI (2+s) + Low TTL (3) to bypass campus Cisco DPI with zero latency penalty.',
      icon: Cpu,
      color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
    },
    {
      id: '2',
      name: 'Cloudflare WARP Privacy Shield',
      mode: 'WireGuard',
      desc: '1-Click full WireGuard encryption via Anycast port 2408 directly to Madras edge.',
      icon: Shield,
      color: 'text-blue-400 bg-blue-500/10 border-blue-500/20',
    },
    {
      id: '3',
      name: 'Personal VPS WireGuard',
      mode: 'Custom VPN',
      desc: 'Mullvad / Proton / Custom VPS tunnel config with ChaCha20 encryption.',
      icon: Server,
      color: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
    },
    {
      id: '4',
      name: 'SOCKS5 Proxy Relay',
      mode: 'Modular Proxy',
      desc: 'Routes applications selectively through an authenticated upstream SOCKS5 node.',
      icon: Globe,
      color: 'text-amber-400 bg-amber-500/10 border-amber-500/20',
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
          return (
            <div
              key={p.id}
              className="bg-surface/60 border border-gray-800 rounded-xl p-4 flex items-center justify-between hover:bg-surface/80 transition"
            >
              <div className="flex items-start space-x-3.5">
                <div className={`p-2.5 rounded-lg border ${p.color}`}>
                  <Icon className="w-5 h-5" />
                </div>
                <div>
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-semibold text-white">{p.name}</span>
                    <span className="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
                      {p.mode}
                    </span>
                  </div>
                  <p className="text-xs text-gray-400 mt-1">{p.desc}</p>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                <button className="px-3 py-1.5 rounded-lg bg-surface border border-gray-700 text-xs text-gray-200 hover:text-white hover:bg-surface-hover transition">
                  Activate
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
