// Cerebras-MAKER: Main Application Entry Point
// PRD Section 6: The Bridge - Tauri Interface

import { useState } from 'react';
import './App.css';
import { AgentState, ChatMessage, PRDFile } from './types';
import { GRAPH_NODES, GRAPH_LINKS } from './constants';

// Components
import { Sidebar, MobileNav } from './components/Sidebar';
import PlanView from './components/PlanView';
import GraphView from './components/GraphView';
import ExecutionPanel from './components/ExecutionPanel';
import TimeSlider from './components/TimeSlider';
import Settings from './components/Settings';
import PRDUpload from './components/PRDUpload';
import ChatInput from './components/ChatInput';

// Tauri hooks
import { openProjectDialog, loadSymbolGraph, initRuntime, analyzePrd, sendInterrogationMessage, transformGraphForD3, ProjectTemplate, createFromTemplate } from './hooks/useTauri';

type ViewType = 'dashboard' | 'topology' | 'execution' | 'history' | 'upload' | 'interrogation';

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('upload');
  const [agentState, setAgentState] = useState<AgentState>(AgentState.IDLE);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [prdFile, setPrdFile] = useState<PRDFile | null>(null);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [graphNodes, setGraphNodes] = useState(GRAPH_NODES);
  const [graphLinks, setGraphLinks] = useState(GRAPH_LINKS);

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
      // Call Tauri to get L1 response
      const response = await sendInterrogationMessage(message, { prd: prdFile?.content });

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: new Date(),
      };
      setChatMessages(prev => [...prev, assistantMessage]);

      // If interrogation is complete, move to planning
      if (response.is_final) {
        setAgentState(AgentState.PLANNING);
        setCurrentView('dashboard');
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
            <div className="hidden lg:block border-l border-zinc-800 p-6 overflow-y-auto">
              <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-wider mb-4">PRD Preview</h3>
              <div className="bg-black rounded border border-zinc-800 p-4">
                <pre className="text-xs text-zinc-400 font-mono whitespace-pre-wrap">{prdFile?.content}</pre>
              </div>
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
          <div className="h-full flex items-center justify-center text-zinc-500">
            <div className="text-center">
              <h2 className="text-xl font-bold text-white mb-2">Shadow Git History</h2>
              <p>Use the time slider below to scrub through commits</p>
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
