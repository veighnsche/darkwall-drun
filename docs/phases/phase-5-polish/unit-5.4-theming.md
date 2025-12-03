# Unit 5.4: Theming (Masterplan)

> **Phase:** 5 - Polish & Features  
> **Complexity:** Medium (split into sub-units)  
> **Skills:** Configuration, UI styling, Layout  
> **Status:** ✅ COMPLETE - Integrated by TEAM_004

---

## Overview

This unit implements the visual theming system for DRUN, matching the rofi-style appearance with a 2-column grid layout, multi-line entry cards, and customizable colors.

**Reference:** See `REQUIREMENTS.md` → Theming Requirements for the full specification.

---

## Target Appearance

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ❯ search query                                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────┐  ┌─────────────────────────────┐            │
│ │ 🦊 Firefox                  │  │ 📁 Files                    │            │
│ │    Web Browser              │  │    File Manager             │            │
│ │    Browse the web           │  │    Access and organize      │            │
│ │    Network;WebBrowser       │  │    System;FileManager       │            │
│ └─────────────────────────────┘  └─────────────────────────────┘            │
│ ┌─────────────────────────────┐  ┌─────────────────────────────┐            │
│ │ 💻 Kitty                    │  │ ⚙️ Settings                 │            │
│ │    Terminal Emulator        │  │    System Settings          │            │
│ │    GPU-accelerated terminal │  │    Configure your system    │            │
│ │    System;TerminalEmulator  │  │    Settings;System          │            │
│ └─────────────────────────────┘  └─────────────────────────────┘            │
│ ... (5 rows × 2 columns = 10 visible entries)                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Sub-Units

| Unit | Name | Description | Complexity |
|------|------|-------------|------------|
| 5.4.1 | Grid Layout | 2-column layout with configurable rows | Low |
| 5.4.2 | Entry Cards | Multi-line card rendering with icon | Medium |
| 5.4.3 | Color System | Theme struct, presets, hex parsing | Low |
| 5.4.4 | Configuration | TOML config loading, validation | Low |

---

## Implementation Order

```
5.4.1 Grid Layout
    ↓
5.4.2 Entry Cards (depends on grid)
    ↓
5.4.3 Color System (independent, can parallel with 5.4.1)
    ↓
5.4.4 Configuration (depends on 5.4.3)
```

---

## Key Data Structures

### Theme

```rust
#[derive(Deserialize, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub accent: Color,           // Border, highlights
    pub dimmed: Color,           // Comments, categories
    pub dimmed_alt: Color,       // Even more dimmed (categories)
    pub search_highlight: Color,
    pub exit_success: Color,
    pub exit_failure: Color,
}
```

### Layout Config

```rust
#[derive(Deserialize, Clone)]
pub struct LayoutConfig {
    pub columns: u16,        // Default: 2
    pub visible_rows: u16,   // Default: 5
}
```

### Entry Display Config

```rust
#[derive(Deserialize, Clone)]
pub struct EntryDisplayConfig {
    pub show_generic: bool,     // Default: true
    pub show_comment: bool,     // Default: true
    pub show_categories: bool,  // Default: true
}
```

---

## Default Theme (Darkwall)

Based on rofi config analysis:

```rust
impl Theme {
    pub fn darkwall() -> Self {
        Self {
            background: Color::Rgb(13, 17, 22),       // #0d1116
            foreground: Color::Rgb(229, 234, 241),    // #e5eaf1
            selection_bg: Color::Rgb(20, 28, 42),     // #141c2a
            selection_fg: Color::Rgb(229, 234, 241),  // #e5eaf1
            accent: Color::Rgb(180, 83, 9),           // #b45309 (amber)
            dimmed: Color::Rgb(156, 163, 175),        // #9ca3af
            dimmed_alt: Color::Rgb(107, 114, 128),    // #6b7280
            search_highlight: Color::Rgb(180, 83, 9), // #b45309
            exit_success: Color::Rgb(34, 197, 94),    // green
            exit_failure: Color::Rgb(239, 68, 68),    // red
        }
    }
}
```

---

## Acceptance Criteria (Overall)

- [ ] 2-column grid layout displays correctly
- [ ] Each entry shows as a card with 4 lines
- [ ] Icons display (Nerd Font glyphs)
- [ ] Selection highlights entire card
- [ ] Colors match darkwall theme by default
- [ ] Theme configurable via TOML
- [ ] Graceful degradation on limited terminals

---

## TUI Considerations

### Character Cell Sizing

| Element | Size (cells) |
|---------|--------------|
| Entry height | 4-5 lines |
| Icon width | 2-3 chars (Nerd Font) |
| Card padding | 1 char horizontal, 0-1 vertical |
| Column gap | 2 chars |

### Terminal Compatibility

| Terminal | Support |
|----------|---------|
| True color (24-bit) | Full theme colors |
| 256 color | Approximate colors |
| 16 color | Basic fallback |

---

## Files to Create/Modify

| File | Purpose |
|------|---------|
| `src/ui/theme.rs` | Theme struct, presets, color parsing |
| `src/ui/layout.rs` | Grid layout logic |
| `src/ui/entry_card.rs` | Multi-line entry rendering |
| `src/config.rs` | Add theme/layout config sections |

---

## Related Units

- **Depends on:** Unit 5.3 (Icons)
- **Related:** Core UI rendering
