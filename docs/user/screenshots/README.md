# Screenshot Placeholders

This directory should contain screenshots for the user documentation. The following screenshots are needed:

## Required Screenshots

| Filename | Description | Documentation Reference |
|----------|-------------|------------------------|
| `welcome-screen.png` | The initial welcome screen showing the three main options (Upload PRD, Open Existing, Start from Template) with the MAKER logo at top | getting-started.md |
| `settings-button.png` | The sidebar with an arrow pointing to the gear/settings icon in the bottom-left corner | getting-started.md |
| `api-key-setup.png` | The Settings panel showing the provider dropdown, API key input field, and Save button | getting-started.md |
| `main-window.png` | The complete application window showing all sections - sidebar with icons, main content area with Blueprint view, header bar with workspace name, and status bar at bottom | ui-guide.md |
| `blueprint-view.png` | The Blueprint 3D view showing colored nodes representing code symbols, with the rotation/zoom controls visible. Show a medium-sized codebase with multiple node types visible | ui-guide.md |
| `topology-view.png` | The Topology 2D graph view showing interconnected nodes with dependency edges, colored by component type (indigo for logic, green for data, amber for UI) | ui-guide.md |
| `cockpit-view.png` | The Cockpit/Swarm view showing a list of execution events with icons, timestamps, and messages. Show multiple event types (Script Start, Atom Spawned, Consensus Vote) to demonstrate variety | ui-guide.md |
| `time-machine-view.png` | The Shadow Git/Time Machine view showing the timeline slider, list of snapshots with timestamps and messages, and the rollback/squash buttons | ui-guide.md |
| `prd-upload.png` | The PRD upload area with a file being dragged over it, showing the dropzone highlight and file preview | tutorial.md |
| `cockpit-execution.png` | The Cockpit view during active code generation showing multiple events scrolling by with different icons and a green "Running" indicator | tutorial.md |
| `time-machine-rollback.png` | The Time Machine view with the timeline slider positioned partway through history, showing a list of snapshots with one selected and the Rollback button highlighted | tutorial.md |

## Screenshot Guidelines

1. **Resolution**: Capture at 1920x1080 or higher
2. **Theme**: Use the default application theme
3. **Content**: Use realistic but non-sensitive sample data
4. **Annotations**: Add arrows/highlights using image editing software if needed
5. **Format**: PNG format with reasonable compression

## How to Capture

1. Launch Cerebras-MAKER
2. Load a sample project (or create one)
3. Navigate to the relevant view
4. Use your OS screenshot tool or a dedicated tool like:
   - Windows: `Win + Shift + S` (Snipping Tool)
   - macOS: `Cmd + Shift + 4`
   - Linux: `gnome-screenshot` or similar
5. Save with the exact filename listed above

