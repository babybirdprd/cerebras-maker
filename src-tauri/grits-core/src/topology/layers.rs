//! Layer configuration loading and validation for architectural constraints
//! 
//! This module handles loading layers.yaml files that define architectural
//! boundaries and allowed dependencies between layers.

use super::analysis::{LayerConfig, Layer};
use anyhow::{Context, Result};
use std::path::Path;

/// Load layer configuration from a workspace's layers.yaml file
/// 
/// # Arguments
/// * `workspace_path` - Path to the workspace root directory
/// 
/// # Returns
/// * `Ok(LayerConfig)` - Loaded configuration, or default if file doesn't exist
/// * `Err` - If file exists but cannot be parsed
pub fn load_layer_config(workspace_path: &Path) -> Result<LayerConfig> {
    let config_paths = [
        workspace_path.join("layers.yaml"),
        workspace_path.join("layers.yml"),
        workspace_path.join(".grits/layers.yaml"),
        workspace_path.join(".grits/layers.yml"),
    ];

    for config_path in &config_paths {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)
                .with_context(|| format!("Failed to read {}", config_path.display()))?;
            
            let config: LayerConfig = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", config_path.display()))?;
            
            tracing::info!("Loaded layer config from {}", config_path.display());
            return Ok(config);
        }
    }

    // Return default config if no file found
    tracing::debug!("No layers.yaml found, using default configuration");
    Ok(LayerConfig::default())
}

impl Default for LayerConfig {
    fn default() -> Self {
        LayerConfig {
            layers: vec![
                Layer {
                    name: "domain".to_string(),
                    patterns: vec!["*/domain/*".to_string(), "*/models/*".to_string(), "*/entities/*".to_string()],
                    allowed_deps: vec![],
                },
                Layer {
                    name: "application".to_string(),
                    patterns: vec!["*/services/*".to_string(), "*/handlers/*".to_string(), "*/use_cases/*".to_string()],
                    allowed_deps: vec!["domain".to_string()],
                },
                Layer {
                    name: "infrastructure".to_string(),
                    patterns: vec!["*/db/*".to_string(), "*/api/*".to_string(), "*/adapters/*".to_string()],
                    allowed_deps: vec!["domain".to_string(), "application".to_string()],
                },
                Layer {
                    name: "presentation".to_string(),
                    patterns: vec!["*/ui/*".to_string(), "*/views/*".to_string(), "*/components/*".to_string()],
                    allowed_deps: vec!["domain".to_string(), "application".to_string()],
                },
            ],
        }
    }
}

/// Validate that a layer configuration is internally consistent
pub fn validate_layer_config(config: &LayerConfig) -> Vec<String> {
    let mut errors = Vec::new();
    let layer_names: Vec<&str> = config.layers.iter().map(|l| l.name.as_str()).collect();

    for layer in &config.layers {
        // Check that all allowed_deps reference existing layers
        for dep in &layer.allowed_deps {
            if !layer_names.contains(&dep.as_str()) {
                errors.push(format!(
                    "Layer '{}' references unknown dependency layer '{}'",
                    layer.name, dep
                ));
            }
        }

        // Check for self-references (always allowed implicitly, but warn if explicit)
        if layer.allowed_deps.contains(&layer.name) {
            errors.push(format!(
                "Layer '{}' explicitly allows self-dependency (this is implicit)",
                layer.name
            ));
        }

        // Check for empty patterns
        if layer.patterns.is_empty() {
            errors.push(format!(
                "Layer '{}' has no patterns defined",
                layer.name
            ));
        }
    }

    // Check for duplicate layer names
    let mut seen = std::collections::HashSet::new();
    for name in &layer_names {
        if !seen.insert(name) {
            errors.push(format!("Duplicate layer name: '{}'", name));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_layer_config() {
        let config = LayerConfig::default();
        assert_eq!(config.layers.len(), 4);
        assert_eq!(config.layers[0].name, "domain");
        assert!(config.layers[0].allowed_deps.is_empty());
    }

    #[test]
    fn test_load_layer_config_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = load_layer_config(temp_dir.path()).unwrap();
        assert_eq!(config.layers.len(), 4); // Default config
    }

    #[test]
    fn test_load_layer_config_from_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_content = r#"
layers:
  - name: core
    patterns:
      - "*/core/*"
    allowed_deps: []
  - name: api
    patterns:
      - "*/api/*"
    allowed_deps:
      - core
"#;
        std::fs::write(temp_dir.path().join("layers.yaml"), yaml_content).unwrap();
        
        let config = load_layer_config(temp_dir.path()).unwrap();
        assert_eq!(config.layers.len(), 2);
        assert_eq!(config.layers[0].name, "core");
        assert_eq!(config.layers[1].allowed_deps, vec!["core"]);
    }

    #[test]
    fn test_validate_layer_config() {
        let config = LayerConfig {
            layers: vec![
                Layer {
                    name: "a".to_string(),
                    patterns: vec!["*/a/*".to_string()],
                    allowed_deps: vec!["nonexistent".to_string()],
                },
            ],
        };
        let errors = validate_layer_config(&config);
        assert!(errors.iter().any(|e| e.contains("nonexistent")));
    }
}

