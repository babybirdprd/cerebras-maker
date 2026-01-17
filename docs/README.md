# Cerebras-MAKER Documentation

Welcome to the Cerebras-MAKER documentation. This documentation is organized into two main sections for different audiences.

---

## 📚 Documentation Index

### For End Users (`docs/user/`)

Getting started and using the application.

| Document | Description |
|----------|-------------|
| [Getting Started](user/getting-started.md) | Installation, system requirements, first launch |
| [Tutorial](user/tutorial.md) | Step-by-step first project walkthrough |
| [UI Guide](user/ui-guide.md) | Complete walkthrough of all UI sections |
| [Features Guide](user/features.md) | Detailed explanation of all features |
| [Troubleshooting](user/troubleshooting.md) | Common issues and solutions |
| [FAQ](user/faq.md) | Frequently asked questions |

### For Developers (`docs/dev/`)

Architecture, API reference, and contributing.

| Document | Description |
|----------|-------------|
| [Architecture](dev/architecture.md) | 4-layer system, Dual-Graph, Shadow Git, RLM |
| [API Reference](dev/api-reference.md) | Complete Tauri command reference |
| [Components](dev/components.md) | Core component documentation |
| [Setup](dev/setup.md) | Development environment setup |
| [Testing](dev/testing.md) | Testing guidelines and patterns |
| [Contributing](dev/contributing.md) | Code style and contribution process |

---

## 🏗️ System Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│                           CEREBRAS-MAKER                                   │
│                    Autonomous AI-Assisted Coding                           │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                │
│   │   Frontend   │◄──►│  Tauri IPC   │◄──►│   Backend    │                │
│   │  React + TS  │    │   Bridge     │    │    Rust      │                │
│   └──────────────┘    └──────────────┘    └──────────────┘                │
│          │                                       │                         │
│          ▼                                       ▼                         │
│   ┌──────────────┐                       ┌──────────────┐                 │
│   │   Zustand    │                       │  grits-core  │                 │
│   │    Store     │                       │  Topology    │                 │
│   └──────────────┘                       └──────────────┘                 │
│                                                 │                          │
│                                                 ▼                          │
│                                          ┌──────────────┐                 │
│                                          │  LLM APIs    │                 │
│                                          │  (Cerebras,  │                 │
│                                          │   OpenAI)    │                 │
│                                          └──────────────┘                 │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## 🔄 The MAKER Framework

```
User Request
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│  L1: Product Orchestrator                                    │
│  • Analyzes PRD documents                                    │
│  • Conducts interrogation to clarify requirements            │
│  • Generates structured PLAN.md                              │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│  L2: Technical Orchestrator                                  │
│  • Rhai scripting runtime                                    │
│  • Task dependency resolution                                │
│  • Execution flow control                                    │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│  L3: Context Engineer                                        │
│  • grits-core topology analysis                              │
│  • MiniCodebase extraction (~50 lines)                       │
│  • Forbidden dependency identification                       │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│  L4: Atoms                                                   │
│  • Individual LLM-powered operations                         │
│  • First-to-ahead-by-k consensus voting                      │
│  • Red-flag checking before commit                           │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
Generated Code
```

---

## 🚀 Quick Links

- **New Users**: Start with [Getting Started](user/getting-started.md)
- **First Project**: Follow the [Tutorial](user/tutorial.md)
- **Developers**: Read the [Architecture](dev/architecture.md) overview
- **API Integration**: See [API Reference](dev/api-reference.md)
- **Contributing**: Check [Contributing Guidelines](dev/contributing.md)

---

## 📖 Version

This documentation is for **Cerebras-MAKER v0.1.0** (PRD v7.0 / PRD-ADD v7.1).

