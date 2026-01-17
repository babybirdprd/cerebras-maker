# Cerebras-MAKER Features Guide

This guide explains the powerful features that make MAKER an effective autonomous coding assistant.

## Autonomous Task Decomposition

MAKER automatically breaks down complex tasks into manageable pieces.

### How It Works
When you describe a project or feature, MAKER's L1 Orchestrator:
1. Analyzes your requirements
2. Identifies all necessary components
3. Creates a hierarchical task tree
4. Assigns specialized agents to each task

### The Four-Layer System

| Layer | Name | Role |
|-------|------|------|
| L1 | Product Orchestrator | Understands goals, creates high-level plan |
| L2 | Technical Orchestrator | Makes architecture and technology decisions |
| L3 | Context Engineer | Gathers code context, analyzes dependencies |
| L4 | Atoms | Specialized workers that write actual code |

### Atom Types
L4 includes specialized agents for different tasks:
- **Coder**: Writes new code
- **Reviewer**: Checks code quality
- **Tester**: Creates and runs tests
- **Architect**: Designs system structure
- **GritsAnalyzer**: Checks for architectural issues
- **RLMProcessor**: Handles large contexts

## Consensus Voting

Multiple AI agents vote on the best solution to ensure quality.

### How Consensus Works

```
                    ┌─────────────────┐
                    │   Task Input    │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
    ┌─────────┐        ┌─────────┐        ┌─────────┐
    │ Agent 1 │        │ Agent 2 │        │ Agent 3 │
    │ (Coder) │        │ (Coder) │        │ (Coder) │
    └────┬────┘        └────┬────┘        └────┬────┘
         │                  │                  │
         ▼                  ▼                  ▼
    ┌─────────┐        ┌─────────┐        ┌─────────┐
    │Solution │        │Solution │        │Solution │
    │    A    │        │    B    │        │    C    │
    └────┬────┘        └────┬────┘        └────┬────┘
         │                  │                  │
         └──────────────────┼──────────────────┘
                            ▼
                    ┌───────────────┐
                    │  🗳️ VOTING    │
                    │               │
                    │  A: ██        │
                    │  B: ████████  │  ← Winner!
                    │  C: ███       │
                    └───────────────┘
```

1. Multiple agents generate solutions independently
2. Each solution is evaluated
3. Agents vote on which solution is best
4. The winner is selected using "first-to-ahead-by-k" logic

### Configuration
- **K Threshold**: How many votes ahead to win (default: 3)
- **Max Atoms**: Maximum agents voting (default: 10)
- **Timeout**: Maximum time for voting
- **Min Votes**: Minimum votes required

### Why It Matters
Consensus voting catches errors that a single agent might miss and produces more reliable code.

## Red-Flag Detection

MAKER automatically detects potential problems in your code.

### What Gets Flagged
- **Circular Dependencies**: Code that depends on itself
- **Layer Violations**: Breaking architectural boundaries
- **Complexity Issues**: Overly complex code structures

### Grits Analysis
The Grits engine performs topological analysis:
- **Betti Numbers**: Mathematical measure of code complexity
- **Cycle Detection**: Finds circular dependencies
- **Triangle Detection**: Identifies tightly coupled code

### Viewing Red Flags
Red flags appear in:
- The Cockpit (real-time alerts)
- The Blueprint view (visual indicators)
- The Topology view (highlighted problem areas)

## Layer Violation Checking

MAKER enforces clean architecture by detecting layer violations.

### What Are Layers?
Code is organized into layers:

```
┌─────────────────────────────────────────────────────────┐
│                    UI LAYER                             │
│           (Components, Views, Handlers)                 │
└─────────────────────────┬───────────────────────────────┘
                          │ ✅ Allowed
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   LOGIC LAYER                           │
│         (Business Rules, Services, Validation)          │
└─────────────────────────┬───────────────────────────────┘
                          │ ✅ Allowed
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   DATA LAYER                            │
│        (Repositories, Database, File Storage)           │
└─────────────────────────────────────────────────────────┘

     🚫 VIOLATIONS: Data → UI, UI → Data (skipping Logic)
```

- **UI Layer**: User interface components
- **Logic Layer**: Business logic
- **Data Layer**: Data storage and retrieval
- **External Layer**: Third-party integrations

### Violation Examples
- UI code directly accessing the database (should go through Logic)
- Data layer calling UI components
- Circular dependencies between layers

### How MAKER Helps
When a violation is detected:
1. A red flag appears in the Cockpit
2. The Blueprint highlights the problematic connection
3. MAKER suggests how to fix the issue

## RLM (Recursive Language Model) Context Handling

MAKER can work with codebases larger than typical AI context limits.

### The Problem
AI models have limited context windows. Large codebases don't fit.

### The Solution: RLM
When context exceeds the threshold (default: 50K characters), RLM:
1. **Peek**: Examines the structure of large content
2. **Chunk**: Breaks content into manageable pieces
3. **Filter**: Uses regex to find relevant sections
4. **Sub-Query**: Processes chunks with focused questions
5. **Synthesize**: Combines results into a coherent answer

### RLM Trajectory
Watch RLM work in the dedicated trajectory panel:
- See each processing step
- Monitor context variables
- Track chunk processing progress

### Configuration
Adjust RLM behavior in Settings:
- **Context Threshold**: When to activate RLM
- **Chunk Size**: How big each piece should be
- **Max Depth**: How deep recursion can go
- **Max Iterations**: Processing limit

## Shadow Git (Time Machine)

Never lose work with automatic version control.

### How It Works
Shadow Git creates snapshots automatically:
- Every significant change is saved
- Snapshots are stored locally
- No manual commits required

### Key Operations

| Operation | Description |
|-----------|-------------|
| **Snapshot** | Save current state |
| **Rollback** | Return to previous state |
| **Rollback To** | Jump to a specific point |
| **Squash** | Combine multiple snapshots |

### Benefits
- **Unlimited Undo**: Go back to any point
- **Safe Experimentation**: Try things without risk
- **Change History**: See what changed and when
- **No Git Knowledge Required**: Works automatically

## The Interrogator

Get clarity before coding begins.

### How It Works
The Interrogator analyzes your request and:
1. Identifies ambiguous requirements
2. Calculates an "ambiguity score"
3. Asks clarifying questions if needed
4. Ensures MAKER understands your intent

### Ambiguity Detection
The Interrogator looks for:
- Vague terms ("make it better")
- Missing details ("add authentication")
- Conflicting requirements
- Undefined scope

### Best Practices
- Answer questions thoroughly
- Provide examples when possible
- Clarify priorities and constraints
- Mention any technical preferences

## Blueprint Visualization

See your code structure in 3D.

### Features
- **Interactive 3D Graph**: Rotate, zoom, and explore
- **Color-Coded Nodes**: Different colors for different code types
- **Relationship Lines**: See how code connects
- **Selection Details**: Click nodes for more information

### Use Cases
- Understanding unfamiliar codebases
- Finding architectural issues
- Planning refactoring
- Explaining code to others

---

**Next**: Check the [Troubleshooting Guide](troubleshooting.md) or [FAQ](faq.md).

