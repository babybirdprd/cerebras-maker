\# Product Requirements Document: Cerebras-MAKER



\*\*Version:\*\* 7.0 (Rhai "Code Mode" Final)

\*\*Core Stack:\*\* Rust (`cerebras-rs`, `rig-core`, `rhai`, `grits-core`, `gitoxide`, `ast-grep-core`, `tauri`), React (`react-three-fiber`)

\*\*Philosophy:\*\* "Massively Decomposed, Topologically Aware, 100% Reliable."



---



\## 1. Executive Summary

\*\*Cerebras-MAKER\*\* is an autonomous coding system designed to solve complex engineering tasks with zero errors. It rejects the standard "chat loop" in favor of a \*\*Reliability Graph\*\* orchestrated by a sandboxed scripting runtime.



It combines:

1\.  \*\*MAKER Framework:\*\* Decomposing tasks into atomic steps ($m=1$) and using "First-to-ahead-by-$k$" voting to eliminate stochastic errors.

2\.  \*\*Rhai Code Mode:\*\* An embedded scripting layer that allows the Orchestrator to solve problems by writing logic (loops, consensus checks) rather than managing fragile JSON state.

3\.  \*\*Grits Topology:\*\* Using `grits-core` as a native library to perform "Semantic Tree-Shaking" and architectural red-flagging.

4\.  \*\*Shadow Reliability:\*\* A transactional file system (`gitoxide`) that snapshots every atomic move, allowing instant time-travel rollback.



---



\## 2. Architecture: The Dual-Graph System

The system is split into two distinct execution graphs to separate \*\*System 2 Thinking\*\* (Planning/Scripting) from \*\*System 1 Execution\*\* (Atomic Work).



\### Phase A: The Planning Graph (System 2)

\* \*\*Goal:\*\* Resolve ambiguity and generate a rigid Technical Spec.

\* \*\*Outputs:\*\* `PLAN.md` (Natural Language), `ARCH.json` (Topology Spec).

\* \*\*Agents:\*\*

&nbsp;   \* \*\*The Interrogator:\*\* Scans user requests for "Known Unknowns". Halts execution to ask the user if ambiguity > threshold.

&nbsp;   \* \*\*The Architect:\*\* Decomposes the PRD into a dependency tree of Atomic Micro-Tasks.



\### Phase B: The Execution Graph (System 1)

\* \*\*Goal:\*\* Implement the plan with surgical precision.

\* \*\*Outputs:\*\* Valid Rust/Python code committed to the Shadow Repo.

\* \*\*Agents:\*\*

&nbsp;   \* \*\*The Orchestrator:\*\* A high-level programmer agent that writes \*\*Rhai Scripts\*\* to coordinate the work. It does not execute tasks itself.

&nbsp;   \* \*\*The Atoms (Micro-Agents):\*\* Ephemeral, stateless agents spawned by the Rhai runtime to perform exactly one tool call or text generation.



---



\## 3. The "Brain": Grits-Core Integration

We perform \*\*Semantic Tree-Shaking\*\* by embedding `grits-core` directly into the agent binary.



\### 3.1. Context Engineering (The "Mini Codebase")

Instead of regex-based context slicing, we use Grits' \*\*Star Neighborhood\*\* algorithm to hydrate strictly relevant code.



\*\*Implementation:\*\*

```rust

use grits\_core::context::MiniCodebase;

use grits\_core::topology::SymbolGraph;



async fn build\_agent\_context(issue\_id: \&str, seed\_symbols: Vec<String>) -> Result<String> {

&nbsp;   // 1. Load the Semantic Graph (Cached in RAM)

&nbsp;   let graph = SymbolGraph::from\_workspace(".").await?;

&nbsp;   

&nbsp;   // 2. Semantic Tree-Shaking

&nbsp;   // "Extract only the topologically-relevant code... a 2000-line file becomes 50 focused lines."

&nbsp;   //

&nbsp;   let mini\_repo = MiniCodebase::assemble(

&nbsp;       \&graph,

&nbsp;       seed\_symbols, 

&nbsp;       2,   // Depth: How far to traverse relations

&nbsp;       0.5, // Strength Threshold

&nbsp;       Some(issue\_id.to\_string())

&nbsp;   );



&nbsp;   // 3. Hydrate \& Serialize

&nbsp;   // Grits automatically fetches the byte-ranges for the code snippets.

&nbsp;   let mut hydrated = mini\_repo.clone();

&nbsp;   hydrated.hydrate\_code(Path::new("."));

&nbsp;   

&nbsp;   Ok(serde\_json::to\_string(\&hydrated)?)

}

```



\### 3.2. Architectural Red-Flagging (The Guardrails)

We use Grits to mechanically enforce architectural invariants.



```rust

use grits\_core::topology::analysis::TopologicalAnalysis;



fn red\_flag\_check(proposed\_code: \&str) -> bool {

&nbsp;   // 1. Virtual Apply (AST-Grep in memory)

&nbsp;   let temp\_graph = apply\_virtual\_edit(proposed\_code);

&nbsp;   

&nbsp;   // 2. Cycle Detection (Betti Number 1)

&nbsp;   // "Detect cycle membership via triangles... tightly coupled."

&nbsp;   //

&nbsp;   let analysis = TopologicalAnalysis::analyze(\&temp\_graph);

&nbsp;   

&nbsp;   if analysis.betti\_1 > PREVIOUS\_BETTI\_1 {

&nbsp;       return true; // RED FLAG: Introduced circular dependency

&nbsp;   }

&nbsp;   

&nbsp;   if violates\_layers(temp\_graph, "layers.yaml") {

&nbsp;       return true; // RED FLAG: Layer violation

&nbsp;   }

&nbsp;   false

}

```



---



\## 4. The Logic Layer: Rhai "Code Mode" Runtime

We invert the standard "Tool Calling" paradigm by providing a \*\*sandboxed, type-safe scripting runtime (Rhai)\*\*. This layer allows the \*\*Orchestrator Agent\*\* to solve problems by writing logic (loops, conditionals, error handling) rather than managing multi-turn JSON state.



This specifically enables \*\*Massively Decomposed Agentic Processes (MDAPs)\*\* by allowing the Orchestrator to dynamically spawn ephemeral "Atom" agents and enforce voting/consensus mechanisms programmatically.



\### 4.1. Core Philosophy

1\.  \*\*The Orchestrator is a Programmer:\*\* The high-level agent (Orchestrator) does not execute tasks. It writes scripts.

2\.  \*\*Atoms are Ephemeral \& Restricted:\*\* The script calls "Atoms". These are fresh, stateless `Rig` agents spun up for a single interaction. They have exactly \*\*one tool\*\* and strict context limits.

3\.  \*\*Logic lives in Script:\*\* Control flow (voting loops, retries, consensus checks) happens in Rhai, not inside the LLM's hidden state.



\### 4.2. The Rhai API Surface

The following Rust types and functions are registered into the Rhai engine to interface with `cerebras-rs` via `Rig`.



\* \*\*`AtomType` (Enum):\*\* Strictly typed worker definitions (e.g., `AtomType.Search`, `AtomType.Coder`, `AtomType.Reviewer`).

\* \*\*`spawn\_atom(type, prompt)`:\*\* Instantiates an ephemeral agent.

&nbsp;   \* \*Constraint:\* No chat history passed (context atomization).

\* \*\*`spawn\_atom\_with\_flags(type, prompt, flags)`:\*\* Enforces strict output validation.

&nbsp;   \* \*MAKER Logic:\* If response > 750 tokens or invalid JSON, Rust throws an error immediately, enabling script-level retry.



\### 4.3. The "MAKER" Standard Library

We inject a `voting.rhai` preamble to implement reliability patterns described in the MAKER paper.



\* \*\*`run\_consensus(atom\_type, task, k\_threshold)`\*\*:

&nbsp;   1.  Runs `spawn\_atom` in a parallel loop (using Cerebras throughput).

&nbsp;   2.  Collects outputs.

&nbsp;   3.  Applies "First-to-ahead-by-k" logic to find the winner.

&nbsp;   4.  Automatically discards "red-flagged" responses.



\### 4.4. Example Execution Script

The Orchestrator generates this Rhai script to solve a task:

```rust

// Orchestrator generated script

let task = "Implement Auth Middleware";



// Pattern: Decomposition \& Voting

let results = run\_consensus(AtomType.Coder, task, 3); // Wait for 3 votes



if results.is\_err() {

&nbsp;   throw "Consensus failed: " + results.error;

}



// Pattern: Validation

let red\_flags = check\_red\_flags(results.value);

if red\_flags {

&nbsp;  throw "Architectural violation detected";

}



return results.value;

```



---



\## 5. Reliability Layer: The Shadow Git

We integrate \*\*`gitoxide`\*\* to treat every atomic step as a database transaction.



\### 5.1. The "Time-Travel" Protocol

1\.  \*\*Snapshot:\*\* Before any Rhai script touches disk, `gitoxide` creates a blob of the current state.

2\.  \*\*Execute:\*\* The Atoms apply edits via `ast-grep`.

3\.  \*\*Verify:\*\* `cargo check` / `pytest` runs.

4\.  \*\*Rollback:\*\* If verification fails or the Rhai script throws an error, `gitoxide` reverts the index to the snapshot instantly.

5\.  \*\*Squash:\*\* Only when the `PLAN.md` is marked "Complete" does the Shadow Repo squash the micro-commits into the main history.



---



\## 6. Interface Layer: Unified Tauri Architecture

We replace fragmented interfaces with a single, high-performance hybrid application using \*\*Tauri v2\*\* and \*\*React\*\*.



\### 6.1. The Stack

\* \*\*Backend:\*\* Rust (`maker\_core` + `tauri`). Handles the Rhai Runtime, Event Stream, Cerebras client, and Grits topology analysis.

\* \*\*Frontend:\*\* React + TypeScript + Vite. Handles visualization.

\* \*\*Bridge:\*\* Tauri IPC for high-frequency event streaming.



\### 6.2. The Dashboard ("The Bridge")

\* \*\*The Cockpit:\*\* Real-time log of the Rhai execution loop.

\* \*\*The Blueprint:\*\* 3D interactive visualization (`react-three-fiber`) of the `grits-core` SymbolGraph.

\* \*\*The Time Machine:\*\* Visual scrubber for `gitoxide` history.



\### 6.3. CLI Integration

\* \*\*Headless Mode:\*\* `maker run --headless` runs the Rhai runtime in CI/CD pipelines.

\* \*\*Interactive Mode:\*\* `maker run` launches the GUI.



---



\## 7. The Workflow: End-to-End



1\.  \*\*Intake:\*\* User submits PRD.

2\.  \*\*Interrogation:\*\* Architect Agent generates `PLAN.md` and `issues.jsonl`.

3\.  \*\*Decomposition:\*\* Plan is broken into Atomic Micro-Tasks (Grits Issues).

4\.  \*\*Rhai Execution Loop:\*\*

&nbsp;   \* \*\*Script Generation:\*\* Orchestrator reads a Micro-Task and writes a `task\_script.rhai`.

&nbsp;   \* \*\*Runtime:\*\* `CodeModeRuntime` executes the script.

&nbsp;   \* \*\*Spawning:\*\* Runtime calls `spawn\_atom` (Rig/Cerebras).

&nbsp;   \* \*\*Voting:\*\* Script runs `run\_consensus` to gather 5 candidates.

&nbsp;   \* \*\*Red-Flagging:\*\* `grits-core` validates topology; `check\_red\_flags` validates format.

&nbsp;   \* \*\*Commit:\*\* If successful, `gitoxide` saves to Shadow Repo.

5\.  \*\*Review:\*\* User watches playback in GUI.

6\.  \*\*Merge:\*\* System squashes Shadow History into Main Branch.



---



\## 8. Technical Stack Reference



| Component | Library/Crate | Role |

| :--- | :--- | :--- |

| \*\*Orchestrator\*\* | `rhai` + `maker\_core` | Scripting Runtime \& Logic |

| \*\*Agent Framework\*\* | `rig-core` | Rust LLM Abstraction |

| \*\*Inference\*\* | `cerebras-rs` | High-speed LLM client |

| \*\*Context\*\* | \*\*`grits-core`\*\* | Topology, Context Slicing, Red-Flagging |

| \*\*Search\*\* | `ripgrep` | Global text search |

| \*\*Editing\*\* | `ast-grep-core` | Structural search \& replace |

| \*\*Reliability\*\* | `gitoxide` | Transactional file system |

| \*\*Frontend\*\* | `react`, `react-three-fiber` | 3D Visualization |

| \*\*App Shell\*\* | \*\*`tauri`\*\* | OS Integration \& CLI Plugin |

