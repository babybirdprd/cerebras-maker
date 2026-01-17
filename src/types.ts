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

// Chat/Message types for L1 interaction (legacy - simple chatbot style)
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

// =============================================================================
// Threaded Messaging System Types (Agent Unblocker)
// =============================================================================

/** Type of thread - determines the nature of the agent's request */
export type ThreadType = 'help_request' | 'clarification' | 'resource_needed' | 'approval_needed';

/** Current status of the thread */
export type ThreadStatus = 'open' | 'resolved' | 'pending';

/** Priority level for thread handling */
export type ThreadPriority = 'low' | 'medium' | 'high' | 'urgent';

/** Attachment type for resources shared in threads */
export type AttachmentType = 'link' | 'file' | 'code_snippet' | 'documentation';

/** Resource attachment that can be shared in thread messages */
export interface ThreadAttachment {
  type: AttachmentType;
  title: string;
  url?: string;          // For links and documentation
  content?: string;      // For code snippets and inline content
  file_path?: string;    // For local file references
  mime_type?: string;    // For file type identification
}

/** Individual message within a thread */
export interface ThreadMessage {
  id: string;
  thread_id: string;
  role: 'agent' | 'human';
  agent_name?: string;   // e.g., "Research Agent", "Coder Agent" (only for agent role)
  content: string;
  attachments?: ThreadAttachment[];
  timestamp: Date;
}

/** A conversation thread - created by agents when blocked, resolved by humans */
export interface Thread {
  id: string;
  type: ThreadType;
  status: ThreadStatus;
  priority: ThreadPriority;
  title: string;
  agent_name: string;    // Agent that created/owns the thread
  task_id?: string;      // Related task (links to Task.id)
  created_at: Date;
  updated_at: Date;
  resolved_at?: Date;    // When thread was marked resolved
  messages: ThreadMessage[];
  is_blocking: boolean;  // Whether agent is blocked waiting for response
}

/** Summary of thread activity for UI display */
export interface ThreadSummary {
  id: string;
  type: ThreadType;
  status: ThreadStatus;
  priority: ThreadPriority;
  title: string;
  agent_name: string;
  message_count: number;
  unread_count: number;
  is_blocking: boolean;
  created_at: Date;
  updated_at: Date;
}

/** Request payload for creating a new thread (from agent) */
export interface CreateThreadRequest {
  type: ThreadType;
  priority: ThreadPriority;
  title: string;
  agent_name: string;
  task_id?: string;
  initial_message: string;
  is_blocking: boolean;
}

/** Request payload for replying to a thread (from human) */
export interface ThreadReplyRequest {
  thread_id: string;
  content: string;
  attachments?: ThreadAttachment[];
  resolve_thread?: boolean; // If true, marks thread as resolved after reply
}

export interface PRDFile {
  name: string;
  content: string;
  type: 'md' | 'txt' | 'pdf';
}

