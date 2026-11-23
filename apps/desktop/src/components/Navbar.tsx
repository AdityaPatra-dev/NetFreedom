import React from 'react';
import { Shield, Activity, Sliders, Stethoscope, Settings as SettingsIcon, Layers } from 'lucide-react';
import { ConnectionStatus } from '../types';

interface NavbarProps {
  currentTab: string;
  setCurrentTab: (tab: string) => void;
  status: ConnectionStatus;
  latency: number | null;
}

export const Navbar: React.FC<NavbarProps> = ({ currentTab, setCurrentTab, status, latency }) => {
  const isConnected = status === 'connected';

  return (
    <header className="h-14 border-b border-gray-800 bg-surface/80 backdrop-blur px-4 flex items-center justify-between z-20">
      <div className="flex items-center space-x-3">
        <div className="w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center">
          <Shield className="w-5 h-5 text-emerald-400" />
        </div>
        <span className="font-bold tracking-tight text-white text-base">NetFreedom</span>
        <span className="text-[10px] uppercase font-semibold px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          v2.0
        </span>
      </div>

      <nav className="flex items-center space-x-1">
        <button
          onClick={() => setCurrentTab('dashboard')}
          className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition ${
            currentTab === 'dashboard'
              ? 'bg-surface-hover text-white border border-gray-700'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          <Activity className="w-3.5 h-3.5" />
          <span>Dashboard</span>
        </button>

        <button
          onClick={() => setCurrentTab('profiles')}
          className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition ${
            currentTab === 'profiles'
              ? 'bg-surface-hover text-white border border-gray-700'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          <Layers className="w-3.5 h-3.5" />
          <span>Profiles</span>
        </button>

        <button
          onClick={() => setCurrentTab('dpi_lab')}
          className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition ${
            currentTab === 'dpi_lab'
              ? 'bg-surface-hover text-white border border-gray-700'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          <Sliders className="w-3.5 h-3.5" />
          <span>DPI Lab</span>
        </button>

        <button
          onClick={() => setCurrentTab('diagnostics')}
          className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition ${
            currentTab === 'diagnostics'
              ? 'bg-surface-hover text-white border border-gray-700'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          <Stethoscope className="w-3.5 h-3.5" />
          <span>Diagnostics</span>
        </button>

        <button
          onClick={() => setCurrentTab('settings')}
          className={`flex items-center space-x-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition ${
            currentTab === 'settings'
              ? 'bg-surface-hover text-white border border-gray-700'
              : 'text-gray-400 hover:text-gray-200'
          }`}
        >
          <SettingsIcon className="w-3.5 h-3.5" />
          <span>Settings</span>
        </button>
      </nav>

      <div className="flex items-center space-x-3">
        <div className="flex items-center space-x-2 text-xs">
          <span
            className={`w-2 h-2 rounded-full ${
              isConnected ? 'bg-emerald-400 shadow-[0_0_8px_rgba(16,185,129,0.8)]' : 'bg-gray-500'
            }`}
          />
          <span className="text-gray-300 capitalize">{status}</span>
        </div>

        {latency !== null && (
          <div className="bg-surface-hover/80 border border-gray-700/60 rounded px-2 py-0.5 text-[11px] font-mono text-emerald-400">
            {latency.toFixed(1)} ms
          </div>
        )}
      </div>
    </header>
  );
};
