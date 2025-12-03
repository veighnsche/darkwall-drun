# TEAM_003: Theming Preparation

> **Date:** 2024-12-03  
> **Units:** 5.3 (Icons), 5.4 (Theming Prep)  
> **Status:** ✅ Complete

---

## Work Completed

### Unit 5.3: Icons (Completed)
- Implemented custom icon lookup (replaced `freedesktop-icons` crate)
- GTK theme detection from `~/.config/gtk-3.0/settings.ini`
- Full XDG_DATA_DIRS search including NixOS paths
- Theme hierarchy: user theme → parent themes → hicolor
- SVG support via `resvg`/`usvg`/`tiny-skia`
- PNG/other formats via `image` crate
- Non-blocking icon loading (one per frame)
- Icon caching per entry ID

### Unit 5.4: Theming (Prepared)
Created new module structure `src/ui/`:
- `mod.rs` - Module exports
- `theme.rs` - Theme struct with 5 presets (darkwall, catppuccin-mocha, catppuccin-latte, nord, gruvbox)
- `layout.rs` - GridLayout for 2-column display with navigation helpers
- `entry_card.rs` - EntryCard widget for multi-line entry rendering
- `draw.rs` - Existing UI code (moved from src/ui.rs)

---

## Known Issues (Documented)

See `/docs/KNOWN_ISSUES.md` for full list:
1. Black background on icons (terminal protocol limitation)
2. Icons don't fill allocated space (aspect ratio)
3. Some icons not found (naming conventions)
4. Initial icon loading delay (intentional, non-blocking)

---

## Files Changed

### New Files
- `src/ui/mod.rs`
- `src/ui/theme.rs`
- `src/ui/layout.rs`
- `src/ui/entry_card.rs`
- `docs/KNOWN_ISSUES.md`
- `docs/teams/TEAM_003_theming_prep.md`

### Modified Files
- `src/icons.rs` - Complete rewrite for proper icon lookup
- `src/ui.rs` → `src/ui/draw.rs` - Moved to module
- `Cargo.toml` - Added `resvg`, `usvg`, `tiny-skia`, `unicode-width`
- `docs/phases/phase-5-polish/unit-5.3-icons.md` - Marked complete
- `docs/phases/phase-5-polish/unit-5.4*.md` - Added status

---

## For Next Team (Unit 5.4 Integration)

### What's Ready
1. **Theme** (`src/ui/theme.rs`)
   - `Theme` struct with all colors
   - 5 presets: darkwall, catppuccin-mocha, catppuccin-latte, nord, gruvbox
   - `parse_hex_color()` for custom colors
   - Serde integration for config loading
   - 256-color fallback

2. **GridLayout** (`src/ui/layout.rs`)
   - Column-major ordering (rofi-style)
   - Navigation: up/down/left/right/tab/page
   - Pagination helpers
   - Fully tested

3. **EntryCard** (`src/ui/entry_card.rs`)
   - Multi-line card widget
   - Configurable display (generic/comment/categories)
   - Text truncation with ellipsis
   - Selection highlighting

### What Needs To Be Done
1. **Update `draw.rs`**
   - Replace hardcoded colors with `Theme`
   - Replace single-column list with `GridLayout`
   - Replace list items with `EntryCard` widgets

2. **Update `config.rs`**
   - Add `ThemeConfig` section
   - Add `LayoutConfig` section
   - Add preset loading
   - Add color override support

3. **Update `main.rs`**
   - Load theme from config
   - Pass theme to UI
   - Add CLI flags for theme/layout

4. **Update navigation in `app.rs`**
   - Add left/right navigation (column jump)
   - Add Tab/Shift+Tab wrap
   - Add Page Up/Down

---

## Test Results

```
running 36 tests
test icons::tests::test_detect_icon_theme ... ok
test icons::tests::test_get_icon_search_paths ... ok
test icons::tests::test_icon_lookup ... ok
test icons::tests::test_svg_loading ... ok
test ui::theme::tests::test_parse_hex_3 ... ok
test ui::theme::tests::test_parse_hex_6 ... ok
test ui::theme::tests::test_parse_hex_8 ... ok
test ui::theme::tests::test_parse_hex_invalid ... ok
test ui::theme::tests::test_presets ... ok
test ui::layout::tests::test_index_to_position ... ok
test ui::layout::tests::test_navigation ... ok
test ui::layout::tests::test_position_to_index ... ok
test ui::layout::tests::test_tab_wrap ... ok
test ui::layout::tests::test_visible_count ... ok
test ui::layout::tests::test_visible_range ... ok
test ui::entry_card::tests::test_card_height ... ok
test ui::entry_card::tests::test_truncate ... ok
... (all 36 tests pass)
```

---

## Handoff Checklist

- [x] Project builds cleanly
- [x] All tests pass (36 tests)
- [x] Team file created
- [x] Known issues documented
- [x] Next steps documented
- [x] Spec files updated with status
