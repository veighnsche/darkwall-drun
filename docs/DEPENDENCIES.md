# Dependencies

> Crate dependencies and their purposes.

---

## Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29 | TUI framework |
| `crossterm` | 0.28 | Terminal backend |
| `freedesktop-desktop-entry` | 0.7 | Parse .desktop files |
| `nucleo-matcher` | 0.3 | Fuzzy matching |
| `tokio` | 1.x | Async runtime |
| `toml` | 0.8 | Config parsing |
| `serde` | 1.x | Serialization |
| `clap` | 4.x | CLI parsing |
| `anyhow` | 1.x | Error handling |
| `tracing` | 0.1 | Logging |

---

## Phase 2+ Dependencies

| Crate | Version | Purpose | Phase |
|-------|---------|---------|-------|
| `portable-pty` | 0.8 | PTY allocation | 2 |
| `vte` | 0.13 | ANSI escape parsing | 2 |

---

## Optional Dependencies

| Crate | Version | Purpose | Feature |
|-------|---------|---------|---------|
| `ratatui-image` | 1.0 | Image rendering | `icons` |
| `image` | 0.25 | Image loading | `icons` |
| `freedesktop-icons` | 0.2 | Icon lookup | `icons` |

---

## Dependency Details

### ratatui

TUI framework for building terminal user interfaces.

```toml
[dependencies]
ratatui = { version = "0.29", default-features = false, features = ["crossterm"] }
```

**Why:** Industry standard for Rust TUIs, excellent documentation, active maintenance.

### crossterm

Cross-platform terminal manipulation.

```toml
[dependencies]
crossterm = "0.28"
```

**Why:** Works on Linux, macOS, Windows. Handles raw mode, events, styling.

### freedesktop-desktop-entry

Parse XDG .desktop files.

```toml
[dependencies]
freedesktop-desktop-entry = "0.7"
```

**Why:** Handles the complexity of the desktop entry spec, including localization.

### nucleo-matcher

High-performance fuzzy matching.

```toml
[dependencies]
nucleo-matcher = "0.3"
```

**Why:** Same algorithm as Helix editor, very fast, good match quality.

### tokio

Async runtime for I/O operations.

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

**Why:** Standard async runtime, needed for PTY I/O and IPC.

### portable-pty

Cross-platform PTY handling.

```toml
[dependencies]
portable-pty = "0.8"
```

**Why:** Abstracts platform differences, handles edge cases.

### vte

ANSI escape sequence parser.

```toml
[dependencies]
vte = "0.13"
```

**Why:** Robust parsing of terminal escape codes for proper output rendering.

---

## Cargo.toml Example

```toml
[package]
name = "darkwall-drun"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core
ratatui = { version = "0.29", default-features = false, features = ["crossterm"] }
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }

# Parsing
freedesktop-desktop-entry = "0.7"
nucleo-matcher = "0.3"
toml = "0.8"
serde = { version = "1", features = ["derive"] }

# CLI & Errors
clap = { version = "4", features = ["derive"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

# Phase 2
portable-pty = "0.8"
vte = "0.13"

# Optional
ratatui-image = { version = "1.0", optional = true }
image = { version = "0.25", optional = true }
freedesktop-icons = { version = "0.2", optional = true }

[features]
default = []
icons = ["ratatui-image", "image", "freedesktop-icons"]

[profile.release]
lto = true
codegen-units = 1
strip = true
```

---

## Version Pinning Strategy

- **Major versions:** Pin to avoid breaking changes
- **Minor versions:** Allow updates for bug fixes
- **Patch versions:** Allow updates

Example:
```toml
ratatui = "0.29"      # Allows 0.29.x
tokio = "1"           # Allows 1.x.x
```

---

## Security Considerations

- All dependencies from crates.io
- `cargo audit` in CI
- Minimal dependency tree
- No native dependencies (except libc)

---

## Updating Dependencies

```bash
# Check for updates
cargo outdated

# Update within semver bounds
cargo update

# Update specific crate
cargo update -p ratatui

# Audit for vulnerabilities
cargo audit
```
