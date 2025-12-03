# Unit 10.6: Post-Migration Cleanup

> **Phase:** 10 - Terminal Emulator Integration  
> **Complexity:** Low  
> **Estimated Time:** 1-2 hours  
> **Prerequisites:** Units 10.1-10.5 complete and tested

---

## Objective

Remove the legacy `OutputBuffer` implementation and feature flag after the new terminal emulator is stable.

---

## When to Execute This Unit

**Execute immediately after Unit 10.5 passes all tests.**

The old `OutputBuffer` is broken - it doesn't handle:
- Carriage returns properly
- ANSI escape sequences
- Cursor positioning
- Colors

There's no point keeping broken code as a "fallback". Delete it as soon as the new implementation works.

### Pre-Cleanup Checklist

- [ ] Unit 10.5 complete
- [ ] `cargo test` passes
- [ ] Manual smoke test: rsync, sudo prompt, colors work

---

## Tasks

### 1. Remove OutputBuffer

Delete or archive the following code:

```rust
// DELETE from src/executor.rs:

/// Buffer for captured command output
pub struct OutputBuffer {
    lines: VecDeque<OutputLine>,
    max_lines: usize,
    scroll_offset: usize,
    partial_line: String,
    follow_mode: bool,
}

// All impl blocks for OutputBuffer
// strip_ansi_escapes function
// OutputLine struct
```

### 2. Update App Struct

```rust
// In app.rs - should already be done in Unit 10.2
pub struct App {
    terminal: EmbeddedTerminal,
}

impl App {
    pub fn terminal(&self) -> &EmbeddedTerminal {
        &self.terminal
    }
    
    pub fn terminal_mut(&mut self) -> &mut EmbeddedTerminal {
        &mut self.terminal
    }
}
```

### 3. Update Tests

```rust
// DELETE tests that reference OutputBuffer
#[test]
fn test_output_buffer_basic() { ... }  // DELETE

#[test]
fn test_output_buffer_scroll() { ... }  // DELETE

// KEEP/UPDATE tests for EmbeddedTerminal
#[test]
fn test_terminal_scroll() { ... }  // KEEP
```

### 6. Clean Up Imports

```rust
// Remove unused imports throughout codebase
// Run: cargo fix --allow-dirty
```

### 7. Update Documentation

Update these files to remove references to OutputBuffer:

| File | Change |
|------|--------|
| `docs/ARCHITECTURE.md` | Update output handling section |
| `docs/SYSTEM_REQUIREMENTS.md` | Update if needed |
| `README.md` | Update feature list if mentioned |
| Phase docs | Archive phase-10 to `archive/` |

---

## Files to Delete

| File/Code | Location |
|-----------|----------|
| `OutputBuffer` struct | `src/executor.rs` |
| `OutputLine` struct | `src/executor.rs` |
| `strip_ansi_escapes` fn | `src/executor.rs` |

---

## Files to Modify

| File | Change |
|------|--------|
| `src/app.rs` | Replace `output_buffer` field with `terminal` |
| `src/executor.rs` | Remove OutputBuffer (keep TerminalMode, CommandStatus) |
| `src/ui/draw.rs` | Use TerminalWidget |
| `src/main.rs` | Update key forwarding to use terminal |

---

## Verification Checklist

Before committing cleanup:

```bash
# 1. Build succeeds
cargo build

# 2. All tests pass
cargo test

# 3. No warnings about dead code (should be none after cleanup)
cargo build 2>&1 | grep -i "warning.*dead_code"

# 4. No unused imports
cargo build 2>&1 | grep -i "warning.*unused"

# 5. Manual testing
cargo run -- drun
# Test: rsync, sudo, htop, vim, colors
```

---

## Rollback Plan

If issues are discovered after cleanup:

1. **Git revert** the cleanup commit
2. Re-enable feature flag
3. Debug the issue with fallback available
4. Fix and re-attempt cleanup

Keep the cleanup as a **single atomic commit** for easy revert:

```bash
git commit -m "chore: remove legacy OutputBuffer after terminal emulator migration"
```

---

## Archive Phase Documentation

After cleanup is complete:

```bash
# Move phase docs to archive
mv docs/phases/phase-10-terminal-emulator docs/phases/archive/

# Or keep a summary
echo "# Phase 10: Terminal Emulator - COMPLETED $(date +%Y-%m-%d)" > docs/phases/archive/phase-10-terminal-emulator.md
```

---

## Acceptance Criteria

- [ ] No references to `OutputBuffer` in codebase
- [ ] No feature flag for terminal emulator
- [ ] `cargo build` produces no dead code warnings
- [ ] All tests pass
- [ ] Manual testing confirms no regressions
- [ ] Documentation updated
- [ ] Single atomic commit for easy rollback

---

## Post-Cleanup

After successful cleanup:

1. **Update CHANGELOG** with terminal emulator as stable feature
2. **Consider version bump** (minor version for new feature)
3. **Update any external documentation** or README badges
