// Cerebras-MAKER: Project Template System
// Pre-configured tech stacks for greenfield projects

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A project template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub tech_stack: Vec<String>,
    pub files: HashMap<String, String>,
    pub dependencies: TemplateDependencies,
    pub agent_config_preset: Option<String>,
}

/// Template categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    Desktop,
    Web,
    Mobile,
    CLI,
    Library,
}

/// Dependencies for a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDependencies {
    pub rust: Vec<String>,
    pub npm: Vec<String>,
    pub python: Vec<String>,
}

impl Default for TemplateDependencies {
    fn default() -> Self {
        Self {
            rust: Vec::new(),
            npm: Vec::new(),
            python: Vec::new(),
        }
    }
}

/// Template registry
pub struct TemplateRegistry {
    templates: HashMap<String, ProjectTemplate>,
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        
        // Register built-in templates
        registry.register(Self::tauri_react_template());
        registry.register(Self::tauri_vanilla_template());
        registry.register(Self::rust_cli_template());
        
        registry
    }
    
    pub fn register(&mut self, template: ProjectTemplate) {
        self.templates.insert(template.id.clone(), template);
    }
    
    pub fn get(&self, id: &str) -> Option<&ProjectTemplate> {
        self.templates.get(id)
    }
    
    pub fn list(&self) -> Vec<&ProjectTemplate> {
        self.templates.values().collect()
    }
    
    /// Create a project from a template
    pub fn create_project(&self, template_id: &str, project_path: &Path, project_name: &str) -> Result<(), String> {
        let template = self.get(template_id)
            .ok_or_else(|| format!("Template not found: {}", template_id))?;
        
        // Create project directory
        std::fs::create_dir_all(project_path)
            .map_err(|e| format!("Failed to create project directory: {}", e))?;
        
        // Write template files
        for (file_path, content) in &template.files {
            let full_path = project_path.join(file_path);
            
            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            
            // Replace placeholders
            let content = content
                .replace("{{PROJECT_NAME}}", project_name)
                .replace("{{project_name}}", &project_name.to_lowercase().replace(' ', "-"));
            
            std::fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write file {}: {}", file_path, e))?;
        }
        
        Ok(())
    }
    
    // Built-in templates
    
    fn tauri_react_template() -> ProjectTemplate {
        let mut files = HashMap::new();
        
        files.insert("package.json".to_string(), include_str!("tauri_react/package.json").to_string());
        files.insert("src-tauri/Cargo.toml".to_string(), include_str!("tauri_react/Cargo.toml").to_string());
        files.insert("src-tauri/tauri.conf.json".to_string(), include_str!("tauri_react/tauri.conf.json").to_string());
        files.insert("src/App.tsx".to_string(), include_str!("tauri_react/App.tsx").to_string());
        files.insert("src/main.tsx".to_string(), include_str!("tauri_react/main.tsx").to_string());
        files.insert("index.html".to_string(), include_str!("tauri_react/index.html").to_string());
        
        ProjectTemplate {
            id: "tauri-react".to_string(),
            name: "Tauri + React".to_string(),
            description: "Desktop application with Tauri backend and React frontend".to_string(),
            category: TemplateCategory::Desktop,
            tech_stack: vec!["Rust".to_string(), "TypeScript".to_string(), "React".to_string(), "Tauri".to_string()],
            files,
            dependencies: TemplateDependencies {
                rust: vec!["tauri".to_string(), "serde".to_string()],
                npm: vec!["react".to_string(), "@tauri-apps/api".to_string()],
                python: vec![],
            },
            agent_config_preset: Some("tauri-react".to_string()),
        }
    }
    
    fn tauri_vanilla_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "tauri-vanilla".to_string(),
            name: "Tauri + Vanilla JS".to_string(),
            description: "Lightweight desktop app with Tauri and vanilla JavaScript".to_string(),
            category: TemplateCategory::Desktop,
            tech_stack: vec!["Rust".to_string(), "JavaScript".to_string(), "Tauri".to_string()],
            files: HashMap::new(), // TODO: Add files
            dependencies: TemplateDependencies::default(),
            agent_config_preset: None,
        }
    }
    
    fn rust_cli_template() -> ProjectTemplate {
        ProjectTemplate {
            id: "rust-cli".to_string(),
            name: "Rust CLI".to_string(),
            description: "Command-line application in Rust".to_string(),
            category: TemplateCategory::CLI,
            tech_stack: vec!["Rust".to_string()],
            files: HashMap::new(), // TODO: Add files
            dependencies: TemplateDependencies {
                rust: vec!["clap".to_string(), "anyhow".to_string()],
                npm: vec![],
                python: vec![],
            },
            agent_config_preset: None,
        }
    }
}

