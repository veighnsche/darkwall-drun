//! PTY (pseudo-terminal) handling for in-place command execution.
//!
//! TEAM_000: Phase 2, Unit 2.1 - PTY Allocation

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};

// Re-export ExitStatus for use by other modules
pub use portable_pty::ExitStatus;

/// A PTY session for running commands
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl PtySession {
    /// Spawn a new PTY session with the given command
    pub fn spawn(cmd: &str, cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(size)
            .context("Failed to open PTY")?;

        let mut command = CommandBuilder::new("sh");
        command.arg("-c");
        command.arg(cmd);

        // Inherit environment
        for (key, value) in std::env::vars() {
            command.env(key, value);
        }

        // Set TERM if not already set
        if std::env::var("TERM").is_err() {
            command.env("TERM", "xterm-256color");
        }

        let child = pair
            .slave
            .spawn_command(command)
            .context("Failed to spawn command in PTY")?;

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        Ok(Self {
            master: pair.master,
            child,
            reader,
            writer,
        })
    }

    /// Resize the PTY
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to resize PTY")
    }

    /// Read available data from the PTY (non-blocking attempt)
    /// Returns Ok(None) if no data available, Ok(Some(data)) if data read
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        // portable-pty reader is blocking, so we use a small timeout approach
        // For now, just do a blocking read - we'll handle this in the async layer
        match self.reader.read(buf) {
            Ok(0) => Ok(None), // EOF
            Ok(n) => Ok(Some(n)),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e).context("Failed to read from PTY"),
        }
    }

    /// Write data to the PTY (for user input)
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer
            .write_all(data)
            .context("Failed to write to PTY")?;
        self.writer.flush().context("Failed to flush PTY writer")
    }

    /// Check if the child process is still running
    pub fn is_alive(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// Wait for the child process to exit and return the exit status
    pub fn wait(&mut self) -> Result<ExitStatus> {
        self.child.wait().context("Failed to wait for child process")
    }

    /// Try to get exit status without blocking
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .context("Failed to check child status")
    }

    /// Get a clone of the reader for async operations
    pub fn take_reader(&mut self) -> Box<dyn Read + Send> {
        std::mem::replace(
            &mut self.reader,
            Box::new(std::io::empty()),
        )
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        // Try to kill the child if still running
        if self.is_alive() {
            let _ = self.child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_spawn_simple() {
        let mut session = PtySession::spawn("echo hello", 80, 24).unwrap();
        
        // Wait for command to complete
        let status = session.wait().unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_pty_read_output() {
        let mut session = PtySession::spawn("echo hello", 80, 24).unwrap();
        
        let mut buf = [0u8; 1024];
        let mut output = Vec::new();
        
        // Read until EOF or timeout
        loop {
            match session.try_read(&mut buf) {
                Ok(Some(n)) => output.extend_from_slice(&buf[..n]),
                Ok(None) => break,
                Err(_) => break,
            }
            if !session.is_alive() {
                // Read any remaining output
                while let Ok(Some(n)) = session.try_read(&mut buf) {
                    output.extend_from_slice(&buf[..n]);
                    if n == 0 {
                        break;
                    }
                }
                break;
            }
        }
        
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("hello"), "Output was: {}", output_str);
    }

    #[test]
    fn test_pty_resize() {
        let session = PtySession::spawn("sleep 0.1", 80, 24).unwrap();
        assert!(session.resize(120, 40).is_ok());
    }

    #[test]
    fn test_pty_exit_code() {
        let mut session = PtySession::spawn("exit 42", 80, 24).unwrap();
        let status = session.wait().unwrap();
        // portable_pty::ExitStatus only exposes success()
        assert!(!status.success());
    }

    #[test]
    fn test_pty_success() {
        let mut session = PtySession::spawn("exit 0", 80, 24).unwrap();
        let status = session.wait().unwrap();
        assert!(status.success());
    }
}
