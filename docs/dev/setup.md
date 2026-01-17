# Setup and Build Instructions

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| Node.js | 18+ | Frontend build |
| Rust | 1.70+ | Backend compilation |
| pnpm/npm/yarn | Latest | Package management |

### Platform-Specific Requirements

**Windows:**
- Visual Studio Build Tools 2019+ with C++ workload
- WebView2 (usually pre-installed on Windows 10/11)

**macOS:**
- Xcode Command Line Tools: `xcode-select --install`

**Linux:**
- Build essentials: `sudo apt install build-essential`
- WebKit: `sudo apt install libwebkit2gtk-4.1-dev`
- Additional libs: `sudo apt install libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/cerebras-maker.git
cd cerebras-maker
```

### 2. Install Frontend Dependencies

```bash
npm install
# or
pnpm install
# or
yarn install
```

### 3. Build grits-core (if needed)

The grits-core library is included as a local dependency. It should build automatically with Cargo, but you can verify:

```bash
cd src-tauri/grits-core
cargo build
cd ../..
```

## Development Workflow

### Start Development Server

```bash
npm run tauri dev
```

This command:
1. Starts the Vite dev server for the frontend
2. Compiles the Rust backend
3. Opens the application window with hot-reload

### Frontend Only (for UI development)

```bash
npm run dev
```

Opens at `http://localhost:5173` - useful for rapid UI iteration without Rust compilation.

### Backend Only (for Rust development)

```bash
cd src-tauri
cargo build
cargo test
```

## Building for Production

### Build Application

```bash
npm run tauri build
```

**Output locations:**
- Windows: `src-tauri/target/release/cerebras-maker.exe`
- macOS: `src-tauri/target/release/bundle/macos/Cerebras-MAKER.app`
- Linux: `src-tauri/target/release/bundle/deb/` or `appimage/`

### Build with Debug Symbols

```bash
npm run tauri build -- --debug
```

### Cross-Platform Builds

Tauri supports cross-compilation, but native builds are recommended:

```bash
# On Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# On macOS (Intel)
npm run tauri build -- --target x86_64-apple-darwin

# On macOS (Apple Silicon)
npm run tauri build -- --target aarch64-apple-darwin

# On Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## Environment Variables

### LLM Provider API Keys

Set at least one of these for LLM functionality:

| Variable | Provider | Required |
|----------|----------|----------|
| `OPENAI_API_KEY` | OpenAI | One of these |
| `ANTHROPIC_API_KEY` | Anthropic | One of these |
| `CEREBRAS_API_KEY` | Cerebras | One of these |
| `OPENROUTER_API_KEY` | OpenRouter | Optional |

**Setting environment variables:**

```bash
# Windows (PowerShell)
$env:OPENAI_API_KEY = "sk-..."

# macOS/Linux
export OPENAI_API_KEY="sk-..."
```

Alternatively, configure API keys in the application Settings panel.

### Development Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `RUST_LOG` | Rust logging level | `info` |
| `TAURI_DEBUG` | Enable Tauri debug mode | `false` |

## Project Structure

```
cerebras-maker/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # React components
│   ├── hooks/              # Custom hooks
│   ├── store/              # Zustand state
│   └── types.ts            # TypeScript types
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── agents/         # L1-L4 agents
│   │   ├── generators/     # Script generators
│   │   ├── llm/            # LLM providers
│   │   ├── maker_core/     # Core runtime
│   │   └── lib.rs          # Tauri commands
│   ├── grits-core/         # Topology library
│   ├── crawl4ai-rs/        # Web crawling
│   └── Cargo.toml          # Rust dependencies
├── docs/                   # Documentation
├── package.json            # Node dependencies
└── tauri.conf.json         # Tauri configuration
```

## Troubleshooting

### Common Issues

**"WebView2 not found" (Windows)**
Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

**Rust compilation errors**
```bash
rustup update
cargo clean
cargo build
```

**Node module issues**
```bash
rm -rf node_modules
npm install
```

**Permission errors (Linux)**
```bash
sudo chown -R $USER:$USER ~/.cargo
```

