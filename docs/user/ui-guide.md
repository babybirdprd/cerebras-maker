# Cerebras-MAKER UI Guide

This guide explains every part of the Cerebras-MAKER interface to help you navigate and use the application effectively.

## Application Layout

The MAKER interface is divided into several key areas:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            HEADER BAR                                   │
│  [Logo]              Workspace Name              [Settings] [Maximize]  │
├─────────┬───────────────────────────────────────────────────────────────┤
│         │                                                               │
│  📊     │                                                               │
│ ─────── │                                                               │
│  🔗     │                    MAIN CONTENT AREA                          │
│ ─────── │                                                               │
│  💻     │            (Changes based on selected view)                   │
│ ─────── │                                                               │
│  🕐     │                                                               │
│         │                                                               │
├─────────┴───────────────────────────────────────────────────────────────┤
│ SIDEBAR │              STATUS BAR: Reliability Indicator                │
│ [⚙️]    │              "100% Solid" or issue count                      │
└─────────┴───────────────────────────────────────────────────────────────┘
```

![Screenshot: Main Application Window](screenshots/main-window.png)
<!-- SCREENSHOT: The complete application window showing all sections - sidebar with icons, main content area with Blueprint view, header bar with workspace name, and status bar at bottom -->

## Sidebar

The sidebar on the left provides navigation between main views.

### Navigation Items

| Icon | Name | Description |
|------|------|-------------|
| 📊 | **Blueprint** | 3D visualization of your code structure |
| 🔗 | **Topology** | Dependency graph and architecture analysis |
| 💻 | **Swarm** | Real-time execution log (Cockpit) |
| 🕐 | **Shadow Git** | Time Machine for undo/rollback |

### Reliability Indicator
At the bottom of the sidebar, you'll see a reliability meter showing the health of your codebase:
- **100% Solid**: No issues detected
- **Lower percentages**: Potential problems found by Grits analysis

### Settings Button
Click the gear icon to open the Settings panel.

## Main Views

### Blueprint View

The Blueprint provides a 3D interactive visualization of your code.

![Screenshot: Blueprint View](screenshots/blueprint-view.png)
<!-- SCREENSHOT: The Blueprint 3D view showing colored nodes representing code symbols, with the rotation/zoom controls visible. Show a medium-sized codebase with multiple node types visible -->

**Features:**
- **Colored Nodes**: Different colors represent different code types
  - 🟢 Green: Functions
  - 🔵 Blue: Structs
  - 🟣 Purple: Classes
  - 🩷 Pink: Interfaces
  - 🟡 Yellow: Modules
  - 🩵 Cyan: Constants
  - 🟠 Orange: Types

- **Navigation Controls:**
  - Click and drag to rotate the view
  - Scroll to zoom in/out
  - Click a node to select it and see details

- **Status Indicators:**
  - ✅ Clean: No architectural issues
  - 🚩 Cycle Detected: Circular dependency found

### Topology View

The Topology view shows a 2D force-directed graph of code dependencies.

![Screenshot: Topology View](screenshots/topology-view.png)
<!-- SCREENSHOT: The Topology 2D graph view showing interconnected nodes with dependency edges, colored by component type (indigo for logic, green for data, amber for UI) -->

**Legend:**
- 🟣 Indigo: Logic components
- 🟢 Green: Data components
- 🟡 Amber: UI components
- 🩷 Pink: External dependencies

**Interactions:**
- Drag nodes to rearrange the layout
- Hover over nodes to see labels
- The graph automatically organizes related code together

### Swarm View (Cockpit)

The Cockpit shows real-time execution events as MAKER works.

![Screenshot: Cockpit View](screenshots/cockpit-view.png)
<!-- SCREENSHOT: The Cockpit/Swarm view showing a list of execution events with icons, timestamps, and messages. Show multiple event types (Script Start, Atom Spawned, Consensus Vote) to demonstrate variety -->

**Event Types:**

| Icon | Event | Description |
|------|-------|-------------|
| 🚀 | Script Start | A new execution script has begun |
| ⚛️ | Atom Spawned | A specialized agent started working |
| 🗳️ | Consensus Vote | Agents voting on the best solution |
| 🚩 | Red Flag | Potential issue detected |
| 📦 | RLM Start | Large context processing began |
| 🔄 | RLM Chunk | Processing a chunk of large context |
| ✅ | RLM Complete | Large context processing finished |
| ⚠️ | Error | Something went wrong |

**Status Indicator:**
- 🟢 Running: MAKER is actively working
- ⚪ Idle: Waiting for input

### Shadow Git View (Time Machine)

Time Machine provides unlimited undo capability through automatic snapshots.

![Screenshot: Time Machine View](screenshots/time-machine-view.png)
<!-- SCREENSHOT: The Shadow Git/Time Machine view showing the timeline slider, list of snapshots with timestamps and messages, and the rollback/squash buttons -->

**Controls:**
- **Timeline Slider**: Drag to browse through history
- **Refresh Button**: Update the snapshot list
- **Rollback Button**: Restore to the selected point
- **Squash Button**: Combine multiple snapshots into one

**Snapshot Information:**
- Timestamp of each snapshot
- Number of files changed
- Commit message (if available)

## Plan View

When MAKER is executing a task, the Plan View shows progress.

**Task Status Icons:**
- ✅ Completed: Task finished successfully
- 🔄 Active: Currently being worked on
- ⚪ Pending: Waiting to start
- ⚠️ Failed: Task encountered an error

**Task Details** (click to expand):
- Notes about the task
- Related issues or tickets
- Code snippets for context

## Settings Panel

Access settings by clicking the gear icon in the sidebar.

### Provider Tab
Configure your AI provider:
- **Provider Selection**: Choose from Cerebras, OpenAI, Anthropic, OpenRouter, or OpenAI-Compatible
- **API Key**: Enter your provider's API key
- **Base URL**: For custom endpoints (OpenAI-Compatible only)

### Models Tab
Customize which model each agent uses:
- **L1 Orchestrator Model**: For high-level planning
- **L2 Technical Model**: For architecture decisions
- **L3 Context Model**: For code analysis
- **L4 Atom Model**: For code generation

### RLM Tab
Configure Recursive Language Model settings:
- **Context Threshold**: When to activate RLM (default: 50K characters)
- **Chunk Size**: Size of context chunks
- **Max Depth**: Maximum recursion depth
- **Max Iterations**: Limit on processing iterations

## Welcome Screen

The welcome screen appears when no project is loaded.

**Options:**
1. **Upload PRD**: Drag and drop a requirements document
2. **Open Existing Project**: Browse to an existing codebase
3. **Start from Template**: Choose a pre-built project template

**Supported File Types for PRD:**
- `.md` (Markdown)
- `.txt` (Plain text)
- `.pdf` (PDF documents)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + S` | Save current state |
| `Ctrl/Cmd + Z` | Quick undo (via Time Machine) |
| `Ctrl/Cmd + ,` | Open Settings |
| `Escape` | Close dialogs |

## Mobile/Responsive Layout

On smaller screens, the sidebar collapses to a bottom navigation bar with icons for quick access to all views.

---

**Next**: Learn about [MAKER's features](features.md) or check the [FAQ](faq.md).

