// Cerebras-MAKER: The Architect Agent
// PRD Section 2 (Phase A): Decomposes PRD into Atomic Micro-Tasks

use super::{Agent, AgentContext, ArchitectureOutput, LayerSpec, MicroTask, PlanOutput};
use crate::llm::{LlmConfig, LlmProvider, Message};
use serde::{Deserialize, Serialize};

/// The Architect Agent
/// Decomposes user requirements into a dependency tree of Atomic Micro-Tasks
pub struct Architect {
    /// Maximum depth for task decomposition
    pub max_decomposition_depth: usize,
    /// Target complexity for micro-tasks (1-5)
    pub target_task_complexity: u8,
    /// LLM configuration for this agent
    llm_config: LlmConfig,
}

impl Default for Architect {
    fn default() -> Self {
        Self {
            max_decomposition_depth: 3,
            target_task_complexity: 2,
            llm_config: LlmConfig::cerebras(), // Default to Cerebras for speed
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

    pub fn with_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = config;
        self
    }

    /// Decompose requirements into a plan using LLM
    pub async fn decompose(&self, requirements: &str, context: &AgentContext) -> Result<DecompositionResult, String> {
        // Build the user prompt with context
        let user_prompt = self.build_user_prompt(requirements, context);

        // Create LLM provider
        let provider = LlmProvider::new(self.llm_config.clone())
            .map_err(|e| format!("Failed to create LLM provider: {}", e))?;

        // Build messages
        let messages = vec![
            Message::system(self.system_prompt()),
            Message::user(&user_prompt),
        ];

        // Execute the LLM call
        let response = provider
            .complete(messages)
            .await
            .map_err(|e| format!("LLM call failed: {}", e))?;

        // Parse the response
        self.parse_response(&response.content)
    }

    /// Build the user prompt with context information
    fn build_user_prompt(&self, requirements: &str, context: &AgentContext) -> String {
        let mut prompt = format!("## Requirements\n\n{}\n\n", requirements);

        prompt.push_str(&format!("## Workspace\n\n{}\n\n", context.workspace_path));

        prompt.push_str(&format!(
            "## Constraints\n\n- Max decomposition depth: {}\n- Target task complexity: {} (1-5 scale)\n\n",
            self.max_decomposition_depth, self.target_task_complexity
        ));

        prompt.push_str("Please decompose these requirements into atomic micro-tasks. Output both PLAN.md and ARCH.json sections.");
        prompt
    }

    /// Parse the LLM response into DecompositionResult
    fn parse_response(&self, response: &str) -> Result<DecompositionResult, String> {
        // Extract PLAN.md section
        let plan = self.parse_plan_section(response)?;

        // Extract ARCH.json section
        let architecture = self.parse_arch_section(response).unwrap_or_else(|_| {
            // Default architecture if not provided
            ArchitectureOutput {
                arch_id: format!("arch_{}", uuid_simple()),
                layers: vec![
                    LayerSpec { name: "domain".to_string(), level: 0, allowed_deps: Vec::new() },
                    LayerSpec { name: "application".to_string(), level: 1, allowed_deps: vec!["domain".to_string()] },
                    LayerSpec { name: "infrastructure".to_string(), level: 2, allowed_deps: vec!["domain".to_string(), "application".to_string()] },
                ],
                modules: Vec::new(),
                constraints: Vec::new(),
            }
        });

        Ok(DecompositionResult { plan, architecture })
    }

    /// Parse the PLAN.md section from response
    fn parse_plan_section(&self, response: &str) -> Result<PlanOutput, String> {
        let mut micro_tasks = Vec::new();
        let mut dependencies = Vec::new();
        let mut title = "Implementation Plan".to_string();

        for line in response.lines() {
            let trimmed = line.trim();

            // Extract title
            if trimmed.starts_with("# Plan:") || trimmed.starts_with("# ") {
                title = trimmed.trim_start_matches('#').trim().to_string();
                continue;
            }

            // Parse task lines: [TASK_001] [AtomType: Coder] - Description
            if let Some(task) = self.parse_task_line(trimmed) {
                micro_tasks.push(task);
            }

            // Parse dependency lines: - depends_on: [TASK_001, TASK_002]
            if trimmed.starts_with("- depends_on:") {
                if let Some(deps) = self.parse_dependencies(trimmed, &micro_tasks) {
                    dependencies.extend(deps);
                }
            }
        }

        Ok(PlanOutput {
            plan_id: format!("plan_{}", uuid_simple()),
            title,
            description: "Generated by Architect agent".to_string(),
            micro_tasks,
            dependencies,
        })
    }

    /// Parse a single task line
    fn parse_task_line(&self, line: &str) -> Option<MicroTask> {
        // Format: [TASK_001] [AtomType: Coder] - Description
        // Or: 1. [TASK_001] [AtomType: Coder] - Description
        if !line.contains("[TASK_") && !line.contains("- [ ]") {
            return None;
        }

        // Handle checkbox format: - [ ] Description
        if line.starts_with("- [ ]") || line.starts_with("- [x]") {
            let description = line
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim()
                .to_string();

            let atom_type = self.infer_atom_type(&description);

            return Some(MicroTask {
                id: format!("t{}", uuid_simple().chars().take(8).collect::<String>()),
                description,
                atom_type,
                estimated_complexity: 3,
                seed_symbols: Vec::new(),
            });
        }

        // Handle structured format: [TASK_001] [AtomType: Coder] - Description
        let task_id = self.extract_between(line, "[TASK_", "]")
            .map(|s| format!("TASK_{}", s))
            .unwrap_or_else(|| format!("t{}", uuid_simple().chars().take(8).collect::<String>()));

        let atom_type = self.extract_between(line, "[AtomType:", "]")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Coder".to_string());

        let description = line
            .split(" - ")
            .last()
            .unwrap_or(line)
            .trim()
            .to_string();

        let complexity = self.extract_between(line, "complexity:", "\n")
            .or_else(|| self.extract_between(line, "complexity: ", " "))
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(3);

        let seed_symbols = self.extract_between(line, "seed_symbols:", "]")
            .map(|s| {
                s.trim_start_matches('[')
                    .split(',')
                    .map(|sym| sym.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Some(MicroTask {
            id: task_id,
            description,
            atom_type,
            estimated_complexity: complexity,
            seed_symbols,
        })
    }

    /// Extract text between two markers
    fn extract_between(&self, text: &str, start: &str, end: &str) -> Option<String> {
        let start_idx = text.find(start)?;
        let after_start = start_idx + start.len();
        let end_idx = text[after_start..].find(end)?;
        Some(text[after_start..after_start + end_idx].to_string())
    }

    /// Parse dependencies from a line
    fn parse_dependencies(&self, line: &str, tasks: &[MicroTask]) -> Option<Vec<(String, String)>> {
        let deps_str = line.trim_start_matches("- depends_on:").trim();
        let deps_str = deps_str.trim_start_matches('[').trim_end_matches(']');

        let current_task = tasks.last()?;

        let deps: Vec<(String, String)> = deps_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .map(|dep| (current_task.id.clone(), dep))
            .collect();

        if deps.is_empty() { None } else { Some(deps) }
    }

    /// Infer atom type from description
    fn infer_atom_type(&self, description: &str) -> String {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("test") || desc_lower.contains("verify") {
            "Tester".to_string()
        } else if desc_lower.contains("review") || desc_lower.contains("check") {
            "Reviewer".to_string()
        } else if desc_lower.contains("design") || desc_lower.contains("architect") {
            "Architect".to_string()
        } else if desc_lower.contains("analyze") || desc_lower.contains("topology") {
            "Analyzer".to_string()
        } else {
            "Coder".to_string()
        }
    }

    /// Parse the ARCH.json section from response
    fn parse_arch_section(&self, response: &str) -> Result<ArchitectureOutput, String> {
        // Find JSON in the response
        let json_str = self.extract_json(response)?;

        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse ARCH.json: {}", e))?;

        let layers = parsed.get("layers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(LayerSpec {
                            name: item.get("name")?.as_str()?.to_string(),
                            level: item.get("level")?.as_u64()? as u8,
                            allowed_deps: item.get("allowed_deps")
                                .and_then(|v| v.as_array())
                                .map(|deps| deps.iter().filter_map(|d| d.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let modules = parsed.get("modules")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(super::ModuleSpec {
                            name: item.get("name")?.as_str()?.to_string(),
                            layer: item.get("layer")?.as_str()?.to_string(),
                            files: item.get("files")
                                .and_then(|v| v.as_array())
                                .map(|files| files.iter().filter_map(|f| f.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let constraints = parsed.get("constraints")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(ArchitectureOutput {
            arch_id: format!("arch_{}", uuid_simple()),
            layers,
            modules,
            constraints,
        })
    }

    /// Extract JSON from response
    fn extract_json(&self, response: &str) -> Result<String, String> {
        // Try to find JSON in code blocks
        if let Some(start) = response.find("```json") {
            if let Some(end) = response[start..].find("```\n").or_else(|| response[start..].rfind("```")) {
                let json_start = start + 7;
                let json_end = start + end;
                if json_start < json_end {
                    return Ok(response[json_start..json_end].trim().to_string());
                }
            }
        }

        // Try to find raw JSON
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if start < end {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        Err("No JSON found in response".to_string())
    }

    /// P2-3: Further decompose a task if it's too complex
    /// Now actually calls LLM to break down complex tasks
    pub async fn decompose_task(&self, task: &MicroTask, depth: usize) -> Result<Vec<MicroTask>, String> {
        // Check recursion limit
        if depth >= self.max_decomposition_depth {
            return Ok(vec![task.clone()]);
        }

        // If task is simple enough, return as-is
        if task.estimated_complexity <= self.target_task_complexity {
            return Ok(vec![task.clone()]);
        }

        // Build decomposition prompt
        let prompt = format!(
            r#"You are decomposing a complex task into smaller atomic sub-tasks.

## Original Task
- **ID**: {}
- **Description**: {}
- **Atom Type**: {}
- **Complexity**: {} (target: {})
- **Seed Symbols**: {:?}

## Instructions
Break this task into 2-4 smaller sub-tasks, each with complexity <= {}.
Each sub-task should be:
1. Self-contained and independently executable
2. Focused on a single responsibility
3. Small enough for a single LLM call

## Output Format (JSON array)
```json
[
  {{
    "id": "sub_task_id",
    "description": "What this sub-task does",
    "atom_type": "Coder|Search|Reviewer|Validator",
    "estimated_complexity": 1-5,
    "seed_symbols": ["relevant", "symbols"]
  }}
]
```

Output ONLY the JSON array, no other text."#,
            task.id, task.description, task.atom_type,
            task.estimated_complexity, self.target_task_complexity,
            task.seed_symbols, self.target_task_complexity
        );

        // Create LLM provider
        let provider = LlmProvider::new(self.llm_config.clone())
            .map_err(|e| format!("Failed to create LLM provider: {}", e))?;

        // Execute LLM call
        let messages = vec![
            Message::system("You are a task decomposition expert. Output only valid JSON."),
            Message::user(&prompt),
        ];

        let response = provider
            .complete(messages)
            .await
            .map_err(|e| format!("LLM call failed: {}", e))?;

        // Parse the response
        let sub_tasks = self.parse_subtasks(&response.content, &task.id)?;

        // Recursively decompose if any sub-task is still too complex
        let mut final_tasks = Vec::new();
        for sub_task in sub_tasks {
            if sub_task.estimated_complexity > self.target_task_complexity && depth + 1 < self.max_decomposition_depth {
                // Recursively decompose (using Box::pin for async recursion)
                let deeper_tasks = Box::pin(self.decompose_task(&sub_task, depth + 1)).await?;
                final_tasks.extend(deeper_tasks);
            } else {
                final_tasks.push(sub_task);
            }
        }

        Ok(final_tasks)
    }

    /// Parse sub-tasks from LLM response
    fn parse_subtasks(&self, response: &str, parent_id: &str) -> Result<Vec<MicroTask>, String> {
        // Find JSON array in response
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                return Err("No closing bracket found".to_string());
            }
        } else {
            return Err("No JSON array found in response".to_string());
        };

        // Parse JSON
        let parsed: Vec<serde_json::Value> = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        // Convert to MicroTask structs
        let mut tasks = Vec::new();
        for (i, item) in parsed.iter().enumerate() {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}_{}", parent_id, i + 1));

            let description = item.get("description")
                .and_then(|v| v.as_str())
                .ok_or("Missing description")?
                .to_string();

            let atom_type = item.get("atom_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Coder")
                .to_string();

            let estimated_complexity = item.get("estimated_complexity")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as u8;

            let seed_symbols = item.get("seed_symbols")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            tasks.push(MicroTask {
                id,
                description,
                atom_type,
                estimated_complexity,
                seed_symbols,
            });
        }

        if tasks.is_empty() {
            return Err("No valid sub-tasks parsed".to_string());
        }

        Ok(tasks)
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

