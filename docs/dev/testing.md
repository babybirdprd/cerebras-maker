# Testing Guidelines

## Overview

Cerebras-MAKER uses different testing approaches for the Rust backend and TypeScript frontend.

## Running Tests

### Rust Backend Tests

```bash
# Run all tests
cd src-tauri
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_runtime_creation

# Run tests in a specific module
cargo test maker_core::runtime::tests

# Run ignored (integration) tests
cargo test -- --ignored
```

### Frontend Tests

```bash
# Run frontend tests (if configured)
npm test
# or
bun test
```

## Test File Locations

### Rust Tests

| Location | Type | Description |
|----------|------|-------------|
| `src-tauri/src/**/*.rs` | Unit | Inline `#[cfg(test)]` modules |
| `src-tauri/grits-core/src/**/*.rs` | Unit | grits-core library tests |
| `src-tauri/crawl4ai-rs/tests/*.rs` | Integration | crawl4ai integration tests |

### Frontend Tests

| Location | Type | Description |
|----------|------|-------------|
| `src/**/*.test.ts` | Unit | Component/function tests |
| `src/**/*.test.tsx` | Unit | React component tests |

## Writing Tests

### Rust Unit Tests

Place tests in a `#[cfg(test)]` module at the bottom of the source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_success() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_function_error() {
        let result = function_under_test(invalid_input);
        assert!(result.is_err());
    }
}
```

### Async Tests

Use `#[tokio::test]` for async functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

For longer-running tests, use `#[ignore]`:

```rust
#[test]
#[ignore]
fn integration_test_full_workflow() {
    // This test is skipped by default
    // Run with: cargo test -- --ignored
}
```

### TypeScript Tests

Using Vitest (recommended):

```typescript
import { describe, it, expect, vi } from 'vitest';
import { myFunction } from './myModule';

describe('myFunction', () => {
  it('should return expected result', () => {
    expect(myFunction(input)).toBe(expected);
  });

  it('should throw for invalid input', () => {
    expect(() => myFunction(invalid)).toThrow();
  });
});
```

## Test Patterns

### Testing Tauri Commands

Mock the Tauri invoke function:

```typescript
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(mockResult)
}));
```

### Testing with Temporary Directories

```rust
use std::env;

fn create_test_runtime() -> CodeModeRuntime {
    let temp_dir = env::temp_dir().join("cerebras_maker_test");
    std::fs::create_dir_all(&temp_dir).ok();
    CodeModeRuntime::new(temp_dir.to_str().unwrap()).unwrap()
}
```

### Testing RLM Operations

```rust
#[test]
fn test_rlm_store_operations() {
    let mut store = RLMContextStore::new();
    store.load_variable("test", "content".to_string(), ContextType::String);
    
    assert!(store.contains("test"));
    assert_eq!(store.length("test"), Some(7));
    
    let peek = store.peek("test", 0, 4);
    assert_eq!(peek, Some("cont".to_string()));
}
```

## Existing Test Suites

### maker_core/runtime.rs
- `test_runtime_creation` - Runtime initialization
- `test_rlm_store_access` - RLM store operations

### maker_core/rlm.rs
- `integration_test_codebase_regex_filtering` - Regex filtering (ignored)
- RLM context store operations

### grits-core/topology/refactor.rs
- `test_comment_out_rust` - Rust refactoring
- `test_comment_out_python` - Python refactoring

### crawl4ai-rs/tests/
- `test_crawler_initialization` - Crawler setup
- `test_crawl_html` - HTML crawling
- `test_bm25_content_filter_basic` - Content filtering
- `test_wait_strategy_configuration` - Wait strategies

## Best Practices

1. **Test naming**: `test_<function>_<scenario>_<expected>`
2. **Arrange-Act-Assert**: Structure tests clearly
3. **One assertion per test**: When practical
4. **Test edge cases**: Empty inputs, errors, boundaries
5. **Use fixtures**: Create helper functions for test data
6. **Clean up**: Remove temporary files/directories
7. **Mock external services**: Don't make real API calls in unit tests

