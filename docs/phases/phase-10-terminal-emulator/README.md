# Phase 10: Terminal Emulator Integration

> **Goal:** Transform darkwall-drun's output viewer into a proper terminal emulator capable of running TUI applications, progress bars, and interactive commands correctly.

---

## Executive Summary

Currently, darkwall-drun captures PTY output as raw text and strips ANSI codes. This breaks:
- Progress bars (rsync, wget, etc.)
- TUI applications (htop, vim - currently use handover mode)
- Cursor positioning
- Colors and styling

This phase integrates a proper terminal emulator library to handle all escape sequences correctly.

---

## Crate Comparison

### Option 1: `vte` (Alacritty)

| Aspect | Details |
|--------|---------|
| **What it is** | Low-level escape sequence parser |
| **Used by** | Alacritty, rio, many others |
| **Pros** | Minimal, fast, well-tested, no dependencies |
| **Cons** | Parser only - YOU must implement the terminal state machine |
| **Complexity** | High - requires building screen buffer, cursor tracking, etc. |

### Option 2: `termwiz` (WezTerm)

| Aspect | Details |
|--------|---------|
| **What it is** | Complete terminal toolkit |
| **Used by** | WezTerm, Zellij |
| **Pros** | Full solution: Surface, Cell, escape parsing, rendering |
| **Cons** | Larger dependency, more complex API |
| **Complexity** | Medium - provides building blocks, some assembly required |

### Option 3: `alacritty_terminal`

| Aspect | Details |
|--------|---------|
| **What it is** | Alacritty's full terminal emulator core |
| **Used by** | Alacritty only |
| **Pros** | Battle-tested, complete |
| **Cons** | Tightly coupled to Alacritty, not designed as library |
| **Complexity** | High - extraction from Alacritty codebase |

---

## Recommendation: `termwiz`

**Why termwiz:**

1. **Complete toolkit** - Provides `Surface` (screen buffer), `Cell` (character + attributes), escape sequence parser, and rendering
2. **Active development** - Maintained by WezTerm author, regular updates
3. **Proven in production** - Powers WezTerm and Zellij
4. **Good abstraction level** - Not too low (vte) or too coupled (alacritty_terminal)
5. **Cross-platform** - Works on Linux, macOS, Windows

**You don't need to switch to WezTerm** - termwiz is a standalone crate that works with any terminal. Kitty will work fine.

---

## Phase Structure

This phase is split into 6 units, each building on the previous:

| Unit | Name | Complexity | Description |
|------|------|------------|-------------|
| 10.1 | [Dependencies & Types](./unit-10.1-dependencies.md) | Low | Add termwiz, define types |
| 10.2 | [Terminal Surface](./unit-10.2-surface.md) | Medium | Replace OutputBuffer with termwiz Surface |
| 10.3 | [Escape Sequence Handling](./unit-10.3-escape-sequences.md) | Medium | Parse and apply escape sequences |
| 10.4 | [Rendering](./unit-10.4-rendering.md) | Medium | Render Surface to ratatui |
| 10.5 | [Input Handling](./unit-10.5-input.md) | Low | Forward keyboard/mouse correctly |
| 10.6 | [Post-Migration Cleanup](./unit-10.6-cleanup.md) | Low | Remove legacy code after stabilization |

---

## Success Criteria

After this phase:

- [ ] rsync progress bars update in place (no line spam)
- [ ] Colors and styling preserved
- [ ] Cursor positioning works
- [ ] TUI apps can run embedded (optional handover)
- [ ] Performance comparable to current implementation

---

## Non-Goals (This Phase)

- Sixel/image support (future phase)
- Kitty graphics protocol (future phase)
- Full xterm compatibility (80% is fine)
- Mouse reporting (basic only)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Performance regression | Medium | High | Benchmark before/after, optimize hot paths |
| API complexity | Medium | Medium | Start with minimal Surface usage |
| Breaking existing features | Low | High | Keep TUI handover as fallback |

---

## Timeline Estimate

| Unit | Estimated Effort |
|------|------------------|
| 10.1 | 1-2 hours |
| 10.2 | 3-4 hours |
| 10.3 | 2-3 hours |
| 10.4 | 3-4 hours |
| 10.5 | 1-2 hours |
| 10.6 | 1-2 hours (after stabilization) |
| **Total** | **11-17 hours** |

> **Note:** Unit 10.6 should only be executed after 1+ week of stable usage.

---

## Related Documentation

- [termwiz docs](https://docs.rs/termwiz/latest/termwiz/)
- [WezTerm source](https://github.com/wez/wezterm)
- [VT100 escape sequences](https://vt100.net/docs/vt100-ug/chapter3.html)
- [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code)
