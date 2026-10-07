# Chotu — A Floating AI Assistant for macOS

A small, spotlight-style AI assistant that floats above your other apps. Press `Cmd + /` from anywhere, even over a full-screen app, ask a question about what you are looking at, and get back to work.

**Where it is going:** Chotu is growing into an **agent that does things on your computer for you**: read your files, open apps, organize folders, run safe commands. It will always show what it is about to do and wait for your approval first. See [the plan](#-where-chotu-is-going) and [`docs/roadmap.md`](docs/roadmap.md).

Built with **Rust end to end**: **Tauri v2** for the native macOS app and **Yew (WebAssembly)** for the interface.

---

## ✅ What works today

| Area | What you can do |
|---|---|
| **Summon** | `Cmd + /` shows or hides Chotu from anywhere, including over full-screen apps. Select text in any app first and it opens with that text attached |
| **Chat** | Streaming answers, Markdown and code blocks, copy, regenerate, a thinking timer. The AI remembers the last 10 messages |
| **Saved chats** | Every chat is saved on your Mac (SQLite). `/new` starts a fresh chat, `/history` lists, filters, opens, renames (F2) and deletes old chats. The AI names each chat |
| **Selected text** | Read through the macOS Accessibility API, with a `Cmd + C` fallback. Shown as a quote above your question |
| **Screen** | `/screen` sends a screenshot to a vision model. `/ss` reads the text on your screen (Apple Vision OCR) and asks about it |
| **Images** | Paste a screenshot, an image from a browser, or an image file copied in Finder. Pasted images stack next to the send button. Click one to see it large. They are sent to a vision model with your question and saved with the chat |
| **Web** | `/web` searches DuckDuckGo, reads the top pages and answers with numbered citations that open the source |
| **Text commands** | `/translate` and `/tldr` (more planned, see below) |
| **Models** | Text and vision model lists from `.env`, with automatic fallback to the next model |

### Commands

| Command | What it does | Example |
|---|---|---|
| `/web <question>` | Search the web and answer with sources | `/web what is new in Rust 1.9x` |
| `/explain <text>` | Explain text or code | `/explain what is a mutex` |
| `/translate <text>` | Translate. Name the language anywhere in your text; English is the default | `/translate how are you in Hindi` |
| `/tldr <text>` | Summarize in 1 to 3 sentences. You can add an instruction | `/tldr in one sentence` |
| `/ss <question>` | Read the text on your screen, then answer | `/ss what is this error?` |
| `/screen <question>` | Show your screen to a vision model | `/screen what is this chart?` |
| `/new` | Start a new chat | |
| `/history` | Open an earlier chat | `/history rust` |

For `/translate` and `/tldr`, highlight text in another app, press `Cmd + /`, then type the command. With text highlighted, the highlighted text is what gets worked on and what you type is the instruction.

---

## 🧭 Where Chotu is going

The big idea: **an agent that does things on your computer for you.** You say what you want ("put the PDFs from Downloads into folders by month"), Chotu works out the steps, shows you exactly what it will do, and does it once you approve.

It is built in small steps, safest first:

| Stage | What | Risk |
|---|---|---|
| 1. Tools and the agent loop | Chotu can decide to use a tool (web, screen, files), see the result, and continue, with step and time limits and a Stop button | none |
| 2. Read-only tools | Read and search files in folders you allow, read the clipboard and the screen | low |
| 3. Permissions | A confirmation card for every change: what will happen, then Allow once, Always, or Deny. A readable action log | the safety base |
| 4. Safe actions | Open an app, a link or a file, copy to the clipboard, create a note or file in an allowed folder | low |
| 5. Changes with undo | Move and rename files, create folders, move things to the Trash (never delete), with a preview and an undo | medium |
| 6. Commands and automation | Run allowlisted shell commands, click and type in apps, always with your approval | high |
| 7. Connections and memory | Calendar, mail, notes as long-term memory | medium |

**Safety rules (they come before any action tool):**
1. Ask first. Anything that changes something waits for your approval.
2. Reversible by default: Trash instead of delete, previews, undo.
3. Least power: only the folders and apps you allow.
4. Everything is logged where you can read it.
5. Text from web pages, files and screenshots is data, never an instruction to follow.
6. Stop works at once, and every run has step and time limits.
7. A warning before text that looks like a password or key leaves your Mac.

None of the agent stages is built yet. Today's features (chat, screen, images, saved chats, web) are the base it stands on.

---

## 🛠️ Tech Stack

- **App**: [Tauri v2](https://v2.tauri.app/) (Rust), with `tauri-nspanel` so the window can float over full-screen apps
- **Interface**: [Yew 0.21](https://yew.rs/) (Rust to WebAssembly), bundled with [Trunk](https://trunkrs.dev/)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) + [DaisyUI v5](https://daisyui.com/)
- **AI**: any OpenAI-compatible endpoint (for example OpenRouter) through `async-openai`, with streaming and model fallback
- **Storage**: SQLite through `rusqlite` (bundled) for chats and messages; images stored in the app's data folder
- **macOS**: Accessibility API (selected text), Apple Vision (OCR), `arboard` (clipboard), `enigo` (the `Cmd + C` fallback)
- **Web search**: DuckDuckGo results via `scraper`, article text via `readability`
- **Markdown**: `pulldown-cmark`

---

## 📁 Project Layout

```text
chotu/
├── src/                          # Frontend (Yew, WebAssembly)
│   ├── api/                      # Calls to the backend (chat, history, images, screen) and command routing
│   ├── components/               # AskBar, chat bubbles, command menu, history list, image stack and viewer
│   ├── hooks/                    # Chat state, input, selected text, attachments, window size
│   ├── models/                   # Types shared with the backend (chat turns, commands, stream events)
│   └── utils/                    # Markdown, highlighting, memory, time, attachment rules
├── src-tauri/                    # Backend (Rust)
│   ├── prompts/                  # System prompt, web prompts, /translate and /tldr prompts
│   └── src/
│       ├── commands/             # Functions the frontend can call (chat, history, images, screen, web)
│       ├── services/             # AI client, web search, saved chats, screen, OCR, selection, shortcut, panel
│       ├── models/               # Backend types
│       ├── utils/                # Prompt building for text commands, image helpers
│       ├── config.rs             # Constants and prompt templates
│       └── db.rs                 # SQLite setup and migrations
└── docs/                         # Roadmap and design notes
```

---

## 🚀 Getting Started

### Prerequisites

1. **Rust** with the WebAssembly target: `rustup target add wasm32-unknown-unknown`
2. **Trunk**: `cargo install trunk`
3. **Bun** (runs the CSS build): [bun.sh](https://bun.sh)
4. **Tauri CLI**: `cargo install tauri-cli --version "^2.0.0"`

### Setup and run

1. Install the frontend packages:
   ```bash
   bun install
   ```
2. Create a `.env` file in the project root:
   ```env
   API_URL=https://openrouter.ai/api/v1
   API_KEY=your_api_key_here
   # One model, or several separated by commas. The next one is tried if a model fails.
   AI_MODEL=model-a,model-b
   # Optional: models that can see images (used for /screen and pasted images)
   AI_VISION_MODEL=vision-model-a,vision-model-b
   ```
3. Build the stylesheet:
   ```bash
   bun run build:css
   ```
4. Start the app:
   ```bash
   cargo tauri dev
   ```

### macOS permissions

Chotu asks for these the first time they are needed:
- **Accessibility**: to read the text you selected in other apps.
- **Screen Recording**: for `/screen` and `/ss`.

Run the tests with `cargo test` inside `src-tauri/`, and `cargo test -p chotu-ui` at the project root.

---

## ⌨️ Shortcuts and Controls

| Shortcut / Action | What it does |
|---|---|
| `Cmd + /` | Show or hide Chotu. With text selected in another app, opens with that text attached |
| `Enter` | Send. `Shift + Enter` adds a new line |
| `Cmd + V` | Paste an image (up to 4). The newest shows on top of the stack, `×` removes it |
| Click an image | See it large; click it again or `×` to close |
| `/` | Open the command menu |
| In `/history` | `↑ ↓` move, `Enter` open, `F2` rename, `Delete` twice to delete, `Esc` close |
| Drag outside the input and buttons | Move the window |

---

## 📌 Status

Early and personal: it works well for daily use on one Mac, but it is not packaged or signed yet, there is no Dock or menu bar icon (so no Quit item yet), and the API key lives in `.env`. Planned fixes are in [`docs/roadmap.md`](docs/roadmap.md).
