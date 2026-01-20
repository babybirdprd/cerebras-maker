// use grits_core::topology::analysis::TopologicalAnalysis;
use serde::{Deserialize, Serialize};

// Re-export grits-core types needed for the API
pub use grits_core::topology::analysis::LayerViolation;
pub use grits_core::topology::virtual_apply::ProposedChange;

use crate::grits;
// use crate::grits::load_workspace_graph;

/// Result of Red Flag checks
/// Combined from MAKER Paper (Section 3.3) and Local PRD (Architectural)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedFlagResult {
    // Architectural Flags (Local PRD)
    pub introduced_cycle: bool,
    pub has_layer_violations: bool,
    pub cycles_detected: Vec<Vec<String>>,
    pub layer_violations: Vec<LayerViolation>,

    // Unreliability Flags (MAKER Paper)
    pub is_verbose: bool,   // Response > threshold
    pub is_malformed: bool, // JSON/Schematic failure

    // Aggregated Status
    pub approved: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub max_tokens: usize,
    pub previous_betti_1: usize,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            max_tokens: 1000,
            previous_betti_1: 0,
        }
    }
}

/// Check output for Red Flags
/// This aggregates both topological checks (if graph provided) and unreliability checks
#[tauri::command]
pub fn check_governance(
    response_text: String,
    changes: Option<Vec<ProposedChange>>,
    config: Option<GovernanceConfig>,
) -> Result<RedFlagResult, String> {
    let config = config.unwrap_or_default();
    let mut flags = RedFlagResult {
        introduced_cycle: false,
        has_layer_violations: false,
        cycles_detected: Vec::new(),
        layer_violations: Vec::new(),
        is_verbose: false,
        is_malformed: false,
        approved: true,
        rejection_reason: None,
    };

    // 1. Check Verbosity (Paper: "Overly long responses")
    let token_estimate = response_text.split_whitespace().count(); // Rough heuristic
    if token_estimate > config.max_tokens {
        flags.is_verbose = true;
        flags.approved = false;
        flags.rejection_reason = Some(format!(
            "Response too verbose ({} tokens > {})",
            token_estimate, config.max_tokens
        ));
        return Ok(flags);
    }

    // 2. Check Formatting (Paper: "Incorrectly formatted")
    // If response is meant to be JSON, try to parse it.
    // This is a naive check; real validation usually happens in the Atom itself,
    // but we can expose a check here if the response text IS the JSON payload.
    if response_text.trim().starts_with('{') || response_text.trim().starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(&response_text).is_err() {
            flags.is_malformed = true;
            flags.approved = false;
            flags.rejection_reason = Some("Malformed JSON".to_string());
            return Ok(flags);
        }
    }

    // 3. Check Architecture (Local PRD: Cycles & Layers)
    if let Some(changes) = changes {
        // We need the graph for this.
        let graph =
            grits::get_cached_graph().ok_or("No cached graph. Call load_symbol_graph first.")?;
        let workspace_path = grits::get_cached_workspace_path();

        let result = grits::virtual_red_flag_check(&graph, &changes, workspace_path.as_deref());

        flags.introduced_cycle = result.introduced_cycle;
        flags.has_layer_violations = result.has_layer_violations;
        flags.cycles_detected = result.cycles_detected;
        flags.layer_violations = result.layer_violations;

        if flags.introduced_cycle {
            flags.approved = false;
            flags.rejection_reason = Some("Introduced circular dependency".to_string());
        } else if flags.has_layer_violations {
            flags.approved = false;
            flags.rejection_reason = Some("Architectural layer violation".to_string());
        }
    }

    Ok(flags)
}

/// Legacy wrapper for just checking topological red flags
#[tauri::command]
pub fn check_architectural_flags(changes: Vec<ProposedChange>) -> Result<RedFlagResult, String> {
    check_governance("".to_string(), Some(changes), None)
}
