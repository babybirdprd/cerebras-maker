//! Dynamic builtin detection for topology analysis.
//!
//! This module provides configurable builtin symbol detection, allowing users
//! to customize which symbols are considered "builtins" and excluded from
//! topology analysis. Builtins can be loaded from TOML configuration files
//! or use embedded defaults.
//!
//! # TOML Configuration Format
//!
//! ```toml
//! [rust]
//! primitives = ["bool", "char", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"]
//! std_types = ["String", "Vec", "HashMap", "Option", "Result", "Box", "Rc", "Arc"]
//! macros = ["println", "format", "vec", "assert", "panic", "dbg"]
//! traits = ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"]
//! methods = ["clone", "into", "from", "unwrap", "expect", "map", "iter"]
//!
//! [javascript]
//! globals = ["console", "window", "document", "Math", "JSON", "Promise"]
//! types = ["Array", "Object", "String", "Number", "Boolean", "Map", "Set"]
//! functions = ["setTimeout", "setInterval", "fetch", "parseInt", "parseFloat"]
//!
//! [python]
//! builtins = ["print", "len", "range", "str", "int", "float", "list", "dict"]
//! exceptions = ["Exception", "ValueError", "TypeError", "KeyError"]
//!
//! [go]
//! builtins = ["fmt", "make", "new", "len", "cap", "append", "panic", "recover"]
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Configuration for language-specific builtin symbols.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageBuiltins {
    /// Primitive types (e.g., bool, i32, str)
    #[serde(default)]
    pub primitives: Vec<String>,
    /// Standard library types (e.g., String, Vec, HashMap)
    #[serde(default)]
    pub std_types: Vec<String>,
    /// Macros (e.g., println!, format!, vec!)
    #[serde(default)]
    pub macros: Vec<String>,
    /// Traits/interfaces (e.g., Clone, Copy, Debug)
    #[serde(default)]
    pub traits: Vec<String>,
    /// Common method names (e.g., clone, into, unwrap)
    #[serde(default)]
    pub methods: Vec<String>,
    /// Global objects/functions (e.g., console, window, Math)
    #[serde(default)]
    pub globals: Vec<String>,
    /// Type constructors (e.g., Array, Object, Map)
    #[serde(default)]
    pub types: Vec<String>,
    /// Standalone functions (e.g., setTimeout, fetch)
    #[serde(default)]
    pub functions: Vec<String>,
    /// Built-in functions (Python/Go style)
    #[serde(default)]
    pub builtins: Vec<String>,
    /// Exception/error types
    #[serde(default)]
    pub exceptions: Vec<String>,
    /// Keywords that might get captured
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl LanguageBuiltins {
    /// Collect all builtins into a single HashSet for fast lookup.
    pub fn to_set(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        set.extend(self.primitives.iter().cloned());
        set.extend(self.std_types.iter().cloned());
        set.extend(self.macros.iter().cloned());
        set.extend(self.traits.iter().cloned());
        set.extend(self.methods.iter().cloned());
        set.extend(self.globals.iter().cloned());
        set.extend(self.types.iter().cloned());
        set.extend(self.functions.iter().cloned());
        set.extend(self.builtins.iter().cloned());
        set.extend(self.exceptions.iter().cloned());
        set.extend(self.keywords.iter().cloned());
        set
    }
}

/// Full builtin configuration across all languages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuiltinConfig {
    #[serde(default)]
    pub rust: LanguageBuiltins,
    #[serde(default)]
    pub javascript: LanguageBuiltins,
    #[serde(default)]
    pub typescript: LanguageBuiltins,
    #[serde(default)]
    pub python: LanguageBuiltins,
    #[serde(default)]
    pub go: LanguageBuiltins,
}

/// Detector for builtin symbols with compiled lookup tables.
#[derive(Debug, Clone)]
pub struct BuiltinDetector {
    rust_builtins: HashSet<String>,
    js_builtins: HashSet<String>,
    ts_builtins: HashSet<String>,
    python_builtins: HashSet<String>,
    go_builtins: HashSet<String>,
}

impl Default for BuiltinDetector {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl BuiltinDetector {
    /// Create a new BuiltinDetector with default embedded builtins.
    pub fn with_defaults() -> Self {
        let config = default_builtin_config();
        Self::from_config(&config)
    }

    /// Create a BuiltinDetector from a BuiltinConfig.
    pub fn from_config(config: &BuiltinConfig) -> Self {
        // Merge typescript with javascript if typescript is empty
        let ts_set = if config.typescript.to_set().is_empty() {
            config.javascript.to_set()
        } else {
            let mut set = config.javascript.to_set();
            set.extend(config.typescript.to_set());
            set
        };

        Self {
            rust_builtins: config.rust.to_set(),
            js_builtins: config.javascript.to_set(),
            ts_builtins: ts_set,
            python_builtins: config.python.to_set(),
            go_builtins: config.go.to_set(),
        }
    }

    /// Load builtins from a TOML configuration file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: BuiltinConfig = toml::from_str(&content)?;
        Ok(Self::from_config(&config))
    }

    /// Load builtins from TOML string.
    pub fn load_from_str(toml_str: &str) -> anyhow::Result<Self> {
        let config: BuiltinConfig = toml::from_str(toml_str)?;
        Ok(Self::from_config(&config))
    }

    /// Check if a symbol is a builtin for the given language.
    ///
    /// Also performs pattern-based checks for common cases:
    /// - Single uppercase letters (generic type params like T, E, F)
    /// - Compound expressions containing "()" or "::"
    /// - Numeric literals and simple strings
    pub fn is_builtin(&self, name: &str, language: &str) -> bool {
        // Quick pattern-based checks (before expensive set lookups)
        if self.is_pattern_builtin(name) {
            return true;
        }

        match language {
            "rust" => self.rust_builtins.contains(name),
            "typescript" | "ts" => self.ts_builtins.contains(name),
            "javascript" | "js" => self.js_builtins.contains(name),
            "python" | "py" => self.python_builtins.contains(name),
            "go" => self.go_builtins.contains(name),
            _ => false,
        }
    }

    /// Pattern-based builtin detection (language-agnostic).
    fn is_pattern_builtin(&self, name: &str) -> bool {
        // Filter single-letter identifiers (generic type params like T, E, F)
        if name.len() == 1
            && name
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_uppercase())
        {
            return true;
        }

        // Filter compound expressions that got captured (e.g., "Ok(())")
        if name.contains("()") || name.contains("::") {
            return true;
        }

        // Filter numeric literals and simple strings
        if name.parse::<f64>().is_ok() || name.starts_with('"') || name.starts_with('\'') {
            return true;
        }

        false
    }

    /// Get the set of builtins for a specific language.
    pub fn get_builtins_for_language(&self, language: &str) -> &HashSet<String> {
        match language {
            "rust" => &self.rust_builtins,
            "typescript" | "ts" => &self.ts_builtins,
            "javascript" | "js" => &self.js_builtins,
            "python" | "py" => &self.python_builtins,
            "go" => &self.go_builtins,
            _ => &self.rust_builtins, // fallback
        }
    }
}

/// Load builtins from a TOML configuration file.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_builtins_from_config(path: &Path) -> anyhow::Result<BuiltinConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: BuiltinConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Convenience function to check if a symbol is a builtin.
/// Uses the default detector.
pub fn is_builtin(symbol: &str, language: &str) -> bool {
    // Use a thread-local cached detector for efficiency
    thread_local! {
        static DETECTOR: BuiltinDetector = BuiltinDetector::with_defaults();
    }
    DETECTOR.with(|d| d.is_builtin(symbol, language))
}

/// Returns the default embedded builtin configuration.
pub fn default_builtin_config() -> BuiltinConfig {
    BuiltinConfig {
        rust: LanguageBuiltins {
            primitives: vec![
                "()", "unit", "bool", "true", "false",
                "i8", "i16", "i32", "i64", "i128", "isize",
                "u8", "u16", "u32", "u64", "u128", "usize",
                "f32", "f64", "str", "char",
            ].into_iter().map(String::from).collect(),
            std_types: vec![
                "String", "Vec", "Box", "Rc", "Arc", "RefCell", "Cell",
                "Mutex", "RwLock", "HashMap", "HashSet", "BTreeMap", "BTreeSet",
                "VecDeque", "LinkedList", "BinaryHeap", "Option", "Result",
                "Ok", "Err", "Some", "None", "Path", "PathBuf", "File",
            ].into_iter().map(String::from).collect(),
            macros: vec![
                "println", "print", "eprintln", "eprint", "format", "panic",
                "assert", "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq",
                "unreachable", "todo", "unimplemented", "dbg", "vec", "write", "writeln",
                "include_str", "include_bytes", "env", "concat", "stringify", "cfg", "matches",
                "log", "trace", "info", "warn", "error",
            ].into_iter().map(String::from).collect(),
            traits: vec![
                "Clone", "Copy", "Default", "Debug", "Display",
                "PartialEq", "Eq", "PartialOrd", "Ord", "Hash",
                "Send", "Sync", "Sized", "Drop", "From", "Into",
                "TryFrom", "TryInto", "AsRef", "AsMut", "Deref", "DerefMut",
                "Iterator", "IntoIterator", "Extend", "FromIterator",
                "Read", "Write", "Seek", "BufRead", "Error",
                "Serialize", "Deserialize", "Fn", "FnMut", "FnOnce",
            ].into_iter().map(String::from).collect(),
            methods: vec![
                "clone", "into", "from", "as_ref", "as_mut",
                "unwrap", "unwrap_or", "unwrap_or_else", "unwrap_or_default", "expect",
                "ok", "err", "is_ok", "is_err", "is_some", "is_none",
                "map", "map_err", "and_then", "or_else", "take", "replace",
                "get", "get_mut", "insert", "remove", "contains",
                "push", "pop", "len", "is_empty",
                "iter", "iter_mut", "into_iter", "collect", "filter", "find", "any", "all",
                "to_string", "to_owned", "as_str", "as_bytes", "parse", "try_into", "try_from",
            ].into_iter().map(String::from).collect(),
            keywords: vec![
                "Self", "self", "static", "mut", "ref", "pub", "move", "drop", "forget",
                "new", "default", "anyhow", "bail", "ensure", "context", "Context", "with_context",
                "fs", "io", "std", "async", "await", "spawn", "block_on", "tokio", "async_std",
            ].into_iter().map(String::from).collect(),
            ..Default::default()
        },
        javascript: LanguageBuiltins {
            globals: vec![
                "console", "log", "error", "warn", "info", "debug",
                "window", "document", "navigator", "location", "history",
                "localStorage", "sessionStorage", "fetch", "require", "module", "exports", "process",
            ].into_iter().map(String::from).collect(),
            types: vec![
                "Array", "Object", "String", "Number", "Boolean", "Symbol",
                "Map", "Set", "WeakMap", "WeakSet", "Promise",
                "Date", "RegExp", "JSON", "Math",
                "Error", "TypeError", "RangeError", "SyntaxError",
            ].into_iter().map(String::from).collect(),
            functions: vec![
                "setTimeout", "setInterval", "clearTimeout", "clearInterval",
                "parseInt", "parseFloat", "isNaN", "isFinite",
                "encodeURI", "decodeURI", "encodeURIComponent", "decodeURIComponent",
            ].into_iter().map(String::from).collect(),
            keywords: vec![
                "async", "await", "undefined", "null", "NaN", "Infinity",
            ].into_iter().map(String::from).collect(),
            ..Default::default()
        },
        typescript: LanguageBuiltins::default(), // Will merge with JS
        python: LanguageBuiltins {
            builtins: vec![
                "print", "len", "range", "str", "int", "float", "bool",
                "list", "dict", "set", "tuple", "open", "input", "type",
                "isinstance", "hasattr", "getattr", "setattr", "delattr",
                "sum", "min", "max", "abs", "round", "sorted", "reversed",
                "enumerate", "zip", "map", "filter", "any", "all", "next", "iter",
                "super", "property", "classmethod", "staticmethod",
            ].into_iter().map(String::from).collect(),
            exceptions: vec![
                "Exception", "ValueError", "TypeError", "KeyError",
                "IndexError", "AttributeError", "RuntimeError", "StopIteration",
            ].into_iter().map(String::from).collect(),
            ..Default::default()
        },
        go: LanguageBuiltins {
            builtins: vec![
                "fmt", "Println", "Printf", "Sprintf", "Errorf", "Print",
                "make", "new", "len", "cap", "append", "copy", "delete", "close",
                "panic", "recover", "error", "nil",
            ].into_iter().map(String::from).collect(),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_detector() {
        let detector = BuiltinDetector::with_defaults();

        // Rust builtins
        assert!(detector.is_builtin("println", "rust"));
        assert!(detector.is_builtin("Vec", "rust"));
        assert!(detector.is_builtin("Option", "rust"));
        assert!(!detector.is_builtin("MyCustomType", "rust"));

        // JS builtins
        assert!(detector.is_builtin("console", "javascript"));
        assert!(detector.is_builtin("Promise", "js"));
        assert!(!detector.is_builtin("myFunction", "javascript"));

        // Python builtins
        assert!(detector.is_builtin("print", "python"));
        assert!(detector.is_builtin("len", "py"));

        // Go builtins
        assert!(detector.is_builtin("fmt", "go"));
        assert!(detector.is_builtin("make", "go"));
    }

    #[test]
    fn test_pattern_builtins() {
        let detector = BuiltinDetector::with_defaults();

        // Single uppercase letters (generic params)
        assert!(detector.is_builtin("T", "rust"));
        assert!(detector.is_builtin("E", "rust"));

        // Compound expressions
        assert!(detector.is_builtin("Ok(())", "rust"));
        assert!(detector.is_builtin("std::vec", "rust"));

        // Numeric literals
        assert!(detector.is_builtin("42", "rust"));
        assert!(detector.is_builtin("3.14", "rust"));
    }

    #[test]
    fn test_is_builtin_function() {
        assert!(is_builtin("println", "rust"));
        assert!(is_builtin("console", "javascript"));
        assert!(!is_builtin("myCustomFunction", "rust"));
    }
}

