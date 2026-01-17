// Cerebras-MAKER: RLM Trajectory Visualization Component
// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
// Displays the execution trace of RLM operations for debugging and understanding

import { useEffect, useRef } from 'react';
import { useMakerStore, RLMTrajectoryStep, RLMConfig } from '../store/makerStore';
import { invoke } from '@tauri-apps/api/core';
import './RLMTrajectory.css';

// Icons for different RLM operations
const OPERATION_ICONS: Record<string, string> = {
  Start: '🚀',
  Peek: '👁️',
  Chunk: '📦',
  SubQuery: '🔄',
  SubResult: '💡',
  RegexFilter: '🔍',
  LoadContext: '📥',
  Final: '✅',
  Error: '❌',
};

// Colors for different RLM operations
const OPERATION_COLORS: Record<string, string> = {
  Start: '#4ade80',
  Peek: '#60a5fa',
  Chunk: '#a78bfa',
  SubQuery: '#fbbf24',
  SubResult: '#22c55e',
  RegexFilter: '#f472b6',
  LoadContext: '#38bdf8',
  Final: '#10b981',
  Error: '#ef4444',
};

function formatTimestamp(ms: number): string {
  const date = new Date(ms);
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }) + '.' + String(ms % 1000).padStart(3, '0');
}

function TrajectoryStep({ 
  step, 
  isSelected, 
  onSelect 
}: { 
  step: RLMTrajectoryStep; 
  isSelected: boolean;
  onSelect: () => void;
}) {
  const opType = step.operation?.type || 'Start';
  const icon = OPERATION_ICONS[opType] || '📝';
  const color = OPERATION_COLORS[opType] || '#9ca3af';
  
  return (
    <div 
      className={`rlm-step ${isSelected ? 'selected' : ''}`}
      style={{ borderLeftColor: color }}
      onClick={onSelect}
    >
      <div className="step-header">
        <span className="step-number">#{step.step}</span>
        <span className="step-icon">{icon}</span>
        <span className="step-type" style={{ color }}>{opType}</span>
        <span className="step-time">{formatTimestamp(step.timestamp_ms)}</span>
      </div>
      <div className="step-description">{step.description}</div>
      {isSelected && step.data !== undefined && step.data !== null && (
        <div className="step-data">
          <pre>{String(JSON.stringify(step.data, null, 2))}</pre>
        </div>
      )}
    </div>
  );
}

function ContextList() {
  const { rlmContexts } = useMakerStore();
  
  if (rlmContexts.length === 0) {
    return <div className="no-contexts">No context variables loaded</div>;
  }
  
  return (
    <div className="context-list">
      {rlmContexts.map((ctx) => (
        <div key={ctx.var_name} className="context-item">
          <span className="context-name">{ctx.var_name}</span>
          <span className="context-length">{(ctx.length / 1000).toFixed(1)}K chars</span>
          <span className="context-type">{ctx.context_type}</span>
        </div>
      ))}
    </div>
  );
}

export function RLMTrajectory() {
  const { 
    rlmTrajectory, 
    rlmIsProcessing, 
    rlmSelectedStep,
    rlmConfig,
    clearRLMTrajectory,
    setRLMTrajectory,
    setRLMSelectedStep,
    setRLMContexts,
    setRLMConfig,
  } = useMakerStore();
  
  const logEndRef = useRef<HTMLDivElement>(null);
  
  // Auto-scroll to bottom on new steps
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [rlmTrajectory]);
  
  // Poll for trajectory updates
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const trajectory = await invoke<RLMTrajectoryStep[]>('get_rlm_trajectory');
        if (trajectory && trajectory.length !== rlmTrajectory.length) {
          setRLMTrajectory(trajectory);
        }
        const contexts = await invoke<string[]>('rlm_list_contexts');
        if (contexts) {
          const contextInfos = await Promise.all(
            contexts.map(async (name) => {
              const length = await invoke<number>('rlm_context_length', { varName: name });
              return { var_name: name, length, context_type: 'unknown' };
            })
          );
          setRLMContexts(contextInfos);
        }
      } catch {
        // RLM store not initialized
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [rlmTrajectory.length, setRLMTrajectory, setRLMContexts]);
  
  // Load RLM config on mount
  useEffect(() => {
    invoke<RLMConfig>('get_rlm_config').then((config) => {
      if (config) setRLMConfig(config);
    }).catch(() => {});
  }, [setRLMConfig]);
  
  const handleClear = async () => {
    try {
      await invoke('clear_rlm_trajectory');
      clearRLMTrajectory();
    } catch (e) {
      console.error('Failed to clear trajectory:', e);
    }
  };
  
  return (
    <div className="rlm-trajectory">
      <div className="rlm-header">
        <h2>🔄 RLM Trajectory</h2>
        <div className="rlm-controls">
          <span className={`rlm-status ${rlmIsProcessing ? 'processing' : 'idle'}`}>
            {rlmIsProcessing ? '● Processing' : '○ Idle'}
          </span>
          <button onClick={handleClear} className="clear-btn">Clear</button>
        </div>
      </div>
      {rlmConfig && (
        <div className="rlm-config-bar">
          <span>Threshold: {((rlmConfig.context_threshold || rlmConfig.rlm_threshold || 50000) / 1000).toFixed(0)}K</span>
          <span>Max Depth: {rlmConfig.max_depth}</span>
          <span>Max Iterations: {rlmConfig.max_iterations}</span>
        </div>
      )}
      <div className="rlm-content">
        <div className="trajectory-panel">
          <h3>Execution Steps</h3>
          <div className="trajectory-list">
            {rlmTrajectory.length === 0 ? (
              <div className="empty-trajectory">
                No RLM execution yet. Load a large context to trigger RLM mode.
              </div>
            ) : (
              rlmTrajectory.map((step) => (
                <TrajectoryStep
                  key={step.step}
                  step={step}
                  isSelected={rlmSelectedStep === step.step}
                  onSelect={() => setRLMSelectedStep(step.step)}
                />
              ))
            )}
            <div ref={logEndRef} />
          </div>
        </div>
        <div className="context-panel">
          <h3>Context Variables</h3>
          <ContextList />
        </div>
      </div>
    </div>
  );
}

