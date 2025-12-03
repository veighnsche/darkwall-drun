# Getting Started

> Quick start guide for new developers.

---

## Prerequisites

- Rust toolchain (rustup recommended)
- A terminal emulator (foot, alacritty, kitty)
- Optional: Niri compositor

---

## Clone and Build

```bash
cd /home/vince/Projects/darkwall-drun
cargo build
```

---

## Run with Debug Logging

```bash
RUST_LOG=debug cargo run
```

---

## Test with foot

```bash
foot --app-id darkwall-drun -e cargo run
```

---

## Check Niri Socket

```bash
echo $NIRI_SOCKET
# or
ls $XDG_RUNTIME_DIR/niri*
```

---

## Project Structure

```
darkwall-drun/
├── Cargo.toml
├── README.md
├── config.example.toml
├── docs/
│   ├── DEVELOPMENT.md          # Main index
│   ├── ARCHITECTURE.md         # System design
│   ├── TESTING.md              # Test strategy
│   ├── DEPENDENCIES.md         # Crate info
│   ├── GETTING_STARTED.md      # This file
│   └── phases/                 # Phase documentation
│       ├── phase-1-basic-launcher.md
│       ├── phase-2-in-place-execution/
│       ├── phase-3-niri-ipc/
│       ├── phase-4-advanced-metadata/
│       └── phase-5-polish/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── config.rs
│   ├── desktop_entry.rs
│   ├── niri.rs
│   └── ui.rs
└── tests/
    └── fixtures/
```

---

## Development Workflow

### 1. Pick a Work Unit

Browse `docs/phases/` and find an unclaimed unit.

### 2. Create a Branch

```bash
git checkout -b feature/unit-2.1-pty-allocation
```

### 3. Implement

Follow the unit's tasks and acceptance criteria.

### 4. Test

```bash
cargo test
cargo clippy
cargo fmt --check
```

### 5. Submit PR

Include:
- Unit reference
- Test results
- Manual testing notes

---

## Useful Commands

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Run tests
cargo test

# Build release
cargo build --release

# Check dependencies
cargo tree
cargo outdated
```

---

## Configuration

Copy the example config:

```bash
mkdir -p ~/.config/darkwall-drun
cp config.example.toml ~/.config/darkwall-drun/config.toml
```

Edit as needed.

---

## Debugging Tips

### TUI Issues

```bash
# Run outside of cargo for cleaner terminal
cargo build && ./target/debug/darkwall-drun
```

### Niri IPC

```bash
# Test IPC manually
echo '{"Request":"Version"}' | nc -U $NIRI_SOCKET
```

### Desktop Entries

```bash
# List system entries
ls /usr/share/applications/

# List user entries
ls ~/.local/share/applications/
```

---

## Next Steps

1. Read [ARCHITECTURE.md](./ARCHITECTURE.md)
2. Review [Phase 1](./phases/phase-1-basic-launcher.md) (complete)
3. Pick a unit from Phase 2-5
4. Start coding!
