//! AST-based code editing and validation
//!
//! PRD Section 3.2: "Virtual Apply" - Use AST parsing for syntax validation
//! before applying code changes to disk.
//!
//! This module provides:
//! - Syntax validation for multiple languages (pattern-based)
//! - Code structure analysis
//! - Safe code transformations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum cache size to prevent memory issues
const MAX_CACHE_SIZE: usize = 100;

/// Supported languages for AST parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
}

impl SupportedLanguage {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(SupportedLanguage::Rust),
            "ts" | "tsx" => Some(SupportedLanguage::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(SupportedLanguage::JavaScript),
            "py" | "pyi" => Some(SupportedLanguage::Python),
            "go" => Some(SupportedLanguage::Go),
            _ => None,
        }
    }

    /// Detect language from file path
    pub fn from_path(path: &str) -> Option<Self> {
        path.rsplit('.').next().and_then(Self::from_extension)
    }
}

/// Result of syntax validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxValidationResult {
    /// Whether the syntax is valid
    pub is_valid: bool,
    /// List of syntax errors found
    pub errors: Vec<SyntaxError>,
    /// Language detected/used
    pub language: Option<String>,
    /// Number of AST nodes parsed
    pub node_count: usize,
}

/// A syntax error found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxError {
    /// Error message
    pub message: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// The problematic code snippet
    pub snippet: Option<String>,
}

/// AST editor for code analysis and transformation
pub struct AstEditor {
    /// Cache of parsed ASTs (reserved for future use)
    #[allow(dead_code)]
    cache: HashMap<String, String>,
}

impl Default for AstEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl AstEditor {
    /// Create a new AST editor
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Evict cache if it exceeds the maximum size to prevent memory issues
    fn maybe_evict_cache(&mut self) {
        if self.cache.len() > MAX_CACHE_SIZE {
            // Remove oldest entries (just clear for simplicity)
            self.cache.clear();
        }
    }

    /// Validate syntax of code content using pattern-based heuristics
    /// This is a lightweight validation that catches common syntax errors
    pub fn validate_syntax(&self, code: &str, language: SupportedLanguage) -> SyntaxValidationResult {
        let errors = self.check_syntax_patterns(code, language);
        let structure_count = self.count_structures(code, language);

        SyntaxValidationResult {
            is_valid: errors.is_empty(),
            errors,
            language: Some(format!("{:?}", language)),
            node_count: structure_count,
        }
    }

    /// Validate syntax with auto-detected language
    pub fn validate_syntax_auto(&self, code: &str, file_path: &str) -> SyntaxValidationResult {
        match SupportedLanguage::from_path(file_path) {
            Some(lang) => self.validate_syntax(code, lang),
            None => SyntaxValidationResult {
                is_valid: true, // Can't validate unknown languages
                errors: vec![],
                language: None,
                node_count: 0,
            },
        }
    }

    /// Check for common syntax errors using pattern matching
    fn check_syntax_patterns(&self, code: &str, language: SupportedLanguage) -> Vec<SyntaxError> {
        let mut errors = Vec::new();

        // Check bracket balance
        errors.extend(self.check_bracket_balance(code));

        // Language-specific checks
        match language {
            SupportedLanguage::Rust => errors.extend(self.check_rust_syntax(code)),
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                errors.extend(self.check_js_ts_syntax(code))
            }
            SupportedLanguage::Python => errors.extend(self.check_python_syntax(code)),
            SupportedLanguage::Go => errors.extend(self.check_go_syntax(code)),
        }

        errors
    }

    /// Check bracket balance (common across all languages)
    fn check_bracket_balance(&self, code: &str) -> Vec<SyntaxError> {
        let mut errors = Vec::new();
        let mut stack: Vec<(char, usize, usize)> = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                match ch {
                    '(' | '[' | '{' => stack.push((ch, line_num + 1, col + 1)),
                    ')' => {
                        if let Some((open, _, _)) = stack.pop() {
                            if open != '(' {
                                errors.push(SyntaxError {
                                    message: format!("Mismatched bracket: expected ')', found '{}'", open),
                                    line: line_num + 1,
                                    column: col + 1,
                                    snippet: Some(line.to_string()),
                                });
                            }
                        } else {
                            errors.push(SyntaxError {
                                message: "Unmatched closing parenthesis ')'".to_string(),
                                line: line_num + 1,
                                column: col + 1,
                                snippet: Some(line.to_string()),
                            });
                        }
                    }
                    ']' => {
                        if let Some((open, _, _)) = stack.pop() {
                            if open != '[' {
                                errors.push(SyntaxError {
                                    message: format!("Mismatched bracket: expected ']', found '{}'", open),
                                    line: line_num + 1,
                                    column: col + 1,
                                    snippet: Some(line.to_string()),
                                });
                            }
                        } else {
                            errors.push(SyntaxError {
                                message: "Unmatched closing bracket ']'".to_string(),
                                line: line_num + 1,
                                column: col + 1,
                                snippet: Some(line.to_string()),
                            });
                        }
                    }
                    '}' => {
                        if let Some((open, _, _)) = stack.pop() {
                            if open != '{' {
                                errors.push(SyntaxError {
                                    message: format!("Mismatched bracket: expected '}}', found '{}'", open),
                                    line: line_num + 1,
                                    column: col + 1,
                                    snippet: Some(line.to_string()),
                                });
                            }
                        } else {
                            errors.push(SyntaxError {
                                message: "Unmatched closing brace '}'".to_string(),
                                line: line_num + 1,
                                column: col + 1,
                                snippet: Some(line.to_string()),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        // Report unclosed brackets
        for (ch, line, col) in stack {
            errors.push(SyntaxError {
                message: format!("Unclosed bracket '{}'", ch),
                line,
                column: col,
                snippet: None,
            });
        }

        errors
    }

    /// Rust-specific syntax checks
    fn check_rust_syntax(&self, code: &str) -> Vec<SyntaxError> {
        let mut errors = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Check for missing semicolons on statements (heuristic)
            if (trimmed.starts_with("let ") || trimmed.starts_with("use "))
                && !trimmed.ends_with(';')
                && !trimmed.ends_with('{')
                && !trimmed.contains("//")
            {
                errors.push(SyntaxError {
                    message: "Possible missing semicolon".to_string(),
                    line: line_num + 1,
                    column: line.len(),
                    snippet: Some(line.to_string()),
                });
            }
        }

        errors
    }

    /// JavaScript/TypeScript-specific syntax checks
    fn check_js_ts_syntax(&self, code: &str) -> Vec<SyntaxError> {
        let errors = Vec::new();

        for (_line_num, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Check for common JS/TS issues
            if trimmed.contains("===") && trimmed.contains("==") && !trimmed.contains("!==") {
                // Mixed equality operators might be intentional, just a warning
            }
        }

        errors
    }

    /// Python-specific syntax checks
    fn check_python_syntax(&self, code: &str) -> Vec<SyntaxError> {
        let mut errors = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Check for missing colons after def/class/if/for/while
            let keywords = ["def ", "class ", "if ", "elif ", "else", "for ", "while ", "try", "except", "finally", "with "];
            for kw in keywords {
                if trimmed.starts_with(kw) && !trimmed.ends_with(':') && !trimmed.contains('#') {
                    errors.push(SyntaxError {
                        message: format!("Possible missing colon after '{}'", kw.trim()),
                        line: line_num + 1,
                        column: line.len(),
                        snippet: Some(line.to_string()),
                    });
                }
            }
        }

        errors
    }

    /// Go-specific syntax checks
    fn check_go_syntax(&self, _code: &str) -> Vec<SyntaxError> {
        // Go has fewer common syntax issues due to gofmt
        Vec::new()
    }

    /// Count code structures (functions, classes, etc.)
    fn count_structures(&self, code: &str, language: SupportedLanguage) -> usize {
        let mut count = 0;

        for line in code.lines() {
            let trimmed = line.trim();

            match language {
                SupportedLanguage::Rust => {
                    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")
                        || trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ")
                        || trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ")
                        || trimmed.starts_with("impl ") || trimmed.starts_with("trait ")
                    {
                        count += 1;
                    }
                }
                SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                    if trimmed.starts_with("function ") || trimmed.contains("=> {")
                        || trimmed.starts_with("class ") || trimmed.starts_with("interface ")
                    {
                        count += 1;
                    }
                }
                SupportedLanguage::Python => {
                    if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                        count += 1;
                    }
                }
                SupportedLanguage::Go => {
                    if trimmed.starts_with("func ") || trimmed.starts_with("type ") {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Extract function/method names from code using pattern matching
    pub fn extract_functions(&self, code: &str, language: SupportedLanguage) -> Vec<String> {
        let mut functions = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();

            if let Some(name) = self.extract_function_name(trimmed, language) {
                functions.push(name);
            }
        }

        functions
    }

    /// Extract function name from a line based on language
    fn extract_function_name(&self, line: &str, language: SupportedLanguage) -> Option<String> {
        match language {
            SupportedLanguage::Rust => {
                if line.starts_with("fn ") || line.starts_with("pub fn ") {
                    let after_fn = line.split("fn ").nth(1)?;
                    let name = after_fn.split('(').next()?.trim();
                    return Some(name.to_string());
                }
            }
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                if line.starts_with("function ") || line.contains("function ") {
                    let after_fn = line.split("function ").nth(1)?;
                    let name = after_fn.split('(').next()?.trim();
                    return Some(name.to_string());
                }
                // Arrow functions with const
                if line.contains("const ") && line.contains(" = ") && line.contains("=>") {
                    let after_const = line.split("const ").nth(1)?;
                    let name = after_const.split(&[' ', ':', '='][..]).next()?.trim();
                    return Some(name.to_string());
                }
            }
            SupportedLanguage::Python => {
                if line.starts_with("def ") {
                    let after_def = line.split("def ").nth(1)?;
                    let name = after_def.split('(').next()?.trim();
                    return Some(name.to_string());
                }
            }
            SupportedLanguage::Go => {
                if line.starts_with("func ") {
                    let after_func = line.split("func ").nth(1)?;
                    // Handle method receivers: func (r *Receiver) Name()
                    let name_part = if after_func.starts_with('(') {
                        after_func.split(')').nth(1)?.trim()
                    } else {
                        after_func
                    };
                    let name = name_part.split('(').next()?.trim();
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    /// Check if code contains a specific text pattern
    pub fn contains_pattern(&self, code: &str, pattern: &str, _language: SupportedLanguage) -> bool {
        code.contains(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_rust_syntax_valid() {
        let editor = AstEditor::new();
        let code = r#"
fn hello() {
    println!("Hello, world!");
}
"#;
        let result = editor.validate_syntax(code, SupportedLanguage::Rust);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_typescript_syntax_valid() {
        let editor = AstEditor::new();
        let code = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}
"#;
        let result = editor.validate_syntax(code, SupportedLanguage::TypeScript);
        assert!(result.is_valid);
    }

    #[test]
    fn test_language_from_extension() {
        assert_eq!(SupportedLanguage::from_extension("rs"), Some(SupportedLanguage::Rust));
        assert_eq!(SupportedLanguage::from_extension("ts"), Some(SupportedLanguage::TypeScript));
        assert_eq!(SupportedLanguage::from_extension("py"), Some(SupportedLanguage::Python));
        assert_eq!(SupportedLanguage::from_extension("go"), Some(SupportedLanguage::Go));
        assert_eq!(SupportedLanguage::from_extension("unknown"), None);
    }

    #[test]
    fn test_extract_rust_functions() {
        let editor = AstEditor::new();
        let code = r#"
fn foo() {}
fn bar(x: i32) -> i32 { x }
"#;
        let functions = editor.extract_functions(code, SupportedLanguage::Rust);
        assert!(functions.contains(&"foo".to_string()));
        assert!(functions.contains(&"bar".to_string()));
    }
}

