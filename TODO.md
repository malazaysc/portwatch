# TODO — portwatch v0.1

## Project Setup
- [x] Initialize Cargo project with dependencies
- [x] Set up module structure
- [x] Create CLAUDE.md with project conventions

## Core Data Layer
- [x] Define shared types (`PortEntry`, `ProcessInfo`, `TechStack`, etc.)
- [x] Port scanner — macOS backend (`lsof` parsing)
- [x] Process resolver — working directory, command line, start time
- [x] Technology detector — command line patterns
- [x] Technology detector — project file scanning (`package.json`, `Cargo.toml`, etc.)
- [x] Technology detector — port heuristic fallback
- [x] Git/worktree detection
- [x] Network exposure classification (local vs exposed)

## TUI
- [x] App state and event loop (tokio + crossterm)
- [x] Three-panel layout (port list, details, action bar)
- [x] Port list table rendering with columns
- [x] Detail view panel for selected port
- [x] Action bar rendering
- [x] Keyboard navigation (j/k, arrows, enter)
- [x] Selected row highlighting and scrolling
- [x] Color scheme (lazygit-inspired)
- [x] Responsive column visibility by terminal width

## Actions
- [x] Kill process (with confirmation)
- [x] Open in browser
- [x] Open folder in terminal
- [x] Copy URL to clipboard
- [x] Manual refresh

## Polish
- [x] Human-friendly uptime display
- [x] Error handling for permission-denied ports
- [x] Graceful terminal restore on panic/exit
- [x] CLI args (--help, --version, --interval)
