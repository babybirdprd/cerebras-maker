// Cerebras-MAKER: Main Application Entry Point
// PRD Section 6: The Bridge - Tauri Interface

import { useEffect, useCallback, useRef } from 'react';
import './App.css';
import { AgentState } from './types';

// Components
import { Sidebar, MobileNav } from './components/Sidebar';
import PlanView from './components/PlanView';
import GraphView from './components/GraphView';
import ExecutionPanel from './components/ExecutionPanel';
import { TimeMachine } from './components/TimeMachine';
import TimeSlider from './components/TimeSlider';
import Settings from './components/Settings';
import PRDUpload from './components/PRDUpload';
import ChatInput from './components/ChatInput';
import ResearchPanel from './components/ResearchPanel';
import KnowledgePanel from './components/KnowledgePanel';
import { TestPanel } from './components/TestPanel';
import { ValidationPanel } from './components/ValidationPanel';
import SessionPanel from './components/SessionPanel';
import ThreadPanel from './components/ThreadPanel';

// Zustand store
import { useMakerStore } from './store/makerStore';

// Tauri API
import { updateSession } from './tauri-api';


function App() {
  const {
    currentView,
    agentState,
    prdFile,
    chatMessages,
    planContent,
    researchContext,
    currentSessionId,
    autoSaveEnabled,
    setLastAutoSave,
    setAutoSaveStatus
  } = useMakerStore();

  const autoSaveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const AUTO_SAVE_INTERVAL = 30000; // 30 seconds

  // Auto-save function
  const performAutoSave = useCallback(async () => {
    if (!currentSessionId || !autoSaveEnabled) return;

    setAutoSaveStatus('saving');
    try {
      await updateSession(
        currentSessionId,
        prdFile?.content || null,
        chatMessages,
        planContent,
        currentView
      );
      setLastAutoSave(new Date());
      setAutoSaveStatus('saved');
      // Reset to idle after 3 seconds
      setTimeout(() => setAutoSaveStatus('idle'), 3000);
    } catch (e) {
      console.error('Auto-save failed:', e);
      setAutoSaveStatus('error');
      setTimeout(() => setAutoSaveStatus('idle'), 5000);
    }
  }, [currentSessionId, autoSaveEnabled, prdFile, chatMessages, planContent, currentView]);

  // Auto-save effect: trigger on state changes with debounce
  useEffect(() => {
    if (!currentSessionId || !autoSaveEnabled) return;

    // Clear existing timeout
    if (autoSaveTimeoutRef.current) {
      clearTimeout(autoSaveTimeoutRef.current);
    }

    // Set new timeout for auto-save
    autoSaveTimeoutRef.current = setTimeout(() => {
      performAutoSave();
    }, AUTO_SAVE_INTERVAL);

    return () => {
      if (autoSaveTimeoutRef.current) {
        clearTimeout(autoSaveTimeoutRef.current);
      }
    };
  }, [currentSessionId, autoSaveEnabled, prdFile, chatMessages, planContent, currentView, performAutoSave]);

  // Handlers moved to store or sub-components
  const renderContent = () => {
    switch (currentView) {
      case 'upload':
        return <PRDUpload />;

      case 'interrogation':
        return (
          <div className="h-full grid grid-cols-1 lg:grid-cols-2 gap-0">
            <ChatInput
              placeholder="Answer the question..."
            />
            <div className="hidden lg:block border-l border-zinc-800 p-6 overflow-y-auto space-y-6">
              {/* Knowledge Base Panel for pre-existing docs */}
              <KnowledgePanel />
              {/* Research Panel for gathering external docs */}
              <ResearchPanel />
              {/* PRD Preview */}
              {prdFile && (
                <div>
                  <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-4">PRD Preview</h3>
                  <div className="bg-black rounded border border-zinc-800 p-4 max-h-48 overflow-y-auto">
                    <pre className="text-xs text-zinc-400 font-mono whitespace-pre-wrap">{prdFile?.content}</pre>
                  </div>
                </div>
              )}
              {/* Research Context Preview */}
              {researchContext && (
                <div>
                  <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-4">Research Context</h3>
                  <div className="bg-black rounded border border-emerald-800/50 p-4 max-h-32 overflow-y-auto">
                    <pre className="text-xs text-emerald-400/70 font-mono whitespace-pre-wrap">{researchContext.slice(0, 300)}...</pre>
                  </div>
                </div>
              )}
            </div>
          </div>
        );

      case 'dashboard':
        return (
          <div className="h-full grid grid-cols-1 lg:grid-cols-2 gap-0">
            <div className="border-r border-zinc-800 overflow-hidden">
              <PlanView />
            </div>
            <div className="flex flex-col overflow-hidden">
              <div className="flex-1 p-4 lg:p-6 min-h-0">
                <GraphView />
              </div>
              <div className="h-48 border-t border-zinc-800 p-4 overflow-y-auto">
                <h3 className="text-xs font-bold text-zinc-400 uppercase tracking-wider mb-2">Activity Log</h3>
                <div className="space-y-1 text-xs font-mono text-zinc-500">
                  <p><span className="text-emerald-500">[System 1]</span> Generated 3 candidates for task #t2b</p>
                  <p><span className="text-indigo-500">[System 2]</span> Decomposed task into 3 subtasks</p>
                  <p><span className="text-amber-500">[Grits]</span> Detected cycle in candidate #2</p>
                </div>
              </div>
            </div>
          </div>
        );

      case 'topology':
        return (
          <div className="h-full p-4 lg:p-6">
            <GraphView />
          </div>
        );

      case 'execution':
        return <ExecutionPanel />;

      case 'history':
        return (
          <div className="h-full p-4 lg:p-6">
            <TimeMachine />
          </div>
        );

      case 'validation':
        return (
          <div className="h-full grid grid-cols-1 lg:grid-cols-2 gap-0">
            <ValidationPanel />
            <div className="hidden lg:block border-l border-zinc-800 p-6 overflow-y-auto">
              <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-4">About Validation</h3>
              <div className="text-sm text-zinc-500 space-y-3">
                <p>The validation panel uses <span className="text-indigo-400">Grits VirtualApply</span> to test proposed changes before applying them.</p>
                <p>It detects:</p>
                <ul className="list-disc list-inside space-y-1 text-xs">
                  <li>Dependency cycles (Betti_1 analysis)</li>
                  <li>Layer architecture violations</li>
                  <li>Cross-file dependency issues</li>
                  <li>Symbol conflicts</li>
                </ul>
              </div>
            </div>
          </div>
        );

      case 'tests':
        return (
          <div className="h-full grid grid-cols-1 lg:grid-cols-2 gap-0">
            <TestPanel />
            <div className="hidden lg:block border-l border-zinc-800 p-6 overflow-y-auto">
              <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-4">Test Generation</h3>
              <div className="text-sm text-zinc-500 space-y-3">
                <p>Generate and run tests using the <span className="text-green-400">Tester Atom</span>.</p>
                <p>Supported frameworks:</p>
                <ul className="list-disc list-inside space-y-1 text-xs">
                  <li><span className="text-orange-400">Rust</span> - cargo test</li>
                  <li><span className="text-blue-400">TypeScript</span> - vitest / jest</li>
                  <li><span className="text-yellow-400">Python</span> - pytest</li>
                  <li><span className="text-cyan-400">Go</span> - go test</li>
                </ul>
              </div>
            </div>
          </div>
        );

      case 'sessions':
        return (
          <div className="h-full grid grid-cols-1 lg:grid-cols-2 gap-0">
            <SessionPanel />
            <div className="hidden lg:block border-l border-zinc-800 p-6 overflow-y-auto">
              <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-4">Session Management</h3>
              <div className="text-sm text-zinc-500 space-y-3">
                <p>Save and restore your work sessions to continue from where you left off.</p>
                <p className="flex items-center gap-2 text-amber-400">
                  <span className="text-lg">⚡</span> Auto-save every 30 seconds when enabled
                </p>
                <p>Sessions include:</p>
                <ul className="list-disc list-inside space-y-1 text-xs">
                  <li><span className="text-indigo-400">PRD content</span> and filename</li>
                  <li><span className="text-green-400">Conversation history</span> with the AI</li>
                  <li><span className="text-amber-400">Knowledge base</span> documents</li>
                  <li><span className="text-purple-400">Execution plan</span></li>
                  <li><span className="text-cyan-400">Workspace path</span></li>
                </ul>
                <p className="text-xs mt-4 text-zinc-600">Sessions are stored in <code className="bg-zinc-800 px-1 py-0.5 rounded">~/.cerebras-maker/sessions/</code></p>
              </div>
            </div>
          </div>
        );

      case 'threads':
        return <ThreadPanel className="h-full" />;

      default:
        return null;
    }
  };

  return (
    <div className="h-screen w-screen bg-zinc-950 text-zinc-100 flex flex-col overflow-hidden">
      <div className="flex-1 flex overflow-hidden">
        {/* Desktop Sidebar */}
        <Sidebar className="hidden md:flex" />

        {/* Main Content */}
        <main className="flex-1 flex flex-col overflow-hidden">
          {/* Header */}
          <header className="h-14 border-b border-zinc-800 flex items-center justify-between px-4 lg:px-6 bg-zinc-900/50 shrink-0">
            <div className="flex items-center gap-3">
              <span className={`px-2 py-1 rounded text-xs font-mono uppercase tracking-wider ${agentState === AgentState.IDLE ? 'bg-zinc-800 text-zinc-400' :
                agentState === AgentState.INTERROGATING ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' :
                  agentState === AgentState.PLANNING ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' :
                    agentState === AgentState.VOTING ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 animate-pulse' :
                      'bg-zinc-800 text-zinc-400'
                }`}>
                {agentState}
              </span>
            </div>
            <div className="flex items-center gap-2 text-xs text-zinc-500">
              <span className="hidden sm:inline">Cerebras MAKER</span>
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
            </div>
          </header>

          {/* Content Area */}
          <div className="flex-1 overflow-hidden">
            {renderContent()}
          </div>
        </main>
      </div>

      {/* Time Slider - only show when not in upload/interrogation */}
      {currentView !== 'upload' && currentView !== 'interrogation' && (
        <TimeSlider />
      )}

      {/* Mobile Navigation */}
      <MobileNav className="md:hidden" />

      {/* Settings Modal */}
      <Settings />
    </div>
  );
}

export default App;
