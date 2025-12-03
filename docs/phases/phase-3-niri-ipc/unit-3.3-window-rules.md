# Unit 3.3: Window Rules Documentation

> **Phase:** 3 - Niri IPC Integration  
> **Complexity:** Low  
> **Skills:** Documentation, Niri configuration

---

## Objective

Document required niri configuration and test various window scenarios.

---

## Tasks

### 1. Document Required Niri Config

Create user-facing documentation for niri setup.

### 2. Test Various Window Sizes

- Small floating window (launcher mode)
- Full-width tiled window (execution mode)
- Multi-monitor behavior

### 3. Handle Multi-Monitor Scenarios

- Launcher on monitor 1, execute on monitor 2
- Window follows focus
- Consistent behavior across outputs

---

## Niri Configuration

### Recommended Window Rules

Add to `~/.config/niri/config.kdl`:

```kdl
window-rule {
    match app-id="darkwall-drun"
    
    // Start floating and centered
    open-floating true
    
    // Floating size (launcher mode)
    default-column-width { proportion 0.4; }
    
    // Center on screen
    // (niri doesn't have center option yet, use fixed position or let it auto-place)
}

// Optional: specific rules for different states
window-rule {
    match app-id="darkwall-drun" title="darkwall-drun - Executing"
    open-floating false
}
```

### Terminal Configuration

For foot terminal:

```ini
# ~/.config/foot/foot.ini
[main]
app-id=darkwall-drun  # When launched with --app-id
```

Launch command:
```bash
foot --app-id darkwall-drun -e darkwall-drun
```

---

## Window Sizes

### Launcher Mode (Floating)

| Aspect | Recommendation |
|--------|----------------|
| Width | 40-50% of screen |
| Height | 50-60% of screen |
| Position | Centered |

### Execution Mode (Tiled)

| Aspect | Recommendation |
|--------|----------------|
| Width | Full column width |
| Height | Full workspace height |
| Position | Standard tiling |

---

## Multi-Monitor Behavior

### Scenario 1: Single Monitor

Standard behavior, no special handling needed.

### Scenario 2: Multiple Monitors, Same Output

- Launcher appears on focused monitor
- Execution tiles on same monitor
- Re-float returns to same position

### Scenario 3: Focus Follows Mouse

- Launcher appears where mouse is
- Be aware of focus stealing

---

## Troubleshooting Guide

### Window Doesn't Float

1. Check app-id matches: `niri msg windows | grep darkwall`
2. Verify window rule syntax
3. Check niri logs: `journalctl --user -u niri`

### Window Floats But Wrong Size

1. Check `default-column-width` setting
2. Terminal might override size
3. Try explicit size in foot config

### IPC Not Working

1. Check socket: `echo $NIRI_SOCKET`
2. Test manually: `echo '{"Request":"Version"}' | nc -U $NIRI_SOCKET`
3. Check permissions on socket

---

## Example Configurations

### Minimal Setup

```kdl
// ~/.config/niri/config.kdl
window-rule {
    match app-id="darkwall-drun"
    open-floating true
}
```

### Full Setup

```kdl
// ~/.config/niri/config.kdl
window-rule {
    match app-id="darkwall-drun"
    open-floating true
    default-column-width { proportion 0.4; }
}

// Keybind to launch
binds {
    Mod+D { spawn "foot" "--app-id" "darkwall-drun" "-e" "darkwall-drun"; }
}
```

---

## Acceptance Criteria

- [ ] README includes niri setup instructions
- [ ] Window rules documented with examples
- [ ] Multi-monitor tested and documented
- [ ] Troubleshooting guide complete
- [ ] Works with foot, alacritty, kitty terminals

---

## Testing

### Manual Tests

1. Fresh niri config - add rules, verify behavior
2. Test on single monitor
3. Test on dual monitor setup
4. Test with different terminals
5. Test window rule edge cases

### Documentation Review

- [ ] Instructions are clear for new users
- [ ] All config snippets are valid syntax
- [ ] Troubleshooting covers common issues

---

## Related Units

- **Depends on:** Unit 3.1, Unit 3.2
- **Output:** README.md updates, docs/NIRI_SETUP.md
