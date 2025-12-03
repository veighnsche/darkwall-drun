# darkwall-drun Development Masterplan

> **Status:** Phase 1 scaffolding complete. Ready for team development.

---

## Quick Links

- [Getting Started](./GETTING_STARTED.md) - New developer guide
- [Architecture](./ARCHITECTURE.md) - System design
- [Testing](./TESTING.md) - Test strategy
- [Dependencies](./DEPENDENCIES.md) - Crate information

---

## Project Vision

Replace rofi with a TUI-based launcher that:
1. Runs inside a floating terminal window
2. Executes commands in-place (no new windows)
3. Seamlessly transitions between launcher and execution modes via niri IPC
4. Returns to launcher after command completion (preserving output)

---

## Development Phases

| Phase | Name | Status | Docs |
|-------|------|--------|------|
| 1 | Basic drun/TUI Launcher | ✅ Complete | [phase-1-basic-launcher.md](./phases/phase-1-basic-launcher.md) |
| 2 | In-Place Execution | 🔲 Not Started | [phase-2-in-place-execution/](./phases/phase-2-in-place-execution/README.md) |
| 3 | Niri IPC Integration | 🔲 Not Started | [phase-3-niri-ipc/](./phases/phase-3-niri-ipc/README.md) |
| 4 | Advanced Metadata | 🔲 Not Started | [phase-4-advanced-metadata/](./phases/phase-4-advanced-metadata/README.md) |
| 5 | Polish & Features | 🔲 Not Started | [phase-5-polish/](./phases/phase-5-polish/README.md) |

---

## Work Units Overview

### Phase 2: In-Place Execution (3-4 days)

| Unit | Name | Complexity |
|------|------|------------|
| 2.1 | [PTY Allocation](./phases/phase-2-in-place-execution/unit-2.1-pty-allocation.md) | High |
| 2.2 | [Output Capture](./phases/phase-2-in-place-execution/unit-2.2-output-capture.md) | Medium |
| 2.3 | [Return to Launcher](./phases/phase-2-in-place-execution/unit-2.3-return-to-launcher.md) | Medium |
| 2.4 | [Interactive Mode](./phases/phase-2-in-place-execution/unit-2.4-interactive-mode.md) | High |

### Phase 3: Niri IPC Integration (1-2 days)

| Unit | Name | Complexity |
|------|------|------------|
| 3.1 | [IPC Protocol](./phases/phase-3-niri-ipc/unit-3.1-ipc-protocol.md) | Low |
| 3.2 | [Window State](./phases/phase-3-niri-ipc/unit-3.2-window-state.md) | Low |
| 3.3 | [Window Rules](./phases/phase-3-niri-ipc/unit-3.3-window-rules.md) | Low |

### Phase 4: Advanced Metadata (2-3 days)

| Unit | Name | Complexity |
|------|------|------------|
| 4.1 | [Terminal Mode Schema](./phases/phase-4-advanced-metadata/unit-4.1-terminal-mode-schema.md) | Medium |
| 4.2 | [SSH Detection](./phases/phase-4-advanced-metadata/unit-4.2-ssh-detection.md) | Low |
| 4.3 | [Output Preservation](./phases/phase-4-advanced-metadata/unit-4.3-output-preservation.md) | Medium |
| 4.4 | [Custom Fields](./phases/phase-4-advanced-metadata/unit-4.4-custom-fields.md) | Medium |

### Phase 5: Polish & Features (3-4 days)

| Unit | Name | Complexity |
|------|------|------------|
| 5.1 | [Frecency Sorting](./phases/phase-5-polish/unit-5.1-frecency-sorting.md) | Medium |
| 5.2 | [Categories](./phases/phase-5-polish/unit-5.2-categories.md) | Medium |
| 5.3 | [Icons](./phases/phase-5-polish/unit-5.3-icons.md) | Low |
| 5.4 | [Theming](./phases/phase-5-polish/unit-5.4-theming.md) | Low |

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

## File Structure

```
darkwall-drun/
├── Cargo.toml
├── README.md
├── config.example.toml
├── docs/
│   ├── DEVELOPMENT.md          # This file (index)
│   ├── ARCHITECTURE.md         # System design
│   ├── TESTING.md              # Test strategy
│   ├── DEPENDENCIES.md         # Crate info
│   ├── GETTING_STARTED.md      # New dev guide
│   └── phases/
│       ├── phase-1-basic-launcher.md
│       ├── phase-2-in-place-execution/
│       │   ├── README.md
│       │   ├── unit-2.1-pty-allocation.md
│       │   ├── unit-2.2-output-capture.md
│       │   ├── unit-2.3-return-to-launcher.md
│       │   └── unit-2.4-interactive-mode.md
│       ├── phase-3-niri-ipc/
│       │   ├── README.md
│       │   ├── unit-3.1-ipc-protocol.md
│       │   ├── unit-3.2-window-state.md
│       │   └── unit-3.3-window-rules.md
│       ├── phase-4-advanced-metadata/
│       │   ├── README.md
│       │   ├── unit-4.1-terminal-mode-schema.md
│       │   ├── unit-4.2-ssh-detection.md
│       │   ├── unit-4.3-output-preservation.md
│       │   └── unit-4.4-custom-fields.md
│       └── phase-5-polish/
│           ├── README.md
│           ├── unit-5.1-frecency-sorting.md
│           ├── unit-5.2-categories.md
│           ├── unit-5.3-icons.md
│           └── unit-5.4-theming.md
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

## Open Questions

1. **PTY vs raw exec:** For TUI apps like btop, do we:
   - Allocate PTY and proxy I/O? (complex)
   - Just exec and reclaim terminal after? (simpler)

2. **Multiple instances:** Should we allow multiple darkwall-drun instances?
   - Single instance with IPC?
   - Multiple independent instances?

3. **Daemon mode:** Should there be a persistent daemon?
   - Faster startup (already loaded entries)
   - Socket for triggering from keybind

4. **Wayland-only:** Do we care about X11 support?
   - Niri is Wayland-only
   - Could support other compositors (sway, hyprland)

---

## NixOS Integration

Package will be added to NixOS config:

```nix
# pkgs/darkwall-drun.nix
{ lib, rustPlatform, ... }:

rustPlatform.buildRustPackage {
  pname = "darkwall-drun";
  version = "0.1.0";
  src = /home/vince/Projects/darkwall-drun;
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

## References

- [ratatui book](https://ratatui.rs/introduction.html)
- [niri IPC wiki](https://github.com/YaLTeR/niri/wiki/IPC)
- [XDG Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [portable-pty docs](https://docs.rs/portable-pty/latest/portable_pty/)
