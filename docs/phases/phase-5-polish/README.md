# Phase 5: Polish & Features

> **Status:** ⚠️ Partially Implemented

---

## Goal

Production-ready launcher with advanced features.

---

## Overview

This phase adds polish and quality-of-life features: frecency-based sorting, category grouping, optional icons, and theming support. These features make darkwall-drun more pleasant to use daily.

---

## Work Units

| Unit | Name | Complexity | Status | File |
|------|------|------------|--------|------|
| 5.1 | Frecency Sorting | Medium | ⚠️ Code exists, not wired | [unit-5.1-frecency-sorting.md](./unit-5.1-frecency-sorting.md) |
| 5.2 | Categories/Groups | Medium | 🟡 Not started | [unit-5.2-categories.md](./unit-5.2-categories.md) |
| 5.3 | Icons (Optional) | Low | 🟡 Field exists, not used | [unit-5.3-icons.md](./unit-5.3-icons.md) |
| 5.4 | Theming | Low | 🟡 Not started | [unit-5.4-theming.md](./unit-5.4-theming.md) |

---

## Requirements Addressed

| ID | Requirement | Unit |
|----|-------------|------|
| FR-15 | Frecency-based sorting (recently/frequently used first) | 5.1 |
| FR-16 | History persistence | 5.1 |

---

## Estimated Effort

**Total:** 3-4 days

| Unit | Estimate |
|------|----------|
| 5.1 | 1 day |
| 5.2 | 1 day |
| 5.3 | 0.5 day |
| 5.4 | 0.5-1 day |

---

## Files to Create

| File | Purpose |
|------|---------|
| `src/history.rs` | Usage history and frecency |
| `src/theme.rs` | Theming support |

---

## Data Files

| File | Purpose |
|------|---------|
| `~/.local/share/darkwall-drun/history.json` | Usage history |
| `~/.config/darkwall-drun/theme.toml` | Custom theme (optional) |

---

## Recommended Order

Units can be implemented in parallel as they are independent:

1. **Unit 5.1** - Frecency (most impactful)
2. **Unit 5.4** - Theming (quick win)
3. **Unit 5.2** - Categories (nice to have)
4. **Unit 5.3** - Icons (optional, terminal-dependent)

---

## Previous Phase

← [Phase 4: Advanced Metadata](../phase-4-advanced-metadata/README.md)

## Next Phase

This is the final planned phase. Future enhancements could include:
- Plugin system
- Custom actions per entry
- Clipboard integration
- Calculator mode
