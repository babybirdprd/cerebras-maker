# Cerebras-MAKER Implementation Plan

> **Status**: ✅ COMPLETED
> **Created**: 2026-01-15
> **Completed**: 2026-01-15
> **Priority Order**: 1 (Critical) → 6 (Enhancement)

This document details each TODO/placeholder in the codebase with specific implementation steps, dependencies, and estimated complexity.

## Summary of Completed Work

| Priority | Component | Status |
|----------|-----------|--------|
| 1.1 | Multi-Provider Support (Cerebras, Anthropic, OpenAI, OpenRouter, OpenAICompatible) | ✅ DONE |
| 2.1 | Interrogator.analyze() with LLM | ✅ DONE |
| 2.2 | complete_interrogation PLAN.md generation | ✅ DONE |
| 3.1 | Architect.decompose() with LLM | ✅ DONE |
| 4.1 | Async spawn_atom in Rhai | ✅ DONE |
| 4.2 | Async run_consensus voting | ✅ DONE |
| 5.1 | Dependency-aware task execution | ✅ DONE |

---

## Priority 1: LLM Integration Foundation

### 1.1 Complete Multi-Provider Support in LlmProvider ✅ DONE

**File**: `src-tauri/src/llm/provider.rs`

**Status**: ✅ **COMPLETED** (2026-01-15)

**Implemented Providers**:
- ✅ **Cerebras**: via `cerebras-rs` native client (high-speed)
- ✅ **Anthropic**: via `rig-core` with `ProviderClient` trait
- ✅ **OpenAI**: via `rig-core` with builder pattern
- ✅ **OpenRouter**: via `rig-core` native openrouter provider
- ✅ **OpenAICompatible**: via `rig-core` with custom `base_url`

**Key Changes**:
- Added `cerebras-rs` crate dependency
- Removed `Ollama` variant, added `OpenRouter` and `OpenAICompatible`
- Implemented `call_cerebras()`, `call_anthropic()`, `call_openrouter()`, `call_openai()` methods
- Used proper rig-core 0.28.0 builder patterns and trait imports

---

## Priority 2: L1 Interrogator Agent

### 2.1 Implement `Interrogator.analyze()` with LLM

**File**: `src-tauri/src/agents/interrogator.rs` (lines 70-80)

**Current State**: Returns hardcoded placeholder `InterrogatorResult`

**Implementation Steps**:

1. Import `crate::llm::{complete_with_system, Message}`
2. Make `analyze` method `async`
3. Build JSON prompt from `system_prompt()` + user request
4. Parse LLM JSON response into `InterrogatorResult`
5. Add structured output validation (JSON schema)

**Code Pattern**:
```rust
pub async fn analyze(&self, request: &str, context: &AgentContext) -> Result<InterrogatorResult, anyhow::Error> {
    let user_prompt = format!(
        "Analyze the following request for ambiguities:\n\n{}\n\nWorkspace: {}",
        request,
        context.workspace_path
    );
    
    let response = crate::llm::complete_with_system(self.system_prompt(), &user_prompt).await?;
    let result: InterrogatorResult = serde_json::from_str(&response)?;
    Ok(result)
}
```

**Dependencies**: Priority 1.1 (LlmProvider)  
**Complexity**: Medium  
**Estimated Time**: 2 hours

---

### 2.2 Implement `complete_interrogation` PLAN.md Generation

**File**: `src-tauri/src/lib.rs` (line 622)

**Current State**: Returns mock PLAN.md with conversation summary

**Implementation Steps**:

1. Extract structured data from conversation history
2. Call Architect agent to decompose into tasks
3. Generate proper PLAN.md markdown format per PRD spec
4. Return structured tasks array alongside markdown

**Code Pattern**:
```rust
async fn complete_interrogation(conversation: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    // 1. Build requirements summary from conversation
    let requirements = extract_requirements_from_conversation(&conversation);
    
    // 2. Call Architect to decompose
    let architect = agents::Architect::new();
    let context = agents::AgentContext::new(&workspace_path);
    let decomposition = architect.decompose(&requirements, &context).await?;
    
    // 3. Generate PLAN.md
    let plan_md = render_plan_md(&decomposition.plan);
    
    Ok(serde_json::json!({
        "status": "complete",
        "plan_md": plan_md,
        "tasks": decomposition.plan.micro_tasks
    }))
}
```

**Dependencies**: Priority 2.1 (Interrogator), Priority 3.1 (Architect)  
**Complexity**: Medium-High  
**Estimated Time**: 3-4 hours

---

## Priority 3: L1 Architect Agent

### 3.1 Implement `Architect.decompose()` with LLM

**File**: `src-tauri/src/agents/architect.rs` (lines 71-105)

**Current State**: Returns empty placeholder `DecompositionResult`

**Implementation Steps**:

1. Make `decompose` method `async`
2. Build structured prompt with requirements + context
3. Request JSON output with tasks, dependencies, architecture
4. Parse response into `PlanOutput` and `ArchitectureOutput`
5. Validate task DAG (no cycles in dependencies)

**Code Pattern**:
```rust
pub async fn decompose(&self, requirements: &str, context: &AgentContext) -> Result<DecompositionResult, anyhow::Error> {
    let user_prompt = format!(
        "Decompose the following requirements into atomic micro-tasks:\n\n{}\n\n\
         Output both PLAN.md content and ARCH.json structure.",
        requirements
    );

    let response = crate::llm::complete_with_system(self.system_prompt(), &user_prompt).await?;

    // Parse the structured response
    let parsed = parse_decomposition_response(&response)?;
    Ok(parsed)
}
```

**Dependencies**: Priority 1.1 (LlmProvider)
**Complexity**: High (structured output parsing)
**Estimated Time**: 4-5 hours

---

## Priority 4: Rhai Runtime - Async LLM Execution

### 4.1 Implement Async `spawn_atom` in Rhai

**File**: `src-tauri/src/maker_core/runtime.rs` (lines 74-78)

**Current State**: Returns mock `AtomResult` synchronously

**Challenge**: Rhai is synchronous; need to bridge async LLM calls

**Implementation Steps**:

1. Create `AsyncRuntime` wrapper that holds a tokio runtime handle
2. Use `tokio::runtime::Handle::block_on` for sync→async bridging in Rhai callbacks
3. Wire up to `AtomExecutor.execute()` for actual LLM completion
4. Add proper error handling and timeout management

**Code Pattern**:
```rust
// Store tokio handle in runtime
let handle = tokio::runtime::Handle::current();

engine.register_fn("spawn_atom", move |atom_type: AtomType, prompt: &str| -> Dynamic {
    let handle = handle.clone();
    let result = handle.block_on(async {
        let executor = AtomExecutor::new();
        executor.execute(AtomInput {
            atom_type,
            task: prompt.to_string(),
            context: None,
            flags: SpawnFlags::default(),
            variables: HashMap::new(),
        }).await
    });

    match result {
        Ok(output) => rhai::serde::to_dynamic(&output).unwrap_or(Dynamic::UNIT),
        Err(e) => {
            // Return error result
            let error_result = AtomResult::error(atom_type, e.to_string());
            rhai::serde::to_dynamic(&error_result).unwrap_or(Dynamic::UNIT)
        }
    }
});
```

**Dependencies**: Priority 1.1 (LlmProvider), AtomExecutor (already implemented)
**Complexity**: High (async bridging)
**Estimated Time**: 4-6 hours

---

### 4.2 Implement Async `run_consensus` in Rhai

**File**: `src-tauri/src/maker_core/runtime.rs` (lines 90-101)

**Current State**: Returns mock failure message

**Implementation Steps**:

1. Create async consensus runner that spawns N atoms in parallel
2. Use `tokio::spawn` for concurrent LLM calls
3. Implement "first-to-ahead-by-k" voting logic with early termination
4. Integrate red flag checking for each candidate
5. Return winning result or timeout/failure

**Code Pattern**:
```rust
engine.register_fn("run_consensus", move |atom_type: AtomType, task: &str, k_threshold: i64| -> Dynamic {
    let handle = handle.clone();
    let result = handle.block_on(async {
        let config = ConsensusConfig {
            k_threshold: k_threshold as usize,
            ..Default::default()
        };

        run_consensus_async(atom_type, task, config).await
    });

    rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
});

async fn run_consensus_async(atom_type: AtomType, task: &str, config: ConsensusConfig) -> ConsensusResult {
    let mut votes: HashMap<String, usize> = HashMap::new();
    let mut handles = Vec::new();

    // Spawn max_atoms in parallel
    for _ in 0..config.max_atoms {
        let task = task.to_string();
        handles.push(tokio::spawn(async move {
            let executor = AtomExecutor::new();
            executor.execute_simple(atom_type, &task).await
        }));
    }

    // Collect results as they complete
    for handle in handles {
        if let Ok(Ok(result)) = handle.await {
            // Check red flags
            if config.discard_red_flags && result.has_red_flags() {
                continue;
            }

            let normalized = normalize_output(&result.output);
            *votes.entry(normalized).or_insert(0) += 1;

            // Check for early consensus
            if let Some(winner) = check_consensus(&votes, config.k_threshold, config.min_votes) {
                return ConsensusResult::success(winner, ...);
            }
        }
    }

    // Return best result or failure
}
```

**Dependencies**: Priority 4.1 (spawn_atom)
**Complexity**: Very High (concurrent execution, voting logic)
**Estimated Time**: 6-8 hours

---

## Priority 5: Orchestrator Task Dependencies

### 5.1 Implement Dependency-Aware Task Execution

**File**: `src-tauri/src/agents/orchestrator.rs` (line 152)

**Current State**: Executes tasks sequentially, ignoring dependencies

**Implementation Steps**:

1. Build dependency graph from `plan.dependencies`
2. Topologically sort tasks (already have grits-core for this)
3. Identify parallelizable task groups (tasks with satisfied deps)
4. Use `tokio::join!` or `futures::join_all` for parallel execution
5. Track task completion and unblock dependents

**Code Pattern**:
```rust
pub async fn generate_execution_plan(&self, plan: &PlanOutput, context: &AgentContext)
    -> Vec<Result<GenerationResult, GeneratorError>>
{
    // Build adjacency list
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    for (task_id, dep_id) in &plan.dependencies {
        deps.entry(task_id.clone()).or_default().push(dep_id.clone());
    }

    // Track completed tasks
    let completed: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
    let mut results = Vec::new();

    // Process in waves
    loop {
        let ready_tasks: Vec<_> = plan.micro_tasks.iter()
            .filter(|t| {
                let completed = completed.read().unwrap();
                !completed.contains(&t.id) &&
                deps.get(&t.id).map(|d| d.iter().all(|dep| completed.contains(dep))).unwrap_or(true)
            })
            .collect();

        if ready_tasks.is_empty() { break; }

        // Execute ready tasks in parallel
        let wave_results = futures::future::join_all(
            ready_tasks.iter().map(|task| self.generate_script(task, context))
        ).await;

        // Mark completed
        for (task, result) in ready_tasks.iter().zip(wave_results.into_iter()) {
            completed.write().unwrap().insert(task.id.clone());
            results.push(result);
        }
    }

    results
}
```

**Dependencies**: None (orchestrator already exists)
**Complexity**: Medium-High
**Estimated Time**: 3-4 hours

---

## Priority 6: Enhancement TODOs

### 6.1 Parse Dependencies from PLAN.md

**File**: `src-tauri/src/agents/orchestrator.rs` (line 233)

**Current State**: `dependencies` always empty in parsed plan

**Implementation**: Parse `depends_on: [...]` annotations from task lines

**Complexity**: Low
**Estimated Time**: 1 hour

---

### 6.2 Extract Forbidden Dependencies from Layer Analysis

**File**: `src-tauri/grits-core/src/context.rs` (line 182)

**Current State**: `forbidden_dependencies` always empty

**Implementation**: Use layer constraints from ArchitectureOutput to populate

**Complexity**: Medium
**Estimated Time**: 2 hours

---

### 6.3 Complete Project Templates

**File**: `src-tauri/src/templates/mod.rs` (lines 152, 166)

**Current State**: `tauri-vanilla` and `rust-cli` templates have empty files

**Implementation**: Add template files similar to `tauri_react/` pattern

**Complexity**: Low
**Estimated Time**: 2 hours

---

## Implementation Order (Recommended)

```
Week 1:
  1.1 → 2.1 → 3.1 → 2.2
  (LLM Foundation → Interrogator → Architect → Complete Interrogation)

Week 2:
  4.1 → 4.2 → 5.1
  (Async Rhai → Consensus → Dependency Execution)

Week 3:
  6.1 → 6.2 → 6.3
  (Enhancements & Templates)
```

## Testing Strategy

Each priority should include:

1. **Unit Tests**: Test LLM response parsing, voting logic, dependency sorting
2. **Integration Tests**: End-to-end PRD → PLAN.md → Rhai → Execution
3. **Mock LLM Responses**: For deterministic testing without API calls

---

## Questions to Resolve Before Implementation

1. **Temperature settings**: Should different agent types use different temperatures?
2. **Timeout values**: What are appropriate timeouts for consensus voting?
3. **Rate limiting**: How should we handle provider rate limits during consensus?
4. **Fallback behavior**: What happens if LLM provider is unavailable?

