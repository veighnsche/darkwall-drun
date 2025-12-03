# Known Issues

> **Last Updated:** 2024-12-03  
> **Status:** Acceptable regressions for current release

---

## Icon System (Unit 5.3)

### 1. Black Background on Icons
**Severity:** Low (cosmetic)  
**Description:** Icons with transparency display with a black background instead of transparent.  
**Root Cause:** Terminal graphics protocols (Kitty/Sixel/iTerm2) don't always handle alpha channel correctly. The image is rendered with transparency preserved, but the terminal composites it onto black.  
**Workaround:** None currently. Would require pre-compositing icons onto the terminal background color.  
**Future Fix:** 
- Detect terminal background color
- Pre-composite RGBA images onto background before sending to protocol
- Or use terminal-specific transparency flags if available

### 2. Icons Don't Fill Allocated Space
**Severity:** Low (cosmetic)  
**Description:** Icons appear smaller than the allocated cell space in some cases.  
**Root Cause:** `ratatui-image` scales to fit but doesn't stretch. Icon aspect ratios vary.  
**Workaround:** Icons are rendered at 128px and scaled down by the terminal.  
**Future Fix:** Consider padding icons to square before rendering.

### 3. Some Icons Not Found
**Severity:** Low  
**Description:** Some applications show no icon even when icon theme has them.  
**Root Cause:** 
- Icon name in `.desktop` file doesn't match theme's icon name
- Some themes use different naming conventions
- Symlinks in icon themes may not resolve correctly in all cases  
**Workaround:** Falls back gracefully (no icon shown).  
**Future Fix:** 
- Parse theme's `index.theme` for proper inheritance
- Add icon name aliasing/mapping
- Search more subdirectory patterns

### 4. SVG Rendering Quality
**Severity:** Very Low  
**Description:** Some complex SVGs may not render perfectly.  
**Root Cause:** `resvg` doesn't support 100% of SVG spec (filters, some gradients).  
**Workaround:** Most icon theme SVGs are simple and render correctly.  
**Future Fix:** None needed unless specific icons are problematic.

---

## Performance

### 5. Initial Icon Loading Delay
**Severity:** Low  
**Description:** Icons load progressively (one per frame) which can take a few seconds for all visible icons.  
**Root Cause:** Intentional - prevents blocking the UI during icon loading.  
**Workaround:** This IS the workaround for the original blocking issue.  
**Future Fix:** 
- Background thread for icon loading
- Icon cache persistence across sessions
- Preload icons for frecent entries

---

## UI/Layout

### 6. Single Column Layout
**Severity:** Medium (feature gap)  
**Description:** Currently only single-column list layout, not the 2-column grid shown in rofi.  
**Root Cause:** Not yet implemented.  
**Future Fix:** Unit 5.4.1 (Grid Layout) - NEXT PRIORITY

---

## Terminal Compatibility

### 7. Graphics Protocol Detection
**Severity:** Low  
**Description:** Some terminals may not be detected correctly for graphics support.  
**Root Cause:** `ratatui-image` queries terminal capabilities which may timeout or fail.  
**Workaround:** Falls back to no icons (graceful degradation).  
**Future Fix:** Add manual protocol override in config.

---

## Notes for Future Teams

1. **Icon issues are cosmetic** - the launcher is fully functional without icons
2. **Don't add emoji fallback** - per user requirement, it's graphics or nothing
3. **Focus on Unit 5.4** - theming/grid layout is the next priority
4. **Test in Kitty terminal** - that's the primary target for graphics support
