# Getting Started with Cerebras-MAKER

Welcome to Cerebras-MAKER, your autonomous AI-assisted coding companion! This guide will help you get up and running quickly.

## What is Cerebras-MAKER?

Cerebras-MAKER is a desktop application that uses AI to help you write code. You describe what you want to build, and MAKER's team of AI agents works together to create it for you. Think of it as having a team of expert programmers at your fingertips.

## System Requirements

### Minimum Requirements
- **Operating System**: Windows 10/11, macOS 10.15+, or Linux (Ubuntu 20.04+)
- **RAM**: 8 GB
- **Storage**: 500 MB free space
- **Internet**: Required for AI features

### Recommended
- **RAM**: 16 GB or more
- **Display**: 1920x1080 or higher resolution

## Installation

### Windows
1. Download the `.msi` installer from the releases page
2. Double-click the installer file
3. Follow the installation wizard
4. Launch Cerebras-MAKER from the Start Menu

### macOS
1. Download the `.dmg` file from the releases page
2. Open the downloaded file
3. Drag Cerebras-MAKER to your Applications folder
4. Launch from Applications (you may need to right-click and select "Open" the first time)

### Linux
1. Download the `.AppImage` or `.deb` file from the releases page
2. For AppImage: Make it executable (`chmod +x`) and run it
3. For .deb: Install with `sudo dpkg -i cerebras-maker.deb`

## First Launch

When you first open Cerebras-MAKER, you'll see the welcome screen with three options:

1. **Upload a PRD** - Start a new project by uploading a Product Requirements Document
2. **Open Existing Project** - Work on an existing codebase (brownfield development)
3. **Start from Template** - Choose from pre-built project templates

![Screenshot: Welcome Screen](screenshots/welcome-screen.png)
<!-- SCREENSHOT: The initial welcome screen showing the three main options (Upload PRD, Open Existing, Start from Template) with the MAKER logo at top -->

## Setting Up Your AI Provider

Before MAKER can generate code, you need to connect it to an AI provider. Here's how:

### Step 1: Open Settings
Click the **Settings** button (gear icon) in the bottom-left corner of the sidebar.

![Screenshot: Settings Button Location](screenshots/settings-button.png)
<!-- SCREENSHOT: The sidebar with an arrow pointing to the gear/settings icon in the bottom-left corner -->

### Step 2: Choose Your Provider
MAKER supports multiple AI providers:

| Provider | Best For | Notes |
|----------|----------|-------|
| **Cerebras** | Fastest inference | Recommended for best performance |
| **OpenAI** | GPT-4 models | Most widely used |
| **Anthropic** | Claude models | Excellent for complex reasoning |
| **OpenRouter** | Multiple models | Access many models with one key |
| **OpenAI-Compatible** | Self-hosted | Use local or custom endpoints |

### Step 3: Enter Your API Key
1. Select your preferred provider from the dropdown
2. Paste your API key in the text field
3. Click **Save**

![Screenshot: API Key Configuration](screenshots/api-key-setup.png)
<!-- SCREENSHOT: The Settings panel showing the provider dropdown, API key input field, and Save button -->

> **Where to get API keys:**
> - Cerebras: [cloud.cerebras.ai](https://cloud.cerebras.ai)
> - OpenAI: [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
> - Anthropic: [console.anthropic.com](https://console.anthropic.com)
> - OpenRouter: [openrouter.ai/keys](https://openrouter.ai/keys)

### Step 4: Configure Models (Optional)
In the Settings panel, you can customize which AI model each agent uses:
- **L1 Orchestrator**: High-level planning (use a powerful model)
- **L2 Technical**: Architecture decisions
- **L3 Context**: Code analysis
- **L4 Atoms**: Code generation (can use faster/cheaper models)

## Verifying Your Setup

To confirm everything is working:

1. Click **Blueprint** in the sidebar
2. You should see the main workspace without any error messages
3. The status indicator in the bottom-left should show "100% Solid"

## Next Steps

Now that you're set up, you're ready to:
- [Follow the First Project Tutorial](tutorial.md) - Build your first project with MAKER
- [Explore the UI Guide](ui-guide.md) - Learn about all the interface elements
- [Read the Features Guide](features.md) - Understand MAKER's powerful capabilities

---

**Need help?** Check the [Troubleshooting Guide](troubleshooting.md) or [FAQ](faq.md).

