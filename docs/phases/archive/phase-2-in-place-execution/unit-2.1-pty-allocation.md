# Unit 2.1: PTY Allocation

> **Phase:** 2 - In-Place Execution  
> **Complexity:** High  
> **Skills:** Unix systems, PTY internals

---

## Objective

Create a PTY (pseudo-terminal) module that allocates terminals for child processes, enabling in-place command execution.

---

## Tasks

### 1. Create `src/pty.rs` module

```rust
// Skeleton structure
pub struct PtySession {
    master: MasterPty,
    child: Child,
    size: PtySize,
}

impl PtySession {
    pub fn spawn(cmd: &str) -> Result<Self>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn read(&mut self) -> Result<Vec<u8>>;
    pub fn write(&mut self, data: &[u8]) -> Result<()>;
    pub fn is_alive(&self) -> bool;
    pub fn wait(&mut self) -> Result<ExitStatus>;
}
```

### 2. PTY Allocation

- Use `portable-pty` crate for cross-platform support
- Configure PTY size from terminal dimensions
- Set up environment variables (TERM, etc.)

### 3. Handle Terminal Resize (SIGWINCH)

- Listen for terminal resize events
- Propagate size changes to PTY
- Update child process window size

---

## Implementation Notes

### Crate Choice: `portable-pty`

```toml
[dependencies]
portable-pty = "0.8"
```

**Why portable-pty:**
- Cross-platform (Linux, macOS, Windows)
- Well-maintained
- Handles edge cases

### Basic Usage Pattern

```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

let pty_system = native_pty_system();
let pair = pty_system.openpty(PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
})?;

let mut cmd = CommandBuilder::new("sh");
cmd.arg("-c").arg(command);
let child = pair.slave.spawn_command(cmd)?;
```

### Resize Handling

```rust
// In event loop
if let Event::Resize(cols, rows) = event {
    pty_session.resize(cols, rows)?;
}
```

---

## Acceptance Criteria

- [ ] `PtySession::spawn()` successfully creates PTY
- [ ] Child process runs within PTY
- [ ] Terminal resize propagates to child
- [ ] PTY cleanup on drop (no zombie processes)
- [ ] Works with simple commands (`ls`, `echo`)
- [ ] Works with interactive commands (`bash`)

---

## Testing

### Unit Tests

```rust
#[test]
fn test_pty_spawn_simple() {
    let session = PtySession::spawn("echo hello").unwrap();
    let output = session.read_all().unwrap();
    assert!(output.contains("hello"));
}

#[test]
fn test_pty_resize() {
    let mut session = PtySession::spawn("bash").unwrap();
    session.resize(120, 40).unwrap();
    // Verify COLUMNS/LINES in child
}
```

### Manual Tests

1. Run `cargo run`, execute `tput cols` - should match terminal width
2. Resize terminal during command - child should see new size
3. Run `htop` - should render correctly

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| PTY leaks on crash | Implement Drop trait, use RAII |
| Signal handling complexity | Use tokio signal handling |
| Platform differences | Rely on portable-pty abstractions |

---

## Related Units

- **Depends on:** None (foundation unit)
- **Blocks:** Unit 2.2 (Output Capture), Unit 2.4 (Interactive Mode)
