// Cerebras-MAKER: Prompt Engineering System
// Structured prompts with templates and context injection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt template with variable substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub required_vars: Vec<String>,
}

impl PromptTemplate {
    pub fn new(name: &str, template: &str) -> Self {
        // Extract variables like {{var_name}}
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let required_vars: Vec<String> = re.captures_iter(template)
            .map(|c| c[1].to_string())
            .collect();

        Self {
            name: name.to_string(),
            template: template.to_string(),
            required_vars,
        }
    }

    /// Render the template with provided variables
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.template.clone();

        for var in &self.required_vars {
            let placeholder = format!("{{{{{}}}}}", var);
            let value = vars.get(var)
                .ok_or_else(|| format!("Missing required variable: {}", var))?;
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

/// Context for prompt rendering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptContext {
    pub workspace_path: Option<String>,
    pub issue_id: Option<String>,
    pub code_context: Option<String>,
    pub previous_outputs: Vec<String>,
    pub constraints: Vec<String>,
    pub custom_vars: HashMap<String, String>,
}

impl PromptContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workspace(mut self, path: &str) -> Self {
        self.workspace_path = Some(path.to_string());
        self
    }

    pub fn with_code(mut self, code: &str) -> Self {
        self.code_context = Some(code.to_string());
        self
    }

    pub fn with_var(mut self, key: &str, value: &str) -> Self {
        self.custom_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Convert to HashMap for template rendering
    pub fn to_vars(&self) -> HashMap<String, String> {
        let mut vars = self.custom_vars.clone();

        if let Some(ref path) = self.workspace_path {
            vars.insert("workspace_path".to_string(), path.clone());
        }
        if let Some(ref issue) = self.issue_id {
            vars.insert("issue_id".to_string(), issue.clone());
        }
        if let Some(ref code) = self.code_context {
            vars.insert("code_context".to_string(), code.clone());
        }
        if !self.constraints.is_empty() {
            vars.insert("constraints".to_string(), self.constraints.join("\n- "));
        }

        vars
    }
}

/// Collection of system prompts for all agents
pub struct SystemPrompts;

impl SystemPrompts {
    /// Interrogator agent system prompt
    pub fn interrogator() -> &'static str {
        include_str!("../../prompts/interrogator.md")
    }

    /// Architect agent system prompt
    pub fn architect() -> &'static str {
        include_str!("../../prompts/architect.md")
    }

    /// Orchestrator agent system prompt
    pub fn orchestrator() -> &'static str {
        include_str!("../../prompts/orchestrator.md")
    }

    /// Script generator system prompt
    pub fn script_generator() -> &'static str {
        include_str!("../../prompts/script_generator.md")
    }

    /// Code generation atom prompt
    pub fn atom_coder() -> &'static str {
        include_str!("../../prompts/atom_coder.md")
    }

    /// Search atom prompt
    pub fn atom_search() -> &'static str {
        include_str!("../../prompts/atom_search.md")
    }

    /// Reviewer atom prompt
    pub fn atom_reviewer() -> &'static str {
        include_str!("../../prompts/atom_reviewer.md")
    }

    /// Tester atom prompt
    pub fn atom_tester() -> &'static str {
        include_str!("../../prompts/atom_tester.md")
    }

    /// GritsAnalyzer atom prompt
    pub fn atom_grits() -> &'static str {
        include_str!("../../prompts/atom_grits.md")
    }

    /// RLM Processor atom prompt
    /// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
    pub fn atom_rlm_processor() -> &'static str {
        include_str!("../../prompts/atom_rlm_processor.md")
    }

    /// L3 Context Engineer prompt
    pub fn context_engineer() -> &'static str {
        include_str!("../../prompts/context_engineer.md")
    }

    /// Get prompt by atom type
    pub fn for_atom_type(atom_type: &str) -> &'static str {
        match atom_type {
            "Coder" => Self::atom_coder(),
            "Search" => Self::atom_search(),
            "Reviewer" => Self::atom_reviewer(),
            "Tester" => Self::atom_tester(),
            "GritsAnalyzer" => Self::atom_grits(),
            "RLMProcessor" => Self::atom_rlm_processor(),
            "Architect" => Self::architect(),
            _ => Self::atom_coder(), // Default to coder
        }
    }
}

