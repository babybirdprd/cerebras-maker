# Frequently Asked Questions

## General Questions

### What is Cerebras-MAKER?

Cerebras-MAKER is a desktop application that uses AI to help you write code autonomously. You describe what you want to build, and MAKER's team of AI agents works together to create it. It's like having a team of expert programmers that can understand your requirements and generate working code.

### Who is MAKER for?

MAKER is designed for:
- **Developers** who want to speed up their workflow
- **Teams** looking to automate repetitive coding tasks
- **Learners** who want to understand how code is structured
- **Product managers** who want to prototype ideas quickly

### Is MAKER free to use?

MAKER itself is open source and free to download. However, you'll need an API key from an AI provider (like Cerebras, OpenAI, or Anthropic), which may have associated costs based on usage.

## AI Providers

### What LLM providers are supported?

MAKER supports multiple AI providers:

| Provider | Description |
|----------|-------------|
| **Cerebras** | Fastest inference, recommended for best performance |
| **OpenAI** | GPT-4 and other OpenAI models |
| **Anthropic** | Claude models, excellent for complex reasoning |
| **OpenRouter** | Access to many models with a single API key |
| **OpenAI-Compatible** | Any API that follows OpenAI's format |

### Can I use my own models?

Yes! Using the **OpenAI-Compatible** provider option, you can connect to:
- Self-hosted models (like Ollama, LM Studio)
- Custom API endpoints
- Any service that implements the OpenAI API format

Just enter your custom Base URL in the Settings.

### Which provider should I choose?

- **For speed**: Cerebras offers the fastest inference
- **For quality**: OpenAI GPT-4 or Anthropic Claude
- **For flexibility**: OpenRouter gives access to many models
- **For privacy**: Self-hosted via OpenAI-Compatible

### How much does it cost to use?

Costs depend on your chosen provider and usage:
- **Cerebras**: Check [cloud.cerebras.ai](https://cloud.cerebras.ai) for pricing
- **OpenAI**: Based on tokens used, see [openai.com/pricing](https://openai.com/pricing)
- **Anthropic**: Based on tokens, see [anthropic.com](https://anthropic.com)
- **Self-hosted**: Only your hardware/electricity costs

Typical project generation might use thousands to millions of tokens depending on complexity.

## Privacy & Security

### Is my code sent to the cloud?

Yes, when MAKER generates or analyzes code, it sends relevant context to your chosen AI provider. This is necessary for the AI to understand and work with your code.

**What gets sent:**
- Code snippets relevant to the current task
- File structure information
- Your requirements and descriptions

**What stays local:**
- Your complete codebase (only relevant parts are sent)
- Shadow Git snapshots
- Application settings and API keys

### How can I keep my code private?

For maximum privacy:
1. Use a **self-hosted model** via OpenAI-Compatible
2. Run models locally with Ollama or LM Studio
3. Review what context is being sent in the Cockpit

### Are my API keys stored securely?

API keys are stored locally on your machine in the application's data folder. They are not sent anywhere except to the respective AI provider when making requests.

## Data Storage

### Where is my data stored?

MAKER stores data in several locations:

| Data Type | Location |
|-----------|----------|
| **Application settings** | OS-specific app data folder |
| **API keys** | Encrypted in app data folder |
| **Shadow Git snapshots** | `.shadow-git` folder in your project |
| **Generated code** | Your project workspace folder |

**App data locations by OS:**
- **Windows**: `%APPDATA%\cerebras-maker`
- **macOS**: `~/Library/Application Support/cerebras-maker`
- **Linux**: `~/.config/cerebras-maker`

### Can I backup my Time Machine history?

Yes! The Shadow Git data is stored in a `.shadow-git` folder within your project. You can:
- Copy this folder to backup your history
- Include it in your regular backups
- Move it with your project to another machine

### How much disk space does MAKER use?

- **Application**: ~100-200 MB
- **Shadow Git**: Varies based on project size and number of snapshots
- **Temporary files**: Minimal, cleaned up automatically

## Features

### What is the Interrogator?

The Interrogator is MAKER's clarification system. Before generating code, it analyzes your request and asks questions if anything is unclear. This ensures MAKER understands exactly what you want before starting work.

### What is Shadow Git / Time Machine?

Shadow Git is MAKER's automatic version control system. It creates snapshots of your code automatically, allowing you to:
- Undo any changes
- Roll back to any previous state
- Experiment without fear of losing work

It works independently of regular Git, so you don't need Git knowledge to use it.

### What are "Red Flags"?

Red flags are warnings about potential problems in your code, such as:
- Circular dependencies
- Layer violations (breaking architectural rules)
- Overly complex code structures

MAKER detects these automatically using the Grits analysis engine.

### What is RLM?

RLM (Recursive Language Model) is MAKER's system for handling large codebases that exceed typical AI context limits. It automatically breaks down large contexts into manageable chunks, processes them, and synthesizes the results.

## Troubleshooting

### How do I report bugs?

To report a bug:
1. Go to the project's GitHub repository
2. Click "Issues" → "New Issue"
3. Include:
   - Steps to reproduce the problem
   - What you expected vs what happened
   - Your system information (OS, version)
   - Any error messages

### Where can I get help?

- **This documentation**: Start with the guides in this folder
- **GitHub Issues**: Search for similar problems or ask questions
- **Community**: Check for community forums or Discord

### MAKER generated wrong code. What do I do?

1. **Use Time Machine**: Roll back to before the bad code
2. **Be more specific**: Provide clearer requirements
3. **Answer questions**: Respond thoroughly to the Interrogator
4. **Iterate**: Ask MAKER to fix specific issues
5. **Try different models**: Some models work better for certain tasks

## Technical Questions

### What technologies does MAKER use?

MAKER is built with:
- **Frontend**: React with TypeScript
- **Backend**: Rust with Tauri framework
- **Visualization**: Three.js (3D) and D3.js (2D graphs)
- **Analysis**: Grits-core for topological analysis

### Can I use MAKER from the command line?

Yes! MAKER supports headless mode for automation:

```bash
# Run on a PRD file
cerebras-maker run --file my-prd.md

# Execute a script directly
cerebras-maker exec --script my-script.rhai

# Specify workspace
cerebras-maker --workspace ./my-project --prd requirements.md
```

### What file types can I upload as a PRD?

MAKER accepts:
- `.md` (Markdown) - Recommended
- `.txt` (Plain text)
- `.pdf` (PDF documents)

### Does MAKER work offline?

MAKER requires an internet connection to communicate with AI providers. However, if you use a self-hosted model (via OpenAI-Compatible), you can work offline as long as your local model server is running.

---

**Still have questions?** Check the [Troubleshooting Guide](troubleshooting.md) or open an issue on GitHub.

