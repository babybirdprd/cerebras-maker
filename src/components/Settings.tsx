import React, { useState, useEffect } from 'react';
import { X, Save, RotateCcw, Key, Cpu, Loader2, ChevronDown, Check, Link as LinkIcon, Zap, Sparkles, Bot, Code, Eye, TestTube, Thermometer, RefreshCw, Layers, Hash, FileText } from 'lucide-react';
import { DEFAULT_AGENT_CONFIG } from '../constants';
import { AgentConfig, ProviderConfig } from '../types';
import { saveSettings, loadSettings, ApiKeys, AppSettings } from '../tauri-api';
import { RLMConfig, useMakerStore } from '../store/makerStore';


// --- Provider Data Configuration ---
type FieldType = 'apiKey' | 'baseUrl';

interface ProviderData {
  id: string;
  name: string;
  color: string;
  bgColor: string;
  textColor: string;
  borderColor: string;
  ringColor: string;
  description: string;
  models: readonly string[];
  fields: readonly FieldType[];
}

const PROVIDERS_CONFIG: Record<string, ProviderData> = {
  openai: {
    id: 'openai',
    name: 'OpenAI',
    color: 'from-emerald-400 to-emerald-600',
    bgColor: 'bg-emerald-500',
    textColor: 'text-emerald-400',
    borderColor: 'border-emerald-500/50',
    ringColor: 'ring-emerald-500/30',
    description: 'GPT-5.2, o4, and GPT-4o models.',
    models: ['gpt-5.2', 'gpt-5.1', 'gpt-5', 'gpt-5-mini', 'gpt-5-codex', 'o4-mini', 'o3', 'o3-mini', 'o1', 'gpt-4o', 'gpt-4o-mini', 'gpt-4.1', 'gpt-4.1-mini'],
    fields: ['apiKey', 'baseUrl'],
  },
  anthropic: {
    id: 'anthropic',
    name: 'Anthropic',
    color: 'from-orange-400 to-orange-600',
    bgColor: 'bg-orange-500',
    textColor: 'text-orange-400',
    borderColor: 'border-orange-500/50',
    ringColor: 'ring-orange-500/30',
    description: 'Claude Opus 4.5 & Sonnet 4.5.',
    models: ['claude-opus-4.5', 'claude-sonnet-4.5', 'claude-opus-4', 'claude-sonnet-4', 'claude-haiku-4.5', 'claude-sonnet-3.7', 'claude-haiku-3.5'],
    fields: ['apiKey'],
  },
  cerebras: {
    id: 'cerebras',
    name: 'Cerebras',
    color: 'from-violet-400 to-violet-600',
    bgColor: 'bg-violet-500',
    textColor: 'text-violet-400',
    borderColor: 'border-violet-500/50',
    ringColor: 'ring-violet-500/30',
    description: 'Ultra-fast GLM-4.7 & Qwen 3 inference.',
    models: ['zai-glm-4.7', 'zai-glm-4.6', 'gpt-oss-120b', 'qwen-3-235b-a22b-instruct-2507'],
    fields: ['apiKey'],
  },
  ollama: {
    id: 'ollama',
    name: 'Ollama',
    color: 'from-slate-400 to-slate-600',
    bgColor: 'bg-slate-500',
    textColor: 'text-slate-400',
    borderColor: 'border-slate-500/50',
    ringColor: 'ring-slate-500/30',
    description: 'Local models on your machine.',
    models: ['llama3.3', 'llama3.2', 'qwen2.5-coder', 'deepseek-coder-v2', 'codellama', 'mistral', 'phi3'],
    fields: ['baseUrl'],
  },
  google: {
    id: 'google',
    name: 'Google',
    color: 'from-blue-400 to-blue-600',
    bgColor: 'bg-blue-500',
    textColor: 'text-blue-400',
    borderColor: 'border-blue-500/50',
    ringColor: 'ring-blue-500/30',
    description: 'Gemini 3 Pro & 2.5 Pro/Flash.',
    models: ['gemini-3-pro-preview', 'gemini-3-flash-preview', 'gemini-2.5-pro', 'gemini-2.5-flash'],
    fields: ['apiKey'],
  },
  openrouter: {
    id: 'openrouter',
    name: 'OpenRouter',
    color: 'from-pink-400 to-pink-600',
    bgColor: 'bg-pink-500',
    textColor: 'text-pink-400',
    borderColor: 'border-pink-500/50',
    ringColor: 'ring-pink-500/30',
    description: 'Access 200+ models via one API.',
    models: ['anthropic/claude-opus-4.5', 'anthropic/claude-sonnet-4.5', 'openai/gpt-5.2', 'openai/o4-mini', 'google/gemini-3-flash', 'google/gemini-2.5-pro'],
    fields: ['apiKey'],
  },
};

type ProviderId = keyof typeof PROVIDERS_CONFIG;

const PROVIDERS = Object.keys(PROVIDERS_CONFIG) as ProviderId[];

const MODELS: Record<string, string[]> = Object.fromEntries(
  Object.entries(PROVIDERS_CONFIG).map(([key, val]) => [key, val.models as unknown as string[]])
);

const AGENT_LABELS: Record<keyof AgentConfig, { label: string; desc: string; icon: React.ReactNode }> = {
  interrogator: { label: 'Interrogator', desc: 'L1 • Analyzes PRD, asks clarifying questions', icon: <Sparkles size={14} /> },
  architect: { label: 'Architect', desc: 'L2 • Decomposes tasks, generates Rhai scripts', icon: <Zap size={14} /> },
  orchestrator: { label: 'Orchestrator', desc: 'Manages workflow and state machine', icon: <Bot size={14} /> },
  coder: { label: 'Coder', desc: 'L4 Atom • Writes code, focused on single tasks', icon: <Code size={14} /> },
  reviewer: { label: 'Reviewer', desc: 'L4 Atom • Reviews code, approves/rejects', icon: <Eye size={14} /> },
  tester: { label: 'Tester', desc: 'L4 Atom • Generates and runs tests', icon: <TestTube size={14} /> },
};

// Provider Logo Component - fetches from models.dev
const ProviderLogo: React.FC<{ provider: string; size?: 'sm' | 'md' | 'lg' }> = ({ provider, size = 'md' }) => {
  const sizeClasses = {
    sm: 'w-5 h-5',
    md: 'w-8 h-8',
    lg: 'w-12 h-12',
  };

  // Try dark mode variant first, then fallback to standard
  return (
    <div className={`${sizeClasses[size]} flex items-center justify-center`}>
      <img
        src={`https://models.dev/logos/${provider}-dark.svg`}
        alt={`${provider} logo`}
        className={`${sizeClasses[size]} object-contain`}
        onError={(e) => {
          const target = e.currentTarget;
          if (target.src.includes('-dark.svg')) {
            target.src = `https://models.dev/logos/${provider}.svg`;
          } else {
            target.style.display = 'none';
          }
        }}
      />
    </div>
  );
};

const AgentConfigRow: React.FC<{
  agentKey: keyof AgentConfig;
  config: ProviderConfig;
  availableModels: string[];
  onChange: (key: keyof AgentConfig, config: ProviderConfig) => void;
}> = ({ agentKey, config, availableModels, onChange }) => {
  const info = AGENT_LABELS[agentKey];
  const providerConfig = PROVIDERS_CONFIG[config.provider as ProviderId] || PROVIDERS_CONFIG.openai;

  return (
    <div className="group relative bg-zinc-900/50 hover:bg-zinc-800/50 rounded-xl p-4 transition-all duration-200 border border-zinc-800/50 hover:border-zinc-700">
      {/* Accent line */}
      <div className={`absolute left-0 top-0 bottom-0 w-1 rounded-l-xl bg-linear-to-b ${providerConfig.color} opacity-60 group-hover:opacity-100 transition-opacity`} />

      <div className="flex items-start gap-4 pl-3">
        {/* Agent info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className={`${providerConfig.textColor}`}>{info.icon}</span>
            <span className="text-sm font-semibold text-white">{info.label}</span>
          </div>
          <p className="text-[11px] text-zinc-500">{info.desc}</p>
        </div>

        {/* Controls */}
        <div className="flex items-center gap-3">
          {/* Provider Select */}
          <div className="relative">
            <select
              value={config.provider}
              onChange={(e) => {
                const newProvider = e.target.value as ProviderConfig['provider'];
                const newModels = MODELS[newProvider] || [];
                onChange(agentKey, {
                  ...config,
                  provider: newProvider,
                  model: newModels[0] || config.model
                });
              }}
              className="appearance-none bg-zinc-800 border border-zinc-700 rounded-lg pl-8 pr-8 py-1.5 text-xs text-white focus:outline-none focus:ring-2 focus:ring-zinc-600 cursor-pointer hover:border-zinc-600 transition-colors"
            >
              {PROVIDERS.map(p => (
                <option key={p} value={p}>{PROVIDERS_CONFIG[p].name}</option>
              ))}
            </select>
            <div className="absolute left-2 top-1/2 -translate-y-1/2 pointer-events-none">
              <ProviderLogo provider={config.provider} size="sm" />
            </div>
            <ChevronDown size={12} className="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-500 pointer-events-none" />
          </div>

          {/* Model Select */}
          <div className="relative">
            <select
              value={config.model}
              onChange={(e) => onChange(agentKey, { ...config, model: e.target.value })}
              className="appearance-none bg-zinc-800 border border-zinc-700 rounded-lg pl-3 pr-8 py-1.5 text-xs text-white focus:outline-none focus:ring-2 focus:ring-zinc-600 cursor-pointer hover:border-zinc-600 transition-colors min-w-[180px]"
            >
              {availableModels.map(m => <option key={m} value={m}>{m}</option>)}
            </select>
            <ChevronDown size={12} className="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-500 pointer-events-none" />
          </div>

          {/* Temperature */}
          <div className="flex items-center gap-2 bg-zinc-800 border border-zinc-700 rounded-lg px-2 py-1">
            <Thermometer size={12} className="text-zinc-500" />
            <input
              type="number"
              min="0"
              max="2"
              step="0.1"
              value={config.temperature}
              onChange={(e) => onChange(agentKey, { ...config, temperature: parseFloat(e.target.value) })}
              className="w-10 bg-transparent text-xs text-white focus:outline-none text-center"
            />
          </div>
        </div>
      </div>
    </div>
  );
};

// Extended API Keys type to include new providers
interface ExtendedApiKeys extends ApiKeys {
  google?: string;
  openrouter?: string;
  [key: string]: string | undefined;
}

// Helper to get API key for a provider
const getProviderApiKey = (keys: ExtendedApiKeys, provider: string): string => {
  if (provider === 'ollama') return keys.ollama_url || '';
  return (keys as Record<string, string | undefined>)[provider] || '';
};

// Helper to set API key for a provider
const setProviderApiKey = (keys: ExtendedApiKeys, provider: string, value: string): ExtendedApiKeys => {
  if (provider === 'ollama') {
    return { ...keys, ollama_url: value };
  }
  return { ...keys, [provider]: value };
};

const Settings: React.FC = () => {
  const { settingsOpen: isOpen, setSettingsOpen } = useMakerStore();
  const onClose = () => setSettingsOpen(false);
  const [config, setConfig] = useState<AgentConfig>(DEFAULT_AGENT_CONFIG as AgentConfig);
  const [apiKeys, setApiKeys] = useState<ExtendedApiKeys>({
    openai: '',
    anthropic: '',
    cerebras: '',
    ollama_url: 'http://localhost:11434',
    google: '',
    openrouter: '',
  });
  const [selectedProvider, setSelectedProvider] = useState<ProviderId>('openai');
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'providers' | 'agents' | 'rlm'>('providers');
  const [providerModels, setProviderModels] = useState<Record<string, string[]>>(() =>
    Object.fromEntries(Object.entries(PROVIDERS_CONFIG).map(([k, v]) => [k, [...v.models]]))
  );

  // Fetch models from models.dev on mount
  useEffect(() => {
    const fetchModels = async () => {
      try {
        const response = await fetch('https://models.dev/api.json');
        if (!response.ok) throw new Error('Failed to fetch models');
        const data = await response.json();

        const updatedModels: Record<string, string[]> = { ...providerModels };
        const supportedProviders = Object.keys(PROVIDERS_CONFIG);

        supportedProviders.forEach(providerId => {
          const providerData = data[providerId];
          if (providerData && providerData.models) {
            const modelNames = Object.keys(providerData.models);
            if (modelNames.length > 0) {
              updatedModels[providerId] = modelNames;
            }
          }
        });

        setProviderModels(updatedModels);
      } catch (err) {
        console.error('Error fetching models from models.dev:', err);
      }
    };

    fetchModels();
  }, []);
  const [rlmConfig, setRlmConfig] = useState<RLMConfig>({
    max_depth: 3,
    max_iterations: 10,
    context_threshold: 50000,
    sub_model_provider: 'cerebras',
    sub_model_name: 'qwen-3-235b-a22b-instruct-2507',
    sub_model_temperature: 0.1,
  });

  useEffect(() => {
    if (isOpen) {
      setIsLoading(true);
      loadSettings().then((settings: AppSettings | null) => {
        if (settings) {
          setConfig(settings.agent_config);
          setApiKeys(prev => ({ ...prev, ...settings.api_keys }));
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
    setApiKeys({ openai: '', anthropic: '', cerebras: '', ollama_url: 'http://localhost:11434', google: '', openrouter: '' });
  };

  const currentProviderData = PROVIDERS_CONFIG[selectedProvider];

  // Count configured providers
  const configuredCount = [
    apiKeys.openai,
    apiKeys.anthropic,
    apiKeys.cerebras,
    apiKeys.google,
    apiKeys.openrouter,
  ].filter(k => k && k.length > 5).length + (apiKeys.ollama_url ? 1 : 0);

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-md z-50 flex items-center justify-center p-4">
      <div className="bg-zinc-950 border border-zinc-800 rounded-3xl w-full max-w-5xl max-h-[90vh] overflow-hidden flex flex-col shadow-2xl">

        {/* Decorative Top Gradient */}
        <div className={`h-1.5 bg-linear-to-r ${currentProviderData.color}`} />

        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800/50">
          <div className="flex items-center gap-4">
            <div className="p-2 bg-zinc-800/50 rounded-xl border border-zinc-700/50">
              <Cpu size={20} className="text-zinc-400" />
            </div>
            <div>
              <h2 className="text-lg font-bold text-white">Configuration</h2>
              <p className="text-xs text-zinc-500">{configuredCount} provider{configuredCount !== 1 ? 's' : ''} configured</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex border-b border-zinc-800/50 px-6">
          <button
            onClick={() => setActiveTab('providers')}
            className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors ${activeTab === 'providers'
              ? 'border-white text-white'
              : 'border-transparent text-zinc-500 hover:text-zinc-300'
              }`}
          >
            Providers & API Keys
          </button>
          <button
            onClick={() => setActiveTab('agents')}
            className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors ${activeTab === 'agents'
              ? 'border-white text-white'
              : 'border-transparent text-zinc-500 hover:text-zinc-300'
              }`}
          >
            Agent Configuration
          </button>
          <button
            onClick={() => setActiveTab('rlm')}
            className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${activeTab === 'rlm'
              ? 'border-violet-400 text-violet-400'
              : 'border-transparent text-zinc-500 hover:text-zinc-300'
              }`}
          >
            <RefreshCw size={14} />
            RLM Settings
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto">
          {activeTab === 'providers' ? (
            <div className="p-6">
              {/* Provider Selection Header */}
              <div className="flex items-center justify-center mb-8">
                <div className="relative">
                  <div className={`absolute inset-0 blur-3xl opacity-20 ${currentProviderData.bgColor} rounded-full`} />
                  <div className="relative z-10 p-6 bg-zinc-900/50 rounded-2xl border border-zinc-800/50 backdrop-blur-sm">
                    <ProviderLogo provider={selectedProvider} size="lg" />
                  </div>
                </div>
                <div className="ml-6">
                  <h3 className="text-xl font-bold text-white">{currentProviderData.name}</h3>
                  <p className="text-sm text-zinc-400 mt-1">{currentProviderData.description}</p>
                </div>
              </div>

              {/* Provider Grid */}
              <div className="mb-8">
                <label className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-3 block">Select Provider</label>
                <div className="grid grid-cols-3 md:grid-cols-6 gap-3">
                  {PROVIDERS.map(providerId => {
                    const prov = PROVIDERS_CONFIG[providerId];
                    const isSelected = selectedProvider === providerId;
                    const hasKey = getProviderApiKey(apiKeys, providerId).length > 0;

                    return (
                      <button
                        key={providerId}
                        onClick={() => setSelectedProvider(providerId)}
                        className={`
                          relative flex flex-col items-center justify-center p-4 rounded-xl border transition-all duration-200
                          ${isSelected
                            ? `bg-zinc-800 ${prov.borderColor} ring-2 ${prov.ringColor}`
                            : 'bg-zinc-900/50 border-zinc-800 hover:bg-zinc-800 hover:border-zinc-700'
                          }
                        `}
                      >
                        <ProviderLogo provider={providerId} size="md" />
                        <span className={`text-xs font-medium mt-2 ${isSelected ? prov.textColor : 'text-zinc-400'}`}>
                          {prov.name}
                        </span>
                        {hasKey && (
                          <div className="absolute top-2 right-2">
                            <Check size={12} className="text-emerald-400" />
                          </div>
                        )}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* API Key Input for Selected Provider */}
              <div className="space-y-4 bg-zinc-900/50 rounded-2xl border border-zinc-800/50 p-6">
                {/* API Key */}
                {currentProviderData.fields.includes('apiKey') && (
                  <div className="group">
                    <label className="block text-xs font-semibold text-zinc-400 mb-2 group-focus-within:text-white transition-colors">
                      API Key
                    </label>
                    <div className="relative">
                      <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-zinc-500">
                        <Key size={16} />
                      </div>
                      <input
                        type="password"
                        value={getProviderApiKey(apiKeys, selectedProvider)}
                        onChange={(e) => setApiKeys(prev => setProviderApiKey(prev, selectedProvider, e.target.value))}
                        placeholder={`sk-...`}
                        className={`w-full bg-zinc-950 border border-zinc-700 text-white rounded-xl py-3 pl-12 pr-4 focus:outline-none focus:ring-2 ${currentProviderData.ringColor} focus:border-transparent transition-all placeholder:text-zinc-600`}
                      />
                      {getProviderApiKey(apiKeys, selectedProvider).length > 5 && (
                        <div className="absolute inset-y-0 right-4 flex items-center">
                          <Check size={16} className="text-emerald-400" />
                        </div>
                      )}
                    </div>
                  </div>
                )}

                {/* Base URL (for OpenAI and Ollama) */}
                {currentProviderData.fields.includes('baseUrl') && (
                  <div className="group">
                    <label className="block text-xs font-semibold text-zinc-400 mb-2 group-focus-within:text-white transition-colors">
                      {selectedProvider === 'ollama' ? 'Server URL' : 'Base URL'}
                      {selectedProvider !== 'ollama' && <span className="text-zinc-600 font-normal ml-1">(Optional)</span>}
                    </label>
                    <div className="relative">
                      <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-zinc-500">
                        <LinkIcon size={16} />
                      </div>
                      <input
                        type="text"
                        value={apiKeys.ollama_url || ''}
                        onChange={(e) => setApiKeys(prev => ({ ...prev, ollama_url: e.target.value }))}
                        placeholder={selectedProvider === 'ollama' ? 'http://localhost:11434' : 'https://api.openai.com/v1'}
                        className="w-full bg-zinc-950 border border-zinc-700 text-white rounded-xl py-3 pl-12 pr-4 focus:outline-none focus:ring-2 focus:ring-zinc-600 focus:border-transparent transition-all placeholder:text-zinc-600"
                      />
                    </div>
                  </div>
                )}

                {/* Available Models Preview */}
                <div>
                  <label className="block text-xs font-semibold text-zinc-400 mb-2">Available Models</label>
                  <div className="flex flex-wrap gap-2">
                    {currentProviderData.models.slice(0, 4).map(model => (
                      <span key={model} className="px-3 py-1 bg-zinc-800 rounded-lg text-xs text-zinc-300 font-mono">
                        {model}
                      </span>
                    ))}
                    {currentProviderData.models.length > 4 && (
                      <span className="px-3 py-1 bg-zinc-800/50 rounded-lg text-xs text-zinc-500">
                        +{currentProviderData.models.length - 4} more
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          ) : activeTab === 'agents' ? (
            <div className="p-6 space-y-3">
              <p className="text-sm text-zinc-400 mb-4">
                Configure which provider and model each agent should use. Each agent can use a different provider.
              </p>
              {(Object.keys(AGENT_LABELS) as Array<keyof AgentConfig>).map(key => (
                <AgentConfigRow
                  key={key}
                  agentKey={key}
                  config={config[key]}
                  availableModels={providerModels[config[key].provider] || []}
                  onChange={handleAgentChange}
                />
              ))}
            </div>
          ) : (
            /* RLM Configuration Tab */
            <div className="p-6 space-y-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="p-3 bg-violet-500/20 rounded-xl border border-violet-500/30">
                  <RefreshCw size={24} className="text-violet-400" />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-white">Recursive Language Model</h3>
                  <p className="text-sm text-zinc-400">Configure RLM for handling large contexts via recursive decomposition</p>
                </div>
              </div>

              {/* Context Threshold */}
              <div className="bg-zinc-900/50 rounded-2xl border border-zinc-800/50 p-6 space-y-4">
                <div className="flex items-center gap-2 mb-2">
                  <FileText size={16} className="text-violet-400" />
                  <h4 className="text-sm font-semibold text-white">Context Threshold</h4>
                </div>
                <p className="text-xs text-zinc-500 mb-3">
                  When context exceeds this character count, RLM mode activates for recursive processing.
                </p>
                <div className="flex items-center gap-4">
                  <input
                    type="range"
                    min="10000"
                    max="200000"
                    step="5000"
                    value={rlmConfig.context_threshold}
                    onChange={(e) => setRlmConfig(prev => ({ ...prev, context_threshold: parseInt(e.target.value) }))}
                    className="flex-1 h-2 bg-zinc-700 rounded-lg appearance-none cursor-pointer accent-violet-500"
                  />
                  <div className="flex items-center gap-2 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2">
                    <Hash size={14} className="text-zinc-500" />
                    <input
                      type="number"
                      value={rlmConfig.context_threshold}
                      onChange={(e) => setRlmConfig(prev => ({ ...prev, context_threshold: parseInt(e.target.value) || 50000 }))}
                      className="w-20 bg-transparent text-sm text-white focus:outline-none text-center"
                    />
                    <span className="text-xs text-zinc-500">chars</span>
                  </div>
                </div>
              </div>

              {/* Recursion Limits */}
              <div className="bg-zinc-900/50 rounded-2xl border border-zinc-800/50 p-6 space-y-4">
                <div className="flex items-center gap-2 mb-2">
                  <Layers size={16} className="text-violet-400" />
                  <h4 className="text-sm font-semibold text-white">Recursion Limits</h4>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs text-zinc-400 mb-2">Max Depth</label>
                    <p className="text-[10px] text-zinc-600 mb-2">How deep nested sub-queries can go</p>
                    <input
                      type="number"
                      min="1"
                      max="10"
                      value={rlmConfig.max_depth}
                      onChange={(e) => setRlmConfig(prev => ({ ...prev, max_depth: parseInt(e.target.value) || 3 }))}
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:ring-2 focus:ring-violet-500/50"
                    />
                  </div>
                  <div>
                    <label className="block text-xs text-zinc-400 mb-2">Max Iterations</label>
                    <p className="text-[10px] text-zinc-600 mb-2">Maximum REPL loop iterations</p>
                    <input
                      type="number"
                      min="1"
                      max="50"
                      value={rlmConfig.max_iterations}
                      onChange={(e) => setRlmConfig(prev => ({ ...prev, max_iterations: parseInt(e.target.value) || 10 }))}
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:ring-2 focus:ring-violet-500/50"
                    />
                  </div>
                </div>
              </div>

              {/* Sub-Model Configuration */}
              <div className="bg-zinc-900/50 rounded-2xl border border-zinc-800/50 p-6 space-y-4">
                <div className="flex items-center gap-2 mb-2">
                  <Bot size={16} className="text-violet-400" />
                  <h4 className="text-sm font-semibold text-white">Sub-Model for Recursive Queries</h4>
                </div>
                <p className="text-xs text-zinc-500 mb-3">
                  Model used for llm_query() sub-calls. Use a fast, cost-effective model for best results.
                </p>
                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <label className="block text-xs text-zinc-400 mb-2">Provider</label>
                    <select
                      value={rlmConfig.sub_model_provider}
                      onChange={(e) => {
                        const newProvider = e.target.value;
                        const newModels = MODELS[newProvider] || [];
                        setRlmConfig(prev => ({
                          ...prev,
                          sub_model_provider: newProvider,
                          sub_model_name: newModels[0] || prev.sub_model_name
                        }));
                      }}
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:ring-2 focus:ring-violet-500/50"
                    >
                      {PROVIDERS.map(p => (
                        <option key={p} value={p}>{PROVIDERS_CONFIG[p].name}</option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-xs text-zinc-400 mb-2">Model</label>
                    <select
                      value={rlmConfig.sub_model_name}
                      onChange={(e) => setRlmConfig(prev => ({ ...prev, sub_model_name: e.target.value }))}
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:ring-2 focus:ring-violet-500/50"
                    >
                      {(providerModels[rlmConfig.sub_model_provider] || []).map(m => (
                        <option key={m} value={m}>{m}</option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-xs text-zinc-400 mb-2">Temperature</label>
                    <div className="flex items-center gap-2 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2">
                      <Thermometer size={14} className="text-zinc-500" />
                      <input
                        type="number"
                        min="0"
                        max="2"
                        step="0.1"
                        value={rlmConfig.sub_model_temperature}
                        onChange={(e) => setRlmConfig(prev => ({ ...prev, sub_model_temperature: parseFloat(e.target.value) || 0.1 }))}
                        className="w-full bg-transparent text-sm text-white focus:outline-none"
                      />
                    </div>
                  </div>
                </div>
              </div>

              {/* Info Box */}
              <div className="bg-violet-500/10 border border-violet-500/30 rounded-xl p-4">
                <p className="text-xs text-violet-300">
                  <strong>RLM Mode:</strong> When context exceeds the threshold, the system treats the prompt as an external
                  environment variable. The LLM can peek at slices, chunk content, filter with regex, and make recursive
                  sub-queries to process arbitrarily large contexts efficiently.
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Footer Status */}
        <div className="bg-zinc-900 px-6 py-4 border-t border-zinc-800/50 flex justify-between items-center">
          <div className="flex items-center gap-3 text-xs text-zinc-500">
            <div className={`w-2 h-2 rounded-full ${configuredCount > 0 ? 'bg-emerald-500' : 'bg-zinc-700'}`} />
            <span>{configuredCount > 0 ? `${configuredCount} provider${configuredCount !== 1 ? 's' : ''} ready` : 'No providers configured'}</span>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={handleReset}
              disabled={isSaving}
              className="flex items-center gap-2 px-4 py-2 text-zinc-400 hover:text-white text-sm disabled:opacity-50 transition-colors"
            >
              <RotateCcw size={14} /> Reset
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving || isLoading}
              className={`flex items-center gap-2 px-6 py-2.5 bg-linear-to-r ${currentProviderData.color} hover:opacity-90 disabled:opacity-50 text-white rounded-xl text-sm font-medium transition-all shadow-lg`}
            >
              {isSaving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
              {isSaving ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Settings;

