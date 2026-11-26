import React, { useState, useEffect } from 'react';
import { Layers, Plus, Shield, Cpu, Server, Globe, CheckCircle, Settings as SettingsIcon, X, Save, CheckCircle2 } from 'lucide-react';
import { OperatingMode, StealthNodeConfig } from '../types';
import { invoke } from '@tauri-apps/api/core';

const isTauriEnv = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

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
  const [modalOpen, setModalOpen] = useState(false);
  const [workerHost, setWorkerHost] = useState('');
  const [workerUuid, setWorkerUuid] = useState('d342d110-d4a3-45c3-b0a3-3b11815b80b2');
  const [cdnIp, setCdnIp] = useState('104.16.132.229');
  const [wsPath, setWsPath] = useState('/?ed=2048');
  const [savingNode, setSavingNode] = useState(false);
  const [nodeSaved, setNodeSaved] = useState(false);

  useEffect(() => {
    const fetchStealthNode = async () => {
      if (isTauriEnv) {
        try {
          const node = await invoke<StealthNodeConfig>('get_stealth_node');
          if (node) {
            setWorkerHost(node.host || '');
            setWorkerUuid(node.uuid || 'd342d110-d4a3-45c3-b0a3-3b11815b80b2');
            setCdnIp(node.server || '104.16.132.229');
            setWsPath(node.path || '/?ed=2048');
          }
        } catch (e) {
          console.error('Failed to get stealth node config:', e);
        }
      }
    };
    fetchStealthNode();
  }, []);

  const handleSaveNode = async () => {
    setSavingNode(true);
    setNodeSaved(false);
    const node: StealthNodeConfig = {
      server: cdnIp.trim() || '104.16.132.229',
      server_port: 443,
      uuid: workerUuid.trim() || 'd342d110-d4a3-45c3-b0a3-3b11815b80b2',
      host: workerHost.trim() || 'cloudflare.com',
      path: wsPath.trim() || '/?ed=2048',
    };

    try {
      if (isTauriEnv) {
        await invoke('save_stealth_node', { node });
      } else {
        localStorage.setItem('netfreedom_stealth_node', JSON.stringify(node));
      }
      setNodeSaved(true);
      setTimeout(() => {
        setNodeSaved(false);
        setModalOpen(false);
      }, 1200);
    } catch (e) {
      console.error('Failed to save stealth node:', e);
    } finally {
      setSavingNode(false);
    }
  };

  const profiles = [
    {
      id: 'stealth',
      name: 'Campus / Enterprise Wi-Fi Stealth (TCP 443)',
      modeKey: 'stealth_tunnel' as OperatingMode,
      modeLabel: 'Stealth Tunnel',
      desc: 'Encrypted TLS 1.3 tunnel over TCP Port 443 via Cloudflare Worker relay. Bypasses strict Fortinet/Cisco category blocks and UDP drop policies.',
      icon: Globe,
      color: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
      configurable: true,
    },
    {
      id: 'byedpi',
      name: 'Campus Wi-Fi ByeDPI (Stateless Filter)',
      modeKey: 'dpi_desync' as OperatingMode,
      modeLabel: 'ByeDPI Desync',
      desc: 'Split-SNI (2+s) + Low TTL (3) to bypass basic ISP DPI with zero latency penalty.',
      icon: Cpu,
      color: 'text-teal-400 bg-teal-500/10 border-teal-500/20',
      configurable: false,
    },
    {
      id: 'autopilot',
      name: 'Smart Auto-Pilot (Automated Bypass)',
      modeKey: 'auto_pilot' as OperatingMode,
      modeLabel: 'Auto-Pilot',
      desc: 'Probes gateway latency, captive portals, and automatically selects optimal desync parameters.',
      icon: Globe,
      color: 'text-amber-400 bg-amber-500/10 border-amber-500/20',
      configurable: false,
    },
    {
      id: 'warp',
      name: 'Cloudflare WARP Privacy Shield',
      modeKey: 'warp_shield' as OperatingMode,
      modeLabel: 'WireGuard',
      desc: '1-Click full WireGuard encryption via Anycast port 2408 directly to Madras edge.',
      icon: Shield,
      color: 'text-blue-400 bg-blue-500/10 border-blue-500/20',
      configurable: false,
    },
    {
      id: 'hybrid',
      name: 'Hybrid Stealth Shield',
      modeKey: 'hybrid_stealth' as OperatingMode,
      modeLabel: 'Hybrid Mode',
      desc: 'Combined packet desynchronization with failover proxy interception.',
      icon: Server,
      color: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
      configurable: false,
    },
  ];

  return (
    <div className="flex-1 p-6 overflow-y-auto max-w-4xl mx-auto w-full relative">
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

        <button
          onClick={() => setModalOpen(true)}
          className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-emerald-500 hover:bg-emerald-600 text-white text-xs font-semibold transition"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Configure Stealth Relay</span>
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
                {p.configurable && (
                  <button
                    onClick={() => setModalOpen(true)}
                    className="p-1.5 rounded-lg bg-surface border border-gray-700 text-gray-400 hover:text-white hover:bg-surface-hover transition text-xs flex items-center space-x-1"
                    title="Configure Node Relay"
                  >
                    <SettingsIcon className="w-4 h-4" />
                    <span className="text-[11px] pr-1">Config</span>
                  </button>
                )}

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

      {/* Stealth Node Configuration Modal */}
      {modalOpen && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-[#12161f] border border-gray-700 rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl animate-in fade-in duration-200">
            <div className="flex items-center justify-between pb-2 border-b border-gray-800">
              <div className="flex items-center space-x-2">
                <Globe className="w-5 h-5 text-emerald-400" />
                <h3 className="text-sm font-bold text-white">Stealth Tunnel Relay Configuration</h3>
              </div>
              <button
                onClick={() => setModalOpen(false)}
                className="text-gray-400 hover:text-white p-1 rounded-lg"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <p className="text-xs text-gray-400">
              Deploy your Cloudflare Worker script from <code className="text-emerald-300">deploy/cloudflare-worker/vless-worker.js</code> and enter your worker details below:
            </p>

            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-gray-300 block mb-1">
                  Cloudflare Worker Hostname / Domain
                </label>
                <input
                  type="text"
                  placeholder="my-tunnel.workers.dev"
                  value={workerHost}
                  onChange={(e) => setWorkerHost(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
                />
              </div>

              <div>
                <label className="text-xs font-medium text-gray-300 block mb-1">
                  User ID (UUIDv4)
                </label>
                <input
                  type="text"
                  value={workerUuid}
                  onChange={(e) => setWorkerUuid(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-medium text-gray-300 block mb-1">
                    Clean Anycast IP
                  </label>
                  <input
                    type="text"
                    value={cdnIp}
                    onChange={(e) => setCdnIp(e.target.value)}
                    className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-300 block mb-1">
                    WebSocket Path
                  </label>
                  <input
                    type="text"
                    value={wsPath}
                    onChange={(e) => setWsPath(e.target.value)}
                    className="w-full px-3 py-2 rounded-lg bg-surface border border-gray-700 text-xs text-white focus:outline-none focus:border-emerald-500 font-mono"
                  />
                </div>
              </div>
            </div>

            <div className="flex justify-end space-x-2 pt-3 border-t border-gray-800">
              <button
                onClick={() => setModalOpen(false)}
                className="px-3.5 py-1.5 rounded-lg border border-gray-700 text-xs text-gray-300 hover:bg-surface transition"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveNode}
                disabled={savingNode}
                className="flex items-center space-x-1.5 px-4 py-1.5 rounded-lg bg-emerald-500 hover:bg-emerald-600 text-white text-xs font-semibold shadow-md transition disabled:opacity-50"
              >
                {nodeSaved ? (
                  <>
                    <CheckCircle2 className="w-4 h-4 text-white" />
                    <span>Saved!</span>
                  </>
                ) : (
                  <>
                    <Save className="w-4 h-4 text-white" />
                    <span>{savingNode ? 'Saving...' : 'Save Relay Node'}</span>
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
