//! MCP (Model Context Protocol) client.
//!
//! Connects to external tool servers over stdio or HTTP/SSE, discovers
//! their tools, and routes tool calls through the MCP JSON-RPC protocol.

use crate::conversation::{FunctionDefinition, ToolDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, error, warn};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// MCP server connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Human-readable name used to key connections in `McpManager`.
    pub name: String,
    /// Transport type: `"stdio"` or `"http"`.
    pub transport: String,
    /// For stdio: command to spawn. Ignored for http.
    pub command: Option<String>,
    /// For stdio: arguments passed to the command.
    pub args: Option<Vec<String>>,
    /// For http: base URL of the MCP server (e.g. `http://localhost:3000`).
    pub url: Option<String>,
    /// Extra environment variables passed to a stdio subprocess.
    pub env: Option<HashMap<String, String>>,
}

/// A tool definition as returned by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Result returned by an MCP `tools/call` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// A single content block inside an `McpToolResult`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

// ---------------------------------------------------------------------------
// JSON-RPC wire types (private)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by the MCP client.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("failed to spawn MCP server: {0}")]
    Spawn(String),
    #[error("MCP server disconnected")]
    Disconnected,
    #[error("JSON-RPC error ({code}): {message}")]
    JsonRpc { code: i64, message: String },
    #[error("MCP protocol error: {0}")]
    Protocol(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("server not found: {0}")]
    ServerNotFound(String),
    #[error("tool not found: {0}")]
    ToolNotFound(String),
}

// ---------------------------------------------------------------------------
// McpConnection — a single MCP server
// ---------------------------------------------------------------------------

/// An active connection to an MCP server.
pub struct McpConnection {
    config: McpServerConfig,
    child: Option<Child>,
    stdin: Option<tokio::process::ChildStdin>,
    stdout_reader: Option<BufReader<tokio::process::ChildStdout>>,
    /// HTTP client for http+sse transport.
    http_client: Option<reqwest::Client>,
    tools: Vec<McpTool>,
    next_id: u64,
}

impl McpConnection {
    // -- constructors -------------------------------------------------------

    /// Connect to an MCP server over stdio (spawn a child process).
    pub async fn connect_stdio(config: McpServerConfig) -> Result<Self, McpError> {
        let cmd = config
            .command
            .as_deref()
            .ok_or_else(|| McpError::Spawn("missing `command` for stdio transport".into()))?;

        let args = config.args.clone().unwrap_or_default();

        let mut command = Command::new(cmd);
        command
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref env) = config.env {
            for (k, v) in env {
                command.env(k, v);
            }
        }

        let mut child = command
            .spawn()
            .map_err(|e| McpError::Spawn(format!("{cmd}: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Spawn("failed to capture stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Spawn("failed to capture stdout".into()))?;

        let mut conn = Self {
            config,
            child: Some(child),
            stdin: Some(stdin),
            stdout_reader: Some(BufReader::new(stdout)),
            http_client: None,
            tools: Vec::new(),
            next_id: 1,
        };

        conn.initialize().await?;
        conn.tools = conn.list_tools().await?;
        debug!(
            server = %conn.config.name,
            tool_count = conn.tools.len(),
            "MCP stdio connection established"
        );
        Ok(conn)
    }

    /// Connect to an MCP server over HTTP (JSON-RPC over HTTP POST).
    pub async fn connect_http(config: McpServerConfig) -> Result<Self, McpError> {
        let _url = config
            .url
            .as_deref()
            .ok_or_else(|| McpError::Http("missing `url` for http transport".into()))?;

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| McpError::Http(e.to_string()))?;

        let mut conn = Self {
            config,
            child: None,
            stdin: None,
            stdout_reader: None,
            http_client: Some(http_client),
            tools: Vec::new(),
            next_id: 1,
        };

        conn.initialize().await?;
        conn.tools = conn.list_tools().await?;
        debug!(
            server = %conn.config.name,
            tool_count = conn.tools.len(),
            "MCP HTTP connection established"
        );
        Ok(conn)
    }

    // -- JSON-RPC transport -------------------------------------------------

    /// Send a JSON-RPC request and read the response.
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, McpError> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method: method.into(),
            params,
        };

        if self.http_client.is_some() {
            self.send_request_http(&request).await
        } else {
            self.send_request_stdio(&request).await
        }
    }

    async fn send_request_stdio(
        &mut self,
        request: &JsonRpcRequest,
    ) -> Result<serde_json::Value, McpError> {
        let stdin = self.stdin.as_mut().ok_or(McpError::Disconnected)?;
        let reader = self.stdout_reader.as_mut().ok_or(McpError::Disconnected)?;

        let mut payload = serde_json::to_string(request)?;
        payload.push('\n');

        stdin.write_all(payload.as_bytes()).await?;
        stdin.flush().await?;

        // Read lines until we get a valid JSON-RPC response matching our id.
        // MCP servers may emit notifications (no `id`) which we skip.
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(McpError::Disconnected);
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let resp: JsonRpcResponse = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(_) => {
                    // Not valid JSON-RPC — may be log output; skip.
                    debug!(line, "skipping non-JSON-RPC line from MCP server");
                    continue;
                }
            };

            // Skip notifications (no id).
            if resp.id != Some(request.id) {
                continue;
            }

            if let Some(err) = resp.error {
                return Err(McpError::JsonRpc {
                    code: err.code,
                    message: err.message,
                });
            }

            return resp
                .result
                .ok_or_else(|| McpError::Protocol("response missing both result and error".into()));
        }
    }

    async fn send_request_http(
        &mut self,
        request: &JsonRpcRequest,
    ) -> Result<serde_json::Value, McpError> {
        let client = self.http_client.as_ref().ok_or(McpError::Disconnected)?;
        let base_url = self
            .config
            .url
            .as_deref()
            .ok_or_else(|| McpError::Http("no URL configured".into()))?;

        // POST to <base>/rpc (common MCP HTTP convention).
        let url = format!("{}/rpc", base_url.trim_end_matches('/'));

        let http_resp = client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| McpError::Http(e.to_string()))?;

        if !http_resp.status().is_success() {
            return Err(McpError::Http(format!(
                "HTTP {} from {}",
                http_resp.status(),
                url
            )));
        }

        let resp: JsonRpcResponse = http_resp
            .json()
            .await
            .map_err(|e| McpError::Http(e.to_string()))?;

        if let Some(err) = resp.error {
            return Err(McpError::JsonRpc {
                code: err.code,
                message: err.message,
            });
        }

        resp.result
            .ok_or_else(|| McpError::Protocol("response missing both result and error".into()))
    }

    // -- MCP protocol methods -----------------------------------------------

    /// Perform the MCP `initialize` handshake.
    pub async fn initialize(&mut self) -> Result<(), McpError> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": { "listChanged": false }
            },
            "clientInfo": {
                "name": "terminal-ai",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let result = self
            .send_request("initialize", Some(params))
            .await?;

        debug!(server = %self.config.name, ?result, "MCP initialize response");

        // Send initialized notification (no id — fire and forget via stdio).
        if self.stdin.is_some() {
            let notification = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            });
            if let Some(stdin) = self.stdin.as_mut() {
                let mut payload = serde_json::to_string(&notification)?;
                payload.push('\n');
                stdin.write_all(payload.as_bytes()).await?;
                stdin.flush().await?;
            }
        }

        Ok(())
    }

    /// Discover tools exposed by the server (`tools/list`).
    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, McpError> {
        let result = self.send_request("tools/list", None).await?;

        #[derive(Deserialize)]
        struct ToolsList {
            tools: Vec<McpTool>,
        }

        let list: ToolsList = serde_json::from_value(result)?;
        Ok(list.tools)
    }

    /// Invoke a tool on the server (`tools/call`).
    pub async fn call_tool(
        &mut self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<McpToolResult, McpError> {
        let params = serde_json::json!({
            "name": name,
            "arguments": args,
        });

        let result = self.send_request("tools/call", Some(params)).await?;
        let tool_result: McpToolResult = serde_json::from_value(result)?;
        Ok(tool_result)
    }

    // -- conversions --------------------------------------------------------

    /// Return cached tool definitions.
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Convert the server's MCP tools into OpenAI function-calling format.
    pub fn to_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .map(|t| {
                let prefixed_name = format!("mcp_{}_{}", self.config.name, t.name);
                ToolDefinition {
                    tool_type: "function".into(),
                    function: FunctionDefinition {
                        name: prefixed_name,
                        description: t
                            .description
                            .clone()
                            .unwrap_or_else(|| format!("MCP tool: {}", t.name)),
                        parameters: t.input_schema.clone(),
                    },
                }
            })
            .collect()
    }

    /// The server name this connection belongs to.
    pub fn name(&self) -> &str {
        &self.config.name
    }

    // -- lifecycle ----------------------------------------------------------

    /// Disconnect from the server and clean up resources.
    pub async fn disconnect(&mut self) {
        // Drop stdin/stdout so the child gets EOF.
        self.stdin.take();
        self.stdout_reader.take();

        if let Some(ref mut child) = self.child {
            // Try a graceful wait, then kill.
            match tokio::time::timeout(std::time::Duration::from_secs(2), child.wait()).await {
                Ok(Ok(status)) => {
                    debug!(server = %self.config.name, ?status, "MCP server exited");
                }
                _ => {
                    warn!(server = %self.config.name, "killing MCP server");
                    let _ = child.kill().await;
                }
            }
        }
        self.child.take();
        self.http_client.take();
        self.tools.clear();
    }
}

impl Drop for McpConnection {
    fn drop(&mut self) {
        // Best-effort sync cleanup — stdin/stdout are dropped automatically.
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
    }
}

// ---------------------------------------------------------------------------
// McpManager — manages multiple MCP connections
// ---------------------------------------------------------------------------

/// Manages multiple simultaneous MCP server connections.
pub struct McpManager {
    connections: HashMap<String, McpConnection>,
}

impl McpManager {
    /// Create a new, empty manager.
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Connect to an MCP server using the supplied configuration.
    pub async fn connect(&mut self, config: McpServerConfig) -> Result<(), McpError> {
        let name = config.name.clone();

        // Disconnect existing connection with the same name, if any.
        if self.connections.contains_key(&name) {
            self.disconnect(&name).await;
        }

        let conn = match config.transport.as_str() {
            "stdio" => McpConnection::connect_stdio(config).await?,
            "http" | "sse" | "http+sse" => McpConnection::connect_http(config).await?,
            other => {
                return Err(McpError::Protocol(format!(
                    "unsupported transport: {other}"
                )))
            }
        };

        self.connections.insert(name, conn);
        Ok(())
    }

    /// Disconnect a specific server by name.
    pub async fn disconnect(&mut self, name: &str) {
        if let Some(mut conn) = self.connections.remove(name) {
            conn.disconnect().await;
        }
    }

    /// Disconnect all servers.
    pub async fn disconnect_all(&mut self) {
        let names: Vec<String> = self.connections.keys().cloned().collect();
        for name in names {
            self.disconnect(&name).await;
        }
    }

    /// Collect tool definitions from every connected server.
    pub fn all_tools(&self) -> Vec<ToolDefinition> {
        self.connections
            .values()
            .flat_map(|c| c.to_tool_definitions())
            .collect()
    }

    /// Call a tool on a specific server.
    pub async fn call_tool(
        &mut self,
        server_name: &str,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<McpToolResult, McpError> {
        let conn = self
            .connections
            .get_mut(server_name)
            .ok_or_else(|| McpError::ServerNotFound(server_name.into()))?;

        conn.call_tool(tool_name, args).await
    }

    /// Find which server owns a given tool name.
    ///
    /// Tool names are prefixed as `mcp_{server}_{tool}` by `to_tool_definitions`.
    /// This method also supports looking up the raw (unprefixed) tool name.
    pub fn find_tool_server(&self, tool_name: &str) -> Option<&str> {
        // First try the prefixed convention: mcp_{server}_{tool}.
        for (server_name, conn) in &self.connections {
            let prefix = format!("mcp_{}_", server_name);
            if tool_name.starts_with(&prefix) {
                return Some(server_name.as_str());
            }
            // Also check unprefixed.
            if conn.tools.iter().any(|t| t.name == tool_name) {
                return Some(server_name.as_str());
            }
        }
        None
    }

    /// Resolve a prefixed tool name (`mcp_{server}_{tool}`) back to the raw
    /// server-side tool name.
    pub fn resolve_tool_name<'a>(&self, prefixed: &'a str) -> Option<(&str, &'a str)> {
        for (server_name, _conn) in &self.connections {
            let prefix = format!("mcp_{}_", server_name);
            if let Some(raw) = prefixed.strip_prefix(&prefix) {
                return Some((server_name.as_str(), raw));
            }
        }
        None
    }

    /// Number of active connections.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// List the names of connected servers.
    pub fn server_names(&self) -> Vec<&str> {
        self.connections.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_to_definition() {
        let tool = McpTool {
            name: "read_file".into(),
            description: Some("Read a file from disk".into()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        };

        let config = McpServerConfig {
            name: "fs".into(),
            transport: "stdio".into(),
            command: Some("mcp-fs".into()),
            args: None,
            url: None,
            env: None,
        };

        let conn = McpConnection {
            config,
            child: None,
            stdin: None,
            stdout_reader: None,
            http_client: None,
            tools: vec![tool],
            next_id: 1,
        };

        let defs = conn.to_tool_definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].function.name, "mcp_fs_read_file");
        assert_eq!(defs[0].tool_type, "function");
    }

    #[test]
    fn test_manager_find_tool_server() {
        let config = McpServerConfig {
            name: "git".into(),
            transport: "stdio".into(),
            command: Some("mcp-git".into()),
            args: None,
            url: None,
            env: None,
        };

        let tool = McpTool {
            name: "log".into(),
            description: None,
            input_schema: serde_json::json!({}),
        };

        let conn = McpConnection {
            config,
            child: None,
            stdin: None,
            stdout_reader: None,
            http_client: None,
            tools: vec![tool],
            next_id: 1,
        };

        let mut mgr = McpManager::new();
        mgr.connections.insert("git".into(), conn);

        assert_eq!(mgr.find_tool_server("mcp_git_log"), Some("git"));
        assert_eq!(mgr.find_tool_server("log"), Some("git"));
        assert_eq!(mgr.find_tool_server("unknown"), None);
    }

    #[test]
    fn test_resolve_tool_name() {
        let config = McpServerConfig {
            name: "db".into(),
            transport: "stdio".into(),
            command: Some("mcp-db".into()),
            args: None,
            url: None,
            env: None,
        };

        let conn = McpConnection {
            config,
            child: None,
            stdin: None,
            stdout_reader: None,
            http_client: None,
            tools: Vec::new(),
            next_id: 1,
        };

        let mut mgr = McpManager::new();
        mgr.connections.insert("db".into(), conn);

        let (server, raw) = mgr.resolve_tool_name("mcp_db_query").unwrap();
        assert_eq!(server, "db");
        assert_eq!(raw, "query");
    }
}
