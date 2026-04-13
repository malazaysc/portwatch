# Design Document

## Overview

portwatch is a developer-focused TUI tool that provides real-time visibility into all listening TCP ports on a local machine. It combines port scanning, process introspection, and framework detection into a single interactive dashboard inspired by lazygit and lazydocker.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   TUI Layer                     │
│              (ratatui + crossterm)               │
│  ┌──────────┐ ┌───────────┐ ┌────────────────┐ │
│  │Port List │ │Detail View│ │ Action Bar     │ │
│  │  Panel   │ │  Panel    │ │  Panel         │ │
│  └──────────┘ └───────────┘ └────────────────┘ │
├─────────────────────────────────────────────────┤
│                App State Layer                  │
│         (event loop + state management)         │
├─────────────────────────────────────────────────┤
│               Data Collection Layer             │
│  ┌──────────┐ ┌───────────┐ ┌────────────────┐ │
│  │  Port    │ │  Process  │ │   Tech         │ │
│  │ Scanner  │ │  Resolver │ │  Detector      │ │
│  └──────────┘ └───────────┘ └────────────────┘ │
├─────────────────────────────────────────────────┤
│             Platform Abstraction                │
│        ┌──────────┐  ┌───────────┐              │
│        │  macOS   │  │   Linux   │              │
│        │ Backend  │  │  Backend  │              │
│        └──────────┘  └───────────┘              │
└─────────────────────────────────────────────────┘
```

### Layers

**TUI Layer** — Rendering and input handling via ratatui. Three-panel lazygit-style layout: port list, detail view, and action bar.

**App State Layer** — Central event loop using tokio. Manages application state, handles keyboard input, dispatches actions, and coordinates data refresh cycles.

**Data Collection Layer** — Three subsystems that gather and enrich port data:
- Port Scanner: discovers listening TCP ports
- Process Resolver: maps ports to processes with full metadata
- Tech Detector: identifies frameworks and runtimes

**Platform Abstraction** — OS-specific implementations behind a common trait interface, enabling macOS-first development with Linux as a clean addition.

## Data Collection

### Port Scanning

**macOS:**
```
lsof -iTCP -sTCP:LISTEN -P -n
```
Returns: port, PID, process name, bind address (local vs 0.0.0.0).

Alternative: use `libproc` FFI directly for lower overhead — avoids spawning a subprocess on each refresh cycle.

**Linux (planned):**
```
ss -tlnp
```
Or parse `/proc/net/tcp` directly for zero-dependency scanning.

**Refresh strategy:** Poll every 2 seconds by default. Diff against previous state to detect new/removed ports and emit events for the UI.

### Process Resolution

Given a PID from port scanning, resolve:

| Data Point        | macOS Source                        | Linux Source            |
|-------------------|-------------------------------------|-------------------------|
| Command line      | `proc_pidinfo` / `ps -o args`       | `/proc/PID/cmdline`     |
| Working directory | `proc_pidinfo(PROC_PIDVNODEPATHINFO)` | `/proc/PID/cwd` readlink |
| Start time        | `proc_pidinfo` / `sysctl kern.proc` | `/proc/PID/stat` field 22 |
| Parent PID        | `sysinfo` crate                     | `/proc/PID/stat` field 4  |
| User              | `sysinfo` crate                     | `/proc/PID/status`      |

### Technology Detection

Detection runs in priority order — first match wins:

1. **Command line analysis** — Parse the full command line for known patterns:
   - `next dev` / `next start` → Next.js
   - `vite` / `vite dev` → Vite
   - `manage.py runserver` → Django
   - `flask run` → Flask
   - `rails server` / `puma` → Rails
   - `uvicorn` → FastAPI/Starlette
   - `cargo run` / target path detection → Rust
   - `go run` → Go
   - `php artisan serve` → Laravel
   - `hugo server` → Hugo

2. **Project file scanning** — Read files in the process working directory:
   - `package.json` → check `dependencies`/`devDependencies` for `next`, `vite`, `express`, `fastify`, `nuxt`, `remix`, `astro`, etc.
   - `Cargo.toml` → check for `axum`, `actix-web`, `rocket`, `warp`
   - `requirements.txt` / `pyproject.toml` → check for `django`, `flask`, `fastapi`
   - `Gemfile` → check for `rails`, `sinatra`
   - `go.mod` → check for `gin`, `echo`, `fiber`
   - `composer.json` → check for `laravel`

3. **Port heuristics** (lowest priority, fallback only):
   - 3000 → likely Node.js
   - 5173 → likely Vite
   - 8000 → likely Django/Python
   - 4200 → likely Angular
   - 5432 → PostgreSQL
   - 6379 → Redis
   - 27017 → MongoDB

Each detection method returns a confidence level. The UI shows the highest-confidence result.

### Git / Worktree Detection

For each process working directory:
1. Run `git rev-parse --is-inside-work-tree` to confirm it's a git repo
2. Run `git rev-parse --show-toplevel` to get repo root
3. Run `git rev-parse --git-common-dir` — if it differs from `.git`, it's a worktree
4. Run `git branch --show-current` for the branch name

Cache results per directory and invalidate every 30 seconds.

### Network Exposure Detection

From the bind address returned by lsof/ss:
- `127.0.0.1` or `::1` → **Local only** (safe)
- `0.0.0.0` or `::` → **Exposed** (accessible from network)
- Specific IP → **Bound to interface** (show which one)

Display with visual indicators: a lock icon for local, a warning for exposed.

### Traffic Monitoring (v0.2)

**macOS:** Use `nettop` or `NetworkStatistics` private framework. Alternatively, periodic `netstat -b` diffs to calculate bytes in/out per port.

**Linux:** Parse `/proc/net/tcp` and `/proc/PID/net/dev` for byte counters.

This is inherently approximate — show aggregate stats, not real-time packet inspection.

### Uptime Calculation

Process start time is retrieved during process resolution. Display as human-friendly relative time:
- Under 1 minute: `<1m`
- Under 1 hour: `45m`
- Under 1 day: `2h 14m`
- Over 1 day: `3d 2h`

## UI Design

### Layout

Lazygit-inspired three-panel vertical layout:

```
┌─ Ports (12) ─────────────────────────────────────────────────┐
│                                                              │
│  Scrollable table with one row per listening port.           │
│  Columns: port, PID, process, tech, directory, uptime.       │
│  Selected row highlighted. Arrow/vim keys to navigate.       │
│                                                              │
├─ Details ────────────────────────────────────────────────────┤
│                                                              │
│  Expanded info for the selected port. Shows full path,       │
│  bind address, git branch, traffic stats, detection method.  │
│                                                              │
├─ Actions ────────────────────────────────────────────────────┤
│  [k]ill  [b]rowser  [f]older  [r]estart  [c]opy URL         │
└──────────────────────────────────────────────────────────────┘
```

### Color Scheme

Follow lazygit conventions:
- **Selected row:** bold white on blue background
- **Exposed ports:** red/yellow warning indicator
- **Local ports:** green lock indicator
- **Tech labels:** colored by ecosystem (blue for Node, green for Python, orange for Rust, etc.)
- **Uptime:** dimmed/gray for long-running, bright for recently started

### Responsive

Adapt column visibility based on terminal width:
- **Narrow (<80 cols):** port, process, tech only
- **Medium (80-120 cols):** add directory, uptime
- **Wide (>120 cols):** full detail including PID, bind address

## Actions

### Kill Process (`k`)
- Send `SIGTERM` to the process
- Show confirmation dialog before killing
- Remove from list on next refresh cycle

### Open in Browser (`b`)
- Construct URL: `http://localhost:<port>`
- Use `open` (macOS) or `xdg-open` (Linux) to launch default browser
- If bind address is not localhost, use the actual bind address

### Open Folder (`f`)
- Open the process working directory in a new terminal tab
- macOS: use AppleScript to open a new Terminal/iTerm2 tab
- Support configurable terminal emulator (env var or config file)

### Copy URL (`c`)
- Copy `http://localhost:<port>` to system clipboard
- Use `pbcopy` (macOS) or `xclip`/`xsel` (Linux)
- Show brief confirmation flash in the UI

### Restart (`r`) — v0.2
- Record the full command line of the process
- Kill the process
- Re-launch with the same command in the same working directory
- Track the new PID

## Module Structure

```
src/
├── main.rs              # Entry point, CLI args, app bootstrap
├── app.rs               # Application state and event loop
├── ui/
│   ├── mod.rs           # UI module root
│   ├── layout.rs        # Panel layout and sizing
│   ├── port_list.rs     # Port list table rendering
│   ├── detail_view.rs   # Detail panel rendering
│   └── action_bar.rs    # Action bar rendering
├── scanner/
│   ├── mod.rs           # Scanner trait and common types
│   ├── macos.rs         # macOS port scanning (lsof / libproc)
│   └── linux.rs         # Linux port scanning (ss / procfs)
├── process/
│   ├── mod.rs           # Process resolution trait
│   ├── macos.rs         # macOS process info
│   └── linux.rs         # Linux process info
├── detect/
│   ├── mod.rs           # Tech detection orchestrator
│   ├── command_line.rs  # Command line pattern matching
│   ├── project_files.rs # Project file scanning
│   └── port_hints.rs    # Port-based heuristics
├── git.rs               # Git/worktree detection
├── actions.rs           # User action handlers (kill, open, copy)
├── config.rs            # Configuration and CLI args
└── types.rs             # Shared data types (PortEntry, etc.)
```

## Performance Considerations

- **Polling interval:** 2s default, configurable. Avoid spinning.
- **Process file scanning:** Cache tech detection results per PID. Invalidate on PID recycle.
- **lsof parsing:** Consider switching to direct `libproc` FFI if subprocess overhead is noticeable with many ports.
- **UI rendering:** Only re-render on state change, not on every tick.
- **Git operations:** Cache per-directory, invalidate every 30s. Don't block the UI on git commands.

## Configuration

Support a config file at `~/.config/portwatch/config.toml`:

```toml
# Refresh interval in seconds
refresh_interval = 2

# Terminal emulator for "open folder" action
terminal = "iterm2"  # or "terminal", "wezterm", "alacritty", "kitty"

# Ports to always hide (system services, databases, etc.)
hidden_ports = [5432, 6379, 27017]

# Ports to always show even if not typically web servers
watch_ports = []

# Color theme
theme = "default"  # or "monokai", "nord"
```

## Future Ideas (beyond v0.2)

- **Port groups** — group ports by project/repo (all ports from the same repo root)
- **Notifications** — alert when a new port opens or a server crashes
- **Log tailing** — show recent stdout/stderr from the server process
- **Docker awareness** — detect and show containerized services alongside native ones
- **tmux/zellij integration** — open folders in splits instead of new tabs
- **Remote monitoring** — connect to remote machines via SSH
- **Plugin system** — custom tech detectors and actions
