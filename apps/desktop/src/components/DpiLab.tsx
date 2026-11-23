import React from 'react';
import { Sliders, Terminal, RotateCcw } from 'lucide-react';
import { DpiSettings } from '../types';

interface DpiLabProps {
  dpiSettings: DpiSettings;
  setDpiSettings: React.Dispatch<React.SetStateAction<DpiSettings>>;
}

export const DpiLab: React.FC<DpiLabProps> = ({ dpiSettings, setDpiSettings }) => {
  const generatedCommand = `ciadpi -i 127.0.0.1 -p ${dpiSettings.local_port} -s ${
    dpiSettings.split_pos
  } -f -1 -t ${dpiSettings.fake_ttl} ${dpiSettings.disorder ? '-d 1+s ' : ''}${
    dpiSettings.fake_rst ? '-r 1+s ' : ''
  }-F`;

  const resetToCampusPreset = () => {
    setDpiSettings({
      split_pos: '2+s',
      fake_ttl: 3,
      disorder: true,
      fake_rst: false,
      local_port: 1080,
    });
  };

  return (
    <div className="flex-1 p-6 overflow-y-auto max-w-4xl mx-auto w-full">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-bold text-white flex items-center space-x-2">
            <Sliders className="w-5 h-5 text-emerald-400" />
            <span>ByeDPI Desynchronization Lab</span>
          </h2>
          <p className="text-xs text-gray-400 mt-1">
            Fine-tune TCP and TLS packet fragmentation parameters to defeat campus and ISP DPI middleboxes.
          </p>
        </div>

        <button
          onClick={resetToCampusPreset}
          className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-surface border border-gray-700 text-xs text-gray-300 hover:text-white hover:bg-surface-hover transition"
        >
          <RotateCcw className="w-3.5 h-3.5" />
          <span>Campus Preset</span>
        </button>
      </div>

      <div className="space-y-6">
        {/* Controls Card */}
        <div className="bg-surface/60 border border-gray-800 rounded-xl p-5 space-y-5">
          {/* Split Position */}
          <div>
            <div className="flex justify-between items-center mb-1.5">
              <label className="text-xs font-semibold text-gray-300">
                TLS SNI Split Position (`-s`)
              </label>
              <span className="text-xs font-mono text-emerald-400">{dpiSettings.split_pos}</span>
            </div>
            <div className="grid grid-cols-4 gap-2">
              {['1+s', '2+s', '3+s', '1+h'].map((pos) => (
                <button
                  key={pos}
                  onClick={() => setDpiSettings((prev) => ({ ...prev, split_pos: pos }))}
                  className={`py-2 rounded-lg border text-xs font-mono transition ${
                    dpiSettings.split_pos === pos
                      ? 'bg-emerald-500/10 border-emerald-500 text-emerald-400 font-bold'
                      : 'bg-surface border-gray-800 text-gray-400 hover:bg-surface-hover hover:text-gray-200'
                  }`}
                >
                  {pos}
                </button>
              ))}
            </div>
            <p className="text-[11px] text-gray-500 mt-1">
              `2+s` slices the TLS ClientHello at byte 2 of the SNI domain name, confusing DPI state parsers.
            </p>
          </div>

          {/* Fake Packet TTL */}
          <div>
            <div className="flex justify-between items-center mb-1.5">
              <label className="text-xs font-semibold text-gray-300">
                Fake Packet Time-To-Live (`-t`)
              </label>
              <span className="text-xs font-mono text-emerald-400">TTL = {dpiSettings.fake_ttl}</span>
            </div>
            <input
              type="range"
              min="1"
              max="12"
              value={dpiSettings.fake_ttl}
              onChange={(e) =>
                setDpiSettings((prev) => ({ ...prev, fake_ttl: parseInt(e.target.value) }))
              }
              className="w-full accent-emerald-500 h-1.5 bg-gray-700 rounded-lg cursor-pointer"
            />
            <div className="flex justify-between text-[10px] text-gray-500 mt-1">
              <span>TTL 1 (Very Close)</span>
              <span>TTL 3 (Campus Gateway - Verified)</span>
              <span>TTL 12 (Deep ISP)</span>
            </div>
          </div>

          {/* Toggles */}
          <div className="grid grid-cols-2 gap-4 pt-2">
            <label className="flex items-center space-x-3 p-3 rounded-lg border border-gray-800 bg-surface/40 cursor-pointer hover:bg-surface/80 transition">
              <input
                type="checkbox"
                checked={dpiSettings.disorder}
                onChange={(e) =>
                  setDpiSettings((prev) => ({ ...prev, disorder: e.target.checked }))
                }
                className="w-4 h-4 rounded text-emerald-500 focus:ring-0 bg-gray-800 border-gray-700 accent-emerald-500"
              />
              <div>
                <span className="text-xs font-semibold text-gray-200 block">
                  TCP Segment Disordering (`-d 1+s`)
                </span>
                <span className="text-[10px] text-gray-500 block">
                  Delivers chunk 2 before chunk 1 to defeat in-line reassembly.
                </span>
              </div>
            </label>

            <label className="flex items-center space-x-3 p-3 rounded-lg border border-gray-800 bg-surface/40 cursor-pointer hover:bg-surface/80 transition">
              <input
                type="checkbox"
                checked={dpiSettings.fake_rst}
                onChange={(e) =>
                  setDpiSettings((prev) => ({ ...prev, fake_rst: e.target.checked }))
                }
                className="w-4 h-4 rounded text-emerald-500 focus:ring-0 bg-gray-800 border-gray-700 accent-emerald-500"
              />
              <div>
                <span className="text-xs font-semibold text-gray-200 block">
                  Fake TCP RST Injection (`-r 1+s`)
                </span>
                <span className="text-[10px] text-gray-500 block">
                  Resets middlebox connection tracking while destination ignores bad checksum.
                </span>
              </div>
            </label>
          </div>
        </div>

        {/* Live CLI Preview */}
        <div className="bg-surface/90 border border-gray-800 rounded-xl p-4">
          <div className="flex items-center space-x-2 text-xs font-bold text-gray-400 mb-2">
            <Terminal className="w-4 h-4 text-emerald-400" />
            <span>Generated C Daemon Parameters</span>
          </div>
          <pre className="p-3 rounded-lg bg-black/60 border border-gray-800/80 font-mono text-xs text-emerald-400 overflow-x-auto select-all">
            {generatedCommand}
          </pre>
        </div>
      </div>
    </div>
  );
};
