# NixOS Setup Request for darkwall-drun

> **For:** NixOS AI  
> **Devices:** blep (local), workstation (remote)  
> **Priority:** Development installation with sync

---

## Overview

Set up `darkwall-drun` as a development package on both `blep` and the `workstation`, similar to how `darkwall-model-sync` is configured.

---

## Requirements

### 1. Project Sync (Like darkwall-model-sync)

Sync the project directory to the workstation:
- **Source:** `~/Projects/darkwall-drun` on blep
- **Target:** Equivalent path on workstation
- Use the same sync mechanism as `darkwall-model-sync`

### 2. Nix Package

Create a proper Nix package for `darkwall-drun`:
- Similar structure to `darkwall-model-sync`
- Should build from the flake.nix in the project
- Available on both blep and workstation

### 3. Development Alias

Create an alias `drun` that runs the development version:

```bash
alias drun='cd ~/Projects/darkwall-drun && nix develop . -c cargo run'
```

This should be available in the shell configuration on both devices.

### 4. Niri Window Rules

Add window rules to niri config for `darkwall-drun`:

```kdl
window-rule {
    match app-id="darkwall-drun"
    open-floating true
}

window-rule {
    match app-id="drun"
    open-floating true
}
```

This ensures the launcher opens as a floating window.

### 5. Desktop Entry / Action for Workstation Control

Create desktop entries/actions so I can launch `drun` on the workstation from blep:

```desktop
[Desktop Entry]
Name=DRUN (Workstation)
Comment=Launch DRUN on workstation
Exec=ssh workstation "cd ~/Projects/darkwall-drun && nix develop . -c cargo run"
Type=Application
Categories=Utility;
Icon=application-x-executable
Terminal=true
```

Or integrate with existing workstation action system if there is one.

---

## Summary Checklist

- [x] Sync `darkwall-drun` project to workstation (like darkwall-model-sync)
- [x] Create Nix package for darkwall-drun
- [x] Add `drun` alias for development builds
- [x] Add niri window rules for floating behavior
- [x] Create desktop entry/action to launch drun on workstation remotely

---

## Technical Notes

### App ID
The application identifies as `darkwall-drun` or `drun` depending on how it's launched. The niri rules should match both.

### Dependencies
The project uses a Nix flake (`flake.nix`) with Rust toolchain and these key dependencies:
- ratatui (TUI framework)
- ratatui-image (graphics protocol support)
- resvg/usvg (SVG rendering)
- tokio (async runtime)

### Graphics Protocol
DRUN uses Kitty graphics protocol for icons. This works in kitty, foot, and other supporting terminals. Over SSH, icons are automatically disabled.

---

## Reference

See `darkwall-model-sync` configuration for the sync and package setup pattern to follow.
