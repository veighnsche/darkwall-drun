# Phase 10: Async PTY

## Overview

Replace the current blocking PTY read with async I/O for better responsiveness and resource usage.

## Current State

The `PtySession` uses blocking reads:

```rust
// Current implementation in pty.rs
pub fn try_read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
    match self.reader.read(buf) {
        Ok(0) => Ok(None),
        Ok(n) => Ok(Some(n)),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e).context("Failed to read from PTY"),
    }
}
```

The `take_reader()` method exists but is unused - it was intended for async operations.

## Problem

1. **Blocking reads** can cause UI lag
2. **Polling loop** wastes CPU when no output
3. **No true async** integration with tokio runtime

## Solution: Async PTY Reading

### Approach 1: Tokio Spawn Blocking

Wrap blocking reads in `spawn_blocking`:

```rust
use tokio::task::spawn_blocking;

impl App {
    pub async fn poll_execution_async(&mut self) -> Result<bool> {
        let Some(ref mut session) = self.pty_session else {
            return Ok(false);
        };
        
        // Take the reader for async operation
        let mut reader = session.take_reader();
        
        // Read in blocking task
        let result = spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            match reader.read(&mut buf) {
                Ok(n) => Ok((reader, buf, n)),
                Err(e) => Err(e),
            }
        }).await?;
        
        match result {
            Ok((reader, buf, n)) => {
                // Return reader to session
                session.return_reader(reader);
                if n > 0 {
                    self.output_buffer.push(&buf[..n]);
                }
                Ok(true)
            }
            Err(e) => {
                tracing::warn!("PTY read error: {}", e);
                Ok(false)
            }
        }
    }
}
```

### Approach 2: Async PTY Crate

Use `tokio-pty-process` or similar:

```rust
use tokio_pty_process::{AsyncPtyMaster, Child};

pub struct AsyncPtySession {
    master: AsyncPtyMaster,
    child: Child,
}

impl AsyncPtySession {
    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.master.read(buf).await
    }
}
```

### Approach 3: Channel-Based

Use channels to decouple reading from main loop:

```rust
use tokio::sync::mpsc;

pub struct PtySession {
    output_rx: mpsc::Receiver<Vec<u8>>,
    input_tx: mpsc::Sender<Vec<u8>>,
    // ...
}

impl PtySession {
    pub fn spawn(cmd: &str, cols: u16, rows: u16) -> Result<Self> {
        let (output_tx, output_rx) = mpsc::channel(100);
        let (input_tx, input_rx) = mpsc::channel(100);
        
        // Spawn reader task
        let reader = /* get reader */;
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if output_tx.send(buf[..n].to_vec()).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        // Spawn writer task
        // ...
        
        Ok(Self { output_rx, input_tx, ... })
    }
    
    pub async fn try_read(&mut self) -> Option<Vec<u8>> {
        self.output_rx.try_recv().ok()
    }
}
```

## Recommended Approach

**Approach 3 (Channel-Based)** is recommended because:

1. Clean separation of concerns
2. Non-blocking main loop
3. Works with existing tokio runtime
4. Easy to add backpressure

## Implementation Plan

### Unit 10.1: Channel Infrastructure

```rust
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    output_rx: mpsc::Receiver<PtyOutput>,
    input_tx: mpsc::Sender<Vec<u8>>,
    _reader_handle: JoinHandle<()>,
    _writer_handle: JoinHandle<()>,
}

pub enum PtyOutput {
    Data(Vec<u8>),
    Eof,
    Error(String),
}
```

### Unit 10.2: Reader Task

```rust
fn spawn_reader(
    mut reader: Box<dyn Read + Send>,
    tx: mpsc::Sender<PtyOutput>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = tx.blocking_send(PtyOutput::Eof);
                    break;
                }
                Ok(n) => {
                    if tx.blocking_send(PtyOutput::Data(buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.blocking_send(PtyOutput::Error(e.to_string()));
                    break;
                }
            }
        }
    })
}
```

### Unit 10.3: Writer Task

```rust
fn spawn_writer(
    mut writer: Box<dyn Write + Send>,
    mut rx: mpsc::Receiver<Vec<u8>>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        while let Some(data) = rx.blocking_recv() {
            if writer.write_all(&data).is_err() {
                break;
            }
            if writer.flush().is_err() {
                break;
            }
        }
    })
}
```

### Unit 10.4: Integration

```rust
impl App {
    pub async fn poll_execution(&mut self) -> Result<bool> {
        let Some(ref mut session) = self.pty_session else {
            return Ok(false);
        };
        
        // Non-blocking receive
        while let Ok(output) = session.output_rx.try_recv() {
            match output {
                PtyOutput::Data(data) => self.output_buffer.push(&data),
                PtyOutput::Eof => {
                    // Handle EOF
                }
                PtyOutput::Error(e) => {
                    tracing::warn!("PTY error: {}", e);
                }
            }
        }
        
        // Check if process exited
        // ...
    }
}
```

## Benefits

1. **No UI blocking** - Main loop stays responsive
2. **Efficient** - No busy-waiting, uses OS-level blocking
3. **Backpressure** - Channel buffers prevent memory issues
4. **Clean shutdown** - Dropping channels signals tasks to exit

## Configuration

```toml
[execution]
# Buffer size for PTY output channel
pty_buffer_size = 100

# Read buffer size
pty_read_size = 4096
```

## Testing

```bash
# Test with high-output command
cargo run
# Run: yes | head -10000

# Test with slow output
# Run: for i in $(seq 10); do echo $i; sleep 0.5; done

# Test with interactive input
# Run: cat (then type)
```

## Implementation Checklist

- [ ] Add `mpsc` channels to `PtySession`
- [ ] Implement reader task with `spawn_blocking`
- [ ] Implement writer task with `spawn_blocking`
- [ ] Update `PtySession::spawn()` to start tasks
- [ ] Add `PtyOutput` enum
- [ ] Update `App::poll_execution()` to use channels
- [ ] Handle graceful shutdown
- [ ] Add tests for async behavior
- [ ] Benchmark vs current implementation
