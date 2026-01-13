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
}));

