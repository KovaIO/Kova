# Kova

Your workspace, one menu away.

Kova is a lightweight desktop utility that lives in your system tray, giving you instant access to system metrics, clipboard history, workspace profiles, disk cleanup, and display brightness — all from a single menu.

## Features

- **System Monitor** — Live CPU, RAM, disk, and network metrics with per-process tree view and historical bar charts
- **Clipboard History** — Automatically captures text and image clipboard entries with search, paste, copy, and image preview
- **Workspace Profiles** — Save and restore app window layouts; position apps on a grid and restore them with one click
- **Disk Cleanup** — Scan for browser caches, dev artifacts, and large folders, categorized by safety level
- **Brightness Control** — Adjust display brightness directly from the app
- **Global Shortcuts** — Configurable keyboard shortcuts for quick access to any feature
- **Auto-Updates** — Built-in updater checks for new versions on startup

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop Runtime | [Tauri](https://tauri.app) v2 |
| Frontend | [SvelteKit](https://kit.svelte.dev) v2 + [Svelte](https://svelte.dev) v5 |
| Backend | [Rust](https://www.rust-lang.org/) (2021 edition) |
| Build Tool | [Vite](https://vite.dev/) v6 |
| Language | TypeScript ~5.6, Rust |
| Local Database | SQLite (via `rusqlite`) |
| Icons | [Lucide](https://lucide.dev/) for Svelte |

## Prerequisites

- [Node.js](https://nodejs.org/) v22+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/):
  - **Windows**: Microsoft Visual Studio C++ Build Tools, WebView2
  - **macOS**: Xcode Command Line Tools

## Getting Started

```bash
# Clone the repository
git clone https://github.com/KovaIO/Kova.git
cd Kova/app

# Install frontend dependencies
npm install

# Start the dev server (runs both Vite + Tauri)
npm run tauri dev
```

## Scripts

| Command | Description |
|---|---|
| `npm run dev` | Start Vite dev server only |
| `npm run build` | Build frontend for production |
| `npm run preview` | Preview production build |
| `npm run check` | Type-check the project |
| `npm run check:watch` | Type-check in watch mode |
| `npm run tauri` | Run Tauri CLI (e.g. `npm run tauri dev`, `npm run tauri build`) |

## Project Structure

```
app/
├── src/                        # Frontend (Svelte/TypeScript)
│   ├── routes/                 # SvelteKit routes (one per window)
│   │   ├── home/               # Tray popover — metrics cards, toggles
│   │   ├── monitor/            # System monitor with graphs + process tree
│   │   ├── clipboard/          # Clipboard history panel
│   │   ├── prefs/              # Preferences window (sidebar + sections)
│   │   ├── profiles/           # Workspace profile ring selector
│   │   ├── onboarding/         # First-run wizard
│   │   └── update/             # Update notification dialog
│   ├── components/             # Reusable Svelte components
│   ├── stores/                 # Svelte writable stores (preferences, license)
│   ├── services/               # Tauri invoke wrappers (IPC layer)
│   ├── types/                  # TypeScript type definitions
│   └── utils/                  # Utility functions
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── commands/           # Tauri command handlers (IPC boundary)
│   │   ├── clipboard/          # Clipboard subsystem (watcher, storage, service)
│   │   ├── metrics/            # System metrics collection
│   │   ├── processes/          # Process enumeration and management
│   │   ├── workspaces/         # Workspace profile logic
│   │   ├── disk/               # Disk cleanup scanner
│   │   ├── preferences/        # User preferences + monitor dimming
│   │   ├── license/            # License verification and activation
│   │   ├── shortcuts/          # Global keyboard shortcuts
│   │   ├── windows/            # Window management utilities
│   │   ├── apps/               # Installed app detection
│   │   └── update/             # Auto-update logic
│   ├── migrations.rs           # SQLite schema creation
│   ├── app_state.rs            # Central application state
│   ├── lib.rs                  # Plugin setup, tray icon, window management
│   └── main.rs                 # Entry point
├── static/                     # Static assets (favicon, icons)
├── tauri.conf.json             # Tauri configuration
├── svelte.config.js            # SvelteKit config (adapter-static, SPA mode)
├── vite.config.js              # Vite config
└── package.json
```

## Architecture

Kova uses a **multi-window architecture**. Each feature runs in its own frameless, transparent window that shows/hides on demand from the system tray:

| Window | Size | Purpose |
|---|---|---|
| `home` | 280x330 | Tray popover — metrics, toggles, brightness |
| `monitor` | 380x720 | System monitor with graphs and process list |
| `process` | 380x720 | Process detail view |
| `clipboard` | 380x720 | Clipboard history panel |
| `profiles` | 380x380 | Workspace profile selector |
| `prefs` | 980x720 | Full preferences window |
| `onboarding` | 520x380 | First-run wizard |

All windows are **frameless**, **transparent**, and **hidden by default** — they appear/disappear via the tray icon.

## Licensing

Kova uses a freemium model:

| Tier | Price | Clipboard | Disk Cleanup | Workspaces | Monitor Dimming |
|---|---|---|---|---|---|
| **Free** | $0 | 50 items | - | - | - |
| **Pro Monthly** | $2.99/mo | Unlimited | Yes | Yes | Yes |
| **Pro Yearly** | $24/yr | Unlimited | Yes | Yes | Yes |
| **Lifetime** | $59 | Unlimited | Yes | Yes | Yes |

Payments are handled via [Polar](https://polar.sh/).

## Platform Support

| Platform | Status |
|---|---|
| Windows | Supported |
| macOS | Supported |

Platform-specific implementations:
- **Windows**: Native clipboard listener (`WM_CLIPBOARDUPDATE`), Win32 APIs for window management, `sysinfo` for process enumeration
- **macOS**: Clipboard polling (450ms), `CGEvent` keyboard hook, Cocoa for window management, LaunchAgent for autostart
