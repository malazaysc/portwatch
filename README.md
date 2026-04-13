# portwatch

A terminal UI tool for developers to monitor and manage local web server ports. Think **lazygit**, but for your dev servers.

Built for developers juggling multiple projects, coding agents, and worktrees simultaneously.

```
┌─ Ports ──────────────────────────────────────────────────────┐
│ PORT │ PID  │ PROCESS    │ TECH       │ DIR          │ UP   │
│►3000 │ 1234 │ node       │ Next.js    │ ~/app        │ 2h   │
│ 3001 │ 1235 │ node       │ Vite       │ ~/frontend   │ 45m  │
│ 8000 │ 5678 │ python     │ Django     │ ~/api        │ 1h   │
│ 8080 │ 9012 │ portwatch  │ Rust/Axum  │ ~/portwatch  │ 5m   │
│ 5432 │ 442  │ postgres   │ PostgreSQL │ —            │ 3d   │
├─ Details ────────────────────────────────────────────────────┤
│ :3000 — node (PID 1234)                                     │
│ Bind: 127.0.0.1 (local only)                                │
│ Tech: Next.js (detected via package.json)                    │
│ Dir:  ~/projects/my-app (worktree: feature-auth)             │
│ Traffic: ↓ 12.4 MB  ↑ 1.2 MB                                │
│ Started: 2h 14m ago                                          │
├─ Actions ────────────────────────────────────────────────────┤
│ [k]ill  [b]rowser  [f]older  [r]estart  [c]opy URL          │
└──────────────────────────────────────────────────────────────┘
```

## Features

### Port Discovery
- Lists all listening TCP ports on the system
- Shows PID, process name, and bind address for each port
- Auto-refreshes to catch new servers as they start

### Technology Detection
- Identifies the framework/runtime behind each port (Next.js, Vite, Django, Flask, Express, Rails, etc.)
- Uses a combination of process command line inspection, project file scanning (`package.json`, `Cargo.toml`, `requirements.txt`), and port heuristics
- Extensible detection system for adding new frameworks

### Project Context
- Resolves the working directory of each process
- Detects git worktree information (branch, worktree name)
- Shows the project folder path for quick identification

### Network Insight
- **Local vs Exposed** — shows whether a port is bound to `127.0.0.1` (local only) or `0.0.0.0` (exposed to network)
- **Traffic stats** — aggregate inbound/outbound bytes per port
- **Uptime** — how long each server process has been running

### Interactive Actions
- **Kill** — stop a running server process
- **Open in browser** — launch `http://localhost:<port>` in default browser
- **Open folder** — open the project directory in a new terminal tab/pane
- **Restart** — kill and re-launch a server (v0.2)
- **Copy URL** — copy the server URL to clipboard

### Keyboard-Driven
- Full keyboard navigation (vim-style `j`/`k`, arrow keys)
- Single-key actions (`k`ill, `b`rowser, `f`older, `c`opy)
- Lazygit-inspired panel layout with detail views

## Installation

> Coming soon — the project is under active development.

```bash
cargo install portwatch
```

Or build from source:

```bash
git clone https://github.com/yourusername/portwatch.git
cd portwatch
cargo build --release
./target/release/portwatch
```

## Usage

```bash
# Launch the TUI
portwatch

# Shorthand alias (suggested)
alias pw="portwatch"
```

### Keybindings

| Key         | Action                            |
|-------------|-----------------------------------|
| `j` / `↓`  | Move selection down               |
| `k` / `↑`  | Move selection up                 |
| `Enter`     | Expand/collapse details           |
| `k`         | Kill selected process             |
| `b`         | Open in browser                   |
| `f`         | Open folder in terminal           |
| `c`         | Copy URL to clipboard             |
| `r`         | Refresh port list                 |
| `/`         | Filter/search ports               |
| `q`         | Quit                              |
| `?`         | Show help                         |

## Platform Support

| Platform | Status  |
|----------|---------|
| macOS    | Primary target (v0.1) |
| Linux    | Planned (v0.2) |

## Tech Stack

| Crate        | Purpose                                |
|--------------|----------------------------------------|
| `ratatui`    | Terminal UI framework                  |
| `crossterm`  | Cross-platform terminal backend        |
| `tokio`      | Async runtime for non-blocking polling |
| `sysinfo`    | Cross-platform process metadata        |
| `nix`        | POSIX helpers (Linux support)          |

## License

MIT
