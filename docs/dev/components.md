# Component Guide

This guide documents the core components of Cerebras-MAKER.

## Backend Components (Rust)

### AtomExecutor

**Location:** `src-tauri/src/agents/atom_executor.rs`

The L4 Atom Executor handles individual LLM-powered operations. Each atom receives focused context and produces structured output.

#### Key Structures

```rust
pub struct AtomInput {
    pub atom_type: AtomType,
    pub task: String,
    pub context: String,
    pub flags: SpawnFlags,
}

pub struct AtomOutput {
    pub output: String,
    pub changes: Vec<CodeChange>,
    pub review_result: Option<ReviewResult>,
    pub validation_result: Option<ValidationResult>,
}

pub struct SpawnFlags {
    pub require_json: bool,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub red_flag_check: bool,
}
```

#### Atom Types

| Type | Purpose | Output Format |
|------|---------|---------------|
| `Search` | Find code/information | Text |
| `Coder` | Generate/modify code | `CodeChange[]` |
| `Reviewer` | Review changes | `ReviewResult` |
| `Planner` | Create plans | Text |
| `Validator` | Validate outputs | `ValidationResult` |
| `Tester` | Generate tests | `CodeChange[]` |
| `Architect` | Architecture analysis | Text |
| `GritsAnalyzer` | Topology analysis | JSON |
| `RLMProcessor` | Large context handling | JSON |

#### Red-Flag Checking

The executor performs architectural validation using `VirtualApply`:
- Detects dangerous patterns (`rm -rf`, `DROP TABLE`, `eval`, etc.)
- Validates against symbol graph constraints
- Checks Betti number preservation

---

### ContextEngineer

**Location:** `src-tauri/src/agents/context_engineer.rs`

The L3 Context Engineer extracts minimal, precise context for atoms using grits-core topology analysis.

#### Key Structures

```rust
pub struct ContextConfig {
    pub max_depth: usize,           // Default: 2
    pub strength_threshold: f32,    // Default: 0.5
    pub target_lines: usize,        // Default: 50
    pub signatures_only_2hop: bool, // Default: true
}

pub struct ContextPackage {
    pub task_id: String,
    pub atom_type: String,
    pub context_lines: usize,
    pub mini_codebase: MiniCodebase,
    pub markdown: String,
    pub constraints: Vec<String>,
    pub metrics: ContextMetrics,
    pub rlm_info: Option<RLMContextInfo>,
}
```

#### Context Extraction Flow

1. Load/use pre-loaded SymbolGraph
2. Determine seed symbols (from task or inferred)
3. Traverse graph with depth/strength limits
4. Assemble MiniCodebase with invariants
5. Hydrate with actual code content
6. Render to markdown for LLM consumption
7. Check if RLM mode needed (>50K chars)

#### Atom-Specific Requirements

Different atom types receive tailored context:
- **Coder**: Include test examples
- **Reviewer**: Include tests + style guide
- **Tester**: Include test examples
- **GritsAnalyzer**: Include dependency info

---

### CodeModeRuntime

**Location:** `src-tauri/src/maker_core/runtime.rs`

The L2 Technical Orchestrator - a Rhai scripting engine for task orchestration.

#### Registered MAKER API

```rhai
// Atom execution
spawn_atom(type, task, context)      // Execute single atom
run_consensus(type, task, context, k) // Run voting consensus

// Validation
check_red_flags(changes)             // Architectural validation

// Version control
snapshot(message)                    // Create checkpoint
rollback()                           // Revert to last snapshot
rollback_to(id)                      // Revert to specific snapshot
```

#### Registered RLM API

```rhai
// Context management
load_context_var(name, content)      // Load context variable
peek_context(name, start, end)       // View slice
context_length(name)                 // Get length
chunk_context(name, size)            // Split into chunks
regex_filter(name, pattern)          // Filter by regex

// LLM operations
llm_query(prompt)                    // Direct LLM call
spawn_rlm(type, task, context_var)   // RLM-aware atom
```

#### Execution Events

The runtime emits events for visualization:


---

### Voting (Consensus)

**Location:** `src-tauri/src/maker_core/voting.rs`

Implements first-to-ahead-by-k voting for atom consensus.

#### Key Structures

```rust
pub struct ConsensusConfig {
    pub k_threshold: usize,      // Votes ahead to win (default: 2)
    pub max_atoms: usize,        // Max atoms to spawn (default: 5)
    pub timeout_ms: u64,         // Timeout per atom
    pub parallel_batch_size: usize, // Parallel execution (default: 3)
}

pub struct ConsensusResult {
    pub winner: Option<AtomOutput>,
    pub votes: HashMap<String, usize>,
    pub candidates: Vec<AtomOutput>,
    pub discarded_count: usize,
    pub iterations: usize,
}
```

#### Algorithm

1. Spawn atoms in parallel batches
2. Collect outputs and hash for deduplication
3. Vote on each unique output
4. Check if any candidate is ahead by k votes
5. If no winner, spawn more atoms (up to max)
6. Return winner or highest-voted candidate

---

### LLM Provider

**Location:** `src-tauri/src/llm/provider.rs`

Unified interface for LLM operations across providers.

#### Supported Providers

| Provider | Client | Default Model |
|----------|--------|---------------|
| OpenAI | rig-core | `gpt-4o` |
| Anthropic | rig-core | `claude-sonnet-4-20250514` |
| Cerebras | cerebras-rs | `llama-4-scout-17b-16e-instruct` |
| OpenRouter | rig-core | `anthropic/claude-sonnet-4` |
| OpenAI-Compatible | rig-core | Custom |

#### Configuration

```rust
pub struct LlmConfig {
    pub provider: ProviderType,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}
```

---

## Frontend Components (React)

### State Management

**Location:** `src/store/makerStore.ts`

Zustand store managing application state:
- Current workspace path
- Execution state and logs
- Symbol graph data
- Snapshots and history
- Settings and API keys

### Key UI Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `Dashboard` | `src/components/Dashboard.tsx` | Main application view |
| `Cockpit` | `src/components/Cockpit.tsx` | Execution control panel |
| `GraphView` | `src/components/GraphView.tsx` | Symbol graph visualization |
| `TimeMachine` | `src/components/TimeMachine.tsx` | Snapshot/history navigation |
| `ExecutionPanel` | `src/components/ExecutionPanel.tsx` | Script execution UI |
| `PlanView` | `src/components/PlanView.tsx` | PLAN.md visualization |
| `RLMTrajectory` | `src/components/RLMTrajectory.tsx` | RLM execution visualization |
| `Settings` | `src/components/Settings.tsx` | API key configuration |
| `PRDUpload` | `src/components/PRDUpload.tsx` | PRD document upload |
| `Blueprint` | `src/components/Blueprint.tsx` | 3D visualization (react-three-fiber) |

### Tauri Integration

**Location:** `src/hooks/useTauri.ts`

Custom hook for invoking Tauri commands:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Example usage
const result = await invoke('execute_atom', {
  atomType: 'Coder',
  task: 'Add error handling',
  context: contextMarkdown
});
```

---

## grits-core Library

**Location:** `src-tauri/grits-core/`

Custom topology analysis library for semantic tree-shaking.

### Key Features

- **SymbolGraph**: Directed graph of code symbols and dependencies
- **MiniCodebase**: Minimal context extraction with invariants
- **Betti Numbers**: Topological analysis for cycle detection
- **SOLID Scoring**: Code quality metrics

### Core Types

```rust
pub struct SymbolGraph {
    pub nodes: HashMap<String, Symbol>,
    pub edges: Vec<Edge>,
}

pub struct MiniCodebase {
    pub symbols: Vec<SymbolInfo>,
    pub files: Vec<FileInfo>,
    pub invariants: Invariants,
    pub metadata: Metadata,
}

pub struct Invariants {
    pub betti_1: usize,           // Cycle count
    pub notes: Vec<String>,       // Constraints
    pub forbidden_dependencies: Vec<String>,
}
```

**Location:** `src-tauri/src/maker_core/shadow_git.rs`

Transactional file system using gitoxide (gix) for version control.

#### Key Structure

```rust
pub struct Snapshot {
    pub id: String,
    pub message: String,
    pub timestamp_ms: u64,
    pub commit_hash: String,
}

pub struct ShadowGit {
    repo: gix::Repository,
    snapshots: Vec<Snapshot>,
}
```

#### Operations

| Method | Description |
|--------|-------------|
| `init(path)` | Initialize for workspace |
| `snapshot(msg)` | Create named checkpoint |
| `rollback()` | Revert to last snapshot |
| `rollback_to(id)` | Revert to specific snapshot |
| `squash(from, to, msg)` | Combine snapshots |
| `get_history(limit)` | Get commit history |

#### Automatic Rollback

The runtime automatically rolls back on script failure, ensuring atomic execution.

