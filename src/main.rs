use anyhow::Result;
use rmcp::{model::*, service::*, ServerHandler};
use std::sync::Arc;
use tracing::{error, info};

mod ast_grep_tools;
mod binary_manager;
pub mod evaluation_client;
mod simple_search;
use ast_grep_tools::AstGrepTools;
use binary_manager::BinaryManager;

// All functionality now handled by AstGrepTools

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting MCP ast-grep server");

    let handler = AstGrepServer::new();
    info!("Handler created successfully");
    
    let transport = rmcp::transport::io::stdio();
    info!("Transport created, serving...");

    // Run the service with stdio transport
    let server = rmcp::service::serve_server(handler, transport).await?;
    info!("Server started, waiting for connections");
    server.waiting().await?;

    Ok(())
}

#[derive(Clone)]
pub struct AstGrepServer {
    tools: Arc<AstGrepTools>,
}

impl Default for AstGrepServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AstGrepServer {
    pub fn new() -> Self {
        let binary_manager =
            Arc::new(BinaryManager::new().expect("Failed to initialize binary manager"));
        let tools = Arc::new(AstGrepTools::new(binary_manager));
        Self { tools }
    }
}

impl ServerHandler for AstGrepServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "mcp-ast-grep".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some(
                "Minimal ast-grep MCP server with scope navigation and rule execution tools"
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::Error> {
        let tools = vec![
            Tool::new(
                "find_scope",
                "Find containing scope around a position using relational rules",
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Code to search within"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language (e.g., 'javascript', 'python', 'rust')"
                        },
                        "position": {
                            "type": "object",
                            "properties": {
                                "line": {"type": "number", "description": "Line number (1-indexed)"},
                                "column": {"type": "number", "description": "Column number (1-indexed)"}
                            },
                            "required": ["line", "column"],
                            "description": "Cursor position to find scope around"
                        },
                        "scope_rule": {
                            "type": "string",
                            "description": "YAML rule defining the scope to find (e.g., function, class, loop)"
                        }
                    },
                    "required": ["code", "language", "position", "scope_rule"]
                })).unwrap()
            ),
            Tool::new(
                "execute_rule",
                "Execute ast-grep rule for search, replace, or scan operations",
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "rule_config": {
                            "type": "string",
                            "description": "Complete YAML rule configuration"
                        },
                        "target": {
                            "type": "string",
                            "description": "File path or directory to apply rule to"
                        },
                        "operation": {
                            "type": "string",
                            "enum": ["search", "replace", "scan"],
                            "description": "Operation to perform",
                            "default": "search"
                        },
                        "dry_run": {
                            "type": "boolean",
                            "description": "If true, preview changes without applying",
                            "default": true
                        }
                    },
                    "required": ["rule_config", "target"]
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
        let args = request
            .arguments
            .map(serde_json::Value::Object)
            .unwrap_or(serde_json::Value::Null);
        match self.tools.call_tool(&request.name, args).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => {
                error!("Tool execution failed: {}", e);
                Err(rmcp::Error::invalid_params(
                    format!("Tool execution failed: {e}"),
                    None,
                ))
            }
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::Error> {
        let resources = self.tools.list_resources();
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::Error> {
        match self.tools.read_resource(&request.uri) {
            Ok(content) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, request.uri.clone())],
            }),
            Err(e) => Err(rmcp::Error::invalid_params(
                format!("Resource not found: {e}"),
                None,
            )),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::Error> {
        let prompts = self.tools.list_prompts();
        Ok(ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, rmcp::Error> {
        let arguments = request.arguments.unwrap_or_default();
        let arguments_hashmap: std::collections::HashMap<String, serde_json::Value> =
            arguments.into_iter().collect();
        match self.tools.get_prompt(&request.name, arguments_hashmap) {
            Ok(content) => Ok(GetPromptResult {
                description: None,
                messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
            }),
            Err(e) => Err(rmcp::Error::invalid_params(
                format!("Prompt not found: {e}"),
                None,
            )),
        }
    }
}
