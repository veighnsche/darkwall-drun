//! Niri IPC client for window state management.
//!
//! TEAM_000: Phase 3 - Niri IPC Integration

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Niri IPC response format
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NiriResponse {
    Ok { ok: serde_json::Value },
    Err { err: String },
}

impl NiriResponse {
    /// Check if response indicates success
    pub fn is_ok(&self) -> bool {
        matches!(self, NiriResponse::Ok { .. })
    }

    /// Get error message if response is an error
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

    /// Check if niri IPC is available
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

    /// Get information about the focused window
    pub async fn focused_window(&self) -> Result<Option<WindowInfo>> {
        let msg = r#"{"Request":"FocusedWindow"}"#;
        let response = self.request(msg).await?;

        match response {
            NiriResponse::Ok { ok } => {
                if ok.is_null() {
                    return Ok(None);
                }
                // Parse window info from the response
                let info: WindowInfo = serde_json::from_value(ok)
                    .context("Failed to parse window info")?;
                Ok(Some(info))
            }
            NiriResponse::Err { err } => {
                anyhow::bail!("niri error: {}", err);
            }
        }
    }

    /// Toggle floating state of current window
    pub async fn toggle_floating(&self) -> Result<()> {
        let msg = r#"{"Action":{"ToggleWindowFloating":{"id":null}}}"#;
        let response = self.request(msg).await?;

        if let Some(err) = response.error() {
            anyhow::bail!("niri error: {}", err);
        }

        Ok(())
    }
}

/// Information about a niri window
#[derive(Debug, Clone, Deserialize)]
pub struct WindowInfo {
    pub id: u64,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub title: String,
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
