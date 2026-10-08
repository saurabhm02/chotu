# Chotu

A small AI assistant for macOS that floats above whatever you are doing. Press `Cmd + /` from anywhere, even over a full-screen app, ask about what is on your screen, and go back to work.

I want it to grow into something that does things on your computer for you: finding files, opening apps, tidying folders, running safe commands. It will always show you what it plans to do and wait for a yes first. The plan is in [`docs/roadmap.md`](docs/roadmap.md).

It is all Rust. The app is **Tauri v2** and the interface is **Yew** (Rust compiled to WebAssembly).

---

## What works today

| Area | What you can do |
|---|---|
| **Open it anywhere** | `Cmd + /` shows or hides Chotu, even over full-screen apps. If you had text selected in another app, it opens with that text attached |
| **Chat** | Answers stream in, with Markdown and code blocks, copy, regenerate and a thinking timer. It remembers the last 10 messages |
| **Saved chats** | Every chat is saved on your Mac. `/new` starts a fresh one, `/history` lets you search, open, rename (F2) and delete old ones. The AI gives each chat a name |
| **Selected text** | Chotu reads it through the macOS Accessibility API and falls back to `Cmd + C` when that fails |
| **Your screen** | `/screen` sends a screenshot to a vision model. `/ss` reads the text on your screen (Apple Vision) and asks about that text instead |
| **Images** | Paste a screenshot, an image from a browser, or an image file you copied in Finder. They stack next to the send button, you can click one to see it large, and they are sent along with your question and saved in the chat |
| **Web search** | `/web` searches DuckDuckGo, reads the top pages and answers with numbered sources you can click |
| **Text commands** | `/translate`, `/tldr`, `/bullets`, `/refine`, `/rewrite`, `/explain` and `/extract`. They work on highlighted text, on text you type, or on a pasted image |
| **Models** | You list your text and vision models in `.env`, and Chotu tries the next one if a model fails |

### Commands

| Command | What it does | Example |
|---|---|---|
| `/web` | Search the web and answer with sources | `/web what is new in Rust` |
| `/explain` | Explain an idea or code in plain words, with an example | `/explain what is a mutex` |
| `/translate` | Translate into another language. Say the language anywhere in your text. English is the default | `/translate how are you in Hindi` |
| `/tldr` | Sum up text in 1 to 3 short sentences | `/tldr in one sentence` |
| `/bullets` | Pull out the key points as a bullet list | `/bullets in Hindi` |
| `/refine` | Fix grammar, spelling and punctuation, and keep your own voice | `/refine hey just wanted to follow up on the thing` |
| `/rewrite` | Make text sound natural and casual | `/rewrite make it shorter` |
| `/extract` | Show all the text in a pasted image, or in a screenshot if you pasted none. No AI involved, so nothing gets changed | `/extract` |
| `/ss` | Read the text on your screen, then answer your question | `/ss what is this error?` |
| `/screen` | Show your screen to a vision model | `/screen what is this chart?` |
| `/new` | Start a new chat | |
| `/history` | Open an earlier chat | `/history rust` |

How the text commands work: highlight some text in any app, press `Cmd + /`, and type the command. The highlighted text is what gets worked on, and whatever you type after the command is treated as an instruction, like `in Hindi` or `make it shorter`. If you have no text handy, paste an image and Chotu reads the text from it.

---

## Where it is going

The idea is an assistant that acts for you. You say "put the PDFs in Downloads into folders by month", Chotu works out the steps, shows you exactly what it will do, and does it after you approve.

I am building it in small steps, safest first:

| Step | What | Risk |
|---|---|---|
| 1. Tools and a loop | Chotu can pick a tool (web, screen, files), look at the result and keep going, with step and time limits and a Stop button | none |
| 2. Read-only tools | Read and search files in folders you allow, read the clipboard and the screen | low |
| 3. Permissions | A card before every change that says what will happen, with Allow once, Always or Deny. Plus a log you can read | this is the safety base |
| 4. Safe actions | Open an app, a link or a file, copy to the clipboard, create a note or a file in an allowed folder | low |
| 5. Changes with undo | Move and rename files, make folders, send things to the Trash (never delete), always with a preview and an undo | medium |
| 6. Commands and clicking | Run allowlisted shell commands, click and type in apps, always with your approval | high |
| 7. Connections and memory | Calendar, mail, notes as long-term memory | medium |

The rules that come before any action tool:
1. Ask first. Anything that changes something waits for your approval.
2. Make it reversible: Trash instead of delete, previews, undo.
3. Give it the least power: only the folders and apps you allow.
4. Log everything somewhere you can read it.
5. Text from web pages, files and screenshots is just data. Chotu never takes orders from it.
6. Stop works immediately, and every run has step and time limits.
7. Warn before anything that looks like a password or a key leaves your Mac.

None of the agent steps are built yet. What exists today (chat, screen, images, saved chats, web, text commands) is what they will stand on.

---

## Tech stack

- **App**: [Tauri v2](https://v2.tauri.app/), with `tauri-nspanel` so the window can float over full-screen apps
- **Interface**: [Yew 0.21](https://yew.rs/), bundled with [Trunk](https://trunkrs.dev/)
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) and [DaisyUI v5](https://daisyui.com/)
- **AI**: any OpenAI-compatible API (OpenRouter, for example) through `async-openai`, with streaming and model fallback
- **Storage**: SQLite through `rusqlite` for chats and messages. Images live in the app's data folder
- **macOS**: the Accessibility API for selected text, Apple Vision for OCR, `arboard` for the clipboard, `enigo` for the `Cmd + C` fallback
- **Web search**: DuckDuckGo results through `scraper`, article text through `readability`
- **Markdown**: `pulldown-cmark`

---

## Project layout

```text
chotu/
├── src/                          # Frontend (Yew, WebAssembly)
│   ├── api/                      # Calls to the backend, and the code that routes each command
│   ├── components/               # Ask bar, chat bubbles, command menu, history list, image stack and viewer
│   ├── hooks/                    # Chat state, input, selected text, attachments, window size
│   ├── models/                   # Types shared with the backend
│   └── utils/                    # Markdown, highlighting, memory, time, attachment rules, /extract formatting
├── src-tauri/                    # Backend (Rust)
│   ├── prompts/                  # System prompt, web prompts, one prompt file per text command
│   └── src/
│       ├── commands/             # Functions the frontend can call
│       ├── services/             # AI client, web search, saved chats, screen, OCR, selection, shortcut, panel
│       ├── models/               # Backend types
│       ├── utils/                # Building text-command prompts, image helpers
│       ├── config.rs             # Constants and prompt templates
│       └── db.rs                 # SQLite setup and migrations
└── docs/                         # Roadmap and design notes
```

---

## Getting started

### What you need

1. **Rust**, plus the WebAssembly target: `rustup target add wasm32-unknown-unknown`
2. **Trunk**: `cargo install trunk`
3. **Bun**, which runs the CSS build: [bun.sh](https://bun.sh)
4. **Tauri CLI**: `cargo install tauri-cli --version "^2.0.0"`

### Run it

1. Install the frontend packages:
   ```bash
   bun install
   ```
2. Make a `.env` file in the project root:
   ```env
   API_URL=https://openrouter.ai/api/v1
   API_KEY=your_api_key_here
   # One model, or a few separated by commas. If one fails, the next is tried.
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

Chotu asks the first time it needs each one:
- **Accessibility**, to read text you selected in other apps.
- **Screen Recording**, for `/screen` and `/ss`.

To run the tests, use `cargo test` inside `src-tauri/`, and `cargo test -p chotu-ui` from the project root.

---

## Shortcuts

| Shortcut | What it does |
|---|---|
| `Cmd + /` | Show or hide Chotu. With text selected in another app, it opens with that text attached |
| `Enter` | Send. `Shift + Enter` adds a new line |
| `Cmd + V` | Paste an image (up to 4). The newest sits on top of the stack and `×` removes it |
| Click an image | See it large. Click it again or press `×` to close |
| `/` | Open the command menu |
| In `/history` | `↑ ↓` to move, `Enter` to open, `F2` to rename, `Delete` twice to delete, `Esc` to close |
| Drag anywhere outside the input and buttons | Move the window |

---

## Status

Early and personal. It works well for daily use on my Mac, but it is not packaged or signed, it has no Dock or menu bar icon (so no Quit item yet), and the API key sits in `.env`. What I plan to fix is in [`docs/roadmap.md`](docs/roadmap.md).
