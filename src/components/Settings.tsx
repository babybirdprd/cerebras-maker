import React, { useState, useEffect } from 'react';
import { X, Save, RotateCcw, Key, Cpu, Thermometer, Globe, Loader2 } from 'lucide-react';
import { DEFAULT_AGENT_CONFIG } from '../constants';
import { AgentConfig, ProviderConfig } from '../types';
import { saveSettings, loadSettings, ApiKeys } from '../hooks/useTauri';

interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const PROVIDERS = ['openai', 'anthropic', 'cerebras', 'ollama'] as const;

const MODELS: Record<string, string[]> = {
  openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'o1', 'o1-mini'],
  anthropic: ['claude-sonnet-4-20250514', 'claude-opus-4-20250514', 'claude-3-5-haiku-20241022'],
  cerebras: ['llama-4-scout-17b-16e-instruct', 'llama-4-maverick-17b-128e-instruct', 'llama3.3-70b'],
  ollama: ['llama3.2', 'codellama', 'mistral', 'deepseek-coder'],
};

const AGENT_LABELS: Record<keyof AgentConfig, { label: string; desc: string }> = {
  interrogator: { label: 'Interrogator (L1)', desc: 'Analyzes PRD, asks clarifying questions' },
  architect: { label: 'Architect (L2)', desc: 'Decomposes tasks, generates Rhai scripts' },
  orchestrator: { label: 'Orchestrator', desc: 'Manages workflow and state' },
  coder: { label: 'Coder Atom (L4)', desc: 'Writes code, focused on single tasks' },
  reviewer: { label: 'Reviewer Atom (L4)', desc: 'Reviews code, approves/rejects' },
  tester: { label: 'Tester Atom (L4)', desc: 'Generates and runs tests' },
};

const AgentConfigRow: React.FC<{
  agentKey: keyof AgentConfig;
  config: ProviderConfig;
  onChange: (key: keyof AgentConfig, config: ProviderConfig) => void;
}> = ({ agentKey, config, onChange }) => {
  const info = AGENT_LABELS[agentKey];
  
  return (
    <div className="grid grid-cols-12 gap-3 items-center py-3 border-b border-zinc-800 last:border-b-0">
      <div className="col-span-3">
        <span className="text-sm text-white font-medium">{info.label}</span>
        <p className="text-[10px] text-zinc-500">{info.desc}</p>
      </div>
      <div className="col-span-3">
        <select
          value={config.provider}
          onChange={(e) => onChange(agentKey, { ...config, provider: e.target.value as ProviderConfig['provider'] })}
          className="w-full bg-zinc-900 border border-zinc-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-indigo-500"
        >
          {PROVIDERS.map(p => <option key={p} value={p}>{p}</option>)}
        </select>
      </div>
      <div className="col-span-4">
        <select
          value={config.model}
          onChange={(e) => onChange(agentKey, { ...config, model: e.target.value })}
          className="w-full bg-zinc-900 border border-zinc-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-indigo-500"
        >
          {MODELS[config.provider]?.map(m => <option key={m} value={m}>{m}</option>)}
        </select>
      </div>
      <div className="col-span-2">
        <input
          type="number"
          min="0"
          max="2"
          step="0.1"
          value={config.temperature}
          onChange={(e) => onChange(agentKey, { ...config, temperature: parseFloat(e.target.value) })}
          className="w-full bg-zinc-900 border border-zinc-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-indigo-500"
        />
      </div>
    </div>
  );
};

const Settings: React.FC<SettingsProps> = ({ isOpen, onClose }) => {
  const [config, setConfig] = useState<AgentConfig>(DEFAULT_AGENT_CONFIG as AgentConfig);
  const [apiKeys, setApiKeys] = useState<ApiKeys>({ openai: '', anthropic: '', cerebras: '', ollama_url: 'http://localhost:11434' });
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (isOpen) {
      setIsLoading(true);
      loadSettings().then((settings) => {
        if (settings) {
          setConfig(settings.agent_config);
          setApiKeys(settings.api_keys);
        }
        setIsLoading(false);
      }).catch(() => setIsLoading(false));
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleAgentChange = (key: keyof AgentConfig, newConfig: ProviderConfig) => {
    setConfig(prev => ({ ...prev, [key]: newConfig }));
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await saveSettings({ agent_config: config, api_keys: apiKeys });
      onClose();
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings. Check console for details.');
    } finally {
      setIsSaving(false);
    }
  };

  const handleReset = () => {
    setConfig(DEFAULT_AGENT_CONFIG as AgentConfig);
    setApiKeys({ openai: '', anthropic: '', cerebras: '', ollama_url: 'http://localhost:11434' });
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-zinc-900 border border-zinc-700 rounded-xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-800">
          <h2 className="text-lg font-bold text-white">Settings</h2>
          <button onClick={onClose} className="text-zinc-400 hover:text-white"><X size={20} /></button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* API Keys Section */}
          <div>
            <h3 className="text-sm font-bold text-zinc-300 uppercase tracking-wider mb-4 flex items-center gap-2">
              <Key size={14} /> API Keys
            </h3>
            <div className="grid grid-cols-2 gap-4">
              {(['openai', 'anthropic', 'cerebras'] as const).map(provider => (
                <div key={provider}>
                  <label className="text-xs text-zinc-400 uppercase mb-1 block">{provider}</label>
                  <input
                    type="password"
                    placeholder={`${provider.toUpperCase()}_API_KEY`}
                    value={apiKeys[provider]}
                    onChange={(e) => setApiKeys(prev => ({ ...prev, [provider]: e.target.value }))}
                    className="w-full bg-zinc-950 border border-zinc-700 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                  />
                </div>
              ))}
              <div>
                <label className="text-xs text-zinc-400 uppercase mb-1 block flex items-center gap-1"><Globe size={12} /> Ollama URL</label>
                <input
                  type="text"
                  value={apiKeys.ollama_url}
                  onChange={(e) => setApiKeys(prev => ({ ...prev, ollama_url: e.target.value }))}
                  className="w-full bg-zinc-950 border border-zinc-700 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>
          </div>

          {/* Agent Configuration Section */}
          <div>
            <h3 className="text-sm font-bold text-zinc-300 uppercase tracking-wider mb-4 flex items-center gap-2">
              <Cpu size={14} /> Agent Configuration
            </h3>
            <div className="bg-zinc-950 border border-zinc-800 rounded-lg p-4">
              <div className="grid grid-cols-12 gap-3 text-[10px] text-zinc-500 uppercase font-bold tracking-wider pb-2 border-b border-zinc-700 mb-2">
                <div className="col-span-3">Agent</div>
                <div className="col-span-3">Provider</div>
                <div className="col-span-4">Model</div>
                <div className="col-span-2 flex items-center gap-1"><Thermometer size={10} /> Temp</div>
              </div>
              {(Object.keys(AGENT_LABELS) as Array<keyof AgentConfig>).map(key => (
                <AgentConfigRow key={key} agentKey={key} config={config[key]} onChange={handleAgentChange} />
              ))}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 p-4 border-t border-zinc-800 bg-zinc-950">
          <button onClick={handleReset} disabled={isSaving} className="flex items-center gap-2 px-4 py-2 text-zinc-400 hover:text-white text-sm disabled:opacity-50">
            <RotateCcw size={14} /> Reset
          </button>
          <button onClick={handleSave} disabled={isSaving || isLoading} className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-700 text-white rounded text-sm font-medium">
            {isSaving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Settings;

