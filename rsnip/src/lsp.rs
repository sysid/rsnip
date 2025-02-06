// src/lsp.rs
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tracing::{debug, error};

pub struct LspClient {
    process: Child,
    request_id: i64,
}

impl LspClient {
    pub async fn new(server_path: &str) -> Result<Self> {
        let process = Command::new(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start LSP server")?;

        let mut client = Self {
            process,
            request_id: 0,
        };

        // Initialize the server
        client.initialize().await?;

        Ok(client)
    }

    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        let request_str = serde_json::to_string(&request)?;
        let content_length = request_str.len();
        let header = format!("Content-Length: {}\r\n\r\n", content_length);

        // Write request
        let stdin = self.process.stdin.as_mut().context("Failed to get stdin")?;

        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.flush().await?;

        // Read response
        let mut stdout = self
            .process
            .stdout
            .as_mut()
            .context("Failed to get stdout")?;

        // Read header
        let mut header = String::new();
        let mut byte = [0u8; 1];
        while !header.ends_with("\r\n\r\n") {
            stdout.read_exact(&mut byte).await?;
            header.push(byte[0] as char);
        }

        // Parse content length
        let content_length = header
            .lines()
            .find(|line| line.starts_with("Content-Length: "))
            .and_then(|line| line.strip_prefix("Content-Length: "))
            .and_then(|len| len.parse::<usize>().ok())
            .context("Invalid Content-Length header")?;

        // Read content
        let mut content = vec![0u8; content_length];
        stdout.read_exact(&mut content).await?;

        let response: Value = serde_json::from_slice(&content)?;

        if let Some(error) = response.get("error") {
            error!("LSP error: {:?}", error);
            anyhow::bail!("LSP error: {:?}", error);
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }

    async fn initialize(&mut self) -> Result<()> {
        let params = json!({
            "processId": std::process::id(),
            "rootUri": null,
            "capabilities": {
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true
                        }
                    }
                }
            }
        });

        let response = self.send_request("initialize", params).await?;
        debug!("Initialize response: {:?}", response);

        // Send initialized notification
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });

        let stdin = self.process.stdin.as_mut().context("Failed to get stdin")?;

        let msg = serde_json::to_string(&initialized)?;
        let header = format!("Content-Length: {}\r\n\r\n", msg.len());

        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(msg.as_bytes()).await?;
        stdin.flush().await?;

        Ok(())
    }

    pub async fn get_completions(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<String>> {
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            },
            "context": {
                "triggerKind": 1
            }
        });

        let response = self.send_request("textDocument/completion", params).await?;

        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .context("No completion items found")?;

        Ok(items
            .iter()
            .filter_map(|item| item.get("label"))
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect())
    }

    pub async fn notify_document_change(
        &mut self,
        uri: &str,
        version: i32,
        text: &str,
    ) -> Result<()> {
        let params = json!({
            "textDocument": {
                "uri": uri,
                "version": version,
                "text": text
            }
        });

        let notification = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": params
        });

        let stdin = self.process.stdin.as_mut().context("Failed to get stdin")?;

        let msg = serde_json::to_string(&notification)?;
        let header = format!("Content-Length: {}\r\n\r\n", msg.len());

        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(msg.as_bytes()).await?;
        stdin.flush().await?;

        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Send shutdown request when client is dropped
        let shutdown = json!({
            "jsonrpc": "2.0",
            "id": self.request_id + 1,
            "method": "shutdown"
        });

        if let Some(mut stdin) = self.process.stdin.take() {
            let msg = serde_json::to_string(&shutdown).unwrap();
            let header = format!("Content-Length: {}\r\n\r\n", msg.len());

            // Use blocking write since we're shutting down
            use std::io::Write;
            let _ = stdin.write_all(header.as_bytes());
            let _ = stdin.write_all(msg.as_bytes());
            let _ = stdin.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::process::Stdio;
    use tempfile::NamedTempFile;
    use tokio::io::{AsyncWriteExt, BufReader};
    use tokio::process::Command;

    // Helper to create a mock LSP server process
    async fn create_mock_server() -> Result<Child> {
        let process = Command::new("cat") // Use cat as mock server that echoes back
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(process)
    }

    #[tokio::test]
    async fn given_valid_server_when_initializing_then_succeeds() -> Result<()> {
        let mut client = LspClient::new("simple-completion-language-server").await?;
        Ok(())
    }

    #[tokio::test]
    async fn given_document_changes_when_notifying_then_succeeds() -> Result<()> {
        let mut client = LspClient::new("simple-completion-language-server").await?;

        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "fn main() {{\n    println!(\"Hello\");\n}}")?;

        client
            .notify_document_change(
                &format!("file://{}", temp_file.path().display()),
                1,
                "fn main() {\n    println!(\"Hello World\");\n}",
            )
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn given_completion_request_when_getting_completions_then_returns_items() -> Result<()> {
        let mut client = LspClient::new("simple-completion-language-server").await?;

        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "fn main() {{\n    println!(\"Hello\");\n}}")?;

        let completions = client
            .get_completions(&format!("file://{}", temp_file.path().display()), 1, 12)
            .await?;

        assert!(!completions.is_empty());
        Ok(())
    }
}
