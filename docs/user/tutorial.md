# Your First Project with Cerebras-MAKER

This tutorial walks you through creating your first project using MAKER. By the end, you'll understand the complete workflow from idea to working code.

## Overview

In this tutorial, you'll:
1. Create a new project from a description
2. Watch the AI agents work in the Cockpit
3. Review the generated code
4. Use Time Machine if needed
5. Finalize your project

**Time needed**: About 15 minutes

## Workflow Diagram

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  1. DESCRIBE    │────▶│  2. OBSERVE     │────▶│  3. REVIEW      │
│                 │     │                 │     │                 │
│  Upload PRD or  │     │  Watch Cockpit  │     │  Check the      │
│  describe task  │     │  as agents work │     │  generated code │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  6. DONE! 🎉    │◀────│  5. FINALIZE    │◀────│  4. ITERATE     │
│                 │     │                 │     │                 │
│  Your project   │     │  Approve and    │     │  Use Time       │
│  is complete    │     │  commit changes │     │  Machine if     │
└─────────────────┘     └─────────────────┘     │  needed         │
                                                └─────────────────┘
```

## Step 1: Start a New Project

### Option A: Upload a PRD Document
If you have a Product Requirements Document (PRD):

1. From the welcome screen, click the **Upload PRD** area
2. Drag and drop your `.md`, `.txt`, or `.pdf` file
3. Review the preview to confirm it loaded correctly
4. Click **Analyze with L1 Orchestrator**

![Screenshot: PRD Upload](screenshots/prd-upload.png)
<!-- SCREENSHOT: The PRD upload area with a file being dragged over it, showing the dropzone highlight and file preview -->

### Option B: Use a Template
For a quick start:

1. Scroll down to the template section
2. Choose a template that matches your needs:
   - **Tauri + React**: Desktop app with modern UI
   - **Tauri + Vanilla JS**: Lightweight desktop app
   - **Rust CLI**: Command-line tool
3. Click on your chosen template

### Option C: Open Existing Code
To enhance an existing project:

1. Click **Open Existing Project (Brownfield)**
2. Navigate to your project folder
3. Select the folder and click Open

## Step 2: Describe Your Task

Once your project is loaded, the **Interrogator** will help clarify your requirements.

### How the Interrogator Works
The Interrogator analyzes your request and asks clarifying questions if anything is unclear. This ensures MAKER understands exactly what you want.

**Example conversation:**
```
You: "Build a todo app with categories"

Interrogator: "I have a few questions to make sure I understand:
1. Should categories be predefined or user-created?
2. Do you need due dates for tasks?
3. Should data persist between sessions?"
```

### Tips for Good Descriptions
- Be specific about features you want
- Mention any technologies you prefer
- Describe the user experience you envision
- Include any constraints or requirements

## Step 3: Watch the Cockpit

After you confirm your requirements, MAKER begins working. The **Cockpit** shows you everything happening in real-time.

![Screenshot: Cockpit During Execution](screenshots/cockpit-execution.png)
<!-- SCREENSHOT: The Cockpit view during active code generation showing multiple events scrolling by with different icons (Script Start, Atom Spawned, Consensus Vote) and a green "Running" indicator -->

### Understanding Cockpit Events

| Icon | Event Type | What It Means |
|------|------------|---------------|
| 🚀 | Script Start | MAKER is beginning a new task |
| ⚛️ | Atom Spawned | A specialized AI agent is starting work |
| 🗳️ | Consensus Vote | Multiple agents are agreeing on a solution |
| 🚩 | Red Flag | A potential issue was detected |
| 📦 | RLM Chunk | Processing a large piece of code |
| ✅ | Complete | A task finished successfully |

### The Agent Hierarchy
MAKER uses a team of specialized agents:

```
        ┌──────────────────────────┐
        │  L1 Product Orchestrator │  ← Understands your goals
        └────────────┬─────────────┘
                     ▼
        ┌──────────────────────────┐
        │ L2 Technical Orchestrator│  ← Makes architecture decisions
        └────────────┬─────────────┘
                     ▼
        ┌──────────────────────────┐
        │   L3 Context Engineer    │  ← Analyzes existing code
        └────────────┬─────────────┘
                     ▼
    ┌────────┬───────┴───────┬────────┐
    ▼        ▼               ▼        ▼
┌──────┐ ┌──────┐       ┌──────┐ ┌──────┐
│Coder │ │Review│       │Tester│ │Archit│  ← L4 Atoms
└──────┘ └──────┘       └──────┘ └──────┘
```

## Step 4: Review Generated Code

As MAKER works, you can see the results in real-time.

### Using the Plan View
The **Plan View** shows the execution plan as a tree:
- ✅ Green checkmarks = completed tasks
- 🔄 Spinning icon = currently active task
- ⚪ Empty circles = pending tasks
- ⚠️ Warning icon = failed task (will be retried)

Click any task to see details like:
- Which AI agent is handling it
- Estimated complexity
- Related code symbols

### Viewing the Blueprint
Switch to the **Blueprint** view to see a 3D visualization of your code structure:
- Colored spheres represent different code elements (functions, classes, modules)
- Lines show relationships between elements
- Click any node to see details

## Step 5: Use Time Machine (If Needed)

Made a wrong turn? Time Machine lets you go back to any previous state.

![Screenshot: Time Machine Rollback](screenshots/time-machine-rollback.png)
<!-- SCREENSHOT: The Time Machine view with the timeline slider positioned partway through history, showing a list of snapshots with one selected and the Rollback button highlighted -->

### How to Roll Back
1. Click **Shadow Git** in the sidebar
2. Use the slider to browse through snapshots
3. Find the point you want to return to
4. Click **Rollback** to restore that state

### Time Machine Features
- **Unlimited Undo**: Every change is automatically saved
- **Snapshot Preview**: See what changed at each point
- **Squash**: Combine multiple small changes into one
- **Safe Experimentation**: Try things without fear of losing work

## Step 6: Finalize Your Project

When you're happy with the results:

1. Review all generated files in your project folder
2. Test the application to make sure it works
3. Use the **Topology** view to check for any architectural issues
4. Commit your changes to your own Git repository

## Example: Building a Simple App

Let's walk through a concrete example:

**Goal**: Create a note-taking app

1. **Start**: Upload a PRD or describe "Build a note-taking app with markdown support"
2. **Clarify**: Answer the Interrogator's questions about features
3. **Watch**: See MAKER create the file structure, components, and logic
4. **Review**: Check the Blueprint to understand the code structure
5. **Test**: Run the generated app
6. **Iterate**: Ask MAKER to add features or fix issues

## Tips for Success

- **Start Small**: Begin with simple projects to learn the workflow
- **Be Specific**: Clear descriptions lead to better results
- **Use Time Machine**: Don't be afraid to experiment
- **Check the Topology**: Look for red flags before finalizing
- **Iterate**: MAKER works best with feedback and refinement

---

**Next**: Learn about all the [UI elements](ui-guide.md) or explore [advanced features](features.md).

