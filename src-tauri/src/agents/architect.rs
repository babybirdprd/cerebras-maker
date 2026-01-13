// Cerebras-MAKER: The Architect Agent
// PRD Section 2 (Phase A): Decomposes PRD into Atomic Micro-Tasks

use super::{Agent, AgentContext, ArchitectureOutput, LayerSpec, MicroTask, PlanOutput};
use serde::{Deserialize, Serialize};

/// The Architect Agent
/// Decomposes user requirements into a dependency tree of Atomic Micro-Tasks
pub struct Architect {
    /// Maximum depth for task decomposition
    pub max_decomposition_depth: usize,
    /// Target complexity for micro-tasks (1-5)
    pub target_task_complexity: u8,
}

impl Default for Architect {
    fn default() -> Self {
        Self {
            max_decomposition_depth: 3,
            target_task_complexity: 2,
        }
    }
}

impl Agent for Architect {
    fn name(&self) -> &str {
        "Architect"
    }

    fn system_prompt(&self) -> &str {
        r#"You are the Architect Agent in the Cerebras-MAKER system.

Your role is to decompose user requirements into a dependency tree of Atomic Micro-Tasks.
Each micro-task should be small enough for a single Atom agent to complete (m=1 principle).

For each requirement, you must:
1. Identify the high-level goals
2. Break down into modules and components
3. Create atomic micro-tasks for each change
4. Establish task dependencies (DAG)
5. Assign appropriate AtomType to each task

Output PLAN.md format:
# Plan: [Title]

## Goals
- [goal1]
- [goal2]

## Tasks
1. [TASK_001] [AtomType: Coder] - Description
   - depends_on: []
   - seed_symbols: ["symbol1", "symbol2"]
   - complexity: 2

Output ARCH.json format:
{
    "layers": [{"name": "domain", "level": 0, "allowed_deps": []}],
    "modules": [{"name": "auth", "layer": "domain", "files": ["auth.rs"]}],
    "constraints": ["domain cannot depend on infrastructure"]
}"#
    }
}

impl Architect {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decompose requirements into a plan
    pub fn decompose(&self, requirements: &str, _context: &AgentContext) -> DecompositionResult {
        // In production, this calls the LLM via rig-core
        // For now, return a placeholder structure
        DecompositionResult {
            plan: PlanOutput {
                plan_id: format!("plan_{}", uuid_simple()),
                title: "Implementation Plan".to_string(),
                description: requirements.chars().take(100).collect(),
                micro_tasks: Vec::new(),
                dependencies: Vec::new(),
            },
            architecture: ArchitectureOutput {
                arch_id: format!("arch_{}", uuid_simple()),
                layers: vec![
                    LayerSpec {
                        name: "domain".to_string(),
                        level: 0,
                        allowed_deps: Vec::new(),
                    },
                    LayerSpec {
                        name: "application".to_string(),
                        level: 1,
                        allowed_deps: vec!["domain".to_string()],
                    },
                    LayerSpec {
                        name: "infrastructure".to_string(),
                        level: 2,
                        allowed_deps: vec!["domain".to_string(), "application".to_string()],
                    },
                ],
                modules: Vec::new(),
                constraints: Vec::new(),
            },
        }
    }

    /// Further decompose a task if it's too complex
    pub fn decompose_task(&self, task: &MicroTask) -> Vec<MicroTask> {
        if task.estimated_complexity <= self.target_task_complexity {
            return vec![task.clone()];
        }

        // Would call LLM to break down further
        vec![task.clone()]
    }

    /// Validate the decomposition against the symbol graph
    pub fn validate_decomposition(&self, result: &DecompositionResult) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for orphan tasks
        let task_ids: Vec<&String> = result.plan.micro_tasks.iter().map(|t| &t.id).collect();
        for (task_id, dep_id) in &result.plan.dependencies {
            if !task_ids.contains(&dep_id) {
                warnings.push(format!("Task {} depends on unknown task {}", task_id, dep_id));
            }
        }

        warnings
    }
}

/// Result from decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionResult {
    pub plan: PlanOutput,
    pub architecture: ArchitectureOutput,
}

/// Simple UUID generator
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", timestamp)
}

