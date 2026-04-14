# portwatch

A terminal UI tool for developers to monitor and manage local web server ports. Think **lazygit**, but for your dev servers.

Built for developers juggling multiple projects, coding agents, and worktrees simultaneously.

```
┌ Ports (28) ──────────────────────────────────────────────────────────────┐
│ ▼ navaris (5 ports)                                                      │
│ ● 3001  node       Next.js                ~/dev/navaris          2h 14m  │
│ ● 54321 com.docke  Docker (navaris)        ~/Library/Containers/  15d     │
│ ● 54322 com.docke  Docker (navaris)        ~/Library/Containers/  15d     │
│ ● 61006 node       @playwright/test        ~/dev/navaris          4h      │
│ ● 16034 Cursor     Cursor (navaris)        /                      2d      │
│ ▼ asado (3 ports)                                                        │
│ ● 8080  test_migr  Axum (Rust)             ~/dev/asado-apps/      1h      │
│ ● 24857 Cursor     Cursor (asado)          /                      2d      │
│ ● 52368 Google     Chrome (debug port)     /                      3d      │
├ Details ─────────────────────────────────────────────────────────────────┤
│ :3001 — node (PID 71775)                                                 │
│   Bind:  0.0.0.0 (exposed to network!)                                   │
│   Tech:  Next.js (via project file)                                       │
│   Dir:   ~/dev/navaris (main)                                             │
│   CPU:   2.3%                                                             │
│   Mem:   145.2 MB                                                         │
│   Up:    2h 14m                                                           │
├──────────────────────────────────────────────────────────────────────────┤
│  [x] kill  [b] browser  [f] folder  [c] copy url  [r] refresh  [?] help │
└──────────────────────────────────────────────────────────────────────────┘
```

## Features

- **Port discovery** — lists all listening TCP ports, auto-refreshes in the background
- **Technology detection** — identifies Next.js, Vite, Django, Express, Axum, and 30+ frameworks via command line, `package.json`, `Cargo.toml`, npm packages, and port heuristics
- **Docker awareness** — maps container ports to container name, image, and compose project
- **IDE detection** — labels Cursor, VS Code, Zed ports with their workspace name
- **Project grouping** — ports grouped by git repo, Docker compose project, or IDE workspace. Collapsible with arrow keys
- **Git context** — shows branch name and worktree info for each process
- **Network exposure** — green dot for local-only, red dot for exposed to network
- **Resource monitoring** — CPU% and RAM per process
- **Search & sort** — filter by name/tech/port, sort by any column
- **Actions** — kill process, open in browser, open folder, copy URL
- **Config file** — customizable refresh interval and terminal emulator

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap malazaysc/portwatch https://github.com/malazaysc/portwatch
brew install portwatch
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/malazaysc/portwatch/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/malazaysc/portwatch/releases/latest/download/portwatch-macos-aarch64.tar.gz | tar xz
sudo mv portwatch /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/malazaysc/portwatch/releases/latest/download/portwatch-macos-x86_64.tar.gz | tar xz
sudo mv portwatch /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/malazaysc/portwatch/releases/latest/download/portwatch-linux-x86_64.tar.gz | tar xz
sudo mv portwatch /usr/local/bin/

# Linux (aarch64)
curl -L https://github.com/malazaysc/portwatch/releases/latest/download/portwatch-linux-aarch64.tar.gz | tar xz
sudo mv portwatch /usr/local/bin/
```

### Cargo (build from source)

```bash
cargo install --git https://github.com/malazaysc/portwatch.git
```

### Build from source

```bash
git clone https://github.com/malazaysc/portwatch.git
cd portwatch
cargo build --release
./target/release/portwatch
```

## Usage

```bash
portwatch                # launch the TUI
portwatch -i 5           # set refresh interval to 5 seconds
```

### Keybindings

| Key        | Action                          |
|------------|---------------------------------|
| `↑` / `↓`  | Move selection up/down          |
| `←`         | Collapse group                  |
| `→`         | Expand group                    |
| `Home`/`End`| Jump to first/last              |
| `x`         | Kill process (with confirmation)|
| `b`         | Open in browser                 |
| `f`         | Open folder                     |
| `c`         | Copy URL to clipboard           |
| `r`         | Refresh                         |
| `/`         | Search/filter ports             |
| `s`         | Cycle sort column               |
| `S`         | Toggle sort direction           |
| `?`         | Show help                       |
| `q` / `Esc` | Quit                            |

## Configuration

Optional config file at `~/.config/portwatch/config.toml`:

```toml
# Refresh interval in seconds
refresh_interval = 3

# Terminal emulator for "open folder" action
# Options: "finder", "iterm2", "terminal", "wezterm", "alacritty", "kitty"
terminal = "finder"
```

CLI flags override config file values.

## Platform Support

| Platform       | Status |
|----------------|--------|
| macOS (ARM)    | Supported |
| macOS (Intel)  | Supported |
| Linux (x86_64) | Supported |
| Linux (ARM)    | Supported |

## License

MIT
