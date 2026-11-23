import React, { useState } from 'react';
import { Settings as SettingsIcon, ShieldAlert, Globe, Network } from 'lucide-react';

export const Settings: React.FC = () => {
  const [killSwitchStrict, setKillSwitchStrict] = useState(false);
  const [allowLan, setAllowLan] = useState(true);
  const [dnsPreset, setDnsPreset] = useState('cloudflare');
  const [mtu, setMtu] = useState(1420);

  return (
    <div className="flex-1 p-6 overflow-y-auto max-w-4xl mx-auto w-full space-y-6">
      <div>
        <h2 className="text-lg font-bold text-white flex items-center space-x-2">
          <SettingsIcon className="w-5 h-5 text-emerald-400" />
          <span>Security & System Settings</span>
        </h2>
        <p className="text-xs text-gray-400 mt-1">
          Configure firewall kill switch rules, encrypted DNS providers, and packet parameters.
        </p>
      </div>

      <div className="space-y-4">
        {/* Kill Switch Card */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-5">
          <div className="flex items-center space-x-2 mb-3">
            <ShieldAlert className="w-4 h-4 text-emerald-400" />
            <h3 className="text-sm font-semibold text-white">Kernel nftables Kill Switch</h3>
          </div>

          <div className="space-y-3">
            <label className="flex items-center justify-between p-3 rounded-lg border border-gray-800 bg-surface/30 cursor-pointer hover:bg-surface/60 transition">
              <div>
                <span className="text-xs font-semibold text-gray-200 block">Strict Kill Switch</span>
                <span className="text-[10px] text-gray-400 block">
                  Blocks all egress traffic immediately if tunnel interface drops unexpectedly.
                </span>
              </div>
              <input
                type="checkbox"
                checked={killSwitchStrict}
                onChange={(e) => setKillSwitchStrict(e.target.checked)}
                className="w-4 h-4 rounded text-emerald-500 accent-emerald-500"
              />
            </label>

            <label className="flex items-center justify-between p-3 rounded-lg border border-gray-800 bg-surface/30 cursor-pointer hover:bg-surface/60 transition">
              <div>
                <span className="text-xs font-semibold text-gray-200 block">Permit Local Subnet (LAN)</span>
                <span className="text-[10px] text-gray-400 block">
                  Preserves access to local printers, Samba shares, and LAN devices on 10.6.80.0/20.
                </span>
              </div>
              <input
                type="checkbox"
                checked={allowLan}
                onChange={(e) => setAllowLan(e.target.checked)}
                className="w-4 h-4 rounded text-emerald-500 accent-emerald-500"
              />
            </label>
          </div>
        </div>

        {/* Encrypted DNS Provider */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-5">
          <div className="flex items-center space-x-2 mb-3">
            <Globe className="w-4 h-4 text-emerald-400" />
            <h3 className="text-sm font-semibold text-white">Encrypted DNS Resolver (DoH)</h3>
          </div>

          <div className="grid grid-cols-4 gap-2">
            {[
              { id: 'cloudflare', name: 'Cloudflare', ip: '1.1.1.1' },
              { id: 'quad9', name: 'Quad9 Security', ip: '9.9.9.9' },
              { id: 'adguard', name: 'AdGuard AdBlock', ip: '94.140.14.14' },
              { id: 'google', name: 'Google DNS', ip: '8.8.8.8' },
            ].map((p) => (
              <button
                key={p.id}
                onClick={() => setDnsPreset(p.id)}
                className={`p-3 rounded-lg border text-left transition ${
                  dnsPreset === p.id
                    ? 'bg-emerald-500/10 border-emerald-500 text-emerald-400 font-bold'
                    : 'bg-surface border-gray-800 text-gray-400 hover:bg-surface-hover hover:text-gray-200'
                }`}
              >
                <div className="text-xs font-semibold text-white">{p.name}</div>
                <div className="text-[10px] font-mono text-gray-400 mt-0.5">{p.ip}</div>
              </button>
            ))}
          </div>
        </div>

        {/* MTU Slider */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-5">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center space-x-2">
              <Network className="w-4 h-4 text-emerald-400" />
              <h3 className="text-sm font-semibold text-white">Tunnel Interface MTU</h3>
            </div>
            <span className="text-xs font-mono text-emerald-400">{mtu} bytes</span>
          </div>

          <input
            type="range"
            min="1280"
            max="1500"
            value={mtu}
            onChange={(e) => setMtu(parseInt(e.target.value))}
            className="w-full accent-emerald-500 h-1.5 bg-gray-700 rounded-lg cursor-pointer"
          />
          <div className="flex justify-between text-[10px] text-gray-500 mt-1">
            <span>1280 (IPv6 Minimum)</span>
            <span>1420 (WireGuard Standard)</span>
            <span>1500 (Ethernet Frame)</span>
          </div>
        </div>
      </div>
    </div>
  );
};
