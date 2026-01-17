//! Virtual Apply: Pre-commit validation of code changes
//!
//! This module implements the PRD's "Virtual Apply" concept: testing proposed
//! code changes against the SymbolGraph to detect architectural violations
//! BEFORE the code is written to disk.
//!
//! Key features:
//! - Parse proposed changes to extract new symbols and dependencies
//! - Build a temporary "virtual" graph with changes merged in
//! - Check for Betti_1 increases (new cycles)
//! - Check for layer violations
//! - Return detailed validation results

use super::analysis::{InvariantResult, LayerConfig, TopologicalAnalysis};
use super::{Symbol, SymbolGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A proposed code change to be virtually applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedChange {
    /// File path where the change will be applied
    pub file_path: String,
    /// The type of change
    pub change_type: ChangeType,
    /// Raw code content (for analysis)
    pub code_content: String,
    /// Programming language
    pub language: String,
}

/// Type of code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// New file creation
    CreateFile,
    /// Modification to existing file
    ModifyFile,
    /// File deletion
    DeleteFile,
}

/// Result of virtual apply validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualApplyResult {
    /// Whether the change is safe to apply
    pub is_safe: bool,
    /// Original Betti_1 (cycles) count
    pub original_betti_1: usize,
    /// New Betti_1 count after virtual apply
    pub new_betti_1: usize,
    /// Whether new cycles would be introduced
    pub introduces_cycles: bool,
    /// Layer violations that would be introduced
    pub layer_violations: Vec<LayerViolationDetail>,
    /// New symbols that would be added
    pub new_symbols: Vec<String>,
    /// New dependencies that would be added
    pub new_dependencies: Vec<(String, String, String)>, // (from, to, relation)
    /// Warnings (non-blocking issues)
    pub warnings: Vec<String>,
    /// Errors (blocking issues)
    pub errors: Vec<String>,
}

/// Detailed layer violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerViolationDetail {
    pub from_symbol: String,
    pub from_layer: String,
    pub to_symbol: String,
    pub to_layer: String,
    pub message: String,
}

impl VirtualApplyResult {
    /// Create a result indicating the change is safe
    pub fn safe(original_betti_1: usize, new_symbols: Vec<String>) -> Self {
        Self {
            is_safe: true,
            original_betti_1,
            new_betti_1: original_betti_1,
            introduces_cycles: false,
            layer_violations: Vec::new(),
            new_symbols,
            new_dependencies: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a result indicating the change would introduce problems
    pub fn unsafe_change(
        original_betti_1: usize,
        new_betti_1: usize,
        errors: Vec<String>,
    ) -> Self {
        Self {
            is_safe: false,
            original_betti_1,
            new_betti_1,
            introduces_cycles: new_betti_1 > original_betti_1,
            layer_violations: Vec::new(),
            new_symbols: Vec::new(),
            new_dependencies: Vec::new(),
            warnings: Vec::new(),
            errors,
        }
    }
}

/// Virtual apply engine for testing changes before applying
pub struct VirtualApply {
    /// The base symbol graph
    base_graph: SymbolGraph,
    /// Optional layer configuration
    layer_config: Option<LayerConfig>,
}

impl VirtualApply {
    /// Create a new VirtualApply instance
    pub fn new(base_graph: SymbolGraph, layer_config: Option<LayerConfig>) -> Self {
        Self {
            base_graph,
            layer_config,
        }
    }

    /// Validate proposed changes without applying them
    pub fn validate(&self, changes: &[ProposedChange]) -> VirtualApplyResult {
        // Get current analysis
        let original_analysis = TopologicalAnalysis::analyze(&self.base_graph);
        let original_betti_1 = original_analysis.betti_1;

        // Build virtual graph with changes applied
        let (virtual_graph, new_symbols, new_deps) = self.build_virtual_graph(changes);

        // Analyze virtual graph
        let virtual_analysis = TopologicalAnalysis::analyze(&virtual_graph);
        let new_betti_1 = virtual_analysis.betti_1;

        // Check for layer violations if config is available
        let layer_violations = if let Some(ref config) = self.layer_config {
            let invariant_result = InvariantResult::check(&virtual_graph, config);
            invariant_result
                .layer_violations
                .into_iter()
                .map(|v| {
                    let message = format!("Disallowed dependency: {} -> {}", &v.from_layer, &v.to_layer);
                    LayerViolationDetail {
                        from_symbol: v.from_node,
                        from_layer: v.from_layer,
                        to_symbol: v.to_node,
                        to_layer: v.to_layer,
                        message,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // Build errors list
        let mut errors = Vec::new();
        if new_betti_1 > original_betti_1 {
            errors.push(format!(
                "Would introduce {} new cycle(s) (Betti_1: {} -> {})",
                new_betti_1 - original_betti_1,
                original_betti_1,
                new_betti_1
            ));
        }
        for violation in &layer_violations {
            errors.push(violation.message.clone());
        }

        VirtualApplyResult {
            is_safe: errors.is_empty(),
            original_betti_1,
            new_betti_1,
            introduces_cycles: new_betti_1 > original_betti_1,
            layer_violations,
            new_symbols,
            new_dependencies: new_deps,
            warnings: Vec::new(),
            errors,
        }
    }

    /// Build a virtual graph with proposed changes applied
    fn build_virtual_graph(
        &self,
        changes: &[ProposedChange],
    ) -> (SymbolGraph, Vec<String>, Vec<(String, String, String)>) {
        let mut virtual_graph = self.base_graph.clone();
        let mut new_symbols = Vec::new();
        let mut new_deps = Vec::new();

        for change in changes {
            match change.change_type {
                ChangeType::CreateFile | ChangeType::ModifyFile => {
                    // Extract symbols and dependencies from the code content
                    let (symbols, deps) = self.extract_symbols_and_deps(change);

                    for symbol in symbols {
                        new_symbols.push(symbol.id.clone());
                        virtual_graph.add_symbol(symbol);
                    }

                    for (from, to, relation) in deps {
                        new_deps.push((from.clone(), to.clone(), relation.clone()));
                        virtual_graph.add_dependency(&from, &to, &relation);
                    }
                }
                ChangeType::DeleteFile => {
                    // Remove symbols from the file
                    let to_remove: Vec<String> = virtual_graph
                        .nodes
                        .iter()
                        .filter(|(_, s)| s.file_path == change.file_path)
                        .map(|(id, _)| id.clone())
                        .collect();

                    for id in to_remove {
                        virtual_graph.nodes.remove(&id);
                    }

                    // Remove edges involving deleted symbols
                    virtual_graph.edges.retain(|(from, to, _)| {
                        virtual_graph.nodes.contains_key(from) &&
                        virtual_graph.nodes.contains_key(to)
                    });
                }
            }
        }

        (virtual_graph, new_symbols, new_deps)
    }

    /// Extract symbols and dependencies from code content using pattern matching
    /// This is a simplified version - for production, use tree-sitter/ast-grep
    fn extract_symbols_and_deps(
        &self,
        change: &ProposedChange,
    ) -> (Vec<Symbol>, Vec<(String, String, String)>) {
        let mut symbols = Vec::new();
        let mut deps = Vec::new();
        let file_id_prefix = change.file_path.replace(['/', '\\', '.'], "_");

        // Simple pattern-based extraction (language-agnostic)
        for line in change.code_content.lines() {
            let trimmed = line.trim();

            // Detect function/method definitions
            if let Some(name) = self.extract_function_name(trimmed, &change.language) {
                let symbol_id = format!("{}::{}", file_id_prefix, name);
                symbols.push(Symbol {
                    id: symbol_id,
                    name,
                    file_path: change.file_path.clone(),
                    package: None,
                    language: change.language.clone(),
                    kind: "function".to_string(),
                    byte_range: None,
                    metadata: HashMap::new(),
                });
            }

            // Detect struct/class definitions
            if let Some(name) = self.extract_type_name(trimmed, &change.language) {
                let symbol_id = format!("{}::{}", file_id_prefix, name);
                symbols.push(Symbol {
                    id: symbol_id,
                    name,
                    file_path: change.file_path.clone(),
                    package: None,
                    language: change.language.clone(),
                    kind: "struct".to_string(),
                    byte_range: None,
                    metadata: HashMap::new(),
                });
            }

            // Detect imports/dependencies
            if let Some((from, to)) = self.extract_import(trimmed, &change.language, &file_id_prefix) {
                deps.push((from, to, "imports".to_string()));
            }
        }

        (symbols, deps)
    }

    /// Extract function name based on language
    fn extract_function_name(&self, line: &str, language: &str) -> Option<String> {
        match language {
            "rust" => {
                if line.starts_with("fn ") || line.starts_with("pub fn ") {
                    let after_fn = line.split("fn ").nth(1)?;
                    let name = after_fn.split('(').next()?.trim();
                    return Some(name.to_string());
                }
            }
            "typescript" | "javascript" => {
                if line.starts_with("function ") || line.contains("function ") {
                    let after_fn = line.split("function ").nth(1)?;
                    let name = after_fn.split('(').next()?.trim();
                    return Some(name.to_string());
                }
                // Arrow functions
                if line.contains("const ") && line.contains(" = ") && line.contains("=>") {
                    let after_const = line.split("const ").nth(1)?;
                    let name = after_const.split(&[' ', ':', '='][..]).next()?.trim();
                    return Some(name.to_string());
                }
            }
            "python" => {
                if line.starts_with("def ") {
                    let after_def = line.split("def ").nth(1)?;
                    let name = after_def.split('(').next()?.trim();
                    return Some(name.to_string());
                }
            }
            "go" => {
                if line.starts_with("func ") {
                    let after_func = line.split("func ").nth(1)?;
                    let name = after_func.split('(').next()?.trim();
                    return Some(name.to_string());
                }
            }
            _ => {}
        }
        None
    }

    /// Extract type (struct/class) name based on language
    fn extract_type_name(&self, line: &str, language: &str) -> Option<String> {
        match language {
            "rust" => {
                if line.starts_with("struct ") || line.starts_with("pub struct ") {
                    let after_struct = line.split("struct ").nth(1)?;
                    let name = after_struct.split(&[' ', '{', '('][..]).next()?.trim();
                    return Some(name.to_string());
                }
                if line.starts_with("enum ") || line.starts_with("pub enum ") {
                    let after_enum = line.split("enum ").nth(1)?;
                    let name = after_enum.split(&[' ', '{'][..]).next()?.trim();
                    return Some(name.to_string());
                }
            }
            "typescript" | "javascript" => {
                if line.starts_with("class ") || line.starts_with("export class ") {
                    let after_class = line.split("class ").nth(1)?;
                    let name = after_class.split(&[' ', '{', 'e'][..]).next()?.trim();
                    return Some(name.to_string());
                }
                if line.starts_with("interface ") || line.starts_with("export interface ") {
                    let after_interface = line.split("interface ").nth(1)?;
                    let name = after_interface.split(&[' ', '{'][..]).next()?.trim();
                    return Some(name.to_string());
                }
            }
            "python" => {
                if line.starts_with("class ") {
                    let after_class = line.split("class ").nth(1)?;
                    let name = after_class.split(&[':', '('][..]).next()?.trim();
                    return Some(name.to_string());
                }
            }
            "go" => {
                if line.starts_with("type ") && line.contains("struct") {
                    let after_type = line.split("type ").nth(1)?;
                    let name = after_type.split(' ').next()?.trim();
                    return Some(name.to_string());
                }
            }
            _ => {}
        }
        None
    }

    /// Extract import dependencies based on language
    fn extract_import(&self, line: &str, language: &str, current_file_id: &str) -> Option<(String, String)> {
        match language {
            "rust" => {
                if line.starts_with("use ") {
                    let after_use = line.strip_prefix("use ")?.trim_end_matches(';');
                    let module = after_use.split("::").next()?.to_string();
                    return Some((current_file_id.to_string(), module));
                }
            }
            "typescript" | "javascript" => {
                if line.contains("import ") && line.contains(" from ") {
                    let from_part = line.split(" from ").nth(1)?;
                    let module = from_part.trim().trim_matches(&['\'', '"', ';'][..]);
                    return Some((current_file_id.to_string(), module.to_string()));
                }
            }
            "python" => {
                if line.starts_with("import ") {
                    let module = line.strip_prefix("import ")?.split(&[' ', '.'][..]).next()?;
                    return Some((current_file_id.to_string(), module.to_string()));
                }
                if line.starts_with("from ") {
                    let module = line.strip_prefix("from ")?.split(' ').next()?;
                    return Some((current_file_id.to_string(), module.to_string()));
                }
            }
            "go" => {
                if line.contains("import ") || line.trim().starts_with('"') {
                    let pkg = line.trim().trim_matches(&['"', '(', ')'][..]);
                    if !pkg.is_empty() {
                        return Some((current_file_id.to_string(), pkg.to_string()));
                    }
                }
            }
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_apply_safe_change() {
        let graph = SymbolGraph::new();
        let va = VirtualApply::new(graph, None);

        let changes = vec![ProposedChange {
            file_path: "test.rs".to_string(),
            change_type: ChangeType::CreateFile,
            code_content: "fn hello() {}".to_string(),
            language: "rust".to_string(),
        }];

        let result = va.validate(&changes);
        assert!(result.is_safe);
        assert!(!result.introduces_cycles);
    }

    #[test]
    fn test_extract_rust_function() {
        let graph = SymbolGraph::new();
        let va = VirtualApply::new(graph, None);

        let name = va.extract_function_name("pub fn my_function(arg: i32) -> bool {", "rust");
        assert_eq!(name, Some("my_function".to_string()));
    }

    #[test]
    fn test_extract_typescript_class() {
        let graph = SymbolGraph::new();
        let va = VirtualApply::new(graph, None);

        let name = va.extract_type_name("export class MyComponent {", "typescript");
        assert_eq!(name, Some("MyComponent".to_string()));
    }
}

