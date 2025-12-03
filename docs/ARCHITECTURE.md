# Architecture

> darkwall-drun system architecture and module overview.

---

## System Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        darkwall-drun                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   main.rs   │  │   app.rs    │  │      ui.rs          │  │
│  │  (CLI/init) │  │   (state)   │  │  (ratatui views)    │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────────▼──────────┐  │
│  │  config.rs  │  │ executor.rs │  │   desktop_entry.rs  │  │
│  │  (TOML)     │  │ (PTY/spawn) │  │   (XDG parsing)     │  │
│  └─────────────┘  └──────┬──────┘  └─────────────────────┘  │
│                          │                                   │
│                   ┌──────▼──────┐                            │
│                   │   niri.rs   │                            │
│                   │   (IPC)     │                            │
│                   └─────────────┘                            │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Responsibilities

### `main.rs`
- CLI argument parsing (clap)
- Application initialization
- Main event loop
- Signal handling

### `app.rs`
- Application state management
- Mode transitions (Launcher ↔ Executing)
- Input handling dispatch
- Coordination between modules

### `ui.rs`
- Ratatui widget rendering
- Layout calculations
- Visual state representation
- Scroll management

### `config.rs`
- TOML configuration parsing
- Default values
- Configuration validation
- Hot reload (future)

### `desktop_entry.rs`
- XDG .desktop file parsing
- Entry filtering and sorting
- Custom field extraction
- Category handling

### `niri.rs`
- Niri IPC client
- Window state management
- Graceful degradation
- Reconnection logic

### `pty.rs` (Phase 2)
- PTY allocation
- Process spawning
- Terminal resize handling
- I/O streaming

### `executor.rs` (Phase 2)
- Command execution
- Output capture
- Exit status handling
- Mode detection

### `history.rs` (Phase 5)
- Usage tracking
- Frecency calculation
- Persistence

---

## Data Flow

### Launcher Mode

```
User Input → app.rs → desktop_entry.rs (filter)
                   → ui.rs (render)
                   → Terminal
```

### Execution Mode

```
User Select → app.rs → niri.rs (unfloat)
                    → pty.rs (spawn)
                    → executor.rs (run)
                    → ui.rs (output)
                    → niri.rs (re-float)
                    → app.rs (return)
```

---

## State Machine

```
                    ┌─────────────┐
                    │   Startup   │
                    └──────┬──────┘
                           │
                           ▼
              ┌────────────────────────┐
              │       Launcher         │◄────────┐
              │  (floating, searching) │         │
              └───────────┬────────────┘         │
                          │ Enter                │
                          ▼                      │
              ┌────────────────────────┐         │
              │      Executing         │         │
              │  (unfloated, running)  │         │
              └───────────┬────────────┘         │
                          │ Exit                 │
                          ▼                      │
              ┌────────────────────────┐         │
              │    Post-Execution      │─────────┘
              │  (showing output)      │
              └────────────────────────┘
```

---

## External Interfaces

### Niri IPC
- **Socket:** `$NIRI_SOCKET` or `$XDG_RUNTIME_DIR/niri-socket`
- **Protocol:** JSON-RPC over Unix socket
- **Commands:** `SetWindowFloating`, `FocusedWindow`

### XDG Desktop Entries
- **Locations:** `$XDG_DATA_DIRS/applications/`, `~/.local/share/applications/`
- **Format:** INI-style .desktop files
- **Spec:** freedesktop.org Desktop Entry Specification

### Configuration
- **Location:** `~/.config/darkwall-drun/config.toml`
- **Format:** TOML

### History
- **Location:** `~/.local/share/darkwall-drun/history.json`
- **Format:** JSON

---

## Error Handling Strategy

| Layer | Strategy |
|-------|----------|
| Config | Use defaults, log warnings |
| Desktop entries | Skip invalid, continue |
| Niri IPC | Graceful degradation |
| PTY | Propagate errors, cleanup |
| UI | Never panic, show error state |

---

## Threading Model

- **Main thread:** Event loop, UI rendering
- **Async runtime:** Tokio for I/O operations
- **PTY I/O:** Async read/write tasks

---

## Future Considerations

- **Plugin system:** Dynamic loading of extensions
- **D-Bus integration:** System notifications, portal access
- **Wayland protocols:** Direct compositor integration
