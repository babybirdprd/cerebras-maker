// Cerebras-MAKER: Main Application Entry Point
// PRD Section 6: The Bridge - Tauri Interface

import { useState, useEffect, useCallback, useRef } from 'react';
import './App.css';
import { AgentState, ChatMessage, PRDFile, GraphNode, GraphLink } from './types';

// Components
import { Sidebar, MobileNav } from './components/Sidebar';
import PlanView from './components/PlanView';
import GraphView from './components/GraphView';
import ExecutionPanel from './components/ExecutionPanel';
import TimeSlider from './components/TimeSlider';
import { TimeMachine } from './components/TimeMachine';
import Settings from './components/Settings';
import PRDUpload from './components/PRDUpload';
import ChatInput from './components/ChatInput';
import ResearchPanel from './components/ResearchPanel';
import KnowledgePanel from './components/KnowledgePanel';
import { TestPanel } from './components/TestPanel';
import { ValidationPanel } from './components/ValidationPanel';
import SessionPanel from './components/SessionPanel';

// Tauri hooks
import { openProjectDialog, loadSymbolGraph, initRuntime, analyzePrd, sendInterrogationMessage, completeInterrogation, transformGraphForD3, ProjectTemplate, createFromTemplate, kbCompileForInterrogator, SessionData, updateSession } from './hooks/useTauri';

type ViewType = 'dashboard' | 'topology' | 'execution' | 'history' | 'upload' | 'interrogation' | 'validation' | 'tests' | 'sessions';

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('upload');
  const [agentState, setAgentState] = useState<AgentState>(AgentState.IDLE);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [prdFile, setPrdFile] = useState<PRDFile | null>(null);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [graphNodes, setGraphNodes] = useState<GraphNode[]>([]);
  const [graphLinks, setGraphLinks] = useState<GraphLink[]>([]);
  const [planContent, setPlanContent] = useState<string | null>(null);
  const [researchContext, setResearchContext] = useState<string | null>(null);
  const [kbContext, setKbContext] = useState<string | null>(null);
  const [workspacePath, setWorkspacePath] = useState<string>('');

  // Auto-save state
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [autoSaveEnabled, setAutoSaveEnabled] = useState(true);
  const [lastAutoSave, setLastAutoSave] = useState<Date | null>(null);
  const [autoSaveStatus, setAutoSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
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

  // Session loading handler
  const handleLoadSession = (session: SessionData) => {
    // Set session ID to enable auto-save for this session
    setCurrentSessionId(session.id);
    setLastAutoSave(null);
    setAutoSaveStatus('idle');

    setWorkspacePath(session.workspace_path);
    if (session.prd_content && session.prd_filename) {
      // Determine file type from extension
      const ext = session.prd_filename.split('.').pop()?.toLowerCase();
      const fileType: 'md' | 'txt' | 'pdf' = ext === 'md' ? 'md' : ext === 'pdf' ? 'pdf' : 'txt';
      setPrdFile({ name: session.prd_filename, content: session.prd_content, type: fileType });
    }
    if (session.plan_content) {
      setPlanContent(session.plan_content);
    }
    // Restore conversation history
    if (session.conversation_history && session.conversation_history.length > 0) {
      setChatMessages(session.conversation_history as ChatMessage[]);
    }
    // Navigate to the view the user was last on, or default to chat
    const validViews: ViewType[] = ['dashboard', 'topology', 'execution', 'history', 'upload', 'interrogation', 'validation', 'tests', 'sessions'];
    if (validViews.includes(session.current_view as ViewType)) {
      setCurrentView(session.current_view as ViewType);
    } else {
      setCurrentView('interrogation');
    }
  };

  // Handler when a new session is created (called from SessionPanel)
  const handleSessionCreated = (sessionId: string) => {
    setCurrentSessionId(sessionId);
    setLastAutoSave(new Date());
    setAutoSaveStatus('saved');
  };

  const handlePRDUpload = async (file: PRDFile) => {
    setPrdFile(file);
    setAgentState(AgentState.INTERROGATING);
    setIsLoading(true);

    try {
      // Call Tauri to analyze the PRD
      const result = await analyzePrd(file.content, file.name);

      setChatMessages([{
        id: '1',
        role: 'assistant',
        content: result.initial_message,
        timestamp: new Date(),
      }]);
    } catch (error) {
      // Fallback to local message if Tauri not available
      setChatMessages([{
        id: '1',
        role: 'assistant',
        content: `I've analyzed your PRD "${file.name}". Let me ask a few clarifying questions to ensure I understand your requirements correctly.\n\n**Question 1:** What is the primary target platform for this application? (Desktop, Web, Mobile, or Cross-platform)`,
        timestamp: new Date(),
      }]);
    }

    setIsLoading(false);
    setCurrentView('interrogation');
  };

  const handleOpenProject = async () => {
    try {
      const path = await openProjectDialog();
      if (path) {
        setIsLoading(true);
        setWorkspacePath(path);
        // Initialize the runtime and load symbol graph
        await initRuntime(path);
        const graphData = await loadSymbolGraph(path);

        // Transform to D3 format
        const { nodes, links } = transformGraphForD3(graphData);
        setGraphNodes(nodes);
        setGraphLinks(links);

        setAgentState(AgentState.PLANNING);
        setCurrentView('dashboard');
        setIsLoading(false);
      }
    } catch (error) {
      console.error('Failed to open project:', error);
      setIsLoading(false);
    }
  };

  const handleSendMessage = async (message: string) => {
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: message,
      timestamp: new Date(),
    };
    setChatMessages(prev => [...prev, userMessage]);
    setIsLoading(true);

    try {
      // Fetch latest KB context if not already loaded
      let currentKbContext = kbContext;
      if (!currentKbContext) {
        try {
          currentKbContext = await kbCompileForInterrogator();
          if (currentKbContext && currentKbContext.trim()) {
            setKbContext(currentKbContext);
          }
        } catch {
          // KB might be empty, that's fine
        }
      }

      // Call Tauri to get L1 response (include research and KB context)
      const response = await sendInterrogationMessage(message, {
        prd: prdFile?.content,
        research: researchContext,
        knowledge_base: currentKbContext
      });

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: new Date(),
      };
      setChatMessages(prev => [...prev, assistantMessage]);

      // If interrogation is complete, generate the plan
      if (response.is_final) {
        try {
          // Build conversation history for plan generation
          const conversation = [...chatMessages, userMessage, assistantMessage].map(m => ({
            role: m.role,
            content: m.content,
          }));

          const planResult = await completeInterrogation(conversation);
          setPlanContent(planResult.plan_md);

          setAgentState(AgentState.PLANNING);
          setCurrentView('dashboard');
        } catch (planError) {
          console.error('Failed to generate plan:', planError);
          // Still move to dashboard even if plan generation fails
          setAgentState(AgentState.PLANNING);
          setCurrentView('dashboard');
        }
      }
    } catch (error) {
      // Fallback response if Tauri not available
      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'Thank you for that clarification. Based on your response, I have a follow-up question...\n\n**Question 2:** What authentication method would you prefer? (OAuth, JWT, Session-based, or None)',
        timestamp: new Date(),
      };
      setChatMessages(prev => [...prev, assistantMessage]);
    }

    setIsLoading(false);
  };

  const handleSelectTemplate = async (template: ProjectTemplate) => {
    // For now, prompt for project name and location
    const projectName = prompt('Enter project name:', 'my-app');
    if (!projectName) return;

    try {
      // Use a default location for now (could use file dialog later)
      const projectPath = `./projects/${projectName}`;
      const result = await createFromTemplate(template.id, projectPath, projectName);
      console.log(result);

      // After creating, switch to interrogation to define the PRD
      setChatMessages([{
        id: '1',
        role: 'assistant',
        content: `Great! I've created a new **${template.name}** project called "${projectName}".\n\nNow let's define what you want to build. Please describe your project requirements, or upload a PRD document.`,
        timestamp: new Date(),
      }]);
      setCurrentView('interrogation');
      setAgentState(AgentState.INTERROGATING);
    } catch (error) {
      // Fallback for non-Tauri environment
      setChatMessages([{
        id: '1',
        role: 'assistant',
        content: `Starting a new **${template.name}** project called "${projectName}".\n\nPlease describe what you want to build with this ${template.tech_stack.join(' + ')} stack.`,
        timestamp: new Date(),
      }]);
      setCurrentView('interrogation');
      setAgentState(AgentState.INTERROGATING);
    }
  };

  const renderContent = () => {
    switch (currentView) {
      case 'upload':
        return <PRDUpload onUpload={handlePRDUpload} onOpenProject={handleOpenProject} onSelectTemplate={handleSelectTemplate} />;

      case 'interrogation':
        return (
          <div className="h-full grid grid-cols-1 lg:grid-cols-2 gap-0">
            <ChatInput
              messages={chatMessages}
              onSendMessage={handleSendMessage}
              isLoading={isLoading}
              placeholder="Answer the question..."
            />
            <div className="hidden lg:block border-l border-zinc-800 p-6 overflow-y-auto space-y-6">
              {/* Knowledge Base Panel for pre-existing docs */}
              <KnowledgePanel
                onContextChange={(context) => setKbContext(context)}
              />
              {/* Research Panel for gathering external docs */}
              <ResearchPanel
                onResearchComplete={(content) => setResearchContext(content)}
              />
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
              <PlanView planContent={planContent || undefined} />
            </div>
            <div className="flex flex-col overflow-hidden">
              <div className="flex-1 p-4 lg:p-6 min-h-0">
                <GraphView nodes={graphNodes} links={graphLinks} />
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
            <GraphView nodes={graphNodes} links={graphLinks} />
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
            <ValidationPanel workspacePath={workspacePath} />
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
            <TestPanel workspacePath={workspacePath} />
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
            <SessionPanel
              workspacePath={workspacePath}
              prdContent={prdFile?.content || null}
              prdFilename={prdFile?.name || null}
              conversationHistory={chatMessages}
              planContent={planContent}
              currentView={currentView}
              currentSessionId={currentSessionId}
              autoSaveEnabled={autoSaveEnabled}
              autoSaveStatus={autoSaveStatus}
              lastAutoSave={lastAutoSave}
              onLoadSession={handleLoadSession}
              onSessionCreated={handleSessionCreated}
              onAutoSaveToggle={setAutoSaveEnabled}
            />
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

      default:
        return null;
    }
  };

  return (
    <div className="h-screen w-screen bg-zinc-950 text-zinc-100 flex flex-col overflow-hidden">
      <div className="flex-1 flex overflow-hidden">
        {/* Desktop Sidebar */}
        <Sidebar
          currentView={currentView}
          onChangeView={(view) => setCurrentView(view as ViewType)}
          onOpenSettings={() => setSettingsOpen(true)}
          autoSaveStatus={autoSaveStatus}
          currentSessionId={currentSessionId}
          className="hidden md:flex"
        />

        {/* Main Content */}
        <main className="flex-1 flex flex-col overflow-hidden">
          {/* Header */}
          <header className="h-14 border-b border-zinc-800 flex items-center justify-between px-4 lg:px-6 bg-zinc-900/50 shrink-0">
            <div className="flex items-center gap-3">
              <span className={`px-2 py-1 rounded text-xs font-mono uppercase tracking-wider ${
                agentState === AgentState.IDLE ? 'bg-zinc-800 text-zinc-400' :
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
      <MobileNav
        currentView={currentView}
        onChangeView={(view) => setCurrentView(view as ViewType)}
        onOpenSettings={() => setSettingsOpen(true)}
        className="md:hidden"
      />

      {/* Settings Modal */}
      <Settings isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

export default App;
