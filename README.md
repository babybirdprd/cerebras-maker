# Cerebras MAKER

**Massively Atomized Knowledge-driven Execution Runtime**

An autonomous coding framework that transforms Product Requirements Documents (PRDs) into working software through intelligent task decomposition, multi-agent collaboration, and topology-aware code generation.

---

## 🎯 Vision

You describe what you want to build. MAKER builds it.

```
User: "Build me a CLI tool that converts markdown to HTML with syntax highlighting"
MAKER: *analyzes requirements* → *decomposes into micro-tasks* → *generates code* → *validates* → *commits*
```

Unlike traditional AI coding assistants that operate on single prompts, MAKER orchestrates a **swarm of specialized agents** that collaborate to deliver complete, production-ready software.

---

## 🏗️ Core Architecture: The Quad-Level Context Funnel

To achieve "Zero Error" coding, tasks pass through **4 distinct layers of resolution**. Each layer strips away ambiguity before passing a stricter task to the layer below.

### The 4 Levels

| Level | Agent Role | Analogy | Input → Output | Responsibility |
|-------|------------|---------|----------------|----------------|
| **L1** | Product Orchestrator | "The PM" | User Prompt → `PLAN.md` | **"The What"** - Resolves ambiguity, defines scope |
| **L2** | Technical Orchestrator | "Staff Engineer" | `PLAN.md` → `script.rhai` | **"The How"** - Converts requirements into control flow |
| **L3** | Context Engineer | "The Librarian" | Rhai task → `MiniCodebase` | **"The Where"** - Hydrates only necessary ~50 lines |
| **L4** | The Atom | "Intern on Coffee" | `MiniCodebase` → `Result<JSON>` | **"The Do"** - Stateless, 1 tool, 1 task, 100% focus |

### Data Flow (Context Reduction Waterfall)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USER INPUT                                      │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│  │    PRD      │    │  Template   │    │  Mid-Task   │                      │
│  │  (New Proj) │    │  Selection  │    │  Addition   │                      │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                      │
└─────────┼──────────────────┼──────────────────┼─────────────────────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  L1: PRODUCT ORCHESTRATOR (System 2 - Planning)                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  • Analyzes requirements         • Asks clarifying questions        │    │
│  │  • Identifies ambiguities        • Creates PLAN.md                  │    │
│  │  • Rejects impossible requests   • Defines success criteria         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│  Output: PLAN.md (Strategy Document)                                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  L2: TECHNICAL ORCHESTRATOR (System 2 - Logic)                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  • Translates PLAN.md → Rhai     • Knows architecture, not syntax   │    │
│  │  • Defines control flow          • Spawns atoms via spawn_atom()    │    │
│  │  • Enforces red-flag checks      • Manages variable passing         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│  Output: script.rhai (Executable Logic)                                      │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                         ┌────────────┴────────────┐
                         ▼                         ▼
              ┌──────────────────┐      ┌──────────────────┐
              │   Rhai Runtime   │◄────►│   Shadow Git     │
              │  (Sandboxed VM)  │      │   (gitoxide)     │
              └────────┬─────────┘      └──────────────────┘
                       │
          spawn_atom() │ calls
                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  L3: CONTEXT ENGINEER (System 1 - Hydration)                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  • Receives task from Rhai       • Uses Grits for topology analysis │    │
│  │  • Smart context pruning         • Extracts only relevant symbols   │    │
│  │  • Strips unrelated code         • Builds MiniCodebase (~50 lines)  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│  Output: MiniCodebase (Surgical Context)                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  L4: THE ATOM LAYER (System 1 - Execution)                                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐           │
│  │ Coder   │  │Reviewer │  │ Tester  │  │Architect│  │ Grits   │           │
│  │  Atom   │  │  Atom   │  │  Atom   │  │  Atom   │  │ Analyzer│           │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘           │
│       │            │            │            │            │                 │
│  Stateless • Hyper-focused • 1 Tool per Atom • JSON Output Only             │
│       │            │            │            │            │                 │
│       └────────────┴────────────┼────────────┴────────────┘                 │
│                                 ▼                                           │
│                    ┌─────────────────────────┐                              │
│                    │  First-to-Ahead-by-K    │                              │
│                    │     Voting Consensus    │                              │
│                    └─────────────────────────┘                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                         Results flow back to Rhai
                                      │
                                      ▼
                    ┌─────────────────────────────┐
                    │  if grits.validate(code) {  │
                    │      shadow.commit(code);   │
                    │  } else {                   │
                    │      retry_or_escalate();   │
                    │  }                          │
                    └─────────────────────────────┘
```

### Why 4 Levels? (Accuracy Analysis)

| Depth | Error Rate | Why It Fails |
|-------|------------|--------------|
| **2 Levels** | ~15% | "Manager" hallucinates file paths (holds whole codebase in head) |
| **3 Levels** | ~5% | "Tech Lead" misses subtle dependency conflicts (circular imports) |
| **4 Levels** | ~0% | **L3 guarantees** Atom cannot see irrelevant code; **L2 enforces** red-flag checks before commit |

---

## 🔄 User Experience Flow

### Stage 1: PRD Upload & Refinement

**Greenfield (New Project)**
```
┌─────────────────────────────────────────────────────────────────┐
│  User uploads PRD file (.md, .txt, .pdf) or rough outline       │
│  ───────────────────────────────────────────────────────────── │
│                              ↓                                  │
│  L1 (Product Orchestrator) analyzes and identifies gaps         │
│  ───────────────────────────────────────────────────────────── │
│                              ↓                                  │
│  💬 Interrogation Phase (Chat-style Q&A)                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ PM Agent: "I see you want a Tauri app. A few questions: │   │
│  │           1. What's the primary use case?               │   │
│  │           2. Do you need offline support?               │   │
│  │           3. Target platforms (Windows/Mac/Linux)?"     │   │
│  │                                                         │   │
│  │ User: "Task management, yes offline, Windows first"     │   │
│  │                                                         │   │
│  │ PM Agent: "Got it. Database preference for offline?     │   │
│  │           SQLite, IndexedDB, or file-based?"            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              ↓                                  │
│  L1 outputs: PLAN.md (fully-defined requirements)               │
└─────────────────────────────────────────────────────────────────┘
```

The PRD doesn't need to be complete—L1 refines it through conversation until all ambiguity is resolved.

**Brownfield (Existing Codebase)**
```
User: *Opens existing project folder*
MAKER: *Indexes with Grits*
       "Analyzed 847 symbols across 92 files.
        Detected: Rust + Tauri + React + SQLite
        What would you like to modify?"

User: *Uploads change request or describes modification*
L1:   *Analyzes impact on existing codebase*
      "This will affect 3 modules. Confirming scope..."
```

### Stage 2: Active Development (Plan → Execute → Validate)

The UI adapts based on workflow stage:

| Stage | Primary View | User Action |
|-------|--------------|-------------|
| **Interrogation** | Chat interface | Answer L1's questions |
| **Planning** | Task tree + Dependency graph | Review/approve PLAN.md |
| **Execution** | Voting arena + Progress | Watch atoms compete |
| **Validation** | Topology + Red flags | Review Grits analysis |
| **History** | Time slider | Scrub through commits |

### Stage 3: Mid-Session Task Addition

Users can add/modify tasks anytime during execution:
```
User: "Actually, also add dark mode support"
L1:   *Analyzes new requirement*
      "This requires: theme context, CSS variables, toggle component.
       Adding 3 new tasks to the plan..."
L2:   *Regenerates Rhai script with new tasks*
      *Preserves completed work, inserts new branch*
```

---

## 📦 Template System

Pre-configured stacks for instant project scaffolding:

| Template | Stack | Description |
|----------|-------|-------------|
| `tauri-react` | Tauri + React + TypeScript + Tailwind | Desktop app (like this one) |
| `tauri-solid` | Tauri + SolidJS + TypeScript | Lightweight desktop app |
| `axum-api` | Axum + SQLx + PostgreSQL | REST API backend |
| `cli-clap` | Rust + Clap + Tokio | Command-line tool |
| `fullstack` | Tauri + React + Axum + SQLite | Complete application |

Templates include:
- Project structure
- Recommended dependencies
- Common patterns (auth, config, logging)
- Agent configuration presets

---

## ⚙️ Rhai: The L2 Output & Dynamic Tool System

Rhai scripts are **the output of the Technical Orchestrator (L2)**, not user-written code. They serve two critical purposes:

1. **Architectural Control Flow** - The "Staff Engineer" writes logic that knows *architecture*, not *syntax*
2. **Dynamic Tool Loading** - Atoms only get tools they're *allowed* to use for that specific task

### The L2 → Rhai → L3 → L4 Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  L2: Technical Orchestrator                                      │
│  Input: PLAN.md ("Add rate limiting with Redis")                │
│  Output: script.rhai                                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  script.rhai (L2's Architectural Logic)                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  let middleware_file = "src/middleware/rate_limit.rs";    │  │
│  │                                                           │  │
│  │  // Step 1: Define the Struct                             │  │
│  │  let struct_def = spawn_atom(                             │  │
│  │      AtomType::Architect,                                 │  │
│  │      "Define RateLimiter struct for Redis"                │  │
│  │  );                                                       │  │
│  │                                                           │  │
│  │  // Step 2: Check for collisions (Red Flagging)           │  │
│  │  if grits.has_symbol(struct_def.name) {                   │  │
│  │      throw "Struct collision detected";                   │  │
│  │  }                                                        │  │
│  │                                                           │  │
│  │  // Step 3: Implementation                                │  │
│  │  let impl_code = spawn_atom(                              │  │
│  │      AtomType::Coder,                                     │  │
│  │      "Impl RateLimiter using redis-rs"                    │  │
│  │  );                                                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
              spawn_atom() triggers L3
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  L3: Context Engineer                                            │
│  Task: "Impl RateLimiter using redis-rs"                        │
│  Grits Action:                                                   │
│    • Finds `use redis::Client;` in Cargo.toml                   │
│    • Finds existing `Middleware` trait signature                │
│    • Strips out all Auth/Logging code                           │
│  Output: MiniCodebase (50 lines of surgical context)            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  L4: Atom (Coder)                                                │
│  Sees: 20 lines (Router + Middleware trait + imports)           │
│  Task: Write the `impl Middleware for RateLimiter` block        │
│  Tools Available: [ast-grep, code_write] (Coder permissions)    │
│  Output: JSON with exact AST edit pattern                       │
└─────────────────────────────────────────────────────────────────┘
```

### Dynamic Tool Loading (Per-Atom Permissions)

The key insight: **different AtomTypes have different tool access**. This is controlled by the Rhai script and enforced by the runtime:

| AtomType | Available Tools | Permissions |
|----------|-----------------|-------------|
| `Architect` | `design_struct`, `define_interface` | Read-only codebase, design output |
| `Coder` | `ast_grep`, `code_write`, `add_import` | Write code, verify syntax |
| `Reviewer` | `code_read`, `approve`, `reject` | Read-only, decision output |
| `Tester` | `run_tests`, `code_read`, `assert` | Execute tests, read results |
| `GritsAnalyzer` | `check_cycles`, `check_layers`, `red_flag` | Topology analysis only |

```rhai
// L2 controls which tools each atom gets
let coder = spawn_atom(AtomType::Coder, task);     // Gets: ast_grep, code_write
let reviewer = spawn_atom(AtomType::Reviewer, code); // Gets: code_read, approve/reject

// Coder CANNOT approve (no tool access)
// Reviewer CANNOT write code (no tool access)
```

### Rhai ↔ Rust Bridge

The Rhai VM exposes these native functions (implemented in Rust):

| Function | Level | Purpose |
|----------|-------|---------|
| `spawn_atom(AtomType, task)` | L2→L3→L4 | Trigger context hydration + atom execution |
| `grits.has_symbol(name)` | L2 | Red-flag check before spawning |
| `grits.get_topology()` | L2 | Get current dependency graph |
| `grits.check_cycles(code)` | L2 | Detect circular dependencies |
| `grits.check_layers(code)` | L2 | Detect layer violations |
| `shadow.commit(code)` | L2 | Atomic commit to shadow repo |
| `shadow.rollback(n)` | L2 | Revert n commits |
| `emit(event, data)` | L2 | Send event to UI (progress, errors) |
| `interrogator.ask(question)` | L1↔User | Pause for user input (async) |
| `voting.first_to_ahead_by_k(results, k)` | L4→L2 | Consensus on multiple atom outputs |

### Script Generator (L1 → L2)

The Technical Orchestrator (L2) doesn't write Rhai manually—it uses the `ScriptGenerator`:

```rust
pub trait ScriptGenerator: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, plan: &Plan) -> bool;
    fn generate(&self, plan: &Plan, context: &GritsContext) -> Result<RhaiScript>;
}

// Built-in generators:
// - TaskScriptGenerator     → Generic task execution
// - TemplateScriptGenerator → Project scaffolding from templates
// - RefactorScriptGenerator → Brownfield modifications
// - TestScriptGenerator     → Test generation & execution
```

---

## 🖥️ UI Components

| Component | Purpose |
|-----------|---------|
| **Sidebar** | Navigation + Reliability indicator |
| **Blueprint (Dashboard)** | Main view: Plan + Graph + Execution |
| **PlanView** | Hierarchical task tree with status |
| **GraphView** | D3 topology visualization (Grits) |
| **ExecutionPanel** | Swarm stats + Voting arena |
| **TimeSlider** | Shadow Git commit scrubber |
| **Settings** | API keys, model config per agent |
| **PRDUpload** | File upload (.md, .txt, .pdf) + drag-and-drop |
| **ChatInput** | L1 Q&A messaging interface |

### Stage-Adaptive Interface

The UI transforms based on workflow stage:

| Stage | Primary View | Secondary |
|-------|--------------|-----------|
| **Input** | PRDUpload + Template picker | Chat for quick descriptions |
| **Interrogation** | Chat (Q&A with L1) | Extracted requirements summary |
| **Planning** | PlanView (task tree) | GraphView (dependencies) |
| **Execution** | ExecutionPanel (voting) | PlanView (progress) |
| **Review** | TimeSlider (history) | GraphView (final topology) |

---

## 🔧 Configuration

### Per-Agent Model Selection

Each agent type can use a different LLM optimized for its role:

```yaml
# Settings UI stores this configuration
agents:
  interrogator:
    provider: anthropic
    model: claude-sonnet-4-20250514
    temperature: 0.3
    # Good at: understanding requirements, asking questions

  architect:
    provider: openai
    model: gpt-4o
    temperature: 0.5
    # Good at: system design, decomposition

  orchestrator:
    provider: cerebras
    model: llama-4-scout-17b-16e-instruct
    temperature: 0.7
    # Good at: workflow decisions, coordination

  atoms:
    coder:
      provider: cerebras
      model: llama-4-scout-17b-16e-instruct
      temperature: 0.2
      # Fast inference for swarm (50 parallel)

    reviewer:
      provider: anthropic
      model: claude-sonnet-4-20250514
      temperature: 0.3
      # Thorough code review
```

### Supported Providers

| Provider | Models | Best For |
|----------|--------|----------|
| **Cerebras** | Llama 4 Scout | Swarm atoms (fastest) |
| **OpenAI** | GPT-4o, GPT-4o-mini | Architecture, planning |
| **Anthropic** | Claude Opus/Sonnet | Review, interrogation |
| **Ollama** | Local models | Offline/privacy |

---

## 🚀 Getting Started

```bash
# Clone
git clone https://github.com/babybirdprd/cerebras-maker.git
cd cerebras-maker

# Install frontend dependencies
bun install

# Run development build
bun run tauri dev
```

### Environment Variables

```env
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
CEREBRAS_API_KEY=csk-...
# Optional for local models
OLLAMA_HOST=http://localhost:11434
```

---

## 📁 Project Structure

```
cerebras-maker/
├── src/                          # React Frontend
│   ├── components/
│   │   ├── Sidebar.tsx           # Navigation
│   │   ├── Dashboard.tsx         # Main layout
│   │   ├── PlanView.tsx          # Task tree
│   │   ├── GraphView.tsx         # D3 topology
│   │   ├── ExecutionPanel.tsx    # Voting arena
│   │   ├── TimeSlider.tsx        # Git scrubber
│   │   ├── Settings.tsx          # Configuration
│   │   ├── PRDUpload.tsx         # File upload (drag-and-drop)
│   │   └── ChatInput.tsx         # L1 Q&A messaging
│   ├── store/
│   │   └── makerStore.ts         # Zustand state
│   ├── types.ts                  # TypeScript types
│   └── App.tsx
│
├── src-tauri/
│   ├── src/
│   │   ├── agents/               # Agent implementations
│   │   │   ├── mod.rs
│   │   │   ├── interrogator.rs   # Requirement analysis
│   │   │   ├── architect.rs      # Task decomposition
│   │   │   └── orchestrator.rs   # Workflow control
│   │   ├── generators/           # Rhai script generators
│   │   │   ├── mod.rs
│   │   │   ├── registry.rs       # Plugin registry
│   │   │   └── rhai_generator.rs # Task → Rhai conversion
│   │   ├── llm/                  # LLM provider abstraction
│   │   │   ├── mod.rs
│   │   │   └── provider.rs       # Unified API (rig-core)
│   │   ├── runtime/              # Rhai engine + voting
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs         # Rhai execution
│   │   │   └── voting.rs         # First-to-ahead-by-k
│   │   ├── shadow/               # Shadow Git (gitoxide)
│   │   └── lib.rs                # Tauri commands
│   │
│   ├── grits-core/               # Topology analysis library
│   │   ├── src/
│   │   │   ├── symbol_graph.rs   # Dependency graph
│   │   │   ├── mini_codebase.rs  # Context extraction
│   │   │   └── red_flag.rs       # Violation detection
│   │   └── Cargo.toml
│   │
│   ├── crawl4ai-rs/              # Web crawling (docs/research)
│   └── prompts/                  # Agent prompt templates
│       ├── interrogator.md
│       ├── architect.md
│       ├── orchestrator.md
│       └── atom_coder.md
│
└── templates/                    # Project scaffolding templates
    ├── tauri-react/
    │   ├── template.toml         # Template metadata
    │   ├── src/
    │   └── src-tauri/
    ├── axum-api/
    └── cli-clap/
```

---

## 🎯 Key Differentiators

| Feature | Traditional AI Coding | MAKER |
|---------|----------------------|-------|
| Input | Single prompt | Full PRD |
| Architecture | 1-2 levels | **4-Level Context Funnel** |
| Agents | 1 | 50+ specialized atoms |
| Context | Full file/codebase | **~50 lines (MiniCodebase)** |
| Tool Access | All tools always | **Dynamic per-AtomType** |
| Validation | None | Grits topology + red-flags |
| History | Git commits | Atomic shadow commits |
| Rollback | Manual | Instant (any point) |
| Voting | None | First-to-ahead-by-k |
| UI | Chat | Stage-adaptive |
| Error Rate | ~15% | **~0% (L3 isolation)** |

---

## 🛣️ Roadmap

### Phase 1: Core Architecture
- [ ] Implement L1 (Product Orchestrator) → PLAN.md output
- [ ] Implement L2 (Technical Orchestrator) → Rhai script generation
- [ ] Implement L3 (Context Engineer) → Grits MiniCodebase extraction
- [ ] Implement L4 (Atom Layer) → Dynamic tool loading per AtomType
- [ ] Wire spawn_atom() to trigger L3→L4 flow

### Phase 2: Frontend Integration
- [ ] Integrate cerebra-maker-fe UI components
- [ ] Settings UI (API keys, per-agent model config)
- [ ] ChatInput component (PRD + Interrogator Q&A)
- [ ] Wire UI to Tauri backend commands
- [ ] Stage-adaptive interface transitions

### Phase 3: Templates & Tooling
- [ ] Template system implementation (tauri-react first)
- [ ] crawl4ai integration for docs research
- [ ] Multi-file edit support with Grits validation
- [ ] Test generation & execution atoms
- [ ] Deployment automation
