// Cerebras-MAKER: RLM Demo Script
// Demonstrates Recursive Language Model patterns with a large codebase
// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
//
// This script showcases the full RLM workflow:
// 1. Loading a large codebase into the context store
// 2. Inspecting context size and structure
// 3. Chunking for parallel processing
// 4. Regex filtering for targeted code extraction
// 5. Peek operations for random access
// 6. Simulated sub-LM queries for analysis

// =============================================================================
// Configuration
// =============================================================================

const RLM_THRESHOLD = 50000;  // 50K character threshold for RLM mode
const CHUNK_SIZE = 50000;     // Chunk size for processing
const CONTEXT_VAR = "sidecar_codebase";

// =============================================================================
// Step 1: Load the codebase
// =============================================================================

fn demo_load_codebase(content) {
    log("=== Step 1: Loading Codebase ===");

    let start = timestamp();
    load_context_var(CONTEXT_VAR, content);
    let load_time = timestamp() - start;

    let size = context_length(CONTEXT_VAR);
    log("Loaded " + size + " characters in " + load_time + "ms");
    log("RLM Mode: " + (size >= RLM_THRESHOLD ? "ENABLED" : "disabled"));

    return #{
        size: size,
        load_time: load_time,
        rlm_mode: size >= RLM_THRESHOLD
    };
}

// =============================================================================
// Step 2: Analyze codebase structure
// =============================================================================

fn demo_analyze_structure() {
    log("=== Step 2: Analyzing Structure ===");

    let size = context_length(CONTEXT_VAR);

    // Peek at the beginning to understand format
    let header = peek_context(CONTEXT_VAR, 0, 2000);
    log("Header preview (first 2000 chars):");
    log(header.sub_string(0, 500) + "...");

    // Count key patterns using regex
    let structs = regex_filter(CONTEXT_VAR, "^pub struct ");
    let functions = regex_filter(CONTEXT_VAR, "^pub fn ");
    let impls = regex_filter(CONTEXT_VAR, "^impl ");
    let traits = regex_filter(CONTEXT_VAR, "^pub trait ");
    let mods = regex_filter(CONTEXT_VAR, "^pub mod ");

    log("Found:");
    log("  - " + structs.len() + " public structs");
    log("  - " + functions.len() + " public functions");
    log("  - " + impls.len() + " impl blocks");
    log("  - " + traits.len() + " public traits");
    log("  - " + mods.len() + " public modules");

    return #{
        total_size: size,
        structs: structs.len(),
        functions: functions.len(),
        impls: impls.len(),
        traits: traits.len(),
        modules: mods.len()
    };
}

// =============================================================================
// Step 3: Chunk the codebase
// =============================================================================

fn demo_chunking() {
    log("=== Step 3: Chunking Codebase ===");

    let start = timestamp();
    let chunks = chunk_context(CONTEXT_VAR, CHUNK_SIZE);
    let chunk_time = timestamp() - start;

    let num_chunks = chunks.len();
    log("Created " + num_chunks + " chunks of ~" + CHUNK_SIZE + " chars each");
    log("Chunking time: " + chunk_time + "ms");

    // Verify chunk sizes
    let total_size = 0;
    for chunk in chunks {
        total_size += chunk.len();
    }

    log("Total reconstructed size: " + total_size + " chars");
    log("Original size: " + context_length(CONTEXT_VAR) + " chars");

    return #{
        num_chunks: num_chunks,
        chunk_time: chunk_time,
        total_size: total_size,
        avg_chunk_size: total_size / num_chunks
    };
}

// =============================================================================
// Step 4: Targeted code extraction
// =============================================================================

fn demo_code_extraction() {
    log("=== Step 4: Targeted Code Extraction ===");

    // Find error handling patterns
    let errors = regex_filter(CONTEXT_VAR, "Result<.*Error>");
    log("Found " + errors.len() + " Result<..., Error> patterns");

    // Find async functions
    let async_fns = regex_filter(CONTEXT_VAR, "^pub async fn ");
    log("Found " + async_fns.len() + " async functions");

    // Find test functions
    let tests = regex_filter(CONTEXT_VAR, "#\\[test\\]");
    log("Found " + tests.len() + " test functions");

    // Find derive macros
    let derives = regex_filter(CONTEXT_VAR, "#\\[derive\\(");
    log("Found " + derives.len() + " derive macros");

    return #{
        error_patterns: errors.len(),
        async_functions: async_fns.len(),
        tests: tests.len(),
        derives: derives.len()
    };
}

// =============================================================================
// Step 5: Peek operations for random access
// =============================================================================

fn demo_peek_operations() {
    log("=== Step 5: Peek Operations ===");

    let size = context_length(CONTEXT_VAR);

    // Peek at different positions
    let positions = [0, size / 4, size / 2, size * 3 / 4, size - 1000];

    for pos in positions {
        let start = timestamp();
        let snippet = peek_context(CONTEXT_VAR, pos, pos + 500);
        let peek_time = timestamp() - start;
        log("Peek at " + pos + ": " + peek_time + "μs - " + snippet.len() + " chars");
    }

    return #{ positions_tested: positions.len() };
}

// =============================================================================
// Step 6: Simulated sub-LM queries
// =============================================================================

fn demo_sub_lm_queries() {
    log("=== Step 6: Sub-LM Queries ===");

    // In a real scenario, these would call actual LLM endpoints
    // Here we simulate the pattern

    // Find all struct definitions
    let structs = regex_filter(CONTEXT_VAR, "^pub struct ");

    if structs.len() > 0 {
        log("Analyzing first 5 structs with sub-LM...");

        let analyzed = 0;
        for i in 0..min(5, structs.len()) {
            let struct_def = structs[i];
            // Simulated sub-LM query
            let query = "Analyze this struct definition and describe its purpose:\n" + struct_def;
            log("  Sub-LM query " + (i + 1) + ": " + struct_def.sub_string(0, 50) + "...");
            analyzed += 1;
        }

        return #{ structs_analyzed: analyzed };
    }

    return #{ structs_analyzed: 0 };
}

// =============================================================================
// Step 7: Full RLM workflow
// =============================================================================

fn demo_full_workflow(content) {
    log("========================================");
    log("  RLM Demo: Full Workflow");
    log("  Codebase: codestoryai/sidecar");
    log("========================================");
    log("");

    // Step 1: Load
    let load_result = demo_load_codebase(content);
    log("");

    // Step 2: Analyze
    let structure = demo_analyze_structure();
    log("");

    // Step 3: Chunk
    let chunks = demo_chunking();
    log("");

    // Step 4: Extract
    let extraction = demo_code_extraction();
    log("");

    // Step 5: Peek
    let peeks = demo_peek_operations();
    log("");

    // Step 6: Sub-LM
    let sub_lm = demo_sub_lm_queries();
    log("");

    // Summary
    log("========================================");
    log("  RLM Demo Complete!");
    log("========================================");
    log("");
    log("Summary:");
    log("  - Codebase size: " + load_result.size + " chars");
    log("  - RLM mode: " + (load_result.rlm_mode ? "ENABLED" : "disabled"));
    log("  - Structs found: " + structure.structs);
    log("  - Functions found: " + structure.functions);
    log("  - Chunks created: " + chunks.num_chunks);
    log("  - Avg chunk size: " + chunks.avg_chunk_size + " chars");
    log("");

    // Cleanup
    clear_context(CONTEXT_VAR);
    log("Context cleaned up.");

    return #{
        load: load_result,
        structure: structure,
        chunks: chunks,
        extraction: extraction,
        peeks: peeks,
        sub_lm: sub_lm
    };
}

// =============================================================================
// Helper functions
// =============================================================================

fn min(a, b) {
    if a < b { a } else { b }
}

// Entry point - call demo_full_workflow(content) with your codebase content
