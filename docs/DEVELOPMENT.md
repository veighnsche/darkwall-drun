# darkwall-tui Development Masterplan

> **Status:** Phase 1 scaffolding complete. Ready for team development.

---

## Project Vision

Replace rofi with a TUI-based launcher that:
1. Runs inside a floating terminal window
2. Executes commands in-place (no new windows)
3. Seamlessly transitions between launcher and execution modes via niri IPC
4. Returns to launcher after command completion (preserving output)

---

## Requirements

### Functional Requirements

| ID | Requirement | Priority | Phase |
|----|-------------|----------|-------|
| FR-01 | Parse XDG `.desktop` files from configurable directories | Must | 1 |
| FR-02 | Display entries with name, generic name, comment, categories | Must | 1 |
| FR-03 | Fuzzy search/filter entries by name, keywords, categories | Must | 1 |
| FR-04 | Execute selected entry command | Must | 1 |
| FR-05 | Keyboard navigation (j/k, arrows, Enter, Esc) | Must | 1 |
| FR-06 | Execute commands in same terminal (not spawn new) | Must | 2 |
| FR-07 | Capture and display command output | Should | 2 |
| FR-08 | Return to launcher after command exits | Must | 2 |
| FR-09 | Preserve N lines of output when returning | Should | 2 |
| FR-10 | Niri IPC: unfloat window on command start | Must | 3 |
| FR-11 | Niri IPC: re-float window on command end | Must | 3 |
| FR-12 | Detect terminal mode from desktop entry metadata | Should | 4 |
| FR-13 | Handle interactive TUIs (btop, htop) properly | Should | 4 |
| FR-14 | Detect SSH commands and show appropriate UI | Could | 4 |
| FR-15 | Frecency-based sorting (recently/frequently used first) | Could | 5 |
| FR-16 | History persistence | Could | 5 |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | Startup time < 100ms | Must |
| NFR-02 | Memory usage < 50MB | Should |
| NFR-03 | Works without niri (graceful degradation) | Must |
| NFR-04 | Single binary, no runtime dependencies | Should |
| NFR-05 | Configuration via TOML file | Must |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        darkwall-tui                          │
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

## Development Phases

### Phase 1: Basic TUI Launcher ✅ COMPLETE

**Goal:** Functional launcher that parses desktop entries and spawns commands.

**Deliverables:**
- [x] Project scaffolding (Cargo.toml, src structure)
- [x] Desktop entry parsing (`desktop_entry.rs`)
- [x] Fuzzy filtering with nucleo
- [x] Basic TUI with ratatui (`ui.rs`)
- [x] Configuration loading (`config.rs`)
- [x] Niri IPC client stub (`niri.rs`)
- [x] App state management (`app.rs`)

**Current Limitations:**
- Commands spawn via `sh -c`, not in-place
- No PTY handling
- Niri IPC untested

---

### Phase 2: In-Place Execution

**Goal:** Execute commands within the same terminal, capture output.

**Work Units:**

#### Unit 2.1: PTY Allocation
- Create `src/pty.rs` module
- Allocate PTY for child processes
- Handle terminal resize (SIGWINCH)
- **Crates:** `portable-pty` or `pty-process`

#### Unit 2.2: Output Capture
- Stream stdout/stderr to buffer
- Display output in TUI (scrollable)
- Handle ANSI escape codes

#### Unit 2.3: Return to Launcher
- Detect command exit
- Show exit code
- Preserve last N lines of output
- Re-render launcher below output

#### Unit 2.4: Interactive Mode Detection
- Detect if command needs raw terminal (TUI apps)
- Hand over full terminal control
- Reclaim after exit

**Estimated Effort:** 3-4 days

---

### Phase 3: Niri IPC Integration

**Goal:** Seamless window state transitions.

**Work Units:**

#### Unit 3.1: IPC Protocol Implementation
- Parse niri JSON-RPC responses properly
- Handle connection errors gracefully
- Add reconnection logic

#### Unit 3.2: Window State Management
- `SetWindowFloating` on idle
- `SetWindowFloating false` on execute
- Query current window state

#### Unit 3.3: Window Rules Documentation
- Document required niri config
- Test with various window sizes
- Handle multi-monitor scenarios

**Estimated Effort:** 1-2 days

---

### Phase 4: Advanced Metadata

**Goal:** Intelligent behavior based on entry metadata.

**Work Units:**

#### Unit 4.1: Terminal Mode Schema
- Define `terminalMode` enum: `oneshot`, `interactive`, `tui`, `long-running`
- Parse from desktop entry `X-DarkwallTerminalMode` field
- Infer from command patterns if not specified

#### Unit 4.2: SSH Detection
- Parse command for `ssh` prefix
- Show "Connecting to X..." spinner
- Handle connection failures

#### Unit 4.3: Output Preservation Logic
- `keepOutput` per-entry setting
- Clear screen for TUI apps
- Preserve for oneshot commands

#### Unit 4.4: Custom Desktop Entry Fields
- `X-DarkwallTerminalMode`
- `X-DarkwallKeepOutput`
- `X-DarkwallUnfloatOnRun`
- Document in README

**Estimated Effort:** 2-3 days

---

### Phase 5: Polish & Features

**Goal:** Production-ready launcher.

**Work Units:**

#### Unit 5.1: Frecency Sorting
- Track usage count per entry
- Track last-used timestamp
- Frecency algorithm: `frequency * recency_weight`
- Persist to `~/.local/share/darkwall-tui/history.json`

#### Unit 5.2: Categories/Groups
- Group entries by category
- Collapsible sections in TUI
- Category filter shortcuts

#### Unit 5.3: Icons (Optional)
- Load icons via `freedesktop-icons`
- Render in TUI (if terminal supports)
- Fallback to text indicators

#### Unit 5.4: Theming
- Color scheme configuration
- Match system theme (dark/light)
- Border styles

**Estimated Effort:** 3-4 days

---

## File Structure

```
darkwall-tui/
├── Cargo.toml
├── README.md
├── config.example.toml
├── docs/
│   └── DEVELOPMENT.md          # This file
├── src/
│   ├── main.rs                 # CLI, init, main loop
│   ├── app.rs                  # Application state
│   ├── config.rs               # Configuration parsing
│   ├── desktop_entry.rs        # XDG desktop entry parser
│   ├── niri.rs                 # Niri IPC client
│   ├── ui.rs                   # Ratatui UI rendering
│   ├── pty.rs                  # [Phase 2] PTY handling
│   ├── executor.rs             # [Phase 2] Command execution
│   └── history.rs              # [Phase 5] Usage history
└── tests/
    ├── desktop_entry_test.rs
    └── fixtures/
        └── *.desktop           # Test desktop files
```

---

## Testing Strategy

### Unit Tests
- Desktop entry parsing (various formats)
- Fuzzy matching accuracy
- Config loading (defaults, overrides)

### Integration Tests
- Full TUI render cycle
- Niri IPC (mock socket)
- Command execution

### Manual Testing
- Real desktop entries
- Various terminal emulators (foot, alacritty, kitty)
- Niri window transitions

---

## Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `ratatui` | TUI framework | 0.29 |
| `crossterm` | Terminal backend | 0.28 |
| `freedesktop-desktop-entry` | Parse .desktop files | 0.7 |
| `nucleo-matcher` | Fuzzy matching | 0.3 |
| `tokio` | Async runtime | 1.x |
| `toml` | Config parsing | 0.8 |
| `serde` | Serialization | 1.x |
| `clap` | CLI parsing | 4.x |
| `anyhow` | Error handling | 1.x |
| `tracing` | Logging | 0.1 |

### Future Dependencies (Phase 2+)
| Crate | Purpose |
|-------|---------|
| `portable-pty` | PTY allocation |
| `vte` | ANSI escape parsing |

---

## NixOS Integration

Package will be added to NixOS config:

```nix
# pkgs/darkwall-tui.nix
{ lib, rustPlatform, ... }:

rustPlatform.buildRustPackage {
  pname = "darkwall-tui";
  version = "0.1.0";
  src = /home/vince/Projects/darkwall-tui;
  cargoLock.lockFile = ./Cargo.lock;
  
  meta = {
    description = "TUI application launcher with niri integration";
    license = lib.licenses.mit;
  };
}
```

Desktop entry schema extensions in `lib/desktop-entries.nix`:

```nix
terminalMode = lib.mkOption {
  type = types.enum [ "oneshot" "interactive" "tui" "long-running" ];
  default = "oneshot";
};

keepOutput = lib.mkOption {
  type = types.bool;
  default = true;
};

unfloatOnRun = lib.mkOption {
  type = types.nullOr types.bool;
  default = null;  # Auto based on terminalMode
};
```

---

## Open Questions

1. **PTY vs raw exec:** For TUI apps like btop, do we:
   - Allocate PTY and proxy I/O? (complex)
   - Just exec and reclaim terminal after? (simpler)

2. **Multiple instances:** Should we allow multiple darkwall-tui instances?
   - Single instance with IPC?
   - Multiple independent instances?

3. **Daemon mode:** Should there be a persistent daemon?
   - Faster startup (already loaded entries)
   - Socket for triggering from keybind

4. **Wayland-only:** Do we care about X11 support?
   - Niri is Wayland-only
   - Could support other compositors (sway, hyprland)

---

## Team Assignment Guide

| Phase | Complexity | Skills Needed |
|-------|------------|---------------|
| 2.1 PTY | High | Unix systems, PTY internals |
| 2.2 Output | Medium | Async Rust, TUI |
| 2.3 Return | Medium | State management |
| 2.4 Interactive | High | Terminal emulation |
| 3.x Niri | Low | JSON-RPC, Unix sockets |
| 4.x Metadata | Medium | Schema design |
| 5.x Polish | Low-Medium | UX, persistence |

---

## Getting Started (For New Teams)

1. Clone and build:
   ```bash
   cd /home/vince/Projects/darkwall-tui
   cargo build
   ```

2. Run with debug logging:
   ```bash
   RUST_LOG=debug cargo run
   ```

3. Test with foot:
   ```bash
   foot --app-id darkwall-tui -e cargo run
   ```

4. Check niri socket:
   ```bash
   echo $NIRI_SOCKET
   # or
   ls $XDG_RUNTIME_DIR/niri*
   ```

5. Pick a work unit from Phase 2 or 3 and create a branch.

---

## References

- [ratatui book](https://ratatui.rs/introduction.html)
- [niri IPC wiki](https://github.com/YaLTeR/niri/wiki/IPC)
- [XDG Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [portable-pty docs](https://docs.rs/portable-pty/latest/portable_pty/)
