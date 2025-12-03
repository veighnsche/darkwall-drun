# Phase 4: Advanced Metadata

> **Status:** 🔲 Not Started

---

## Goal

Intelligent behavior based on desktop entry metadata.

---

## Overview

This phase adds smart behavior detection and custom desktop entry fields. The launcher will automatically determine how to handle different types of applications (TUI apps, SSH connections, oneshot commands) and allow users to customize behavior via extended desktop entry fields.

---

## Work Units

| Unit | Name | Complexity | File |
|------|------|------------|------|
| 4.1 | Terminal Mode Schema | Medium | [unit-4.1-terminal-mode-schema.md](./unit-4.1-terminal-mode-schema.md) |
| 4.2 | SSH Detection | Low | [unit-4.2-ssh-detection.md](./unit-4.2-ssh-detection.md) |
| 4.3 | Output Preservation Logic | Medium | [unit-4.3-output-preservation.md](./unit-4.3-output-preservation.md) |
| 4.4 | Custom Desktop Entry Fields | Medium | [unit-4.4-custom-fields.md](./unit-4.4-custom-fields.md) |

---

## Requirements Addressed

| ID | Requirement | Unit |
|----|-------------|------|
| FR-12 | Detect terminal mode from desktop entry metadata | 4.1 |
| FR-13 | Handle interactive TUIs (btop, htop) properly | 4.1 |
| FR-14 | Detect SSH commands and show appropriate UI | 4.2 |

---

## Estimated Effort

**Total:** 2-3 days

| Unit | Estimate |
|------|----------|
| 4.1 | 0.5-1 day |
| 4.2 | 0.5 day |
| 4.3 | 0.5 day |
| 4.4 | 0.5-1 day |

---

## Custom Fields Overview

| Field | Type | Description |
|-------|------|-------------|
| `X-DarkwallTerminalMode` | enum | `oneshot`, `interactive`, `tui`, `long-running` |
| `X-DarkwallKeepOutput` | bool | Preserve output after command exits |
| `X-DarkwallUnfloatOnRun` | bool | Whether to unfloat window during execution |

---

## Recommended Order

1. **Unit 4.1** - Terminal Mode Schema (foundation)
2. **Unit 4.4** - Custom Fields (depends on 4.1)
3. **Unit 4.2** - SSH Detection (parallel)
4. **Unit 4.3** - Output Preservation (depends on 4.1)

---

## Previous Phase

← [Phase 3: Niri IPC Integration](../phase-3-niri-ipc/README.md)

## Next Phase

→ [Phase 5: Polish & Features](../phase-5-polish/README.md)
