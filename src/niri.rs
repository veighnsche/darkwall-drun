//! Niri IPC client for window state management.
//!
//! TEAM_000: Phase 3 - Niri IPC Integration
//!
//! # Architecture
//!
//! This module provides async IPC communication with the niri compositor.
//! Niri uses a JSON-based protocol over Unix sockets.
//!
//! # Connection Lifecycle
//!
//! 1. On startup, `NiriClient::try_new()` attempts to find the socket
//! 2. If found, the client is stored in `App.niri`
//! 3. Each IPC call opens a new connection (niri doesn't support persistent connections)
//! 4. If the socket disappears (niri crash), calls will fail gracefully
//!
//! # Graceful Degradation
//!
//! All niri features are optional. When unavailable:
//! - Over SSH: Socket doesn't exist, `try_new()` returns None
//! - Niri crash: `is_available()` returns false, calls return errors
//! - Non-niri session: Same as SSH case

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Niri IPC response format
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NiriResponse {
    Ok {
        #[allow(dead_code)] // Used for parsing, value accessed via pattern matching
        ok: serde_json::Value,
    },
    Err { err: String },
}

impl NiriResponse {
    /// Check if response indicates success.
    ///
    /// # Usage
    ///
    /// Use this for simple success/failure checks without inspecting the payload:
    ///
    /// ```ignore
    /// let response = client.request(msg).await?;
    /// if !response.is_ok() {
    ///     // Handle error
    /// }
    /// ```
    ///
    /// For responses with data (like `focused_window`), pattern match instead
    /// to access the `ok` field's contents.
    #[allow(dead_code)] // Phase 9: Will be used for health checks
    pub fn is_ok(&self) -> bool {
        matches!(self, NiriResponse::Ok { .. })
    }

    /// Get error message if response is an error.
    ///
    /// Returns `None` for successful responses.
    pub fn error(&self) -> Option<&str> {
        match self {
            NiriResponse::Err { err } => Some(err),
            _ => None,
        }
    }
}

/// Client for niri IPC
/// TEAM_000: Phase 3, Unit 3.1 - IPC Protocol
#[derive(Clone)]
pub struct NiriClient {
    socket_path: PathBuf,
}

impl NiriClient {
    /// Create a new niri client, auto-detecting socket path.
    /// 
    /// Returns None if niri socket is not found (e.g., over SSH or non-niri session).
    /// This is expected behavior - DRUN works fine without niri.
    pub fn new() -> Result<Self> {
        let socket_path = Self::find_socket()?;
        tracing::info!("Using niri socket: {}", socket_path.display());
        Ok(Self { socket_path })
    }

    /// Try to create a niri client, returning None if unavailable.
    /// 
    /// This is the preferred way to create a client when niri is optional.
    /// Common case: running over SSH where niri socket doesn't exist.
    pub fn try_new() -> Option<Self> {
        match Self::new() {
            Ok(client) => Some(client),
            Err(e) => {
                tracing::debug!("Niri IPC not available: {}", e);
                None
            }
        }
    }

    /// Check if niri IPC is currently available.
    ///
    /// # Behavior
    ///
    /// Returns `true` if the socket file exists. This is a quick filesystem check,
    /// not a full connection test.
    ///
    /// # Use Cases
    ///
    /// 1. **Health indicator in UI**: Show green/red dot in status bar
    /// 2. **Feature gating**: Skip niri-specific UI elements when unavailable
    /// 3. **Reconnection logic**: Periodically check if niri has restarted
    ///
    /// # Limitations
    ///
    /// - Socket existing doesn't guarantee niri is responsive
    /// - For full health check, use `ping()` (not yet implemented)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // In status bar rendering
    /// let indicator = if client.is_available() { "◉" } else { "◎" };
    /// ```
    #[allow(dead_code)] // Phase 9: Will be used for health indicator
    pub fn is_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Find the niri socket path
    fn find_socket() -> Result<PathBuf> {
        // Check NIRI_SOCKET env var first
        if let Ok(path) = std::env::var("NIRI_SOCKET") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
            // Socket path set but doesn't exist - likely stale env var
            tracing::debug!("NIRI_SOCKET set but path doesn't exist: {}", path.display());
        }

        // Try XDG_RUNTIME_DIR
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(runtime_dir).join("niri-socket");
            if path.exists() {
                return Ok(path);
            }
        }

        // No socket found - this is normal over SSH or in non-niri sessions
        anyhow::bail!("Niri socket not found (normal if not running under niri or via SSH)")
    }

    /// Send a request to niri and get parsed response
    async fn request(&self, msg: &str) -> Result<NiriResponse> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to niri socket")?;

        stream
            .write_all(msg.as_bytes())
            .await
            .context("Failed to write to niri socket")?;

        stream.shutdown().await?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .context("Failed to read from niri socket")?;

        tracing::debug!("niri response: {}", response);

        serde_json::from_str(&response)
            .context("Failed to parse niri response")
    }

    /// Set the current window's floating state
    /// TEAM_000: Phase 3, Unit 3.2 - Window State Management
    pub async fn set_floating(&self, floating: bool) -> Result<()> {
        // niri IPC format for setting window floating
        // The null id means "focused window"
        let msg = format!(
            r#"{{"Action":{{"SetWindowFloating":{{"id":null,"floating":{}}}}}}}"#,
            floating
        );

        let response = self.request(&msg).await?;

        if let Some(err) = response.error() {
            anyhow::bail!("niri error: {}", err);
        }

        Ok(())
    }

    /// Get information about the currently focused window.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(WindowInfo))` - Window is focused, info retrieved
    /// - `Ok(None)` - No window is focused (e.g., empty workspace)
    /// - `Err(_)` - IPC error (socket gone, parse error, etc.)
    ///
    /// # Use Cases
    ///
    /// 1. **Smart float/unfloat**: Remember original state before execution
    ///    ```ignore
    ///    let was_floating = client.focused_window().await?
    ///        .map(|w| w.is_floating)
    ///        .unwrap_or(false);
    ///    // ... execute command ...
    ///    if was_floating {
    ///        client.set_floating(true).await?;
    ///    }
    ///    ```
    ///
    /// 2. **Debug display**: Show current window info in status bar
    ///    ```ignore
    ///    if let Some(info) = client.focused_window().await? {
    ///        println!("Window: {} ({})", info.title, info.app_id);
    ///    }
    ///    ```
    ///
    /// 3. **Context-aware behavior**: Different actions based on current app
    ///    ```ignore
    ///    if info.app_id == "firefox" {
    ///        // Special handling for browser
    ///    }
    ///    ```
    ///
    /// # Performance
    ///
    /// Each call opens a new socket connection. For frequent polling,
    /// consider caching with a refresh interval (e.g., 1 second).
    #[allow(dead_code)] // Phase 9: Will be used for smart float/unfloat
    pub async fn focused_window(&self) -> Result<Option<WindowInfo>> {
        let msg = r#"{"Request":"FocusedWindow"}"#;
        let response = self.request(msg).await?;

        match response {
            NiriResponse::Ok { ok } => {
                if ok.is_null() {
                    return Ok(None);
                }
                let info: WindowInfo = serde_json::from_value(ok)
                    .context("Failed to parse window info")?;
                Ok(Some(info))
            }
            NiriResponse::Err { err } => {
                anyhow::bail!("niri error: {}", err);
            }
        }
    }

    /// Toggle floating state of the focused window.
    ///
    /// # Behavior
    ///
    /// - If window is tiled → becomes floating
    /// - If window is floating → becomes tiled
    /// - If no window focused → returns error from niri
    ///
    /// # Use Cases
    ///
    /// 1. **User keybind**: Ctrl+F to toggle float from within drun
    ///    ```ignore
    ///    // In key handler
    ///    KeyCode::Char('f') if ctrl => {
    ///        if let Some(ref niri) = app.niri {
    ///            niri.toggle_floating().await.ok();
    ///        }
    ///    }
    ///    ```
    ///
    /// 2. **Quick resize workflow**: Float → resize with mouse → unfloat
    ///
    /// # Difference from `set_floating()`
    ///
    /// - `toggle_floating()`: Inverts current state (don't need to know current state)
    /// - `set_floating(bool)`: Sets explicit state (idempotent, predictable)
    ///
    /// Use `toggle_floating()` for user-triggered actions.
    /// Use `set_floating()` for programmatic state management.
    #[allow(dead_code)] // Phase 9: Will be used for Ctrl+F keybind
    pub async fn toggle_floating(&self) -> Result<()> {
        let msg = r#"{"Action":{"ToggleWindowFloating":{"id":null}}}"#;
        let response = self.request(msg).await?;

        if let Some(err) = response.error() {
            anyhow::bail!("niri error: {}", err);
        }

        Ok(())
    }
}

/// Information about a niri window.
///
/// # Fields
///
/// - `id`: Unique window identifier (stable for window lifetime)
/// - `app_id`: Wayland app_id (e.g., "firefox", "org.gnome.Nautilus")
/// - `title`: Current window title (may change dynamically)
/// - `is_floating`: Whether window is in floating state
///
/// # Use Cases
///
/// ```ignore
/// let info = client.focused_window().await?.unwrap();
///
/// // Check if this is our own window
/// if info.app_id == "darkwall-drun" {
///     // We're focused, good
/// }
///
/// // Remember float state for later restoration
/// let was_floating = info.is_floating;
///
/// // Log for debugging
/// tracing::debug!("Focused: {} ({})", info.title, info.app_id);
/// ```
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Phase 9: Will be used with focused_window()
pub struct WindowInfo {
    /// Unique window identifier assigned by niri.
    /// Stable for the lifetime of the window.
    pub id: u64,
    
    /// Wayland app_id (similar to X11 WM_CLASS).
    /// Set by the application, e.g., "firefox", "kitty".
    #[serde(default)]
    pub app_id: String,
    
    /// Current window title.
    /// May change dynamically (e.g., browser tab changes).
    #[serde(default)]
    pub title: String,
    
    /// Whether the window is currently floating.
    /// `false` means tiled in the layout.
    #[serde(default)]
    pub is_floating: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ok_response() {
        let json = r#"{"ok": null}"#;
        let resp: NiriResponse = serde_json::from_str(json).unwrap();
        assert!(resp.is_ok());
        assert!(resp.error().is_none());
    }

    #[test]
    fn test_parse_err_response() {
        let json = r#"{"err": "window not found"}"#;
        let resp: NiriResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.is_ok());
        assert_eq!(resp.error(), Some("window not found"));
    }

    #[test]
    fn test_parse_window_info() {
        let json = r#"{"id": 123, "app_id": "darkwall-drun", "title": "Test"}"#;
        let info: WindowInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, 123);
        assert_eq!(info.app_id, "darkwall-drun");
        assert_eq!(info.title, "Test");
    }
}
