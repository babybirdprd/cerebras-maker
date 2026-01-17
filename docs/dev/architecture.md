# Cerebras-MAKER Architecture

## Overview

Cerebras-MAKER implements the **MAKER Framework** (Massively Atomized Knowledge-driven Execution Runtime) - a 4-level hierarchical autonomous coding agent system built on Tauri v2 + React.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    L1: Product Orchestrator                      │
│         User requirements → PLAN.md decomposition                │
│                    (Interrogation Phase)                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  L2: Technical Orchestrator                      │
│              Rhai scripting runtime for task flow                │
│                   (CodeModeRuntime engine)                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    L3: Context Engineer                          │
│         grits-core topology analysis → MiniCodebase              │
│              Semantic tree-shaking (~50 lines)                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        L4: Atoms                                 │
│           Individual LLM-powered code operations                 │
│              with first-to-ahead-by-k voting                     │
└─────────────────────────────────────────────────────────────────┘
```

## The Four Layers

### L1: Product Orchestrator (Interrogation)

The entry point for user requirements. Analyzes PRD documents and conducts an interrogation phase to clarify ambiguities before generating a structured PLAN.md.

**Key Files:**
- `src-tauri/src/agents/interrogator.rs` - Interrogation agent
- `src-tauri/src/agents/architect.rs` - Architecture analysis

**Tauri Commands:** `analyze_prd`, `send_interrogation_message`, `complete_interrogation`

### L2: Technical Orchestrator (Rhai Runtime)

Executes task orchestration via Rhai scripts. The `CodeModeRuntime` provides a sandboxed scripting environment with access to MAKER APIs.

**Key Files:**
- `src-tauri/src/maker_core/runtime.rs` - Rhai execution engine
- `src-tauri/src/generators/rhai_generator.rs` - Script generation

**Registered Rhai Functions:**
- `spawn_atom(type, task, context)` - Execute an L4 atom
- `run_consensus(type, task, context, k)` - Run voting consensus
- `check_red_flags(changes)` - Architectural validation
- `snapshot(message)` / `rollback()` - Version control

**Tauri Commands:** `init_runtime`, `execute_script`, `get_execution_log`

### L3: Context Engineer

Extracts minimal, precise context for L4 atoms using grits-core topology analysis. Implements "semantic tree-shaking" to provide ~50 lines of relevant code.

**Key Files:**
- `src-tauri/src/agents/context_engineer.rs` - Context extraction
- `src-tauri/grits-core/` - Topology analysis library

**Features:**
- Symbol graph traversal (1-hop direct deps, 2-hop signatures)
- Betti number analysis for cycle detection
- MiniCodebase assembly with invariants
- RLM integration for large contexts (>50K chars)

**Tauri Commands:** `extract_task_context`, `extract_task_context_cached`, `get_task_context_markdown`

### L4: Atoms

Individual LLM-powered operations. Each atom receives focused context and produces structured output.

**Atom Types:**
| Type | Purpose |
|------|---------|
| `Search` | Find relevant code/information |
| `Coder` | Generate/modify code |
| `Reviewer` | Review code changes |
| `Planner` | Create execution plans |
| `Validator` | Validate outputs |
| `Tester` | Generate tests |
| `Architect` | Architectural analysis |
| `GritsAnalyzer` | Topology analysis |
| `RLMProcessor` | Handle large contexts |

**Key Files:**
- `src-tauri/src/agents/atom_executor.rs` - Atom execution
- `src-tauri/src/maker_core/voting.rs` - Consensus voting

**Tauri Commands:** `execute_atom`, `execute_atom_with_context`, `get_atom_types`

## Dual-Graph System

MAKER maintains two distinct graphs:

### Planning Graph
- Generated during L1 interrogation
- Represents task dependencies and execution order
- Stored in PLAN.md format

### Execution Graph
- Built during L2 runtime execution
- Tracks actual execution flow and results
- Supports rollback and replay

## Shadow Git

Transactional file system using gitoxide (gix) for version control operations.

**Key Files:** `src-tauri/src/maker_core/shadow_git.rs`

**Features:**
- `snapshot(message)` - Create named checkpoint
- `rollback()` - Revert to last snapshot
- `rollback_to(id)` - Revert to specific snapshot
- `squash(from, to, message)` - Combine snapshots
- Automatic rollback on script failure

**Tauri Commands:** `create_snapshot`, `rollback_snapshot`, `rollback_to_snapshot`, `squash_snapshots`, `get_snapshots`

## RLM (Recursive Language Model)

Pattern for handling arbitrarily long contexts through iterative exploration.

**Key Files:** `src-tauri/src/maker_core/rlm.rs`

**Operations:**
- `peek(var, start, end)` - View context slice
- `chunk(var, size)` - Split into chunks
- `regex_filter(var, pattern)` - Filter by pattern
- `llm_query(prompt)` - Sub-query execution
- `final(answer)` - Return result

**Configuration:**
- `rlm_threshold`: 50,000 characters (triggers RLM mode)
- `max_iterations`: 20 (iteration limit)
- `max_depth`: 1 (sub-query depth)
- `default_chunk_size`: 10,000 characters

**Tauri Commands:** `init_rlm_store`, `rlm_load_context`, `rlm_peek_context`, `rlm_chunk_context`, `rlm_regex_filter`, `execute_rlm_atom`

## Technology Stack

### Backend (Rust)
- **Tauri v2** - Desktop application framework
- **cerebras-rs** - Native Cerebras API client
- **rig-core** - LLM abstraction (OpenAI, Anthropic)
- **rhai** - Embedded scripting language
- **grits-core** - Custom topology analysis
- **gix (gitoxide)** - Git operations
- **ast-grep-core** - AST manipulation
- **tokio** - Async runtime

### Frontend (TypeScript)
- **React 19** - UI framework
- **Zustand** - State management
- **react-three-fiber** - 3D visualization
- **Tailwind CSS v4** - Styling
- **Vite** - Build tool

