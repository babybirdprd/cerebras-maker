// Cerebras-MAKER: Voting Preamble
// PRD Section 4.3: The "MAKER" Standard Library
// This script is loaded before any task script to provide reliability patterns

// Default configuration
const DEFAULT_K_THRESHOLD = 3;
const DEFAULT_MAX_ATOMS = 10;
const DEFAULT_MIN_VOTES = 2;

/// Run consensus voting on an atom task
/// Spawns multiple atoms and uses "First-to-ahead-by-k" logic
/// 
/// # Arguments
/// * `atom_type` - The AtomType enum value
/// * `task` - The task description/prompt
/// * `k_threshold` - Number of votes winner must be ahead by (default: 3)
///
/// # Returns
/// Object with 'value' (winning output) or 'error' (failure message)
fn vote(atom_type, task, k_threshold) {
    let k = if k_threshold == () { DEFAULT_K_THRESHOLD } else { k_threshold };
    let max_atoms = DEFAULT_MAX_ATOMS;
    let min_votes = DEFAULT_MIN_VOTES;
    
    // Vote storage: output -> count
    let votes = #{};
    let atoms_spawned = 0;
    let discarded = 0;
    
    log("Starting consensus voting for: " + task);
    
    // Keep spawning atoms until consensus or max reached
    while atoms_spawned < max_atoms {
        atoms_spawned += 1;
        
        // Spawn an atom
        let result = spawn_atom(atom_type, task);
        
        // Check if result is valid
        if result.valid == false {
            log("Atom " + atoms_spawned + " returned invalid result");
            continue;
        }
        
        // Check for red flags
        if check_red_flags(result.output) {
            log("Atom " + atoms_spawned + " red-flagged - discarding");
            discarded += 1;
            continue;
        }
        
        // Normalize and count vote
        let output = result.output;
        if votes.contains(output) {
            votes[output] += 1;
        } else {
            votes[output] = 1;
        }
        
        log("Atom " + atoms_spawned + " voted. Current votes: " + votes[output]);
        
        // Check for consensus
        let winner = find_winner(votes, k, min_votes);
        if winner != () {
            log("Consensus reached with " + votes[winner] + " votes!");
            return #{
                value: winner,
                votes: votes[winner],
                atoms_spawned: atoms_spawned,
                discarded: discarded,
                is_err: false
            };
        }
    }
    
    // No consensus reached
    log("Consensus failed after " + atoms_spawned + " atoms");
    return #{
        error: "Consensus not reached within " + max_atoms + " atoms",
        votes: votes,
        atoms_spawned: atoms_spawned,
        discarded: discarded,
        is_err: true
    };
}

/// Find the winner using First-to-ahead-by-k logic
fn find_winner(votes, k, min_votes) {
    let candidates = [];
    
    // Collect candidates
    for output in votes.keys() {
        candidates.push(#{ output: output, count: votes[output] });
    }
    
    // Sort by count descending
    candidates.sort(|a, b| b.count - a.count);
    
    if candidates.len() == 0 {
        return ();
    }
    
    let leader = candidates[0];
    
    // Single candidate check
    if candidates.len() == 1 {
        if leader.count >= min_votes {
            return leader.output;
        }
        return ();
    }
    
    let second = candidates[1];
    
    // First-to-ahead-by-k check
    if leader.count >= min_votes && (leader.count - second.count) >= k {
        return leader.output;
    }
    
    return ();
}

/// Simple majority vote (no k-threshold)
fn simple_vote(atom_type, task, required_votes) {
    let votes = #{};
    let atoms_spawned = 0;
    
    while atoms_spawned < required_votes * 2 {
        atoms_spawned += 1;
        let result = spawn_atom(atom_type, task);
        
        if result.valid && !check_red_flags(result.output) {
            let output = result.output;
            if votes.contains(output) {
                votes[output] += 1;
            } else {
                votes[output] = 1;
            }
            
            if votes[output] >= required_votes {
                return #{ value: output, is_err: false };
            }
        }
    }
    
    return #{ error: "Required votes not reached", is_err: true };
}

/// Validate code using Validator atoms
fn validate_code(code) {
    let result = spawn_atom(AtomType.Validator, code);
    return result.valid && !check_red_flags(code);
}

/// Review code using Reviewer atoms with consensus
fn review_code(code, approvals_needed) {
    let n = if approvals_needed == () { 2 } else { approvals_needed };
    return simple_vote(AtomType.Reviewer, code, n);
}

