# ty — A Floating AI Overlay for macOS with Notes System

A lightweight, spotlight-style ambient AI overlay for macOS with a built-in notes and personal knowledge base system. Always one keystroke away (`Cmd + /`), designed to answer questions, run fast slash commands, and capture notes without breaking your focus or switching away from your active apps.

Built with **100% Rust** — powered by **Tauri v2** for the native macOS desktop runtime and **Yew (WebAssembly)** for the reactive frontend interface.

---

## 🎯 Project Goal

Modern work involves constant context switching between browsers, code editors, terminals, note apps, and chat tools. `ty` eliminates that friction:
- **Always One Keystroke Away**: Summon the floating overlay with `Cmd + /` from anywhere (even over fullscreen apps), ask questions, and dismiss instantly.
- **Minimal & Distraction-Free**: A clean spotlight ask bar that expands dynamically only when answers or multi-line prompts are present.
- **Built-in Notes & Second Brain**: Save structured insights directly into local notes and query your knowledge base later with *"What did I learn about X?"*.

---

## 🛠️ Tech Stack

- **Desktop Runtime**: [Tauri v2](https://v2.tauri.app/) (Rust native backend)
- **Frontend**: [Yew 0.21](https://yew.rs/) (Rust WebAssembly)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) + [DaisyUI v5](https://daisyui.com/)
- **Build Tools**: [Trunk](https://trunkrs.dev/) (WASM bundler)
- **AI Engine**: OpenAI-compatible LLM client via `async-openai`
- **Markdown**: `pulldown-cmark` for structured answer and code rendering

---

## 🗺️ Roadmap & Current Status

We are building this project in iterative phases:

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

### ⏳ Phase 8: Personal Knowledge Base & Notes System
- [ ] "Save as Note" action directly from any AI response.
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
| `Cmd + /` (macOS) / `Ctrl + /` | Toggle floating overlay visibility |
| `Enter` | Submit prompt to AI |
| `Shift + Enter` | Insert newline in input |
| Drag outside input/buttons | Move the floating window anywhere on screen |
| `/` in input | Open slash command suggestions (`/web`, `/notes`, `/analysis`) |
