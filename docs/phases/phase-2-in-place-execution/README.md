# Phase 2: In-Place Execution

> **Status:** 🔲 Not Started

---

## Goal

Execute commands within the same terminal, capture output.

---

## Overview

This phase transforms darkwall-drun from a simple launcher into a terminal-integrated execution environment. Instead of spawning new terminal windows, commands run in-place with output captured and displayed.

---

## Work Units

| Unit | Name | Complexity | File |
|------|------|------------|------|
| 2.1 | PTY Allocation | High | [unit-2.1-pty-allocation.md](./unit-2.1-pty-allocation.md) |
| 2.2 | Output Capture | Medium | [unit-2.2-output-capture.md](./unit-2.2-output-capture.md) |
| 2.3 | Return to Launcher | Medium | [unit-2.3-return-to-launcher.md](./unit-2.3-return-to-launcher.md) |
| 2.4 | Interactive Mode Detection | High | [unit-2.4-interactive-mode.md](./unit-2.4-interactive-mode.md) |

---

## Requirements Addressed

| ID | Requirement | Unit |
|----|-------------|------|
| FR-06 | Execute commands in same terminal (not spawn new) | 2.1 |
| FR-07 | Capture and display command output | 2.2 |
| FR-08 | Return to launcher after command exits | 2.3 |
| FR-09 | Preserve N lines of output when returning | 2.3 |

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `portable-pty` | PTY allocation |
| `vte` | ANSI escape parsing |

---

## Estimated Effort

**Total:** 3-4 days

| Unit | Estimate |
|------|----------|
| 2.1 | 1-1.5 days |
| 2.2 | 0.5-1 day |
| 2.3 | 0.5-1 day |
| 2.4 | 1 day |

---

## Files to Create

| File | Purpose |
|------|---------|
| `src/pty.rs` | PTY handling |
| `src/executor.rs` | Command execution |

---

## Recommended Order

1. **Unit 2.1** - PTY Allocation (foundation)
2. **Unit 2.2** - Output Capture (depends on 2.1)
3. **Unit 2.3** - Return to Launcher (depends on 2.2)
4. **Unit 2.4** - Interactive Mode (can be parallel with 2.3)

---

## Previous Phase

← [Phase 1: Basic Launcher](../phase-1-basic-launcher.md)

## Next Phase

→ [Phase 3: Niri IPC Integration](../phase-3-niri-ipc/README.md)
