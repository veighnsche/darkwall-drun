use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Client for niri IPC
pub struct NiriClient {
    socket_path: PathBuf,
}

impl NiriClient {
    /// Create a new niri client, auto-detecting socket path
    pub fn new() -> Result<Self> {
        let socket_path = Self::find_socket()?;
        tracing::info!("Using niri socket: {}", socket_path.display());
        Ok(Self { socket_path })
    }

    /// Find the niri socket path
    fn find_socket() -> Result<PathBuf> {
        // Check NIRI_SOCKET env var first
        if let Ok(path) = std::env::var("NIRI_SOCKET") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try XDG_RUNTIME_DIR
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(runtime_dir).join("niri-socket");
            if path.exists() {
                return Ok(path);
            }
        }

        anyhow::bail!("Could not find niri socket. Is niri running?")
    }

    /// Send a request to niri and get response
    async fn request(&self, msg: &str) -> Result<String> {
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

        Ok(response)
    }

    /// Set the current window's floating state
    pub async fn set_floating(&self, floating: bool) -> Result<()> {
        // niri IPC format for setting window floating
        // The null id means "focused window"
        let msg = format!(
            r#"{{"Action":{{"SetWindowFloating":{{"id":null,"floating":{}}}}}}}"#,
            floating
        );

        let response = self.request(&msg).await?;
        tracing::debug!("niri response: {}", response);

        // Check for error in response
        if response.contains("Error") {
            anyhow::bail!("niri error: {}", response);
        }

        Ok(())
    }

    /// Get information about the focused window
    pub async fn focused_window(&self) -> Result<Option<WindowInfo>> {
        let msg = r#"{"Request":"FocusedWindow"}"#;
        let response = self.request(msg).await?;

        // Parse response - simplified, real impl would use serde
        if response.contains("null") {
            return Ok(None);
        }

        // TODO: Parse actual window info
        Ok(Some(WindowInfo {
            id: 0,
            app_id: String::new(),
            title: String::new(),
            is_floating: false,
        }))
    }

    /// Toggle floating state of current window
    pub async fn toggle_floating(&self) -> Result<()> {
        let msg = r#"{"Action":{"ToggleWindowFloating":{"id":null}}}"#;
        let response = self.request(msg).await?;
        tracing::debug!("niri response: {}", response);

        if response.contains("Error") {
            anyhow::bail!("niri error: {}", response);
        }

        Ok(())
    }
}

/// Information about a niri window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: u64,
    pub app_id: String,
    pub title: String,
    pub is_floating: bool,
}
