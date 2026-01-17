# Tauri Command API Reference

This document provides a comprehensive reference for all Tauri commands exposed by the Cerebras-MAKER backend.

## Grits Commands

### `load_symbol_graph`
Load and analyze the symbol graph for a workspace.

```typescript
await invoke('load_symbol_graph', { workspacePath: string }): Promise<SymbolGraph>
```

**Parameters:**
- `workspacePath` - Absolute path to the workspace directory

**Returns:** `SymbolGraph` object containing nodes and edges

---

### `assemble_mini_codebase`
Assemble a minimal codebase context from seed symbols.

```typescript
await invoke('assemble_mini_codebase', {
  workspacePath: string,
  seedSymbols: string[],
  maxDepth: number,
  strengthThreshold: number
}): Promise<MiniCodebase>
```

**Parameters:**
- `workspacePath` - Workspace path
- `seedSymbols` - Array of symbol IDs to start from
- `maxDepth` - Maximum traversal depth (1-2 recommended)
- `strengthThreshold` - Minimum edge strength (0.0-1.0)

---

### `check_red_flags`
Check proposed changes for architectural violations.

```typescript
await invoke('check_red_flags', {
  workspacePath: string,
  changes: CodeChange[]
}): Promise<RedFlagResult>
```

**Parameters:**
- `workspacePath` - Workspace path
- `changes` - Array of proposed code changes

**Returns:** Object with `passed: boolean` and `violations: string[]`

---

### `check_proposed_changes`
Validate proposed changes against the symbol graph.

```typescript
await invoke('check_proposed_changes', {
  workspacePath: string,
  changes: CodeChange[]
}): Promise<ValidationResult>
```

---

### `analyze_topology`
Perform topology analysis on the codebase.

```typescript
await invoke('analyze_topology', { workspacePath: string }): Promise<TopologyResult>
```

**Returns:** Betti numbers, cycle information, and SOLID scores

---

## MAKER Runtime Commands

### `init_runtime`
Initialize the Rhai scripting runtime.

```typescript
await invoke('init_runtime', { workspacePath: string }): Promise<string>
```

---

### `execute_script`
Execute a Rhai script in the MAKER runtime.

```typescript
await invoke('execute_script', { script: string }): Promise<ExecutionResult>
```

**Parameters:**
- `script` - Rhai script content

**Returns:** Execution result with output and any errors

---

### `get_execution_log`
Get the execution log from the runtime.

```typescript
await invoke('get_execution_log'): Promise<ExecutionEvent[]>
```

---

### `get_cwd`
Get the current working directory.

```typescript
await invoke('get_cwd'): Promise<string>
```

---

## Shadow Git Commands

### `create_snapshot`
Create a named snapshot (checkpoint).

```typescript
await invoke('create_snapshot', {
  workspacePath: string,
  message: string
}): Promise<Snapshot>
```

**Returns:** `Snapshot` with `id`, `message`, `timestamp_ms`, `commit_hash`

---

### `rollback_snapshot`
Rollback to the most recent snapshot.

```typescript
await invoke('rollback_snapshot', { workspacePath: string }): Promise<string>
```

---


---

## Template Commands

### `list_templates`
List available project templates.

```typescript
await invoke('list_templates'): Promise<Template[]>
```

---

### `create_from_template`
Create a new project from a template.

```typescript
await invoke('create_from_template', {
  templateId: string,
  targetPath: string,
  projectName: string
}): Promise<string>
```

---

## L2 Orchestrator Commands

### `generate_execution_script`
Generate a Rhai execution script from a plan.

```typescript
await invoke('generate_execution_script', { plan: Plan }): Promise<string>
```

**Returns:** Generated Rhai script content

---

### `parse_plan`
Parse a PLAN.md file into structured format.

```typescript
await invoke('parse_plan', { planContent: string }): Promise<Plan>
```

---

## L3 Context Engineer Commands

### `extract_task_context`
Extract context for a specific task.

```typescript
await invoke('extract_task_context', {
  workspacePath: string,
  taskId: string,
  atomType: string,
  seedSymbols: string[],
  description: string
}): Promise<ContextPackage>
```

**Returns:** `ContextPackage` with `mini_codebase`, `markdown`, `constraints`, `metrics`, `rlm_info`

---

### `extract_task_context_cached`
Extract context with caching support.

```typescript
await invoke('extract_task_context_cached', {
  workspacePath: string,
  taskId: string,
  atomType: string,
  seedSymbols: string[],
  description: string
}): Promise<ContextPackage>
```

---

### `get_task_context_markdown`
Get rendered markdown for a task context.

```typescript
await invoke('get_task_context_markdown', { taskId: string }): Promise<string>
```

---

## L4 Atom Execution Commands

### `execute_atom`
Execute an atom with the given input.

```typescript
await invoke('execute_atom', {
  atomType: string,
  task: string,
  context: string
}): Promise<AtomOutput>
```

**Parameters:**
- `atomType` - One of: `Search`, `Coder`, `Reviewer`, `Planner`, `Validator`, `Tester`, `Architect`, `GritsAnalyzer`, `RLMProcessor`
- `task` - Task description
- `context` - Context markdown

**Returns:** `AtomOutput` with `output`, `changes`, `review_result`, `validation_result`

---

### `execute_atom_with_context`
Execute an atom with pre-extracted context package.

```typescript
await invoke('execute_atom_with_context', {
  atomType: string,
  task: string,
  contextPackage: ContextPackage
}): Promise<AtomOutput>
```

---

### `parse_coder_output`
Parse coder atom output into structured changes.

```typescript
await invoke('parse_coder_output', { output: string }): Promise<CodeChange[]>
```

---

### `parse_reviewer_output`
Parse reviewer atom output.

```typescript
await invoke('parse_reviewer_output', { output: string }): Promise<ReviewResult>
```

---

### `get_atom_types`
Get list of available atom types.

```typescript
await invoke('get_atom_types'): Promise<string[]>
```

---

## RLM (Recursive Language Model) Commands

### `init_rlm_store`
Initialize the RLM context store.

```typescript
await invoke('init_rlm_store'): Promise<string>
```

---

### `rlm_load_context`
Load a context variable into the RLM store.

```typescript
await invoke('rlm_load_context', {
  varName: string,
  content: string
}): Promise<{ length: number }>
```

---

### `rlm_peek_context`
Peek at a slice of context.

```typescript
await invoke('rlm_peek_context', {
  varName: string,
  start: number,
  end: number
}): Promise<string>
```

---

### `rlm_context_length`
Get the length of a context variable.

```typescript
await invoke('rlm_context_length', { varName: string }): Promise<number>
```

---

### `rlm_chunk_context`
Split context into chunks.

```typescript
await invoke('rlm_chunk_context', {
  varName: string,
  chunkSize: number
}): Promise<string[]>
```

---

### `rlm_regex_filter`
Filter context by regex pattern.

```typescript
await invoke('rlm_regex_filter', {
  varName: string,
  pattern: string
}): Promise<{ num_matches: number, matches: string[] }>
```

---

### `execute_rlm_atom`
Execute an RLM-aware atom with iterative loop.

```typescript
await invoke('execute_rlm_atom', {
  atomType: string,
  task: string,
  contextVar: string,
  maxIterations: number
}): Promise<RLMResult>
```

**Returns:** `RLMResult` with `success`, `output`, `iterations`, `sub_calls`, `trajectory`

---

### `get_rlm_trajectory`
Get the RLM execution trajectory for visualization.

```typescript
await invoke('get_rlm_trajectory'): Promise<RLMTrajectoryStep[]>
```

---

### `clear_rlm_trajectory`
Clear the RLM trajectory.

```typescript
await invoke('clear_rlm_trajectory'): Promise<string>
```

---

### `rlm_list_contexts`
List all context variables in the RLM store.

```typescript
await invoke('rlm_list_contexts'): Promise<string[]>
```

---

### `rlm_clear_context`
Clear a context variable from the store.

```typescript
await invoke('rlm_clear_context', { varName: string }): Promise<boolean>
```

---

### `get_rlm_config`
Get RLM configuration.

```typescript
await invoke('get_rlm_config'): Promise<RLMConfig>
```

**Returns:**
```typescript
{
  max_depth: number,      // Default: 1
  max_iterations: number, // Default: 20
  default_chunk_size: number, // Default: 10000
  rlm_threshold: number,  // Default: 50000
  use_sub_model: boolean
}
```

---

### `should_use_rlm`
Check if context size exceeds RLM threshold.

```typescript
await invoke('should_use_rlm', { contextLength: number }): Promise<boolean>
```

---

## Type Definitions

### `CodeChange`
```typescript
interface CodeChange {
  file_path: string;
  operation: 'create' | 'modify' | 'delete';
  content?: string;
  diff?: string;
}
```

### `Snapshot`
```typescript
interface Snapshot {
  id: string;
  message: string;
  timestamp_ms: number;
  commit_hash: string;
}
```

### `ContextPackage`
```typescript
interface ContextPackage {
  task_id: string;
  atom_type: string;
  context_lines: number;
  mini_codebase: MiniCodebase;
  markdown: string;
  constraints: string[];
  metrics: ContextMetrics;
  rlm_info?: RLMContextInfo;
}
```

### `AtomOutput`
```typescript
interface AtomOutput {
  output: string;
  changes?: CodeChange[];
  review_result?: ReviewResult;
  validation_result?: ValidationResult;
}
```

## Settings Commands

### `save_settings`
Save application settings.

```typescript
await invoke('save_settings', { settings: AppSettings }): Promise<string>
```

---

### `load_settings`
Load application settings.

```typescript
await invoke('load_settings'): Promise<AppSettings>
```

**Returns:** `AppSettings` object with API keys and preferences

---

## L1 Interrogation Commands

### `analyze_prd`
Analyze a PRD document.

```typescript
await invoke('analyze_prd', { prdContent: string }): Promise<PRDAnalysis>
```

---

### `send_interrogation_message`
Send a message during interrogation.

```typescript
await invoke('send_interrogation_message', {
  sessionId: string,
  message: string
}): Promise<InterrogationResponse>
```

---

### `complete_interrogation`
Complete the interrogation phase.

```typescript
await invoke('complete_interrogation', { sessionId: string }): Promise<Plan>
```

