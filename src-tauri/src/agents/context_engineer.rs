// Cerebras-MAKER: The L3 Context Engineer
// PRD Section 3.1: "Semantic Tree-Shaking" - Extract only topologically-relevant code
// Takes a task from L2's Rhai script and produces a MiniCodebase for L4 Atoms

use super::{Agent, MicroTask};
use crate::grits;
use crate::llm::SystemPrompts;
use grits_core::context::MiniCodebase;
use grits_core::topology::SymbolGraph;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for context extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum depth for symbol traversal (1-hop = direct deps, 2-hop = signatures)
    pub max_depth: usize,
    /// Minimum edge strength to include
    pub strength_threshold: f32,
    /// Target line count for context (~50 lines)
    pub target_lines: usize,
    /// Whether to include full bodies or just signatures for 2-hop deps
    pub signatures_only_2hop: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            strength_threshold: 0.5,
            target_lines: 50,
            signatures_only_2hop: true,
        }
    }
}

/// Atom-specific context requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomContextRequirements {
    /// The type of atom requesting context
    pub atom_type: String,
    /// Whether to include test examples
    pub include_tests: bool,
    /// Whether to include style guide excerpts
    pub include_style_guide: bool,
    /// Whether to include dependency graph info
    pub include_dependency_info: bool,
}

impl AtomContextRequirements {
    /// Create requirements based on atom type
    pub fn for_atom_type(atom_type: &str) -> Self {
        match atom_type {
            "Coder" => Self {
                atom_type: atom_type.to_string(),
                include_tests: true,
                include_style_guide: false,
                include_dependency_info: false,
            },
            "Reviewer" => Self {
                atom_type: atom_type.to_string(),
                include_tests: true,
                include_style_guide: true,
                include_dependency_info: false,
            },
            "Tester" => Self {
                atom_type: atom_type.to_string(),
                include_tests: true,
                include_style_guide: false,
                include_dependency_info: false,
            },
            "GritsAnalyzer" => Self {
                atom_type: atom_type.to_string(),
                include_tests: false,
                include_style_guide: false,
                include_dependency_info: true,
            },
            _ => Self {
                atom_type: atom_type.to_string(),
                include_tests: false,
                include_style_guide: false,
                include_dependency_info: false,
            },
        }
    }
}

/// The output of the Context Engineer - a focused context package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPackage {
    /// The task this context was assembled for
    pub task_id: String,
    /// The atom type this context is tailored for
    pub atom_type: String,
    /// Total lines of context provided
    pub context_lines: usize,
    /// The assembled MiniCodebase
    pub mini_codebase: MiniCodebase,
    /// Rendered markdown for LLM consumption
    pub markdown: String,
    /// Constraints and invariants to preserve
    pub constraints: Vec<String>,
    /// Quality metrics
    pub metrics: ContextMetrics,
}

/// Quality metrics for the context extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetrics {
    /// Number of seed symbols used
    pub seed_count: usize,
    /// Number of symbols extracted
    pub symbol_count: usize,
    /// Number of files involved
    pub file_count: usize,
    /// Estimated precision (% of context likely to be used)
    pub estimated_precision: f32,
    /// Solid score from grits analysis
    pub solid_score: f32,
}

/// The L3 Context Engineer Agent
/// Responsible for extracting minimal, precise context for L4 Atoms
pub struct ContextEngineer {
    config: ContextConfig,
}

impl Default for ContextEngineer {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ContextEngineer {
    fn name(&self) -> &str {
        "ContextEngineer"
    }

    fn system_prompt(&self) -> &str {
        SystemPrompts::context_engineer()
    }
}

impl ContextEngineer {
    pub fn new() -> Self {
        Self {
            config: ContextConfig::default(),
        }
    }

    pub fn with_config(config: ContextConfig) -> Self {
        Self { config }
    }

    /// Extract context for a micro-task
    /// This is the main L3 entry point
    pub fn extract_context(
        &self,
        task: &MicroTask,
        workspace_path: &Path,
    ) -> Result<ContextPackage, String> {
        // Get or load the symbol graph
        let graph = grits::load_workspace_graph(workspace_path.to_string_lossy().as_ref())?;

        self.extract_context_with_graph(task, &graph, workspace_path)
    }

    /// Extract context using a pre-loaded symbol graph
    pub fn extract_context_with_graph(
        &self,
        task: &MicroTask,
        graph: &SymbolGraph,
        workspace_path: &Path,
    ) -> Result<ContextPackage, String> {
        // Get atom-specific requirements
        let requirements = AtomContextRequirements::for_atom_type(&task.atom_type);

        // Determine seed symbols - use task's seeds or infer from description
        let seed_symbols = if task.seed_symbols.is_empty() {
            self.infer_seed_symbols(&task.description, graph)
        } else {
            task.seed_symbols.clone()
        };

        if seed_symbols.is_empty() {
            return Err("No seed symbols found for context extraction".to_string());
        }

        // Assemble the MiniCodebase using grits-core
        let mut mini_codebase = grits::assemble_context(
            graph,
            seed_symbols.clone(),
            self.config.max_depth,
            self.config.strength_threshold,
            Some(task.id.clone()),
        );

        // Hydrate with actual code content
        mini_codebase.hydrate_code(workspace_path);

        // Build constraints from invariants
        let mut constraints = Vec::new();
        for note in &mini_codebase.invariants.notes {
            constraints.push(note.clone());
        }
        if mini_codebase.invariants.betti_1 > 0 {
            constraints.push(format!(
                "Preserve cycle count (Betti₁ = {})",
                mini_codebase.invariants.betti_1
            ));
        }
        for forbidden in &mini_codebase.invariants.forbidden_dependencies {
            constraints.push(format!("Do not depend on: {}", forbidden));
        }

        // Calculate metrics
        let metrics = ContextMetrics {
            seed_count: seed_symbols.len(),
            symbol_count: mini_codebase.symbols.len(),
            file_count: mini_codebase.files.len(),
            estimated_precision: self.estimate_precision(&requirements, &mini_codebase),
            solid_score: mini_codebase.metadata.solid_score,
        };

        // Render to markdown for LLM consumption
        let markdown = self.render_for_atom(&task.atom_type, &mini_codebase, &requirements);

        // Count actual lines
        let context_lines = markdown.lines().count();

        Ok(ContextPackage {
            task_id: task.id.clone(),
            atom_type: task.atom_type.clone(),
            context_lines,
            mini_codebase,
            markdown,
            constraints,
            metrics,
        })
    }

    /// Infer seed symbols from task description by matching against graph nodes
    fn infer_seed_symbols(&self, description: &str, graph: &SymbolGraph) -> Vec<String> {
        let desc_lower = description.to_lowercase();
        let mut seeds = Vec::new();

        // Look for symbol names mentioned in the description
        for (id, symbol) in &graph.nodes {
            let name_lower = symbol.name.to_lowercase();
            if desc_lower.contains(&name_lower) && name_lower.len() > 3 {
                seeds.push(id.clone());
            }
        }

        // Limit to top 5 seeds to avoid context explosion
        seeds.truncate(5);
        seeds
    }

    /// Estimate precision based on atom type and context size
    fn estimate_precision(
        &self,
        requirements: &AtomContextRequirements,
        mini_codebase: &MiniCodebase,
    ) -> f32 {
        // Base precision from solid score
        let base = mini_codebase.metadata.solid_score;

        // Adjust based on context size (smaller is more precise)
        let size_factor = if mini_codebase.symbols.len() <= 5 {
            1.0
        } else if mini_codebase.symbols.len() <= 10 {
            0.95
        } else {
            0.85
        };

        // Adjust based on atom type specificity
        let type_factor = match requirements.atom_type.as_str() {
            "GritsAnalyzer" => 0.95, // Very focused
            "Tester" => 0.90,
            "Coder" => 0.85,
            "Reviewer" => 0.80, // Needs broader context
            _ => 0.85,
        };

        (base * size_factor * type_factor).min(1.0)
    }

    /// Render the MiniCodebase as markdown tailored for the atom type
    fn render_for_atom(
        &self,
        atom_type: &str,
        mini_codebase: &MiniCodebase,
        requirements: &AtomContextRequirements,
    ) -> String {
        let mut md = String::new();

        // Header with context info
        md.push_str(&format!("# Context for {} Atom\n\n", atom_type));
        md.push_str(&format!(
            "**Symbols**: {} | **Files**: {} | **Solid Score**: {:.0}%\n\n",
            mini_codebase.symbols.len(),
            mini_codebase.files.len(),
            mini_codebase.metadata.solid_score * 100.0
        ));

        // Constraints section
        if !mini_codebase.invariants.notes.is_empty()
            || mini_codebase.invariants.betti_1 > 0
            || !mini_codebase.invariants.forbidden_dependencies.is_empty()
        {
            md.push_str("## Constraints\n\n");
            for note in &mini_codebase.invariants.notes {
                md.push_str(&format!("- {}\n", note));
            }
            if mini_codebase.invariants.betti_1 > 0 {
                md.push_str(&format!(
                    "- ⚠️ Betti₁ = {} (do not introduce new cycles)\n",
                    mini_codebase.invariants.betti_1
                ));
            }
            for forbidden in &mini_codebase.invariants.forbidden_dependencies {
                md.push_str(&format!("- ❌ Do not depend on: `{}`\n", forbidden));
            }
            md.push_str("\n");
        }

        // Dependency info for GritsAnalyzer
        if requirements.include_dependency_info {
            md.push_str("## Dependency Graph\n\n");
            md.push_str("```\n");
            for symbol in &mini_codebase.symbols {
                if symbol.in_cycle {
                    md.push_str(&format!("🔄 {} (in cycle)\n", symbol.name));
                } else {
                    md.push_str(&format!("   {}\n", symbol.name));
                }
            }
            md.push_str("```\n\n");
        }

        // Code sections
        md.push_str("## Code\n\n");
        for symbol in &mini_codebase.symbols {
            md.push_str(&format!(
                "### `{}` ({})\n",
                symbol.name, symbol.kind
            ));
            md.push_str(&format!("*File: {}*\n\n", symbol.file_path));

            if let Some(code) = &symbol.code {
                md.push_str("```\n");
                md.push_str(code);
                if !code.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }
        }

        md
    }
}

