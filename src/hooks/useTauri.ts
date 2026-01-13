// Tauri command hooks for frontend-backend communication
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { AgentConfig } from '../types';

// ============================================================================
// Types
// ============================================================================

export interface ApiKeys {
  openai: string;
  anthropic: string;
  cerebras: string;
  ollama_url: string;
}

export interface AppSettings {
  agent_config: AgentConfig;
  api_keys: ApiKeys;
}

export interface PRDAnalysisResult {
  status: string;
  filename: string;
  initial_message: string;
  detected_features: string[];
}

export interface InterrogationResponse {
  role: 'assistant';
  content: string;
  is_final: boolean;
}

// ============================================================================
// Settings Commands
// ============================================================================

export async function saveSettings(settings: AppSettings): Promise<void> {
  await invoke('save_settings', { settings });
}

export async function loadSettings(): Promise<AppSettings | null> {
  try {
    return await invoke<AppSettings>('load_settings');
  } catch {
    return null;
  }
}

// ============================================================================
// L1 Interrogation Commands
// ============================================================================

export async function analyzePrd(content: string, filename: string): Promise<PRDAnalysisResult> {
  return await invoke<PRDAnalysisResult>('analyze_prd', { content, filename });
}

export async function sendInterrogationMessage(
  message: string, 
  context: Record<string, unknown>
): Promise<InterrogationResponse> {
  return await invoke<InterrogationResponse>('send_interrogation_message', { message, context });
}

export async function completeInterrogation(
  conversation: Array<{ role: string; content: string }>
): Promise<{ status: string; plan_md: string; tasks: unknown[] }> {
  return await invoke('complete_interrogation', { conversation });
}

// ============================================================================
// Project Commands
// ============================================================================

export async function openProjectDialog(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Open Project Folder',
  });
  return selected as string | null;
}

export interface SymbolGraphData {
  symbols: Array<{
    id: string;
    name: string;
    kind: string;
    file_path: string;
    line_start: number;
    line_end: number;
  }>;
  edges: Array<{
    source: string;
    target: string;
    kind: string;
    strength: number;
  }>;
}

export async function loadSymbolGraph(workspacePath: string): Promise<SymbolGraphData> {
  return await invoke<SymbolGraphData>('load_symbol_graph', { workspacePath });
}

// Transform SymbolGraph to D3-compatible format
export function transformGraphForD3(graph: SymbolGraphData): {
  nodes: Array<{ id: string; label: string; group: number; val: number }>;
  links: Array<{ source: string; target: string; value: number }>;
} {
  const kindToGroup: Record<string, number> = {
    'function': 1,
    'struct': 2,
    'enum': 2,
    'trait': 2,
    'impl': 1,
    'module': 3,
    'const': 4,
    'type': 2,
  };

  const nodes = graph.symbols.map(s => ({
    id: s.id,
    label: s.name,
    group: kindToGroup[s.kind.toLowerCase()] || 1,
    val: 8, // Default node size
  }));

  const links = graph.edges.map(e => ({
    source: e.source,
    target: e.target,
    value: Math.max(1, Math.round(e.strength * 3)),
  }));

  return { nodes, links };
}

export async function initRuntime(workspacePath: string): Promise<string> {
  return await invoke<string>('init_runtime', { workspacePath });
}

// ============================================================================
// Grits Commands
// ============================================================================

export async function checkRedFlags(previousBetti1: number): Promise<{
  introduced_cycle: boolean;
  betti_1: number;
  betti_0: number;
  triangle_count: number;
  solid_score: number;
  cycles_detected: string[][];
}> {
  return await invoke('check_red_flags', { previousBetti1 });
}

export async function analyzeTopology(): Promise<unknown> {
  return await invoke('analyze_topology');
}

// ============================================================================
// Runtime Commands
// ============================================================================

export async function executeScript(script: string): Promise<unknown> {
  return await invoke('execute_script', { script });
}

export async function getExecutionLog(): Promise<unknown[]> {
  return await invoke('get_execution_log');
}

export async function getGitHistory(limit: number): Promise<Array<{ hash: string; message: string }>> {
  return await invoke('get_git_history', { limit });
}

export async function createSnapshot(message: string): Promise<unknown> {
  return await invoke('create_snapshot', { message });
}

// ============================================================================
// Template Commands
// ============================================================================

export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  tech_stack: string[];
}

export async function listTemplates(): Promise<ProjectTemplate[]> {
  return await invoke<ProjectTemplate[]>('list_templates');
}

export async function createFromTemplate(
  templateId: string,
  projectPath: string,
  projectName: string
): Promise<string> {
  return await invoke<string>('create_from_template', {
    templateId,
    projectPath,
    projectName
  });
}

// ============================================================================
// L2 Technical Orchestrator Commands
// ============================================================================

export interface ParsedTask {
  id: string;
  description: string;
  atom_type: string;
  estimated_complexity: number;
  seed_symbols: string[];
}

export interface ParsedPlan {
  plan_id: string;
  title: string;
  task_count: number;
  tasks: ParsedTask[];
  dependencies: [string, string][];
}

export interface ExecutionScriptResult {
  script: string;
  plan_id: string;
  task_count: number;
  tasks: { id: string; description: string; atom_type: string }[];
}

export async function generateExecutionScript(
  planContent: string,
  workspacePath: string
): Promise<ExecutionScriptResult> {
  return await invoke<ExecutionScriptResult>('generate_execution_script', {
    planContent,
    workspacePath,
  });
}

export async function parsePlan(planContent: string): Promise<ParsedPlan> {
  return await invoke<ParsedPlan>('parse_plan', { planContent });
}

