# Custom Desktop Entry Fields

> TEAM_000: Phase 4, Unit 4.4 - Custom Desktop Entry Fields

darkwall-drun supports custom `X-Darkwall*` fields in `.desktop` files to control execution behavior.

---

## Available Fields

### X-DarkwallTerminalMode

**Type:** enum  
**Values:** `oneshot`, `interactive`, `tui`, `long-running`  
**Default:** Inferred from command

Controls how the command is executed:

| Value | Behavior |
|-------|----------|
| `oneshot` | Quick command, capture output, stay floating |
| `interactive` | Needs input, partial capture, unfloat window |
| `tui` | Full screen app, no capture, terminal handover |
| `long-running` | Server/watch, capture output, unfloat window |

### X-DarkwallKeepOutput

**Type:** boolean  
**Values:** `true`, `false`  
**Default:** Based on terminal mode

Whether to preserve output after the command exits:

- `true`: Keep last N lines visible above launcher
- `false`: Clear screen immediately after exit

### X-DarkwallUnfloatOnRun

**Type:** boolean  
**Values:** `true`, `false`  
**Default:** Based on terminal mode

Whether to unfloat the window when the command starts:

- `true`: Window tiles with other windows during execution
- `false`: Window stays floating (useful for quick lookups)

### X-DarkwallPreserveLines

**Type:** integer  
**Default:** From config (`preserve_output_lines`)

Number of output lines to preserve when returning to launcher.
Overrides the global config for this specific entry.

---

## Example Desktop Entry

```ini
[Desktop Entry]
Name=My TUI App
Comment=A custom TUI application
Exec=my-tui-app
Type=Application
Terminal=true
Categories=Utility;

# Darkwall-specific fields
X-DarkwallTerminalMode=tui
X-DarkwallKeepOutput=false
X-DarkwallUnfloatOnRun=true
```

---

## Common Use Cases

### TUI Application (htop, btop, vim)

```ini
X-DarkwallTerminalMode=tui
X-DarkwallKeepOutput=false
X-DarkwallUnfloatOnRun=true
```

### Quick Lookup Command

```ini
X-DarkwallTerminalMode=oneshot
X-DarkwallKeepOutput=true
X-DarkwallUnfloatOnRun=false
```

### Interactive REPL (python, node)

```ini
X-DarkwallTerminalMode=interactive
X-DarkwallKeepOutput=true
X-DarkwallUnfloatOnRun=true
```

### Long-Running Server

```ini
X-DarkwallTerminalMode=long-running
X-DarkwallKeepOutput=true
X-DarkwallUnfloatOnRun=true
X-DarkwallPreserveLines=50
```

---

## Automatic Detection

If no `X-DarkwallTerminalMode` is specified, darkwall-drun infers the mode from the command:

### TUI Apps (detected automatically)
- htop, btop, top, atop
- vim, nvim, nano, emacs
- less, more, man
- mc, ranger, nnn, lf
- tmux, screen
- lazygit, tig, gitui

### Interactive Apps (detected automatically)
- bash, zsh, fish, sh
- python, python3, ipython
- node, deno, bun
- ruby, irb
- sqlite3, psql, mysql

### Long-Running Patterns (detected automatically)
- Commands containing `watch `
- Commands containing `tail -f`
- Commands containing `journalctl -f`

### Default
All other commands are treated as `oneshot`.

---

## NixOS Integration

For NixOS users, you can define custom fields in your desktop entry definitions:

```nix
{
  xdg.desktopEntries.my-app = {
    name = "My App";
    exec = "my-app";
    terminal = true;
    settings = {
      "X-DarkwallTerminalMode" = "tui";
      "X-DarkwallKeepOutput" = "false";
    };
  };
}
```

---

## See Also

- [Configuration Guide](./GETTING_STARTED.md)
- [Niri Setup](./NIRI_SETUP.md)
