# RLM (Recursive Language Model) API Documentation

## Overview

The RLM module enables Cerebras-MAKER to handle arbitrarily long contexts by treating prompts as external environment variables that can be programmatically accessed through a REPL-style interface. This is based on the research paper "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition".

## Key Concepts

### Context Variables
Large content (code, documents, analysis results) is stored as named "context variables" in an RLM Context Store. Instead of including the full content in prompts, agents can:
- **Peek** at slices of content
- **Chunk** content into processable pieces
- **Filter** content using regex patterns
- **Query** sub-LMs for specific analysis tasks

### RLM Threshold
When context exceeds 50K characters (configurable), the system automatically switches to RLM mode, enabling efficient processing of large codebases.

## Rhai API Functions

### `load_context_var(name: string, content: string) -> bool`
Stores content as a named context variable.

```rhai
load_context_var("codebase", file_content);
load_context_var("analysis", json_data);
```

### `peek_context(var_name: string, start: int, end: int) -> string`
Returns a slice of the context without tokenizing the full content.

```rhai
let preview = peek_context("codebase", 0, 1000);  // First 1000 chars
let middle = peek_context("codebase", 5000, 6000);  // Chars 5000-6000
```

### `context_length(var_name: string) -> int`
Returns the length of a context variable (-1 if not found).

```rhai
let len = context_length("codebase");
if len > 50000 {
    // Use chunked processing
}
```

### `chunk_context(var_name: string, chunk_size: int) -> array`
Splits context into chunks of specified size.

```rhai
let chunks = chunk_context("codebase", 10000);
for chunk in chunks {
    let result = llm_query("Analyze this code section: " + chunk);
    // Process result...
}
```

### `regex_filter(var_name: string, pattern: string) -> array`
Filters context lines matching a regex pattern.

```rhai
let fn_defs = regex_filter("codebase", "^fn ");
let imports = regex_filter("codebase", "^use |^import ");
let todos = regex_filter("codebase", "TODO|FIXME");
```

### `llm_query(prompt: string) -> dynamic`
Makes a recursive sub-LM call for specific analysis.

```rhai
let summary = llm_query("Summarize the key functions in this code: " + chunk);
let issues = llm_query("Find potential bugs in: " + code_section);
```

### `spawn_rlm(atom_type: AtomType, task: string, context_var: string) -> dynamic`
Spawns an RLM-aware atom with access to a context variable.

```rhai
let result = spawn_rlm(AtomType::Coder, "Implement the missing function", "codebase");
```

### `has_context(var_name: string) -> bool`
Checks if a context variable exists.

```rhai
if has_context("analysis_results") {
    let data = peek_context("analysis_results", 0, -1);
}
```

### `clear_context(var_name: string) -> bool`
Removes a context variable from the store.

```rhai
clear_context("temp_data");
```

### `list_contexts() -> array`
Lists all context variable names.

```rhai
let vars = list_contexts();
for var_name in vars {
    print("Context: " + var_name + " (" + context_length(var_name) + " chars)");
}
```

## Configuration

RLM behavior can be configured in the Settings UI under "RLM Settings":

| Setting | Default | Description |
|---------|---------|-------------|
| Context Threshold | 50,000 | Characters before switching to RLM mode |
| Max Depth | 1 | Maximum recursion depth for llm_query |
| Max Iterations | 20 | Maximum iterations per RLM execution |
| Sub-model Provider | (same) | LLM provider for sub-calls |
| Sub-model Name | (same) | Model name for sub-calls |
| Sub-model Temperature | 0.1 | Temperature for sub-calls |

## Trajectory Visualization

The Cockpit dashboard includes an "RLM Trace" tab showing:
- Step-by-step execution trace
- Context variable operations (load, peek, chunk, filter)
- Sub-LM calls and results
- Timing information

## Example: Processing Large Codebase

```rhai
// Load the full codebase
let codebase = read_file("src/**/*.rs");
load_context_var("code", codebase);

// Check if RLM mode is needed
let len = context_length("code");
if len > 50000 {
    // Use chunked analysis
    let chunks = chunk_context("code", 20000);
    let summaries = [];
    
    for chunk in chunks {
        let summary = llm_query("Summarize the key components: " + chunk);
        summaries.push(summary);
    }
    
    // Combine summaries
    load_context_var("summaries", summaries.join("\n"));
    let final_analysis = spawn_rlm(AtomType::Architect, "Create architecture overview", "summaries");
} else {
    // Direct analysis for smaller codebases
    let result = spawn_atom(AtomType::Architect, "Analyze: " + codebase);
}
```

## Best Practices

1. **Use peek before chunk**: Preview content structure before deciding chunk size
2. **Filter first**: Use regex_filter to reduce content before analysis
3. **Combine results**: Aggregate sub-query results for final synthesis
4. **Clear unused contexts**: Free memory by clearing temporary context variables
5. **Monitor trajectory**: Use the RLM Trace tab to debug complex workflows

