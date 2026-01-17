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

// =============================================================================
// RLM (Recursive Language Model) Voting Patterns
// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
// =============================================================================

/// RLM threshold for switching to RLM mode (50K characters)
const RLM_THRESHOLD = 50000;

/// RLM-aware vote function
/// Automatically switches to RLM mode when context exceeds threshold
///
/// # Arguments
/// * `atom_type` - The AtomType enum value
/// * `task` - The task description/prompt
/// * `context` - The context content (may be large)
/// * `k_threshold` - Number of votes winner must be ahead by (default: 3)
///
/// # Returns
/// Object with 'value' (winning output) or 'error' (failure message)
fn rlm_vote(atom_type, task, context, k_threshold) {
    let context_len = context.len();

    // Check if we should use RLM mode
    if context_len >= RLM_THRESHOLD {
        log("Context size " + context_len + " exceeds RLM threshold, using RLM mode");
        return rlm_vote_internal(atom_type, task, context, k_threshold);
    } else {
        // Standard voting with context included in task
        let full_task = task + "\n\n## Context\n" + context;
        return vote(atom_type, full_task, k_threshold);
    }
}

/// Internal RLM voting implementation
/// Uses RLM context store and RLMProcessor atoms
fn rlm_vote_internal(atom_type, task, context, k_threshold) {
    let k = if k_threshold == () { DEFAULT_K_THRESHOLD } else { k_threshold };
    let max_atoms = DEFAULT_MAX_ATOMS;
    let min_votes = DEFAULT_MIN_VOTES;

    // Store context as environment variable
    let context_var = "rlm_context_" + timestamp();
    load_context_var(context_var, context);

    log("Loaded context as '" + context_var + "' (" + context_length(context_var) + " chars)");

    // Vote storage
    let votes = #{};
    let atoms_spawned = 0;
    let discarded = 0;

    // Build RLM-aware task prompt
    let rlm_task = build_rlm_task(task, context_var, atom_type);

    while atoms_spawned < max_atoms {
        atoms_spawned += 1;

        // Use RLMProcessor for large context handling
        let result = spawn_rlm(AtomType.RLMProcessor, rlm_task, context_var);

        if result.valid == false {
            log("RLM Atom " + atoms_spawned + " returned invalid result");
            continue;
        }

        if check_red_flags(result.output) {
            log("RLM Atom " + atoms_spawned + " red-flagged - discarding");
            discarded += 1;
            continue;
        }

        let output = result.output;
        if votes.contains(output) {
            votes[output] += 1;
        } else {
            votes[output] = 1;
        }

        log("RLM Atom " + atoms_spawned + " voted. Current votes: " + votes[output]);

        let winner = find_winner(votes, k, min_votes);
        if winner != () {
            log("RLM Consensus reached with " + votes[winner] + " votes!");
            // Clean up context variable
            clear_context(context_var);
            return #{
                value: winner,
                votes: votes[winner],
                atoms_spawned: atoms_spawned,
                discarded: discarded,
                rlm_mode: true,
                is_err: false
            };
        }
    }

    // Clean up context variable
    clear_context(context_var);

    log("RLM Consensus failed after " + atoms_spawned + " atoms");
    return #{
        error: "RLM Consensus not reached within " + max_atoms + " atoms",
        votes: votes,
        atoms_spawned: atoms_spawned,
        discarded: discarded,
        rlm_mode: true,
        is_err: true
    };
}

/// Build an RLM-aware task prompt
fn build_rlm_task(task, context_var, target_atom_type) {
    let prompt = "## RLM Task\n\n";
    prompt += "**Target Atom Type**: " + target_atom_type + "\n";
    prompt += "**Context Variable**: `" + context_var + "`\n";
    prompt += "**Context Length**: " + context_length(context_var) + " characters\n\n";
    prompt += "### Task Description\n" + task + "\n\n";
    prompt += "### Instructions\n";
    prompt += "1. Use `peek_context(\"" + context_var + "\", start, end)` to examine portions\n";
    prompt += "2. Use `chunk_context(\"" + context_var + "\", size)` to split into chunks\n";
    prompt += "3. Use `regex_filter(\"" + context_var + "\", pattern)` for code filtering\n";
    prompt += "4. Use `llm_query(prompt)` for focused sub-reasoning\n";
    prompt += "5. Return structured JSON with your answer\n";
    return prompt;
}

/// Chunked processing for very large contexts
/// Processes context in chunks and aggregates results
fn rlm_chunked_process(atom_type, task, context, chunk_size) {
    let size = if chunk_size == () { RLM_THRESHOLD } else { chunk_size };

    // Store full context
    let context_var = "rlm_full_" + timestamp();
    load_context_var(context_var, context);

    // Get chunks
    let chunks = chunk_context(context_var, size);
    let num_chunks = chunks.len();

    log("Processing " + num_chunks + " chunks of size " + size);

    let partial_results = [];

    // Process each chunk
    for i in 0..num_chunks {
        let chunk_var = "chunk_" + i;
        load_context_var(chunk_var, chunks[i]);

        let chunk_task = task + "\n\n[Processing chunk " + (i + 1) + " of " + num_chunks + "]";
        let result = spawn_atom(atom_type, chunk_task + "\n\n" + chunks[i]);

        if result.valid && !check_red_flags(result.output) {
            partial_results.push(result.output);
        }

        clear_context(chunk_var);
    }

    // Aggregate results
    if partial_results.len() > 0 {
        let aggregation_task = "Synthesize these partial results into a final answer:\n\n";
        for i in 0..partial_results.len() {
            aggregation_task += "### Result " + (i + 1) + "\n" + partial_results[i] + "\n\n";
        }

        let final_result = spawn_atom(AtomType.Planner, aggregation_task);

        clear_context(context_var);

        return #{
            value: final_result.output,
            chunks_processed: num_chunks,
            partial_results: partial_results.len(),
            is_err: false
        };
    }

    clear_context(context_var);

    return #{
        error: "No valid results from chunk processing",
        chunks_processed: num_chunks,
        is_err: true
    };
}

/// RLM-aware code search with regex filtering
fn rlm_search(context, pattern, task) {
    let context_var = "search_context_" + timestamp();
    load_context_var(context_var, context);

    // Filter using regex
    let matches = regex_filter(context_var, pattern);

    if matches.len() == 0 {
        clear_context(context_var);
        return #{
            error: "No matches found for pattern: " + pattern,
            is_err: true
        };
    }

    log("Found " + matches.len() + " matches for pattern");

    // Process matches with Search atom
    let matches_context = matches.join("\n---\n");
    let search_task = task + "\n\n## Matching Code\n```\n" + matches_context + "\n```";

    let result = spawn_atom(AtomType.Search, search_task);

    clear_context(context_var);

    return #{
        value: result.output,
        matches_found: matches.len(),
        is_err: !result.valid
    };
}

/// Helper to get current timestamp for unique variable names
fn timestamp() {
    // Simple counter-based approach since we don't have real timestamps in Rhai
    // In practice, this would use a proper timestamp
    return "t" + rand(1000000);
}

