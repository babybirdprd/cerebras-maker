// Cerebras-MAKER: Global State Store
// Uses Zustand for state management

import { create } from 'zustand';

// Types for the MAKER system
export interface ExecutionEvent {
  timestamp_ms: number;
  event_type: string;
  message: string;
  data?: unknown;
}

export interface SymbolNode {
  id: string;
  name: string;
  kind: string;
  file: string;
  position: [number, number, number];
}

export interface SymbolEdge {
  source: string;
  target: string;
  weight: number;
}

export interface HistoryEntry {
  hash: string;
  message: string;
}

export interface RedFlagResult {
  introduced_cycle: boolean;
  betti_1: number;
  betti_0: number;
  triangle_count: number;
  solid_score: number;
  cycles_detected: string[][];
}

// RLM (Recursive Language Model) Types
export interface RLMOperation {
  type: 'Start' | 'Peek' | 'Chunk' | 'SubQuery' | 'SubResult' | 'RegexFilter' | 'LoadContext' | 'Final' | 'Error';
  var_name?: string;
  start?: number;
  end?: number;
  num_chunks?: number;
  prompt_preview?: string;
  result_preview?: string;
  pattern?: string;
  matches?: number;
  length?: number;
  answer_preview?: string;
  message?: string;
  depth?: number;
}

export interface RLMTrajectoryStep {
  step: number;
  operation: RLMOperation;
  description: string;
  data?: unknown;
  timestamp_ms: number;
}

export interface RLMContextInfo {
  var_name: string;
  length: number;
  context_type: string;
}

export interface RLMConfig {
  max_depth: number;
  max_iterations: number;
  default_chunk_size?: number;
  rlm_threshold?: number;
  use_sub_model?: boolean;
  context_threshold: number;
  sub_model_provider: string;
  sub_model_name: string;
  sub_model_temperature: number;
}

// P2-1: Conversation message type for L1 Interrogator history
export interface ConversationMessage {
  role: 'user' | 'assistant';
  content: string;
}

interface MakerState {
  // Runtime state
  runtimeInitialized: boolean;
  workspacePath: string | null;

  // Execution log (Cockpit)
  executionLog: ExecutionEvent[];
  isExecuting: boolean;

  // Symbol graph (Blueprint)
  symbolNodes: SymbolNode[];
  symbolEdges: SymbolEdge[];
  selectedNode: string | null;

  // Git history (Time Machine)
  gitHistory: HistoryEntry[];
  currentCommit: string | null;

  // Red flag status
  redFlagResult: RedFlagResult | null;

  // P2-1: L1 Interrogator conversation history
  interrogationHistory: ConversationMessage[];
  interrogationPrd: string | null;

  // RLM (Recursive Language Model) state
  rlmTrajectory: RLMTrajectoryStep[];
  rlmContexts: RLMContextInfo[];
  rlmConfig: RLMConfig | null;
  rlmIsProcessing: boolean;
  rlmSelectedStep: number | null;

  // Actions
  setRuntimeInitialized: (initialized: boolean) => void;
  setWorkspacePath: (path: string) => void;
  addExecutionEvent: (event: ExecutionEvent) => void;
  clearExecutionLog: () => void;
  setIsExecuting: (executing: boolean) => void;
  setSymbolGraph: (nodes: SymbolNode[], edges: SymbolEdge[]) => void;
  setSelectedNode: (nodeId: string | null) => void;
  setGitHistory: (history: HistoryEntry[]) => void;
  setCurrentCommit: (hash: string | null) => void;
  setRedFlagResult: (result: RedFlagResult | null) => void;

  // P2-1: L1 Interrogator Actions
  addInterrogationMessage: (message: ConversationMessage) => void;
  clearInterrogationHistory: () => void;
  setInterrogationPrd: (prd: string | null) => void;

  // RLM Actions
  setRLMTrajectory: (trajectory: RLMTrajectoryStep[]) => void;
  addRLMStep: (step: RLMTrajectoryStep) => void;
  clearRLMTrajectory: () => void;
  setRLMContexts: (contexts: RLMContextInfo[]) => void;
  setRLMConfig: (config: RLMConfig) => void;
  setRLMIsProcessing: (processing: boolean) => void;
  setRLMSelectedStep: (step: number | null) => void;
}

export const useMakerStore = create<MakerState>((set) => ({
  // Initial state
  runtimeInitialized: false,
  workspacePath: null,
  executionLog: [],
  isExecuting: false,
  symbolNodes: [],
  symbolEdges: [],
  selectedNode: null,
  gitHistory: [],
  currentCommit: null,
  redFlagResult: null,

  // P2-1: L1 Interrogator initial state
  interrogationHistory: [],
  interrogationPrd: null,

  // RLM initial state
  rlmTrajectory: [],
  rlmContexts: [],
  rlmConfig: null,
  rlmIsProcessing: false,
  rlmSelectedStep: null,

  // Actions
  setRuntimeInitialized: (initialized) => set({ runtimeInitialized: initialized }),

  setWorkspacePath: (path) => set({ workspacePath: path }),

  addExecutionEvent: (event) => set((state) => ({
    executionLog: [...state.executionLog, event]
  })),

  clearExecutionLog: () => set({ executionLog: [] }),

  setIsExecuting: (executing) => set({ isExecuting: executing }),

  setSymbolGraph: (nodes, edges) => set({
    symbolNodes: nodes,
    symbolEdges: edges
  }),

  setSelectedNode: (nodeId) => set({ selectedNode: nodeId }),

  setGitHistory: (history) => set({ gitHistory: history }),

  setCurrentCommit: (hash) => set({ currentCommit: hash }),

  setRedFlagResult: (result) => set({ redFlagResult: result }),

  // P2-1: L1 Interrogator Actions
  addInterrogationMessage: (message) => set((state) => ({
    interrogationHistory: [...state.interrogationHistory, message]
  })),

  clearInterrogationHistory: () => set({ interrogationHistory: [], interrogationPrd: null }),

  setInterrogationPrd: (prd) => set({ interrogationPrd: prd }),

  // RLM Actions
  setRLMTrajectory: (trajectory) => set({ rlmTrajectory: trajectory }),

  addRLMStep: (step) => set((state) => ({
    rlmTrajectory: [...state.rlmTrajectory, step]
  })),

  clearRLMTrajectory: () => set({ rlmTrajectory: [], rlmSelectedStep: null }),

  setRLMContexts: (contexts) => set({ rlmContexts: contexts }),

  setRLMConfig: (config) => set({ rlmConfig: config }),

  setRLMIsProcessing: (processing) => set({ rlmIsProcessing: processing }),

  setRLMSelectedStep: (step) => set({ rlmSelectedStep: step }),
}));

