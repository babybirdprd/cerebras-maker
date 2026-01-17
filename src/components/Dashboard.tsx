// Cerebras-MAKER: The Dashboard Component
// PRD Section 6: The Bridge - Tauri Interface

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useMakerStore } from '../store/makerStore';
import { Cockpit } from './Cockpit';
import { Blueprint } from './Blueprint';
import { TimeMachine } from './TimeMachine';
import { RLMTrajectory } from './RLMTrajectory';
import './Dashboard.css';

type TabType = 'cockpit' | 'blueprint' | 'timemachine' | 'rlm';

export function Dashboard() {
  const { runtimeInitialized, setRuntimeInitialized, workspacePath, setWorkspacePath } = useMakerStore();
  const [activeTab, setActiveTab] = useState<TabType>('cockpit');
  const [initError, setInitError] = useState<string | null>(null);
  const [scriptInput, setScriptInput] = useState('');
  
  // Initialize runtime on mount
  useEffect(() => {
    initializeRuntime();
  }, []);
  
  async function initializeRuntime() {
    try {
      // Get current working directory as workspace
      const cwd = await invoke<string>('get_cwd');
      setWorkspacePath(cwd);

      await invoke('init_runtime', { workspacePath: cwd });

      // Initialize RLM store for large context handling
      try {
        await invoke('init_rlm_store');
      } catch (rlmError) {
        console.warn('RLM store initialization failed:', rlmError);
      }

      setRuntimeInitialized(true);
      setInitError(null);
    } catch (e) {
      setInitError(String(e));
      setRuntimeInitialized(false);
    }
  }
  
  async function executeScript() {
    if (!scriptInput.trim()) return;
    
    try {
      useMakerStore.getState().setIsExecuting(true);
      const result = await invoke('execute_script', { script: scriptInput });
      console.log('Script result:', result);
    } catch (e) {
      console.error('Script error:', e);
    } finally {
      useMakerStore.getState().setIsExecuting(false);
    }
  }
  
  async function loadSymbolGraph() {
    if (!workspacePath) return;
    
    try {
      const result = await invoke<{ nodes: unknown[]; edges: unknown[] }>('load_symbol_graph', { 
        workspacePath 
      });
      // Transform to our format with positions
      const nodes = (result.nodes as Array<{ id: string; name: string; kind: string; file: string }>).map((n, i) => ({
        ...n,
        position: [
          Math.cos(i * 0.5) * 3,
          Math.sin(i * 0.3) * 2,
          Math.sin(i * 0.5) * 3
        ] as [number, number, number]
      }));
      useMakerStore.getState().setSymbolGraph(nodes, result.edges as never[]);
    } catch (e) {
      console.error('Failed to load symbol graph:', e);
    }
  }
  
  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <div className="logo">
          <span className="logo-icon">🧠</span>
          <h1>Cerebras-MAKER</h1>
        </div>
        <div className="status-bar">
          <span className={`runtime-status ${runtimeInitialized ? 'ready' : 'error'}`}>
            {runtimeInitialized ? '✓ Runtime Ready' : '✗ Runtime Error'}
          </span>
          <span className="workspace">{workspacePath || 'No workspace'}</span>
        </div>
      </header>
      
      {initError && (
        <div className="error-banner">
          <span>⚠️ {initError}</span>
          <button onClick={initializeRuntime}>Retry</button>
        </div>
      )}
      
      <nav className="tab-nav">
        <button
          className={activeTab === 'cockpit' ? 'active' : ''}
          onClick={() => setActiveTab('cockpit')}
        >
          🎮 Cockpit
        </button>
        <button
          className={activeTab === 'blueprint' ? 'active' : ''}
          onClick={() => { setActiveTab('blueprint'); loadSymbolGraph(); }}
        >
          🏗️ Blueprint
        </button>
        <button
          className={activeTab === 'timemachine' ? 'active' : ''}
          onClick={() => setActiveTab('timemachine')}
        >
          ⏰ Time Machine
        </button>
        <button
          className={activeTab === 'rlm' ? 'active' : ''}
          onClick={() => setActiveTab('rlm')}
        >
          🔄 RLM Trace
        </button>
      </nav>

      <main className="dashboard-main">
        <div className="panel-container">
          {activeTab === 'cockpit' && <Cockpit />}
          {activeTab === 'blueprint' && <Blueprint />}
          {activeTab === 'timemachine' && <TimeMachine />}
          {activeTab === 'rlm' && <RLMTrajectory />}
        </div>
        
        {activeTab === 'cockpit' && (
          <div className="script-input">
            <textarea
              value={scriptInput}
              onChange={(e) => setScriptInput(e.target.value)}
              placeholder="Enter Rhai script..."
              rows={4}
            />
            <button onClick={executeScript} disabled={!runtimeInitialized}>
              ▶ Execute
            </button>
          </div>
        )}
      </main>
    </div>
  );
}

