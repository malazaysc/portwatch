# Roadmap

## v0.1 — MVP

The minimum viable product focused on macOS, core port visibility, and basic actions.

### Port Discovery & Display
- [ ] Scan listening TCP ports via `lsof -iTCP -sTCP:LISTEN -P -n`
- [ ] Parse output into structured port entries (port, PID, process name, bind address)
- [ ] Auto-refresh on a 2-second polling interval
- [ ] Diff-based updates (detect new/removed ports between cycles)

### Process Resolution
- [ ] Resolve working directory for each PID (`proc_pidinfo`)
- [ ] Resolve full command line for each PID
- [ ] Calculate uptime from process start time
- [ ] Display human-friendly relative uptime (`2h 14m`, `3d`, etc.)

### Technology Detection
- [ ] Command line pattern matching (next dev, vite, django, flask, uvicorn, etc.)
- [ ] Project file scanning (package.json, Cargo.toml, requirements.txt, etc.)
- [ ] Port-based fallback heuristics

### Git Context
- [ ] Detect if working directory is a git repository
- [ ] Show current branch name
- [ ] Detect and display git worktree info

### Network Exposure
- [ ] Classify ports as local (`127.0.0.1`) or exposed (`0.0.0.0`)
- [ ] Visual indicators for exposure status

### TUI
- [ ] Three-panel lazygit-style layout (port list, details, action bar)
- [ ] Keyboard navigation (vim keys + arrows)
- [ ] Selected row highlighting
- [ ] Responsive column visibility based on terminal width
- [ ] Color-coded tech labels and exposure indicators

### Actions
- [ ] Kill process (with confirmation dialog)
- [ ] Open in browser (`open http://localhost:<port>`)
- [ ] Open folder in terminal (new tab)
- [ ] Copy URL to clipboard
- [ ] Manual refresh

### Infrastructure
- [ ] CLI argument parsing (help, version)
- [ ] Error handling for permission issues (ports owned by root)
- [ ] Graceful terminal cleanup on exit

---

## v0.2 — Enhanced Monitoring & Linux Support

### Linux Support
- [ ] Port scanning via `ss -tlnp` or `/proc/net/tcp`
- [ ] Process resolution via `/proc/PID/` filesystem
- [ ] Clipboard via `xclip` / `xsel`
- [ ] Terminal launch via `xdg-open` and common terminal emulators

### Traffic Monitoring
- [ ] Aggregate inbound/outbound byte counts per port
- [ ] macOS: `nettop` or `netstat -b` diff-based approach
- [ ] Linux: `/proc/net/tcp` byte counter parsing
- [ ] Display as human-friendly sizes (KB, MB, GB)

### Restart Action
- [ ] Record full command line before kill
- [ ] Re-launch process in the same working directory
- [ ] Track the new PID after restart

### Configuration
- [ ] Config file support (`~/.config/portwatch/config.toml`)
- [ ] Configurable refresh interval
- [ ] Configurable terminal emulator
- [ ] Hidden ports list (filter out databases, system services)
- [ ] Theme support

### UX Polish
- [ ] Search/filter ports (`/` key)
- [ ] Sort by column (port, uptime, tech)
- [ ] Help overlay (`?` key)
- [ ] Flash notifications for actions (copied, killed, etc.)

---

## v0.3 — Grouping & Intelligence

### Port Groups
- [ ] Group ports by project (same git repo root)
- [ ] Collapsible groups in the port list
- [ ] Group-level actions (kill all ports in a project)

### Notifications
- [ ] Detect when a new port opens (flash/highlight)
- [ ] Detect when a server crashes (port disappears)
- [ ] Optional desktop notifications

### Log Tailing
- [ ] Show recent stdout/stderr from server processes
- [ ] Scrollable log panel (toggle with a key)

### Resource Monitoring (per process)
- [ ] CPU usage via `sysinfo` crate (already a dependency)
- [ ] Memory (RSS) via `sysinfo` crate
- [ ] Show CPU/RAM columns in port list and detail view
- [ ] Disk I/O — Linux: `/proc/PID/io`, macOS: `ioreg` or skip
- [ ] Network bytes per process — macOS: `nettop`, Linux: `/proc/PID/net`
- [ ] Sparkline or mini-graph for CPU over time in detail view

### Docker Awareness (moved to v0.2)

---

## Future

- tmux / zellij integration (open in panes/splits)
- Remote monitoring via SSH
- Plugin system for custom tech detectors and actions
- Custom keybinding configuration
- Export port list (JSON, CSV)
- Startup profiles (launch predefined sets of servers)
