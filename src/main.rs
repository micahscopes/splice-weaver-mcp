use anyhow::Result;
use rmcp::{
    ServerHandler,
    model::*,
    service::*,
};
use std::sync::Arc;
use tracing::{info, error};

mod ast_grep_tools;
use ast_grep_tools::AstGrepTools;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting MCP ast-grep server");
    
    let handler = AstGrepServer::new();
    let transport = rmcp::transport::io::stdio();
    
    // Run the service with stdio transport
    let server = rmcp::service::serve_server(handler, transport).await?;
    server.waiting().await?;
    
    Ok(())
}

#[derive(Clone)]
pub struct AstGrepServer {
    tools: Arc<AstGrepTools>,
}

impl AstGrepServer {
    pub fn new() -> Self {
        let tools = Arc::new(AstGrepTools::new());
        Self { tools }
    }
}

impl ServerHandler for AstGrepServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "mcp-ast-grep".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: None,
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::Error> {
        let tools = vec![
            Tool::new(
                "ast_grep_search",
                "Search for AST patterns in code using ast-grep",
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The AST pattern to search for"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language (e.g., 'javascript', 'python', 'rust')"
                        },
                        "path": {
                            "type": "string",
                            "description": "Path to search in (file or directory)"
                        }
                    },
                    "required": ["pattern", "language", "path"]
                })).unwrap()
            ),
            Tool::new(
                "ast_grep_replace",
                "Replace AST patterns in code using ast-grep",
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The AST pattern to search for"
                        },
                        "replacement": {
                            "type": "string",
                            "description": "The replacement pattern"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language (e.g., 'javascript', 'python', 'rust')"
                        },
                        "path": {
                            "type": "string",
                            "description": "Path to search in (file or directory)"
                        },
                        "dry_run": {
                            "type": "boolean",
                            "description": "If true, show what would be changed without applying changes",
                            "default": true
                        }
                    },
                    "required": ["pattern", "replacement", "language", "path"]
                })).unwrap()
            ),
            Tool::new(
                "ast_grep_scan",
                "Scan code for potential issues using ast-grep rules",
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "rule": {
                            "type": "string",
                            "description": "The ast-grep rule to apply (YAML format)"
                        },
                        "path": {
                            "type": "string",
                            "description": "Path to scan (file or directory)"
                        }
                    },
                    "required": ["rule", "path"]
                })).unwrap()
            ),
        ];
        
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        let args = request.arguments.map(|args| serde_json::Value::Object(args)).unwrap_or(serde_json::Value::Null);
        match self.tools.call_tool(&request.name, args).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => {
                error!("Tool execution failed: {}", e);
                Err(rmcp::Error::invalid_params(format!("Tool execution failed: {}", e), None))
            }
        }
    }
}