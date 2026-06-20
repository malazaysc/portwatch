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
├── app.rs            # App state (ports, selection, flags), grouping, refresh logic
├── config.rs         # ~/.config/portwatch/config.toml loading
├── types.rs          # PortEntry, BindAddress, TechInfo, GitInfo, Protocol, DockerInfo
├── actions.rs        # User actions: kill, open browser, copy URL, copy dir
├── git.rs            # Git repo/branch/worktree detection
├── resources.rs      # Per-process CPU/memory (sysinfo) + network I/O (nettop)
├── scanner/
│   ├── mod.rs        # PortScanner trait + platform factory
│   ├── macos.rs      # macOS lsof-based scanner
│   └── linux.rs      # Linux ss-based scanner
├── process/
│   ├── mod.rs        # Platform module declarations
│   ├── macos.rs      # macOS process info (cwd, cmdline, uptime via ps/lsof)
│   └── linux.rs      # Linux process info via /proc
├── detect/
│   ├── mod.rs        # Tech detection orchestrator (priority: cmdline > project files > port)
│   ├── command_line.rs  # Framework/app/runtime detection from process command line
│   ├── npm_package.rs   # Tech from node_modules package.json referenced in cmdline
│   ├── project_files.rs # Framework detection from package.json, Cargo.toml, etc.
│   ├── docker.rs        # Container/compose-project enrichment via `docker ps`
│   └── port_hints.rs    # Fallback port-based heuristics
└── ui/
    ├── mod.rs        # Layout + popup rendering
    ├── port_list.rs  # Port table panel
    ├── detail_view.rs # Detail panel for selected port
    ├── status_bar.rs  # Top status/summary bar
    └── action_bar.rs  # Bottom action hints bar
```

## Conventions

- macOS is the primary target; Linux backends exist (`linux.rs` siblings in scanner/process) but are less tested
- Platform scanners implement the `PortScanner` trait behind `scanner::create_scanner()`; process resolution dispatches per-OS via `cfg`-gated `batch_resolve`
- Tech detection is ordered by confidence: command line > npm package > known apps > project files > runtime > port heuristics
- TUI uses ratatui with crossterm backend, synchronous event loop with 200ms poll tick
- Key bindings: arrows for navigation, Home/End; single-letter actions (x=kill, b=browser, c=copy URL, d=copy dir, r=refresh, s/S=sort, /=filter, ?=help, q=quit)
