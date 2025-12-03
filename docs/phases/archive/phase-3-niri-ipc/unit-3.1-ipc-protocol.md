# Unit 3.1: IPC Protocol Implementation

> **Phase:** 3 - Niri IPC Integration  
> **Complexity:** Low  
> **Skills:** JSON-RPC, Unix sockets

---

## Objective

Implement robust Niri IPC communication with proper error handling and reconnection logic.

---

## Tasks

### 1. Parse Niri JSON-RPC Responses

```rust
#[derive(Deserialize)]
#[serde(untagged)]
pub enum NiriResponse {
    Ok { ok: serde_json::Value },
    Err { err: String },
}

impl NiriClient {
    pub async fn send_request(&mut self, request: &str) -> Result<NiriResponse>;
}
```

### 2. Handle Connection Errors

- Socket not found (niri not running)
- Connection refused
- Timeout on response

### 3. Add Reconnection Logic

```rust
impl NiriClient {
    pub async fn ensure_connected(&mut self) -> Result<()>;
    pub async fn reconnect(&mut self) -> Result<()>;
}
```

---

## Implementation Notes

### Socket Discovery

```rust
fn find_niri_socket() -> Option<PathBuf> {
    // 1. Check NIRI_SOCKET env var
    if let Ok(path) = std::env::var("NIRI_SOCKET") {
        return Some(PathBuf::from(path));
    }
    
    // 2. Check XDG_RUNTIME_DIR
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir).join("niri-socket");
        if path.exists() {
            return Some(path);
        }
    }
    
    None
}
```

### Request/Response Format

Niri uses newline-delimited JSON:

```rust
async fn send_request(&mut self, request: &str) -> Result<NiriResponse> {
    // Send request with newline
    self.stream.write_all(request.as_bytes()).await?;
    self.stream.write_all(b"\n").await?;
    
    // Read response line
    let mut response = String::new();
    self.reader.read_line(&mut response).await?;
    
    Ok(serde_json::from_str(&response)?)
}
```

### Graceful Degradation

```rust
pub struct NiriClient {
    stream: Option<UnixStream>,
    enabled: bool,
}

impl NiriClient {
    pub fn new() -> Self {
        let stream = find_niri_socket()
            .and_then(|path| UnixStream::connect(path).ok());
        
        Self {
            enabled: stream.is_some(),
            stream,
        }
    }
    
    pub fn is_available(&self) -> bool {
        self.enabled && self.stream.is_some()
    }
}
```

---

## Error Handling

| Error | Handling |
|-------|----------|
| Socket not found | Log warning, disable IPC |
| Connection refused | Retry once, then disable |
| Timeout | Log error, continue without IPC |
| Invalid response | Log error, ignore response |

---

## Acceptance Criteria

- [ ] Connects to Niri socket successfully
- [ ] Parses JSON-RPC responses correctly
- [ ] Handles missing socket gracefully
- [ ] Reconnects after temporary disconnection
- [ ] App works without niri (NFR-03)

---

## Testing

### Unit Tests

```rust
#[test]
fn test_parse_ok_response() {
    let json = r#"{"ok": null}"#;
    let resp: NiriResponse = serde_json::from_str(json).unwrap();
    assert!(matches!(resp, NiriResponse::Ok { .. }));
}

#[test]
fn test_parse_err_response() {
    let json = r#"{"err": "window not found"}"#;
    let resp: NiriResponse = serde_json::from_str(json).unwrap();
    assert!(matches!(resp, NiriResponse::Err { .. }));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_niri_connection() {
    let client = NiriClient::new();
    if client.is_available() {
        // Test actual connection
    } else {
        // Skip test, niri not running
    }
}
```

### Manual Tests

1. Run without niri - should work, log "Niri IPC disabled"
2. Run with niri - should connect successfully
3. Kill niri during run - should handle gracefully

---

## Related Units

- **Depends on:** None
- **Blocks:** Unit 3.2 (Window State Management)
