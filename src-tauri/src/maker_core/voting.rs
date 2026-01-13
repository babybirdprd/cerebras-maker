// Cerebras-MAKER: Voting and Consensus Logic
// PRD Section 4.3: The "MAKER" Standard Library - First-to-ahead-by-k voting

use super::atom::AtomType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Configuration for consensus voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// The k threshold - winner must be ahead by this many votes
    pub k_threshold: usize,
    /// Maximum number of atoms to spawn
    pub max_atoms: usize,
    /// Timeout for the entire consensus operation
    pub timeout: Duration,
    /// Whether to discard red-flagged responses
    pub discard_red_flags: bool,
    /// Minimum votes required for any candidate
    pub min_votes: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            k_threshold: 3,
            max_atoms: 10,
            timeout: Duration::from_secs(60),
            discard_red_flags: true,
            min_votes: 2,
        }
    }
}

/// Result of a consensus operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// The winning output
    pub winner: Option<String>,
    /// The vote count for the winner
    pub winning_votes: usize,
    /// All candidates with their vote counts
    pub candidates: HashMap<String, usize>,
    /// Total atoms spawned
    pub atoms_spawned: usize,
    /// Red-flagged responses that were discarded
    pub discarded_count: usize,
    /// Whether consensus was reached
    pub consensus_reached: bool,
    /// Error message if consensus failed
    pub error: Option<String>,
    /// Total time taken
    pub elapsed_ms: u64,
}

impl ConsensusResult {
    /// Create a successful consensus result
    pub fn success(winner: String, winning_votes: usize, candidates: HashMap<String, usize>, 
                   atoms_spawned: usize, discarded_count: usize, elapsed_ms: u64) -> Self {
        Self {
            winner: Some(winner),
            winning_votes,
            candidates,
            atoms_spawned,
            discarded_count,
            consensus_reached: true,
            error: None,
            elapsed_ms,
        }
    }

    /// Create a failed consensus result
    pub fn failure(error: String, candidates: HashMap<String, usize>, 
                   atoms_spawned: usize, discarded_count: usize, elapsed_ms: u64) -> Self {
        Self {
            winner: None,
            winning_votes: 0,
            candidates,
            atoms_spawned,
            discarded_count,
            consensus_reached: false,
            error: Some(error),
            elapsed_ms,
        }
    }
}

/// Normalize an output for comparison (hash-based)
fn normalize_output(output: &str) -> String {
    // Remove whitespace variations and normalize
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Run consensus voting on an atom task
/// This is a synchronous placeholder - actual implementation requires async runtime
pub fn run_consensus(
    _atom_type: AtomType,
    _task: &str,
    config: ConsensusConfig,
) -> ConsensusResult {
    // This is a placeholder - the actual implementation happens in runtime.rs
    // using async execution with rig-core
    let start = Instant::now();
    
    ConsensusResult::failure(
        "Direct call not supported. Use CodeModeRuntime.execute_script()".to_string(),
        HashMap::new(),
        0,
        0,
        start.elapsed().as_millis() as u64,
    )
}

/// Check if consensus has been reached based on vote counts
pub fn check_consensus(votes: &HashMap<String, usize>, k_threshold: usize, min_votes: usize) -> Option<String> {
    if votes.is_empty() {
        return None;
    }

    let mut sorted: Vec<_> = votes.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    if sorted.len() == 1 {
        let (winner, count) = sorted[0];
        if *count >= min_votes {
            return Some(winner.clone());
        }
        return None;
    }

    let (winner, winner_count) = sorted[0];
    let (_, second_count) = sorted[1];

    // First-to-ahead-by-k logic
    if *winner_count >= min_votes && (*winner_count - *second_count) >= k_threshold {
        return Some(winner.clone());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_consensus_basic() {
        let mut votes = HashMap::new();
        votes.insert("result_a".to_string(), 5);
        votes.insert("result_b".to_string(), 2);
        
        // With k_threshold=3, winner_count=5, second_count=2, diff=3 >= k
        assert_eq!(check_consensus(&votes, 3, 2), Some("result_a".to_string()));
    }

    #[test]
    fn test_check_consensus_not_reached() {
        let mut votes = HashMap::new();
        votes.insert("result_a".to_string(), 3);
        votes.insert("result_b".to_string(), 2);
        
        // With k_threshold=3, diff=1 < k
        assert_eq!(check_consensus(&votes, 3, 2), None);
    }
}

