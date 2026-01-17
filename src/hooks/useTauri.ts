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

// P2-1: Conversation message type for history tracking
export interface ConversationMessage {
  role: 'user' | 'assistant';
  content: string;
}

export async function sendInterrogationMessage(
  message: string,
  context: Record<string, unknown>,
  conversationHistory?: ConversationMessage[]
): Promise<InterrogationResponse> {
  return await invoke<InterrogationResponse>('send_interrogation_message', {
    message,
    context,
    conversationHistory: conversationHistory ?? null
  });
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

// P3-1: Layer violation type matching Rust LayerViolation struct (analysis.rs:758)
export interface LayerViolation {
  from_node: string;
  from_layer: string;
  to_node: string;
  to_layer: string;
  violation_type: string; // "upstream_dependency", "cycle", etc.
}

// P3-1: Updated to match Rust RedFlagResult struct
export interface RedFlagResult {
  introduced_cycle: boolean;
  has_layer_violations: boolean;
  betti_1: number;
  betti_0: number;
  triangle_count: number;
  solid_score: number;
  cycles_detected: string[][];
  layer_violations: LayerViolation[];
  layer_config_loaded: boolean;
}

export async function checkRedFlags(previousBetti1: number): Promise<RedFlagResult> {
  return await invoke<RedFlagResult>('check_red_flags', { previousBetti1 });
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

// ============================================================================
// Execution Metrics Commands
// ============================================================================

export interface ExecutionMetrics {
  active_atoms: number;
  total_atoms_spawned: number;
  total_tokens: number;
  tokens_per_second: number;
  red_flag_count: number;
  shadow_commits: number;
  last_updated_ms: number;
}

export async function getExecutionMetrics(): Promise<ExecutionMetrics> {
  return await invoke<ExecutionMetrics>('get_execution_metrics');
}

export async function updateExecutionMetrics(
  activeAtoms?: number,
  tokensAdded?: number,
  redFlagsAdded?: number
): Promise<void> {
  await invoke('update_execution_metrics', {
    active_atoms: activeAtoms,
    tokens_added: tokensAdded,
    red_flags_added: redFlagsAdded,
  });
}

export async function recordAtomSpawned(): Promise<void> {
  await invoke('record_atom_spawned');
}

export async function recordAtomCompleted(tokensUsed: number, hadRedFlag: boolean): Promise<void> {
  await invoke('record_atom_completed', { tokensUsed, hadRedFlag });
}

export async function recordShadowCommit(): Promise<void> {
  await invoke('record_shadow_commit');
}

export async function resetExecutionMetrics(): Promise<void> {
  await invoke('reset_execution_metrics');
}

// ============================================================================
// Voting State Commands
// ============================================================================

export interface VotingCandidate {
  id: number;
  snippet: string;
  score: number;
  red_flags: string[];
  status: string; // "pending", "accepted", "rejected"
  votes: number;
}

export interface VotingState {
  task_id: string;
  task_description: string;
  candidates: VotingCandidate[];
  is_voting: boolean;
  winner_id: number | null;
}

export async function getVotingState(): Promise<VotingState> {
  return await invoke<VotingState>('get_voting_state');
}

export async function startVoting(taskId: string, taskDescription: string): Promise<void> {
  await invoke('start_voting', { taskId, taskDescription });
}

export async function addVotingCandidate(
  snippet: string,
  score: number,
  redFlags: string[]
): Promise<number> {
  return await invoke<number>('add_voting_candidate', { snippet, score, redFlags });
}

export async function recordVote(candidateId: number): Promise<void> {
  await invoke('record_vote', { candidateId });
}

export async function completeVoting(winnerId: number): Promise<void> {
  await invoke('complete_voting', { winnerId });
}

export async function clearVotingState(): Promise<void> {
  await invoke('clear_voting_state');
}

// ============================================================================
// Shadow Git Commands - Transactional File System
// ============================================================================

export interface Snapshot {
  id: string;
  message: string;
  timestamp_ms: number;
  commit_hash: string | null;
}

export interface HistoryEntry {
  hash: string;
  message: string;
}

/**
 * Create a snapshot of the current state
 * PRD 5.1: "Before any Rhai script touches disk, gitoxide creates a blob"
 */
export async function createSnapshot(message: string): Promise<Snapshot> {
  return await invoke<Snapshot>('create_snapshot', { message });
}

/**
 * Rollback to the previous snapshot
 * PRD 5.1: "gitoxide reverts the index to the snapshot instantly"
 */
export async function rollbackSnapshot(): Promise<string> {
  return await invoke<string>('rollback_snapshot');
}

/**
 * Rollback to a specific snapshot by ID
 */
export async function rollbackToSnapshot(snapshotId: string): Promise<string> {
  return await invoke<string>('rollback_to_snapshot', { snapshotId });
}

/**
 * Squash all snapshots into a single commit
 * PRD 5.1: "Only when PLAN.md is marked Complete does Shadow Repo squash"
 */
export async function squashSnapshots(finalMessage: string): Promise<{ message: string; commit_hash: string }> {
  return await invoke('squash_snapshots', { finalMessage });
}

/**
 * Get all current snapshots
 */
export async function getSnapshots(): Promise<Snapshot[]> {
  return await invoke<Snapshot[]>('get_snapshots');
}

/**
 * Checkout to a specific git commit (for time travel)
 */
export async function checkoutCommit(commitHash: string): Promise<string> {
  return await invoke<string>('checkout_commit', { commitHash });
}

/**
 * Get git history for Time Machine
 */
export async function getGitHistory(limit: number): Promise<HistoryEntry[]> {
  return await invoke<HistoryEntry[]>('get_git_history', { limit });
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

// ============================================================================
// L3 Context Engineer Commands
// ============================================================================

export interface ContextMetrics {
  seed_count: number;
  symbol_count: number;
  file_count: number;
  estimated_precision: number;
  solid_score: number;
}

export interface SymbolEntry {
  id: string;
  name: string;
  file_path: string;
  kind: string;
  code?: string;
  byte_range?: [number, number];
  pagerank?: number;
  in_cycle: boolean;
}

export interface MiniCodebase {
  seed_issue?: string;
  seed_symbols: string[];
  symbols: SymbolEntry[];
  files: string[];
  invariants: {
    betti_1: number;
    forbidden_dependencies: string[];
    layer_constraints: string[];
    notes: string[];
  };
  metadata: {
    assembled_at: string;
    neighborhood_depth: number;
    strength_threshold: number;
    total_symbols_in_graph: number;
    solid_score: number;
  };
}

// P3-1: RLM context info for large context handling
export interface RLMContextInfo {
  total_length: number;
  context_type: string;
  use_rlm: boolean;
  suggested_chunk_size: number;
  estimated_chunks: number;
  full_context: string;
  context_var_name: string;
}

export interface ContextPackage {
  task_id: string;
  atom_type: string;
  context_lines: number;
  mini_codebase: MiniCodebase;
  markdown: string;
  constraints: string[];
  metrics: ContextMetrics;
  rlm_info?: RLMContextInfo; // P3-1: Added missing optional field
}

/**
 * Extract context for a micro-task using the L3 Context Engineer
 * This loads the symbol graph fresh from the workspace
 */
export async function extractTaskContext(
  taskId: string,
  taskDescription: string,
  atomType: string,
  seedSymbols: string[],
  workspacePath: string
): Promise<ContextPackage> {
  return await invoke<ContextPackage>('extract_task_context', {
    taskId,
    taskDescription,
    atomType,
    seedSymbols,
    workspacePath,
  });
}

/**
 * Extract context using a cached symbol graph (more efficient for batch operations)
 * Requires load_symbol_graph to be called first
 */
export async function extractTaskContextCached(
  taskId: string,
  taskDescription: string,
  atomType: string,
  seedSymbols: string[],
  workspacePath: string
): Promise<ContextPackage> {
  return await invoke<ContextPackage>('extract_task_context_cached', {
    taskId,
    taskDescription,
    atomType,
    seedSymbols,
    workspacePath,
  });
}

/**
 * Get the rendered markdown context for a task (for LLM consumption)
 */
export async function getTaskContextMarkdown(
  taskId: string,
  taskDescription: string,
  atomType: string,
  seedSymbols: string[],
  workspacePath: string
): Promise<string> {
  return await invoke<string>('get_task_context_markdown', {
    taskId,
    taskDescription,
    atomType,
    seedSymbols,
    workspacePath,
  });
}

// ============================================================================
// L4 Atom Execution Commands
// ============================================================================

export interface AtomResult {
  atom_type: string;
  output: string;
  valid: boolean;
  errors: string[];
  execution_time_ms: number;
  tokens_used: number;
  metadata: Record<string, string>;
}

export interface CodeChange {
  file_path: string;
  content: string;
  language?: string;
}

export interface ReviewResult {
  approved: boolean;
  issues: string[];
  suggestions: string[];
}

export interface AtomTypeInfo {
  id: string;
  name: string;
  description: string;
  max_tokens: number;
}

/**
 * Execute a single atom with optional context
 */
export async function executeAtom(
  atomType: string,
  task: string,
  contextPackage?: ContextPackage,
  requireJson = false,
  temperature = 0.1
): Promise<AtomResult> {
  return await invoke<AtomResult>('execute_atom', {
    atomType,
    task,
    contextPackage,
    requireJson,
    temperature,
  });
}

/**
 * Execute an atom with full context extraction (L3 + L4 pipeline)
 */
export async function executeAtomWithContext(
  atomType: string,
  taskId: string,
  taskDescription: string,
  seedSymbols: string[],
  workspacePath: string,
  requireJson = false
): Promise<AtomResult> {
  return await invoke<AtomResult>('execute_atom_with_context', {
    atomType,
    taskId,
    taskDescription,
    seedSymbols,
    workspacePath,
    requireJson,
  });
}

/**
 * Parse code output from a Coder atom into structured changes
 */
export async function parseCoderOutput(rawOutput: string): Promise<CodeChange[]> {
  return await invoke<CodeChange[]>('parse_coder_output', { rawOutput });
}

/**
 * Parse review output from a Reviewer atom
 */
export async function parseReviewerOutput(rawOutput: string): Promise<ReviewResult> {
  return await invoke<ReviewResult>('parse_reviewer_output', { rawOutput });
}

/**
 * Get available atom types
 */
export async function getAtomTypes(): Promise<AtomTypeInfo[]> {
  return await invoke<AtomTypeInfo[]>('get_atom_types');
}

// ============================================================================
// Crawl4AI Commands - Web Crawling & Research
// ============================================================================

export interface CrawlResult {
  url: string;
  status_code: number;
  title: string | null;
  markdown: string | null;
  cleaned_content: string | null;
  duration_ms: number;
}

export interface ResearchResult {
  documents: Array<{
    url: string;
    title: string | null;
    markdown: string | null;
    status_code: number;
  }>;
  errors: Array<{
    url: string;
    error: string;
  }>;
  total_urls: number;
  success_count: number;
  error_count: number;
}

export interface ExtractionResult {
  url: string;
  title: string | null;
  extracted: unknown[];
  count: number;
}

/**
 * Crawl a single URL and return content as markdown
 */
export async function crawlUrl(url: string, convertToMarkdown = true): Promise<CrawlResult> {
  return await invoke<CrawlResult>('crawl_url', { url, convertToMarkdown });
}

/**
 * Research multiple documentation URLs in parallel
 * Useful for gathering context from external APIs, libraries, etc.
 */
export async function researchDocs(urls: string[]): Promise<ResearchResult> {
  return await invoke<ResearchResult>('research_docs', { urls });
}

/**
 * Extract structured content from a URL using CSS or XPath selectors
 * @param url - The URL to crawl
 * @param strategyType - 'css' or 'xpath'
 * @param schema - Extraction schema with baseSelector and fields
 */
export async function extractContent(
  url: string,
  strategyType: 'css' | 'xpath',
  schema: Record<string, unknown>
): Promise<ExtractionResult> {
  return await invoke<ExtractionResult>('extract_content', { url, strategyType, schema });
}

// ============================================================================
// GitHub Integration Hooks
// ============================================================================

export interface GitRemote {
  name: string;
  url: string;
  type: string;
}

export interface GitChange {
  status: string;
  file: string;
}

export interface GitStatus {
  is_clean: boolean;
  changes: GitChange[];
  change_count: number;
}

export async function gitInit(workspacePath: string): Promise<string> {
  return await invoke<string>('git_init', { workspacePath });
}

export async function gitAddRemote(workspacePath: string, name: string, url: string): Promise<string> {
  return await invoke<string>('git_add_remote', { workspacePath, name, url });
}

export async function gitGetRemotes(workspacePath: string): Promise<{ remotes: GitRemote[] }> {
  return await invoke<{ remotes: GitRemote[] }>('git_get_remotes', { workspacePath });
}

export async function gitPush(
  workspacePath: string,
  remote: string,
  branch: string,
  setUpstream: boolean = false
): Promise<string> {
  return await invoke<string>('git_push', { workspacePath, remote, branch, setUpstream });
}

export async function gitCurrentBranch(workspacePath: string): Promise<string> {
  return await invoke<string>('git_current_branch', { workspacePath });
}

export async function gitStatus(workspacePath: string): Promise<GitStatus> {
  return await invoke<GitStatus>('git_status', { workspacePath });
}

export async function gitClone(url: string, targetPath: string): Promise<string> {
  return await invoke<string>('git_clone', { url, targetPath });
}

export async function gitAdd(workspacePath: string, paths: string[]): Promise<string> {
  return await invoke<string>('git_add', { workspacePath, paths });
}

export interface GitCommitResult {
  success: boolean;
  commit_hash?: string;
  message: string;
}

export async function gitCommit(workspacePath: string, message: string): Promise<GitCommitResult> {
  return await invoke<GitCommitResult>('git_commit', { workspacePath, message });
}

export async function gitBranch(workspacePath: string, branchName: string, create: boolean = false): Promise<string> {
  return await invoke<string>('git_branch', { workspacePath, branchName, create });
}

export interface GitBranch {
  name: string;
  commit: string;
  upstream: string;
}

export interface GitBranchList {
  branches: GitBranch[];
  current: string;
}

export async function gitListBranches(workspacePath: string): Promise<GitBranchList> {
  return await invoke<GitBranchList>('git_list_branches', { workspacePath });
}

export async function gitPull(workspacePath: string, remote: string, branch: string, rebase: boolean = false): Promise<string> {
  return await invoke<string>('git_pull', { workspacePath, remote, branch, rebase });
}

export async function gitDiff(workspacePath: string, staged: boolean = false, filePath?: string): Promise<string> {
  return await invoke<string>('git_diff', { workspacePath, staged, filePath });
}

export interface GitCommit {
  hash: string;
  message: string;
  author: string;
  email: string;
  date: string;
}

export interface GitLogResult {
  commits: GitCommit[];
}

export async function gitLog(workspacePath: string, count?: number): Promise<GitLogResult> {
  return await invoke<GitLogResult>('git_log', { workspacePath, count });
}

// ============================================================================
// GitHub Actions & Deployment Commands
// ============================================================================

export interface WorkflowConfig {
  project_type: string;
  node_version?: string;
  rust_version?: string;
  deploy_target?: string;
  run_tests: boolean;
  run_lint: boolean;
}

export interface WorkflowResult {
  success: boolean;
  path: string;
  content: string;
}

export async function generateGithubWorkflow(workspacePath: string, config: WorkflowConfig): Promise<WorkflowResult> {
  return await invoke<WorkflowResult>('generate_github_workflow', { workspacePath, config });
}

export interface DeployConfigResult {
  success: boolean;
  platform: string;
  path: string;
  content: string;
}

export async function generateDeployConfig(workspacePath: string, platform: string): Promise<DeployConfigResult> {
  return await invoke<DeployConfigResult>('generate_deploy_config', { workspacePath, platform });
}

// ============================================================================
// Multi-file Edit Validation Commands
// ============================================================================

export interface MultiFileEdit {
  file_path: string;
  operation: 'create' | 'modify' | 'delete';
  content?: string;
  language?: string;
}

export interface LayerViolation {
  from_symbol: string;
  from_layer: string;
  to_symbol: string;
  to_layer: string;
  message: string;
}

export interface MultiFileValidationResult {
  is_safe: boolean;
  original_betti_1: number;
  new_betti_1: number;
  introduces_cycles: boolean;
  layer_violations: LayerViolation[];
  new_symbols: string[];
  new_dependencies: [string, string, string][];
  warnings: string[];
  errors: string[];
  files_analyzed: number;
  cross_file_issues: string[];
}

export interface EditImpactPreview {
  new_symbols: string[];
  new_dependencies: [string, string, string][];
  files_affected: number;
}

export async function validateMultiFileEdit(
  workspacePath: string,
  edits: MultiFileEdit[]
): Promise<MultiFileValidationResult> {
  return await invoke<MultiFileValidationResult>('validate_multi_file_edit', { workspacePath, edits });
}

export async function previewEditImpact(edits: MultiFileEdit[]): Promise<EditImpactPreview> {
  return await invoke<EditImpactPreview>('preview_edit_impact', { edits });
}

// ============================================================================
// Test Generation & Execution Commands
// ============================================================================

export interface TestFrameworkInfo {
  framework: string;
  test_command: string;
  test_pattern: string;
  config_file: string | null;
}

export interface FailedTest {
  name: string;
  file: string | null;
  error: string;
}

export interface TestExecutionResult {
  success: boolean;
  total_tests: number;
  passed: number;
  failed: number;
  skipped: number;
  duration_ms: number;
  output: string;
  failed_tests: FailedTest[];
}

export interface GeneratedTest {
  test_code: string;
  suggested_file: string;
  source_file: string;
  language: string;
  test_type: string;
}

export async function detectTestFramework(workspacePath: string): Promise<TestFrameworkInfo> {
  return await invoke<TestFrameworkInfo>('detect_test_framework', { workspacePath });
}

export async function runTests(
  workspacePath: string,
  testPattern?: string,
  timeoutSeconds?: number
): Promise<TestExecutionResult> {
  return await invoke<TestExecutionResult>('run_tests', {
    workspacePath,
    testPattern: testPattern ?? null,
    timeoutSeconds: timeoutSeconds ?? null
  });
}

export async function generateTests(
  workspacePath: string,
  sourceFile: string,
  testType?: 'unit' | 'integration' | 'property'
): Promise<GeneratedTest> {
  return await invoke<GeneratedTest>('generate_tests', {
    workspacePath,
    sourceFile,
    testType: testType ?? null
  });
}

// ============================================================================
// Knowledge Base Commands
// ============================================================================

export interface KnowledgeDocument {
  id: string;
  name: string;
  content: string;
  doc_type: string;
  metadata: Record<string, string>;
  auto_classified?: boolean;
  word_count?: number;
}

export interface AutoClassifyResult {
  id: string;
  doc_type: string;
  auto_classified: boolean;
}

export interface KnowledgeBaseStats {
  document_count: number;
  web_research_count: number;
  total_tokens: number;
  documents_by_type: Record<string, number>;
}

export interface WebResearchItem {
  id: string;
  url: string;
  title: string;
  content: string;
  crawled_at: string;
}

export async function kbAddDocument(name: string, content: string, docType: string): Promise<string> {
  return await invoke<string>('kb_add_document', { name, content, docType });
}

/** Add document with auto-classification based on content */
export async function kbAddDocumentAuto(name: string, content: string): Promise<AutoClassifyResult> {
  return await invoke<AutoClassifyResult>('kb_add_document_auto', { name, content });
}

/** Classify document content without adding it to KB */
export async function kbClassifyDocument(content: string, filename: string): Promise<string> {
  return await invoke<string>('kb_classify_document', { content, filename });
}

export async function kbAddWebResearch(url: string, title: string, content: string): Promise<string> {
  return await invoke<string>('kb_add_web_research', { url, title, content });
}

export async function kbRemoveDocument(id: string): Promise<void> {
  return await invoke<void>('kb_remove_document', { id });
}

export async function kbGetDocuments(): Promise<KnowledgeDocument[]> {
  return await invoke<KnowledgeDocument[]>('kb_get_documents');
}

export async function kbCompileContext(): Promise<string> {
  return await invoke<string>('kb_compile_context');
}

/** Compile context with token budget limit */
export async function kbCompileContextWithBudget(maxTokens: number): Promise<string> {
  return await invoke<string>('kb_compile_context_with_budget', { maxTokens });
}

/** Compile context optimized for L1 Interrogator */
export async function kbCompileForInterrogator(): Promise<string> {
  return await invoke<string>('kb_compile_for_interrogator');
}

/** Get knowledge base statistics */
export async function kbGetStats(): Promise<KnowledgeBaseStats> {
  return await invoke<KnowledgeBaseStats>('kb_get_stats');
}

// ============================================================================
// Session Persistence Commands
// ============================================================================

export interface SessionData {
  id: string;
  name: string;
  workspace_path: string;
  prd_content: string | null;
  prd_filename: string | null;
  conversation_history: unknown[];
  plan_content: string | null;
  kb_documents: unknown[];
  current_view: string;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface SessionSummary {
  id: string;
  name: string;
  workspace_path: string;
  created_at_ms: number;
  updated_at_ms: number;
  has_prd: boolean;
  has_plan: boolean;
  message_count: number;
  kb_document_count: number;
}

/** Save current session state */
export async function saveSession(
  sessionName: string,
  workspacePath: string,
  prdContent: string | null,
  prdFilename: string | null,
  conversationHistory: unknown[],
  planContent: string | null,
  currentView: string
): Promise<SessionData> {
  return await invoke<SessionData>('save_session', {
    sessionName,
    workspacePath,
    prdContent,
    prdFilename,
    conversationHistory,
    planContent,
    currentView,
  });
}

/** Update an existing session */
export async function updateSession(
  sessionId: string,
  prdContent: string | null,
  conversationHistory: unknown[],
  planContent: string | null,
  currentView: string
): Promise<SessionData> {
  return await invoke<SessionData>('update_session', {
    sessionId,
    prdContent,
    conversationHistory,
    planContent,
    currentView,
  });
}

/** Load a session by ID */
export async function loadSession(sessionId: string): Promise<SessionData> {
  return await invoke<SessionData>('load_session', { sessionId });
}

/** List all saved sessions */
export async function listSessions(): Promise<SessionSummary[]> {
  return await invoke<SessionSummary[]>('list_sessions');
}

/** Delete a session by ID */
export async function deleteSession(sessionId: string): Promise<void> {
  await invoke('delete_session', { sessionId });
}
