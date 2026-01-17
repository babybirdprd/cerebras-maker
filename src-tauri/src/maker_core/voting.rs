// Cerebras-MAKER: Voting and Consensus Logic
// PRD Section 4.3: The "MAKER" Standard Library - First-to-ahead-by-k voting
// P1-3: Now with parallel atom execution for improved throughput

use super::atom::{AtomType, SpawnFlags};
use crate::agents::{AtomExecutor, AtomInput};
use crate::llm::LlmConfig;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
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
    /// Initial batch size for parallel execution (default: 3)
    /// Spawns this many atoms in parallel at the start
    pub initial_batch_size: usize,
    /// Whether to use parallel execution
    pub parallel_enabled: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            k_threshold: 3,
            max_atoms: 10,
            timeout: Duration::from_secs(60),
            discard_red_flags: true,
            min_votes: 2,
            initial_batch_size: 3,
            parallel_enabled: true,
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
    /// Whether consensus was reached (alias for consensus_reached)
    pub reached: bool,
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
            reached: true,
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
            reached: false,
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
/// Spawns multiple atoms in parallel and uses first-to-ahead-by-k voting
/// P1-3: Now with parallel execution for improved throughput
pub async fn run_consensus(
    atom_type: AtomType,
    task: &str,
    config: ConsensusConfig,
    llm_config: &LlmConfig,
    _workspace_path: &str,
) -> ConsensusResult {
    let start = Instant::now();
    let mut candidates: HashMap<String, usize> = HashMap::new();
    let mut atoms_spawned = 0;
    let mut discarded_count = 0;

    // Create atom executor (wrapped in Arc for sharing across tasks)
    let executor = Arc::new(AtomExecutor::new(llm_config.clone()));

    // Build base input
    let base_input = AtomInput::new(atom_type.clone(), task)
        .with_flags(SpawnFlags {
            red_flag_check: config.discard_red_flags,
            ..Default::default()
        });

    // Use parallel execution if enabled
    if config.parallel_enabled && config.initial_batch_size > 1 {
        // Phase 1: Parallel initial batch
        let batch_size = config.initial_batch_size.min(config.max_atoms);
        let (batch_candidates, batch_spawned, batch_discarded) =
            execute_parallel_batch(&executor, &base_input, batch_size, &config).await;

        atoms_spawned += batch_spawned;
        discarded_count += batch_discarded;

        // Merge batch results
        for (output, count) in batch_candidates {
            *candidates.entry(output).or_insert(0) += count;
        }

        // Check if consensus reached from initial batch
        if let Some(winner) = check_consensus(&candidates, config.k_threshold, config.min_votes) {
            let winning_votes = *candidates.get(&winner).unwrap_or(&0);
            return ConsensusResult::success(
                winner,
                winning_votes,
                candidates,
                atoms_spawned,
                discarded_count,
                start.elapsed().as_millis() as u64,
            );
        }
    }

    // Phase 2: Sequential additional atoms until consensus or max reached
    while atoms_spawned < config.max_atoms {
        // Check timeout
        if start.elapsed() > config.timeout {
            return ConsensusResult::failure(
                "Timeout reached before consensus".to_string(),
                candidates,
                atoms_spawned,
                discarded_count,
                start.elapsed().as_millis() as u64,
            );
        }

        // Execute atom
        let input = base_input.clone();
        atoms_spawned += 1;

        match executor.execute(input).await {
            Ok(result) => {
                // Check for red flags
                if result.is_red_flagged() && config.discard_red_flags {
                    discarded_count += 1;
                    continue;
                }

                // Normalize and count the output
                let normalized = normalize_output(&result.output);
                *candidates.entry(normalized.clone()).or_insert(0) += 1;

                // Check if consensus reached
                if let Some(winner) = check_consensus(&candidates, config.k_threshold, config.min_votes) {
                    // HIGH-5: Cleaner pattern - get vote count directly since we just found the winner
                    let winning_votes = candidates.get(&winner).copied().unwrap_or(0);
                    return ConsensusResult::success(
                        winner,
                        winning_votes,
                        candidates,
                        atoms_spawned,
                        discarded_count,
                        start.elapsed().as_millis() as u64,
                    );
                }
            }
            Err(e) => {
                // Log error but continue trying
                eprintln!("Atom execution failed: {}", e);
                discarded_count += 1;
            }
        }
    }

    // Max atoms reached without consensus
    ConsensusResult::failure(
        format!("Max atoms ({}) reached without consensus", config.max_atoms),
        candidates,
        atoms_spawned,
        discarded_count,
        start.elapsed().as_millis() as u64,
    )
}

/// Execute a batch of atoms in parallel using tokio::spawn
/// Returns (candidates map, atoms spawned, discarded count)
async fn execute_parallel_batch(
    executor: &Arc<AtomExecutor>,
    base_input: &AtomInput,
    batch_size: usize,
    config: &ConsensusConfig,
) -> (HashMap<String, usize>, usize, usize) {
    let mut handles = Vec::with_capacity(batch_size);

    // Spawn all atoms in parallel
    for _ in 0..batch_size {
        let exec = Arc::clone(executor);
        let input = base_input.clone();
        let discard_red_flags = config.discard_red_flags;

        let handle = tokio::spawn(async move {
            match exec.execute(input).await {
                Ok(result) => {
                    if result.is_red_flagged() && discard_red_flags {
                        (None, true) // Discarded
                    } else {
                        (Some(normalize_output(&result.output)), false)
                    }
                }
                Err(_) => (None, true) // Error = discarded
            }
        });
        handles.push(handle);
    }

    // Wait for all to complete and aggregate results
    let results = join_all(handles).await;

    let mut candidates: HashMap<String, usize> = HashMap::new();
    let mut discarded = 0;

    for result in results {
        match result {
            Ok((Some(output), _)) => {
                *candidates.entry(output).or_insert(0) += 1;
            }
            Ok((None, true)) => {
                discarded += 1;
            }
            Err(_) => {
                discarded += 1; // Task panicked
            }
            _ => {}
        }
    }

    (candidates, batch_size, discarded)
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

