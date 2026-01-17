# Contributing Guidelines

## Getting Started

1. Fork the repository
2. Clone your fork
3. Set up the development environment (see [setup.md](./setup.md))
4. Create a feature branch

## Code Style

### Rust

Follow the official Rust style guide with these project conventions:

```rust
// Use descriptive names
pub fn extract_context_for_task(task: &MicroTask) -> Result<ContextPackage, String>

// Document public APIs
/// Extract minimal context for an L4 atom.
///
/// # Arguments
/// * `task` - The micro-task requiring context
///
/// # Returns
/// A `ContextPackage` with the assembled MiniCodebase
pub fn extract_context(&self, task: &MicroTask) -> Result<ContextPackage, String>

// Use Result for fallible operations
pub fn load_graph(path: &str) -> Result<SymbolGraph, String>

// Prefer explicit error messages
Err(format!("Failed to load graph from {}: {}", path, e))
```

**Formatting:**
```bash
cd src-tauri
cargo fmt
cargo clippy
```

### TypeScript

Follow the project's TypeScript conventions:

```typescript
// Use explicit types
interface AtomResult {
  output: string;
  changes: CodeChange[];
}

// Use async/await over promises
async function executeAtom(input: AtomInput): Promise<AtomResult> {
  const result = await invoke('execute_atom', input);
  return result as AtomResult;
}

// Use functional components with hooks
function Dashboard(): JSX.Element {
  const [state, setState] = useState<DashboardState>(initialState);
  // ...
}
```

**Formatting:**
```bash
npm run lint
npm run format
```

## Pull Request Process

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

### 2. Make Changes

- Write clean, documented code
- Add tests for new functionality
- Update documentation if needed

### 3. Test Your Changes

```bash
# Backend
cd src-tauri
cargo test
cargo clippy

# Frontend
npm run build
npm test
```

### 4. Commit Guidelines

Use conventional commits:

```
feat: add RLM trajectory visualization
fix: correct snapshot rollback order
docs: update API reference for new commands
refactor: simplify context extraction logic
test: add tests for voting consensus
```

### 5. Submit PR

- Fill out the PR template
- Link related issues
- Request review from maintainers

### 6. Address Feedback

- Respond to review comments
- Push additional commits as needed
- Squash commits before merge if requested

## Issue Reporting

### Bug Reports

Include:
- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, Rust version, Node version)
- Relevant logs or screenshots

### Feature Requests

Include:
- Use case description
- Proposed solution
- Alternatives considered
- Impact on existing functionality

## Architecture Decisions

Major changes should be discussed before implementation:

1. **Open an issue** describing the proposed change
2. **Discuss** with maintainers
3. **Document** the decision in the PR

### Key Architectural Principles

1. **4-Layer Hierarchy**: Respect L1→L2→L3→L4 flow
2. **Atomic Operations**: Keep atoms stateless and focused
3. **Semantic Tree-Shaking**: Minimize context for LLM calls
4. **Transactional Safety**: Use Shadow Git for all file changes
5. **Red-Flag Checking**: Validate architectural constraints

## Development Tips

### Adding a New Tauri Command

1. Add the function in `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
async fn my_new_command(param: String) -> Result<String, String> {
    // Implementation
}
```

2. Register in the handler:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_new_command,
])
```

3. Call from frontend:
```typescript
const result = await invoke('my_new_command', { param: 'value' });
```

### Adding a New Atom Type

1. Add variant to `AtomType` enum in `atom_executor.rs`
2. Add system prompt in `src-tauri/prompts/`
3. Handle in `AtomExecutor::execute()`
4. Update `get_atom_types()` command

### Adding a New Rhai Function

1. Register in `CodeModeRuntime::new()`:
```rust
engine.register_fn("my_function", |arg: String| -> String {
    // Implementation
});
```

2. Document in `docs/dev/components.md`

## Questions?

- Check existing documentation
- Search closed issues
- Open a discussion for general questions

