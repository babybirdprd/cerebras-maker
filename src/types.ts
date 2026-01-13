export enum AgentState {
  IDLE = 'IDLE',
  PLANNING = 'PLANNING', // System 2
  RESEARCHING = 'RESEARCHING',
  VOTING = 'VOTING', // Cerebras Swarm
  VERIFYING = 'VERIFYING', // Grits/Tests
  COMMITTING = 'COMMITTING', // Gitoxide
  INTERROGATING = 'INTERROGATING', // L1 Q&A
}

export interface TaskDetails {
  issues: string[];
  snippet?: string;
  notes?: string;
}

export interface Task {
  id: string;
  title: string;
  status: 'pending' | 'active' | 'completed' | 'failed';
  depth: number;
  children?: Task[];
  details?: TaskDetails;
}

export interface LogEntry {
  id: number;
  timestamp: string;
  source: 'System 1' | 'System 2' | 'Grits' | 'Gitoxide';
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
}

export interface GraphNode {
  id: string;
  group: number; // 1: Logic, 2: Data, 3: UI, 4: Ext
  val: number; // Size
  label: string;
}

export interface GraphLink {
  source: string;
  target: string;
  value: number;
}

export interface VoteCandidate {
  id: number;
  snippet: string;
  score: number;
  redFlags: string[]; // Grits topological analysis
  status: 'accepted' | 'rejected' | 'pending';
}

// Settings types
export interface ProviderConfig {
  provider: 'openai' | 'anthropic' | 'cerebras' | 'ollama';
  model: string;
  temperature: number;
  apiKey?: string;
  baseUrl?: string;
}

export interface AgentConfig {
  interrogator: ProviderConfig;
  architect: ProviderConfig;
  orchestrator: ProviderConfig;
  coder: ProviderConfig;
  reviewer: ProviderConfig;
  tester: ProviderConfig;
}

// Chat/Message types for L1 interaction
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export interface PRDFile {
  name: string;
  content: string;
  type: 'md' | 'txt' | 'pdf';
}

