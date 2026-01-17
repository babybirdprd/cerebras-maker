// Cerebras-MAKER: The L2 Technical Orchestrator
// PRD Section 2 (Phase B): PLAN.md → script.rhai
// Takes refined requirements from L1 and generates executable Rhai scripts

use super::{Agent, AgentContext, MicroTask, PlanOutput};
use crate::generators::{GeneratorRegistry, GenerationResult, GeneratorError, RhaiScriptGenerator, TaskScriptGenerator};
use crate::llm::{PromptContext, SystemPrompts};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution state for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

/// Execution event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub event_type: String,
    pub task_id: String,
    pub timestamp: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// The Orchestrator Agent
/// Coordinates script generators and manages task execution pipeline
pub struct Orchestrator {
    /// Generator registry for script generation
    registry: Arc<GeneratorRegistry>,
    /// Default k-threshold for consensus voting
    pub default_k_threshold: usize,
    /// Execution log
    execution_log: Arc<RwLock<Vec<ExecutionEvent>>>,
    /// Task states
    task_states: Arc<RwLock<std::collections::HashMap<String, TaskState>>>,
    /// HIGH-6: Maximum retry attempts for failed tasks
    max_retries: usize,
    /// HIGH-6: Delay between retries in milliseconds
    retry_delay_ms: u64,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for Orchestrator {
    fn name(&self) -> &str {
        "Orchestrator"
    }

    fn system_prompt(&self) -> &str {
        SystemPrompts::orchestrator()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        let registry = Arc::new(GeneratorRegistry::new());

        Self {
            registry,
            default_k_threshold: 3,
            execution_log: Arc::new(RwLock::new(Vec::new())),
            task_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }

    /// HIGH-6: Configure retry behavior
    pub fn with_retry_config(mut self, max_retries: usize, retry_delay_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_ms = retry_delay_ms;
        self
    }

    /// Initialize the orchestrator with default generators
    pub async fn init(&self) {
        // Register default generators
        self.registry.register(TaskScriptGenerator::new()).await;
        self.registry.register(RhaiScriptGenerator::new()).await;
    }

    /// Get the generator registry for custom registration
    pub fn registry(&self) -> &GeneratorRegistry {
        &self.registry
    }

    /// Log an execution event
    async fn log_event(&self, event_type: &str, task_id: &str, message: &str, data: Option<serde_json::Value>) {
        let event = ExecutionEvent {
            event_type: event_type.to_string(),
            task_id: task_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message: message.to_string(),
            data,
        };

        let mut log = self.execution_log.write().await;
        log.push(event);
    }

    /// Update task state
    async fn set_task_state(&self, task_id: &str, state: TaskState) {
        let mut states = self.task_states.write().await;
        states.insert(task_id.to_string(), state);
    }

    /// Generate a Rhai script for a micro-task using the generator pipeline
    /// HIGH-6: Now includes retry logic for transient failures
    pub async fn generate_script(&self, task: &MicroTask, context: &AgentContext) -> Result<GenerationResult, GeneratorError> {
        self.log_event("script_generation_started", &task.id, "Generating script", None).await;
        self.set_task_state(&task.id, TaskState::Running).await;

        // Build prompt context from agent context
        let prompt_context = PromptContext::new()
            .with_workspace(&context.workspace_path)
            .with_var("task_id", &task.id)
            .with_var("task_description", &task.description);

        // HIGH-6: Retry loop for transient failures
        let mut last_error: Option<GeneratorError> = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                self.log_event(
                    "script_generation_retry",
                    &task.id,
                    &format!("Retry attempt {} of {}", attempt, self.max_retries),
                    Some(serde_json::json!({"attempt": attempt})),
                ).await;
                tokio::time::sleep(std::time::Duration::from_millis(self.retry_delay_ms)).await;
            }

            // Use the generator registry to find and use the best generator
            let result = self.registry.generate(task, &prompt_context).await;

            match result {
                Ok(gen_result) => {
                    self.log_event(
                        "script_generation_completed",
                        &task.id,
                        &format!("Script generated by {} (attempt {})", gen_result.metadata.generator_name, attempt + 1),
                        Some(serde_json::json!({
                            "confidence": gen_result.confidence,
                            "generation_time_ms": gen_result.metadata.generation_time_ms,
                            "attempts": attempt + 1
                        })),
                    ).await;
                    self.set_task_state(&task.id, TaskState::Completed).await;
                    return Ok(gen_result);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // All retries exhausted
        let error = last_error.unwrap_or_else(|| GeneratorError::GenerationFailed("Unknown error".to_string()));
        self.log_event(
            "script_generation_failed",
            &task.id,
            &format!("Generation failed after {} attempts: {}", self.max_retries + 1, error),
            Some(serde_json::json!({"attempts": self.max_retries + 1})),
        ).await;
        self.set_task_state(&task.id, TaskState::Failed(error.to_string())).await;
        Err(error)
    }

    /// Generate scripts for all tasks in a plan
    pub async fn generate_execution_plan(&self, plan: &PlanOutput, context: &AgentContext) -> Vec<Result<GenerationResult, GeneratorError>> {
        self.log_event("plan_execution_started", &plan.plan_id, &plan.title, None).await;

        let mut results: Vec<Result<GenerationResult, GeneratorError>> = Vec::with_capacity(plan.micro_tasks.len());
        let mut result_map: std::collections::HashMap<String, Result<GenerationResult, GeneratorError>> =
            std::collections::HashMap::new();

        // Build dependency graph: task_id -> [dependent_task_ids]
        let dep_graph = self.build_dependency_graph(plan);

        // Track completed tasks
        let mut completed: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Execute in waves based on dependencies
        while completed.len() < plan.micro_tasks.len() {
            // Find all tasks that can run (dependencies satisfied)
            let ready_tasks: Vec<&MicroTask> = plan.micro_tasks.iter()
                .filter(|t| !completed.contains(&t.id))
                .filter(|t| self.dependencies_satisfied(&t.id, &dep_graph, &completed))
                .collect();

            if ready_tasks.is_empty() {
                // No tasks ready but not all completed - circular dependency
                self.log_event("execution_error", &plan.plan_id,
                    "Circular dependency detected", None).await;
                break;
            }

            // Execute ready tasks in parallel
            let futures: Vec<_> = ready_tasks.iter()
                .map(|task| {
                    let task_id = task.id.clone();
                    let task_clone = (*task).clone();
                    let context_clone = context.clone();
                    async move {
                        let result = self.generate_script(&task_clone, &context_clone).await;
                        (task_id, result)
                    }
                })
                .collect();

            // Wait for all parallel tasks
            let wave_results = futures::future::join_all(futures).await;

            // Process results
            for (task_id, result) in wave_results {
                completed.insert(task_id.clone());
                result_map.insert(task_id, result);
            }
        }

        // Convert result_map to ordered results vector
        for task in &plan.micro_tasks {
            if let Some(result) = result_map.remove(&task.id) {
                results.push(result);
            }
        }

        self.log_event("plan_execution_completed", &plan.plan_id, "All scripts generated", None).await;
        results
    }

    /// Build a dependency graph from the plan
    fn build_dependency_graph(&self, plan: &PlanOutput) -> std::collections::HashMap<String, Vec<String>> {
        let mut graph: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        // Initialize empty dependency lists for all tasks
        for task in &plan.micro_tasks {
            graph.insert(task.id.clone(), Vec::new());
        }

        // Populate dependencies
        for (task_id, depends_on) in &plan.dependencies {
            if let Some(deps) = graph.get_mut(task_id) {
                deps.push(depends_on.clone());
            }
        }

        graph
    }

    /// Check if all dependencies for a task are satisfied
    fn dependencies_satisfied(
        &self,
        task_id: &str,
        dep_graph: &std::collections::HashMap<String, Vec<String>>,
        completed: &std::collections::HashSet<String>,
    ) -> bool {
        match dep_graph.get(task_id) {
            Some(deps) => deps.iter().all(|dep| completed.contains(dep)),
            None => true, // No dependencies
        }
    }

    /// Get the execution log
    pub async fn get_execution_log(&self) -> Vec<ExecutionEvent> {
        let log = self.execution_log.read().await;
        log.clone()
    }

    /// Get all task states
    pub async fn get_task_states(&self) -> std::collections::HashMap<String, TaskState> {
        let states = self.task_states.read().await;
        states.clone()
    }

    /// Clear the execution log
    pub async fn clear_log(&self) {
        let mut log = self.execution_log.write().await;
        log.clear();
    }

    // ========================================================================
    // L2 Technical Orchestrator Methods
    // ========================================================================

    /// Parse a PLAN.md document and extract structured tasks
    /// This is the entry point for L2 processing
    pub fn parse_plan_md(&self, plan_content: &str) -> Result<PlanOutput, String> {
        // Parse markdown to extract tasks
        let mut micro_tasks = Vec::new();
        let mut current_phase = String::new();
        let mut task_counter = 0;
        let mut dependencies: Vec<(String, String)> = Vec::new();
        // Map from user-defined task names to internal IDs
        let mut task_name_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for line in plan_content.lines() {
            let trimmed = line.trim();

            // Detect phase headers (## Phase N: ...)
            if trimmed.starts_with("## Phase") || trimmed.starts_with("## ") {
                current_phase = trimmed.trim_start_matches('#').trim().to_string();
                continue;
            }

            // Detect task items (- [ ] Task description)
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                task_counter += 1;
                let raw_description = trimmed
                    .trim_start_matches("- [ ]")
                    .trim_start_matches("- [x]")
                    .trim();

                // Parse task name, dependencies, and clean description
                let (task_name, clean_description, task_deps) =
                    Self::parse_task_line(raw_description);

                let task_id = format!("t{}", task_counter);

                // If task has an explicit name, map it
                if let Some(name) = &task_name {
                    task_name_to_id.insert(name.clone(), task_id.clone());
                }

                // Determine atom type from task description
                let atom_type = Self::infer_atom_type(&clean_description);

                micro_tasks.push(MicroTask {
                    id: task_id.clone(),
                    description: clean_description,
                    atom_type,
                    estimated_complexity: 3, // Default medium complexity
                    seed_symbols: Vec::new(),
                });

                // Store raw dependency references (resolve after all tasks parsed)
                for dep in task_deps {
                    dependencies.push((task_id.clone(), dep));
                }
            }
        }

        if micro_tasks.is_empty() {
            return Err("No tasks found in PLAN.md".to_string());
        }

        // Resolve dependency references to task IDs
        let resolved_deps: Vec<(String, String)> = dependencies
            .into_iter()
            .filter_map(|(task_id, dep_ref)| {
                // Try to resolve as task name first, then as task ID
                let resolved = task_name_to_id.get(&dep_ref)
                    .cloned()
                    .or_else(|| {
                        // Check if it's already a valid task ID (t1, t2, etc.)
                        if micro_tasks.iter().any(|t| t.id == dep_ref) {
                            Some(dep_ref.clone())
                        } else {
                            None
                        }
                    });

                resolved.map(|dep_id| (task_id, dep_id))
            })
            .collect();

        // Topological sort to ensure valid execution order
        let sorted_tasks = Self::topological_sort(&micro_tasks, &resolved_deps)?;

        Ok(PlanOutput {
            plan_id: format!("plan_{}", chrono::Utc::now().timestamp()),
            title: current_phase.clone(),
            description: "Auto-parsed from PLAN.md".to_string(),
            micro_tasks: sorted_tasks,
            dependencies: resolved_deps,
        })
    }

    /// Parse a task line to extract name, description, and dependencies
    /// Supports formats:
    /// - "Task description (depends: t1, t2)"
    /// - "Task description [after: task_name]"
    /// - "[task_name] Task description (depends: t1)"
    fn parse_task_line(line: &str) -> (Option<String>, String, Vec<String>) {
        let mut task_name = None;
        let mut deps = Vec::new();
        let mut clean_line = line.to_string();

        // Extract task name from [name] prefix
        if let Some(start) = clean_line.find('[') {
            if let Some(end) = clean_line.find(']') {
                if start < end && start == 0 {
                    task_name = Some(clean_line[start + 1..end].trim().to_string());
                    clean_line = clean_line[end + 1..].trim().to_string();
                }
            }
        }

        // Extract dependencies from (depends: ...) or [after: ...]
        let dep_patterns = [
            ("(depends:", ")"),
            ("(after:", ")"),
            ("[depends:", "]"),
            ("[after:", "]"),
        ];

        for (start_pattern, end_char) in dep_patterns {
            if let Some(start) = clean_line.to_lowercase().find(start_pattern) {
                let search_start = start + start_pattern.len();
                if let Some(end) = clean_line[search_start..].find(end_char) {
                    let deps_str = &clean_line[search_start..search_start + end];
                    deps.extend(
                        deps_str.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                    );
                    // Remove the dependency annotation from description
                    clean_line = format!(
                        "{}{}",
                        clean_line[..start].trim(),
                        clean_line[search_start + end + 1..].trim()
                    ).trim().to_string();
                }
            }
        }

        (task_name, clean_line, deps)
    }

    /// Perform topological sort on tasks based on dependencies (Kahn's algorithm)
    fn topological_sort(
        tasks: &[MicroTask],
        dependencies: &[(String, String)],
    ) -> Result<Vec<MicroTask>, String> {
        use std::collections::{HashMap, VecDeque};

        // Build adjacency list and in-degree count
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize all tasks
        for task in tasks {
            in_degree.insert(task.id.clone(), 0);
            adj_list.insert(task.id.clone(), Vec::new());
        }

        // Build graph from dependencies
        for (task_id, depends_on) in dependencies {
            if let Some(degree) = in_degree.get_mut(task_id) {
                *degree += 1;
            }
            if let Some(adj) = adj_list.get_mut(depends_on) {
                adj.push(task_id.clone());
            }
        }

        // Find all tasks with no dependencies
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted_ids = Vec::new();

        while let Some(task_id) = queue.pop_front() {
            sorted_ids.push(task_id.clone());

            if let Some(dependents) = adj_list.get(&task_id) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if sorted_ids.len() != tasks.len() {
            return Err("Circular dependency detected in task plan".to_string());
        }

        // Reorder tasks based on sorted IDs
        let task_map: HashMap<String, MicroTask> = tasks
            .iter()
            .map(|t| (t.id.clone(), t.clone()))
            .collect();

        let sorted_tasks: Vec<MicroTask> = sorted_ids
            .into_iter()
            .filter_map(|id| task_map.get(&id).cloned())
            .collect();

        Ok(sorted_tasks)
    }

    /// Infer the AtomType from task description
    fn infer_atom_type(description: &str) -> String {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("test") || desc_lower.contains("verify") || desc_lower.contains("assert") {
            "Tester".to_string()
        } else if desc_lower.contains("review") || desc_lower.contains("check") || desc_lower.contains("validate") {
            "Reviewer".to_string()
        } else if desc_lower.contains("design") || desc_lower.contains("architect") || desc_lower.contains("interface") {
            "Architect".to_string()
        } else if desc_lower.contains("analyze") || desc_lower.contains("topology") || desc_lower.contains("dependency") {
            "GritsAnalyzer".to_string()
        } else {
            "Coder".to_string() // Default to Coder
        }
    }

    /// Generate a master Rhai script that orchestrates all tasks
    /// This is the main L2 output
    pub async fn generate_master_script(&self, plan: &PlanOutput, _context: &AgentContext) -> Result<String, String> {
        self.log_event("l2_generation_started", &plan.plan_id, "Generating master Rhai script", None).await;

        let mut script = String::new();

        // Header
        script.push_str("// Auto-generated by L2 Technical Orchestrator\n");
        script.push_str(&format!("// Plan: {}\n", plan.title));
        script.push_str(&format!("// Generated: {}\n\n", chrono::Utc::now().to_rfc3339()));

        // Import MAKER preamble
        script.push_str("// MAKER Standard Library\n");
        script.push_str("let k_threshold = 3;\n\n");

        // Generate task execution blocks
        for task in &plan.micro_tasks {
            script.push_str(&format!("// Task: {} - {}\n", task.id, task.description));
            script.push_str(&format!("let {}_result = spawn_atom(\"{}\", \"{}\", k_threshold);\n",
                task.id.replace('-', "_"),
                task.atom_type,
                task.description.replace('"', "\\\"")
            ));
            script.push_str(&format!("if !{}_result.success {{ return #{{ error: \"Task {} failed\" }}; }}\n\n",
                task.id.replace('-', "_"),
                task.id
            ));
        }

        // Final result
        script.push_str("// All tasks completed\n");
        script.push_str("#{ success: true, tasks_completed: ");
        script.push_str(&plan.micro_tasks.len().to_string());
        script.push_str(" }\n");

        self.log_event("l2_generation_completed", &plan.plan_id, "Master script generated",
            Some(serde_json::json!({ "script_length": script.len() }))).await;

        Ok(script)
    }

    /// Full L2 pipeline: PLAN.md → Rhai script
    pub async fn process_plan(&self, plan_content: &str, context: &AgentContext) -> Result<String, String> {
        // Step 1: Parse PLAN.md
        let plan = self.parse_plan_md(plan_content)?;

        // Step 2: Generate master script
        let script = self.generate_master_script(&plan, context).await?;

        Ok(script)
    }
}

