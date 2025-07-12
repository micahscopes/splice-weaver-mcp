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

mod resources;
use resources::ResourceProvider;

mod prompts;
use prompts::PromptManager;

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
    resources: Arc<ResourceProvider>,
    prompts: Arc<PromptManager>,
}

impl AstGrepServer {
    pub fn new() -> Self {
        let tools = Arc::new(AstGrepTools::new());
        let resources = Arc::new(ResourceProvider::new());
        let prompts = Arc::new(PromptManager::new());
        Self { tools, resources, prompts }
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
            instructions: Some("Enhanced ast-grep server with Resources and Prompts for better small LLM support".to_string()),
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

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::Error> {
        let resources = self.resources.list_resources();
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::Error> {
        match self.resources.get_resource(&request.uri) {
            Ok(content) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content.to_string(), request.uri.clone())]
            }),
            Err(e) => Err(rmcp::Error::invalid_params(format!("Resource not found: {}", e), None)),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::Error> {
        let prompts = self.prompts.list_prompts();
        Ok(ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, rmcp::Error> {
        let arguments = request.arguments.unwrap_or_default();
        let arguments_hashmap: std::collections::HashMap<String, serde_json::Value> = arguments.into_iter().collect();
        match self.prompts.get_prompt(&request.name, arguments_hashmap) {
            Ok(content) => Ok(GetPromptResult {
                description: None,
                messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)]
            }),
            Err(e) => Err(rmcp::Error::invalid_params(format!("Prompt not found: {}", e), None)),
        }
    }
}