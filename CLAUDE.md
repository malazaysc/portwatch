# portwatch

Terminal UI tool for monitoring local dev server ports. Rust + ratatui.

## Build & Run

```bash
cargo build          # debug build
cargo run            # run in debug mode
cargo build --release  # release build
```

## Project Structure

```
src/
├── main.rs           # Entry point, terminal setup, event loop, key handling
├── app.rs            # App state (ports, selection, flags), refresh logic
├── types.rs          # PortEntry, BindAddress, TechInfo, GitInfo, Protocol
├── actions.rs        # User actions: kill, open browser, open folder, copy URL
├── git.rs            # Git repo/branch/worktree detection
├── scanner/
│   ├── mod.rs        # PortScanner trait + factory
│   └── macos.rs      # macOS lsof-based scanner
├── process/
│   ├── mod.rs        # ProcessResolver trait + factory
│   └── macos.rs      # macOS process info (cwd, cmdline, uptime via ps/lsof)
├── detect/
│   ├── mod.rs        # Tech detection orchestrator (priority: cmdline > project files > port)
│   ├── command_line.rs  # Framework detection from process command line
│   ├── project_files.rs # Framework detection from package.json, Cargo.toml, etc.
│   └── port_hints.rs    # Fallback port-based heuristics
└── ui/
    ├── mod.rs        # Layout + popup rendering
    ├── port_list.rs  # Port table panel
    ├── detail_view.rs # Detail panel for selected port
    └── action_bar.rs  # Bottom action hints bar
```

## Conventions

- macOS is the primary target; Linux support planned (add `linux.rs` siblings to scanner/process modules)
- Platform backends implement traits (`PortScanner`, `ProcessResolver`) behind factory functions
- Tech detection is ordered by confidence: command line > project files > port heuristics
- TUI uses ratatui with crossterm backend, synchronous event loop with 2s poll tick
- Key bindings: vim-style (j/k) + arrows; single-letter actions (x=kill, b=browser, f=folder, c=copy)
