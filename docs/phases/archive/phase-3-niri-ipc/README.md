# Phase 3: Niri IPC Integration

> **Status:** ✅ COMPLETE (TEAM_000)

---

## Goal

Seamless window state transitions via Niri IPC.

---

## Overview

This phase integrates darkwall-drun with the Niri compositor, enabling automatic window floating/unfloating based on execution state. When idle (launcher mode), the window floats. When executing commands, it unfloats to tile with other windows.

---

## Work Units

| Unit | Name | Complexity | File |
|------|------|------------|------|
| 3.1 | IPC Protocol Implementation | Low | [unit-3.1-ipc-protocol.md](./unit-3.1-ipc-protocol.md) |
| 3.2 | Window State Management | Low | [unit-3.2-window-state.md](./unit-3.2-window-state.md) |
| 3.3 | Window Rules Documentation | Low | [unit-3.3-window-rules.md](./unit-3.3-window-rules.md) |

---

## Requirements Addressed

| ID | Requirement | Unit |
|----|-------------|------|
| FR-10 | Niri IPC: unfloat window on command start | 3.2 |
| FR-11 | Niri IPC: re-float window on command end | 3.2 |
| NFR-03 | Works without niri (graceful degradation) | 3.1 |

---

## Estimated Effort

**Total:** 1-2 days

| Unit | Estimate |
|------|----------|
| 3.1 | 0.5 day |
| 3.2 | 0.5 day |
| 3.3 | 0.5 day |

---

## Prerequisites

- Niri compositor running
- `$NIRI_SOCKET` environment variable set
- Window rules configured in niri config

---

## Recommended Order

1. **Unit 3.1** - IPC Protocol (foundation)
2. **Unit 3.2** - Window State (depends on 3.1)
3. **Unit 3.3** - Documentation (parallel)

---

## Previous Phase

← [Phase 2: In-Place Execution](../phase-2-in-place-execution/README.md)

## Next Phase

→ [Phase 4: Advanced Metadata](../phase-4-advanced-metadata/README.md)
