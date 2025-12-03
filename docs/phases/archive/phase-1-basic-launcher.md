# Phase 1: Basic drun/TUI Launcher

> **Status:** ✅ COMPLETE

---

## Goal

Functional launcher that parses desktop entries and spawns commands.

---

## Deliverables

- [x] Project scaffolding (Cargo.toml, src structure)
- [x] Desktop entry parsing (`desktop_entry.rs`)
- [x] Fuzzy filtering with nucleo
- [x] Basic TUI with ratatui (`ui.rs`)
- [x] Configuration loading (`config.rs`)
- [x] Niri IPC client stub (`niri.rs`)
- [x] App state management (`app.rs`)

---

## Current Limitations

- Commands spawn via `sh -c`, not in-place
- No PTY handling
- Niri IPC untested

---

## Requirements Addressed

| ID | Requirement | Status |
|----|-------------|--------|
| FR-01 | Parse XDG `.desktop` files from configurable directories | ✅ |
| FR-02 | Display entries with name, generic name, comment, categories | ✅ |
| FR-03 | Fuzzy search/filter entries by name, keywords, categories | ✅ |
| FR-04 | Execute selected entry command | ✅ |
| FR-05 | Keyboard navigation (j/k, arrows, Enter, Esc) | ✅ |

---

## Files Created

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI, init, main loop |
| `src/app.rs` | Application state |
| `src/config.rs` | Configuration parsing |
| `src/desktop_entry.rs` | XDG desktop entry parser |
| `src/niri.rs` | Niri IPC client (stub) |
| `src/ui.rs` | Ratatui UI rendering |

---

## Next Phase

→ [Phase 2: In-Place Execution](./phase-2-in-place-execution/README.md)
