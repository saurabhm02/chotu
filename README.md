# ty — Floating AI Assistant & Knowledge Overlay

A lightweight, spotlight-style floating AI assistant for your desktop. Always one keystroke away, designed to answer questions, run quick commands, and help you take notes without switching away from your current workflow.

Built with **100% Rust** — using **Tauri v2** for the native desktop runtime and **Yew (WebAssembly)** for the frontend interface.

---

## 🎯 Project Goal

Modern work involves constant context switching between browsers, terminals, notes, and chat apps. The goal of `ty` is to build a fast, keyboard-first personal assistant that floats directly over your screen:
- **Instant Access**: Summon with a global shortcut from anywhere, ask a question, and get back to work.
- **Minimal & Distraction-Free**: A compact bar that expands smoothly only when needed.
- **Personal Knowledge Hub**: Ask questions, extract text, and save structured notes into a local knowledge base.

---

## 🛠️ Tech Stack

- **Desktop Framework**: [Tauri v2](https://v2.tauri.app/) (Rust backend)
- **Frontend**: [Yew 0.21](https://yew.rs/) (Rust WebAssembly)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) + [DaisyUI v5](https://daisyui.com/)
- **Build Tools**: [Trunk](https://trunkrs.dev/) (WASM bundler)
- **AI Engine**: OpenAI-compatible LLM integration via `async-openai`
- **Markdown**: `pulldown-cmark` for rendering structured AI answers

---

## 🗺️ Roadmap & Current Status

We are building this project in phases:

### ✅ Phase 1: Core Foundation & Scaffolding
- [x] Initialized Tauri v2 + Yew WebAssembly project.
- [x] Configured Trunk build pipeline and Tailwind CSS v4 / DaisyUI integration.

### ✅ Phase 2: Floating Window & Geometry
- [x] Borderless, transparent, spotlight-style floating overlay window.
- [x] Dynamic height auto-resizing based on content and multi-line inputs.
- [x] Window dragging capability with input/button click-through protection.

### ✅ Phase 3: Global Hotkey Activation
- [x] Registered global shortcut (`Cmd + /` on macOS, `Ctrl + /` on Linux/Windows) to summon/hide the assistant from anywhere.

### ✅ Phase 4: AI Backend Integration
- [x] Tauri IPC command bridge (`ask_ai`).
- [x] OpenAI-compatible client integration supporting custom endpoints, API keys, and models.

### ✅ Phase 5: Reactive UI & Modular Architecture *(Current Phase)*
- [x] Cyberpunk-inspired **Aura Dual** glowing input borders with responsive corner brackets.
- [x] Multi-line auto-expanding textarea (up to 180px) with auto-scrolling.
- [x] **LatticeLoader**: 3x3 orbit wave dot matrix loading indicator with live stopwatch timer.
- [x] **Aura Single**: Glowing square stop-loading button state.
- [x] Slash command autocomplete list (`/web`, `/notes`, `/analysis`).
- [x] Markdown response rendering with custom code block and text styling.
- [x] Modular architecture separating Presentation (View), State (Container), Services (Tauri Bridge), Types, and Backend Commands.

### ⏳ Phase 6: Screen & Context Capture *(Next Up)*
- [ ] Active text selection detection via system Accessibility APIs.
- [ ] Screenshot capture command (`/screen`) to send visual context to multimodal models.
- [ ] OCR text extraction from screenshots.

### ⏳ Phase 7: Local Storage & Sessions
- [ ] SQLite database integration for saving conversation history.
- [ ] Session restore and persistent conversation memory.

### ⏳ Phase 8: Personal Knowledge Base & Notes
- [ ] "Save as Note" action from any AI answer.
- [ ] Note tagging, full-text search (SQLite FTS5), and retrieval-augmented context ("what did I learn about X?").

---

## 📁 Project Architecture

```text
ty/
├── public/                               # Static assets & compiled CSS
│   └── tailwind.css                      # Generated Tailwind v4 + DaisyUI CSS
├── src/                                  # Frontend (Yew WASM)
│   ├── app.rs                            # Root App component
│   ├── main.rs                           # WASM entry point
│   ├── components/
│   │   ├── ask_bar/                      # AskBar feature (Container + View)
│   │   ├── common/                       # Icons & corner markers
│   │   └── loaders/                      # LatticeLoader
│   ├── services/
│   │   └── tauri_bridge.rs               # Encapsulated Tauri v2 WASM FFI & helpers
│   ├── types/                            # Domain types (ChatTurn, Commands, etc.)
│   └── utils/                            # Shared utilities
└── src-tauri/                            # Backend (Tauri v2 Native Rust)
    ├── src/
    │   ├── main.rs                       # Tauri binary entry
    │   ├── lib.rs                        # App setup, shortcuts, plugin registry
    │   ├── commands/                     # Tauri command handlers (ask_ai)
    │   ├── ai/                           # AI client implementation
    │   └── types/                        # Backend types & constants
    ├── capabilities/                     # Window permissions & capabilities
    └── tauri.conf.json                   # App configuration
```

---

## 🚀 Getting Started

### Prerequisites

1. **Rust**: Make sure Rust is installed with the WebAssembly target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. **Trunk**: Install the WASM bundler:
   ```bash
   cargo install trunk
   ```
3. **Node.js**: Node 18+ for compiling Tailwind CSS.
4. **Tauri CLI**:
   ```bash
   cargo install tauri-cli --version "^2.0.0"
   ```

### Setup & Run

1. Clone the repository and install frontend dependencies:
   ```bash
   npm install
   ```

2. Create a `.env` file in the root or `src-tauri` directory with your AI credentials:
   ```env
   API_URL=https://api.openai.com/v1
   API_KEY=your_api_key_here
   AI_MODEL=gpt-4o-mini
   ```

3. Build the CSS stylesheet:
   ```bash
   npm run build:css
   ```

4. Launch the application in development mode:
   ```bash
   cargo tauri dev
   ```

---

## ⌨️ Shortcuts & Controls

| Shortcut / Action | Description |
|---|---|
| `Cmd + /` (macOS) / `Ctrl + /` | Toggle floating bar visibility |
| `Enter` | Submit prompt to AI |
| `Shift + Enter` | Insert newline in input |
| Drag outside input/buttons | Move the floating window anywhere on screen |
| `/` in input | Open slash command suggestions (`/web`, `/notes`, `/analysis`) |
