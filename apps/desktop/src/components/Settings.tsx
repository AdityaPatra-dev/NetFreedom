import React, { useState, useEffect } from 'react';
import { Settings as SettingsIcon, ShieldAlert, Globe, Network, Server, Save, CheckCircle2, RotateCcw } from 'lucide-react';
import { UserSettings, StealthNodeConfig } from '../types';
import { invoke } from '@tauri-apps/api/core';

const isTauriEnv = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const Settings: React.FC = () => {
  const [killSwitchStrict, setKillSwitchStrict] = useState(false);
  const [allowLan, setAllowLan] = useState(true);
  const [dnsPreset, setDnsPreset] = useState('cloudflare');
  const [mtu, setMtu] = useState(1420);

  // Stealth Tunnel Node Config
  const [workerHost, setWorkerHost] = useState('');
  const [workerUuid, setWorkerUuid] = useState('d342d110-d4a3-45c3-b0a3-3b11815b80b2');
  const [cdnIp, setCdnIp] = useState('104.16.132.229');
  const [wsPath, setWsPath] = useState('/?ed=2048');

  const [saving, setSaving] = useState(false);
  const [savedSuccess, setSavedSuccess] = useState(false);

  useEffect(() => {
    const loadSettings = async () => {
      if (isTauriEnv) {
        try {
          const s = await invoke<UserSettings>('get_user_settings');
          if (s) {
            setKillSwitchStrict(s.kill_switch_strict);
            setAllowLan(s.allow_lan);
            setDnsPreset(s.dns_preset || 'cloudflare');
            setMtu(s.mtu || 1420);
            if (s.stealth_node) {
              setWorkerHost(s.stealth_node.host || '');
              setWorkerUuid(s.stealth_node.uuid || 'd342d110-d4a3-45c3-b0a3-3b11815b80b2');
              setCdnIp(s.stealth_node.server || '104.16.132.229');
              setWsPath(s.stealth_node.path || '/?ed=2048');
            }
          }
        } catch (e) {
          console.error('Failed to load user settings:', e);
        }
      } else {
        const cached = localStorage.getItem('netfreedom_user_settings');
        if (cached) {
          try {
            const s = JSON.parse(cached);
            setKillSwitchStrict(s.kill_switch_strict);
            setAllowLan(s.allow_lan);
            setDnsPreset(s.dns_preset);
            setMtu(s.mtu);
            if (s.stealth_node) {
              setWorkerHost(s.stealth_node.host);
              setWorkerUuid(s.stealth_node.uuid);
              setCdnIp(s.stealth_node.server);
              setWsPath(s.stealth_node.path);
            }
          } catch {}
        }
      }
    };
    loadSettings();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setSavedSuccess(false);

    const stealth_node: StealthNodeConfig = {
      server: cdnIp.trim() || '104.16.132.229',
      server_port: 443,
      uuid: workerUuid.trim() || 'd342d110-d4a3-45c3-b0a3-3b11815b80b2',
      host: workerHost.trim() || 'cloudflare.com',
      path: wsPath.trim() || '/?ed=2048',
    };

    const payload: UserSettings = {
      kill_switch_strict: killSwitchStrict,
      allow_lan: allowLan,
      dns_preset: dnsPreset,
      mtu: mtu,
      stealth_node: stealth_node,
    };

    try {
      if (isTauriEnv) {
        await invoke('save_user_settings', { settings: payload });
        await invoke('save_stealth_node', { node: stealth_node });
      } else {
        localStorage.setItem('netfreedom_user_settings', JSON.stringify(payload));
      }
      setSavedSuccess(true);
      setTimeout(() => setSavedSuccess(false), 3000);
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      setSaving(false);
    }
  };

  const generateRandomUuid = () => {
    const u = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    setWorkerUuid(u);
  };

  return (
    <div className="flex-1 p-6 overflow-y-auto max-w-4xl mx-auto w-full space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold text-white flex items-center space-x-2">
            <SettingsIcon className="w-5 h-5 text-emerald-400" />
            <span>Security & System Settings</span>
          </h2>
          <p className="text-xs text-gray-400 mt-1">
            Configure kernel firewall kill switch rules, encrypted DNS resolvers, and Stealth Tunnel relay parameters.
          </p>
        </div>

        <button
          onClick={handleSave}
          disabled={saving}
          className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-emerald-500 hover:bg-emerald-600 text-white text-xs font-semibold shadow-md transition disabled:opacity-50"
        >
          {savedSuccess ? (
            <>
              <CheckCircle2 className="w-4 h-4 text-white" />
              <span>Saved!</span>
            </>
          ) : (
            <>
              <Save className="w-4 h-4 text-white" />
              <span>{saving ? 'Saving...' : 'Save Settings'}</span>
            </>
          )}
        </button>
      </div>

      <div className="space-y-4">
        {/* Stealth Tunnel Worker Relay Card */}
        <div className="bg-surface/60 border border-emerald-500/30 rounded-xl p-5 space-y-4 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Server className="w-4 h-4 text-emerald-400" />
              <h3 className="text-sm font-semibold text-white">
                Stealth Tunnel Relay (Cloudflare Worker / VLESS)
              </h3>
            </div>
            <span className="text-[10px] font-mono bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20">
              TCP Port 443 Encrypted
            </span>
          </div>

          <p className="text-xs text-gray-400">
            For campuses (like KIIT Fortinet) that drop UDP and enforce deep TCP packet reassembly, deploy the free script in <code className="text-emerald-300">deploy/cloudflare-worker/</code> and enter your Worker details below.
          </p>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-xs font-semibold text-gray-300 block mb-1">
                Cloudflare Worker Hostname / Domain
              </label>
              <input
                type="text"
                placeholder="my-tunnel.workers.dev"
                value={workerHost}
                onChange={(e) => setWorkerHost(e.target.value)}
                className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
              />
              <span className="text-[10px] text-gray-500 block mt-1">
                Your deployed Cloudflare Worker domain (without https://)
              </span>
            </div>

            <div>
              <div className="flex justify-between items-center mb-1">
                <label className="text-xs font-semibold text-gray-300">
                  User ID (UUIDv4)
                </label>
                <button
                  type="button"
                  onClick={generateRandomUuid}
                  className="text-[10px] text-emerald-400 hover:underline flex items-center space-x-1"
                >
                  <RotateCcw className="w-3 h-3" />
                  <span>Generate New</span>
                </button>
              </div>
              <input
                type="text"
                value={workerUuid}
                onChange={(e) => setWorkerUuid(e.target.value)}
                className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
              />
              <span className="text-[10px] text-gray-500 block mt-1">
                Must match the UUID configured in your Cloudflare Worker script
              </span>
            </div>

            <div>
              <label className="text-xs font-semibold text-gray-300 block mb-1">
                Clean CDN Anycast IP
              </label>
              <input
                type="text"
                value={cdnIp}
                onChange={(e) => setCdnIp(e.target.value)}
                className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
              />
              <span className="text-[10px] text-gray-500 block mt-1">
                Direct Cloudflare Anycast IP (default: 104.16.132.229)
              </span>
            </div>

            <div>
              <label className="text-xs font-semibold text-gray-300 block mb-1">
                WebSocket Path
              </label>
              <input
                type="text"
                value={wsPath}
                onChange={(e) => setWsPath(e.target.value)}
                className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
              />
              <span className="text-[10px] text-gray-500 block mt-1">
                Path for WebSocket upgrade with 2048-byte early data support
              </span>
            </div>
          </div>
        </div>

        {/* Kill Switch Card */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-5">
          <div className="flex items-center space-x-2 mb-3">
            <ShieldAlert className="w-4 h-4 text-emerald-400" />
            <h3 className="text-sm font-semibold text-white">Kernel & Firewall Kill Switch</h3>
          </div>

          <div className="space-y-3">
            <label className="flex items-center justify-between p-3 rounded-lg border border-gray-800 bg-surface/30 cursor-pointer hover:bg-surface/60 transition">
              <div>
                <span className="text-xs font-semibold text-gray-200 block">Strict Kill Switch</span>
                <span className="text-[10px] text-gray-400 block">
                  Enforces Windows Filtering Platform (WFP) or Linux nftables rules to block all traffic if the tunnel drops.
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
                  Preserves access to local printers, Wi-Fi login gateways, and LAN devices on 10.0.0.0/8 and 192.168.0.0/16.
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
            <h3 className="text-sm font-semibold text-white">Encrypted DNS Resolver (DoH / NRPT)</h3>
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
