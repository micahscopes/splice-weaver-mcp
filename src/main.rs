use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, error};

mod ast_grep_tools;

use ast_grep_tools::AstGrepTools;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting MCP ast-grep server");
    
    let tools = AstGrepTools::new();
    let mut server = McpServer::new(tools);
    
    server.run().await
}

struct McpServer {
    tools: AstGrepTools,
}

impl McpServer {
    fn new(tools: AstGrepTools) -> Self {
        Self { tools }
    }
    
    async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(request) = serde_json::from_str::<Value>(&line) {
                        let response = self.handle_request(request).await;
                        let response_str = serde_json::to_string(&response)?;
                        stdout.write_all(response_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_request(&mut self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let params = &request["params"];
        let id = request["id"].clone();
        
        match method {
            "initialize" => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "mcp-ast-grep",
                            "version": "0.1.0"
                        }
                    }
                })
            }
            "tools/list" => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": self.tools.list_tools()
                    }
                })
            }
            "tools/call" => {
                let tool_name = params["name"].as_str().unwrap_or("");
                let arguments = params["arguments"].clone();
                
                match self.tools.call_tool(tool_name, arguments).await {
                    Ok(result) => {
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [
                                    {
                                        "type": "text",
                                        "text": result
                                    }
                                ]
                            }
                        })
                    }
                    Err(e) => {
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": format!("Tool execution failed: {}", e)
                            }
                        })
                    }
                }
            }
            _ => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                })
            }
        }
    }
}
