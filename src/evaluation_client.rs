use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct EvaluationClientConfig {
    pub llm_endpoint: String,
    pub llm_api_key: Option<String>,
    pub model_name: String,
    pub server_command: String,
    pub server_args: Vec<String>,
    pub timeout_seconds: u64,
}

impl Default for EvaluationClientConfig {
    fn default() -> Self {
        Self {
            llm_endpoint: "http://localhost:1234/v1".to_string(),
            llm_api_key: None,
            model_name: "gpt-3.5-turbo".to_string(),
            server_command: "cargo".to_string(),
            server_args: vec!["run".to_string()],
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: OpenAIFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub tools: Vec<OpenAITool>,
    pub tool_choice: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIToolFunction,
}

#[derive(Debug, Serialize)]
pub struct OpenAIToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChatResponse {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIResponseMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIResponseMessage {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug)]
pub struct EvaluationClient {
    pub config: EvaluationClientConfig,
    http_client: Client,
    mcp_process: Option<Child>,
    pub conversation_history: Vec<OpenAIMessage>,
}

impl EvaluationClient {
    pub fn new(config: EvaluationClientConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            mcp_process: None,
            conversation_history: Vec::new(),
        }
    }

    pub async fn connect_to_mcp_server(&mut self) -> Result<()> {
        info!("Starting MCP server process");
        
        let mut cmd = Command::new(&self.config.server_command);
        cmd.args(&self.config.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()
            .context("Failed to spawn MCP server process")?;

        self.mcp_process = Some(child);
        info!("MCP server process started");
        Ok(())
    }

    pub async fn get_available_tools(&self) -> Result<Vec<McpTool>> {
        // For now, return simulated tools that match our MCP server
        Ok(vec![
            McpTool {
                name: "find_scope".to_string(),
                description: "Find containing scope around a position using relational rules".to_string(),
                input_schema: serde_json::json!({
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
                }),
            },
            McpTool {
                name: "execute_rule".to_string(),
                description: "Execute ast-grep rule for search, replace, or scan operations".to_string(),
                input_schema: serde_json::json!({
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
                }),
            },
        ])
    }

    pub async fn call_mcp_tool(&self, name: &str, arguments: Value) -> Result<String> {
        // For now, simulate tool calls
        match name {
            "find_scope" => {
                Ok(format!("Found scope for tool call: {} with args: {}", name, arguments))
            }
            "execute_rule" => {
                Ok(format!("Executed ast-grep rule: {} with args: {}", name, arguments))
            }
            _ => {
                Err(anyhow::anyhow!("Unknown tool: {}", name))
            }
        }
    }

    pub async fn chat_with_llm(&mut self, user_message: &str) -> Result<String> {
        self.conversation_history.push(OpenAIMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            tool_calls: None,
        });

        let tools = self.get_available_tools().await?;
        let openai_tools: Vec<OpenAITool> = tools.into_iter().map(|tool| {
            OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIToolFunction {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.input_schema,
                },
            }
        }).collect();

        let request = OpenAIChatRequest {
            model: self.config.model_name.clone(),
            messages: self.conversation_history.clone(),
            tools: openai_tools,
            tool_choice: "auto".to_string(),
        };

        let mut req_builder = self.http_client
            .post(format!("{}/chat/completions", self.config.llm_endpoint))
            .json(&request);

        if let Some(api_key) = &self.config.llm_api_key {
            req_builder = req_builder.bearer_auth(api_key);
        }

        let response = req_builder.send().await
            .context("Failed to send request to LLM")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("LLM API error: {}", error_text));
        }

        let chat_response: OpenAIChatResponse = response.json().await
            .context("Failed to parse LLM response")?;

        let message = &chat_response.choices[0].message;
        
        if let Some(tool_calls) = &message.tool_calls {
            debug!("LLM requested {} tool calls", tool_calls.len());
            
            let mut tool_results = Vec::new();
            for tool_call in tool_calls {
                let args: Value = serde_json::from_str(&tool_call.function.arguments)
                    .context("Failed to parse tool arguments")?;
                
                info!("Calling tool: {} with args: {}", tool_call.function.name, args);
                
                match self.call_mcp_tool(&tool_call.function.name, args).await {
                    Ok(result) => {
                        tool_results.push(format!("Tool {}: {}", tool_call.function.name, result));
                    }
                    Err(e) => {
                        error!("Tool call failed: {}", e);
                        tool_results.push(format!("Tool {} failed: {}", tool_call.function.name, e));
                    }
                }
            }

            self.conversation_history.push(OpenAIMessage {
                role: "assistant".to_string(),
                content: message.content.clone().unwrap_or_default(),
                tool_calls: Some(tool_calls.clone()),
            });

            for result in tool_results.iter() {
                self.conversation_history.push(OpenAIMessage {
                    role: "tool".to_string(),
                    content: result.clone(),
                    tool_calls: None,
                });
            }

            let follow_up_request = OpenAIChatRequest {
                model: self.config.model_name.clone(),
                messages: self.conversation_history.clone(),
                tools: vec![],
                tool_choice: "none".to_string(),
            };

            let mut follow_up_req = self.http_client
                .post(format!("{}/chat/completions", self.config.llm_endpoint))
                .json(&follow_up_request);

            if let Some(api_key) = &self.config.llm_api_key {
                follow_up_req = follow_up_req.bearer_auth(api_key);
            }

            let follow_up_response = follow_up_req.send().await
                .context("Failed to send follow-up request to LLM")?;

            let follow_up_chat: OpenAIChatResponse = follow_up_response.json().await
                .context("Failed to parse follow-up LLM response")?;

            let final_content = follow_up_chat.choices[0].message.content
                .clone()
                .unwrap_or_default();

            self.conversation_history.push(OpenAIMessage {
                role: "assistant".to_string(),
                content: final_content.clone(),
                tool_calls: None,
            });

            Ok(final_content)
        } else {
            let content = message.content.clone().unwrap_or_default();
            self.conversation_history.push(OpenAIMessage {
                role: "assistant".to_string(),
                content: content.clone(),
                tool_calls: None,
            });
            Ok(content)
        }
    }

    pub fn reset_conversation(&mut self) {
        self.conversation_history.clear();
    }

    pub async fn evaluate_prompt(&mut self, prompt: &str) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();
        
        let response = self.chat_with_llm(prompt).await?;
        
        let duration = start_time.elapsed();
        
        Ok(EvaluationResult {
            prompt: prompt.to_string(),
            response,
            duration_ms: duration.as_millis() as u64,
            tool_calls_made: self.count_tool_calls_in_conversation(),
            success: true,
        })
    }

    fn count_tool_calls_in_conversation(&self) -> usize {
        self.conversation_history
            .iter()
            .filter_map(|msg| msg.tool_calls.as_ref())
            .map(|calls| calls.len())
            .sum()
    }
}

#[derive(Debug, Serialize)]
pub struct EvaluationResult {
    pub prompt: String,
    pub response: String,
    pub duration_ms: u64,
    pub tool_calls_made: usize,
    pub success: bool,
}

#[derive(Debug)]
pub struct EvaluationSuite {
    client: EvaluationClient,
    test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub prompt: String,
    pub expected_tools: Vec<String>,
    pub success_criteria: fn(&EvaluationResult) -> bool,
}

impl EvaluationSuite {
    pub fn new(config: EvaluationClientConfig) -> Self {
        Self {
            client: EvaluationClient::new(config),
            test_cases: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.client.connect_to_mcp_server().await
    }

    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }

    pub async fn run_evaluations(&mut self) -> Result<Vec<(TestCase, EvaluationResult)>> {
        let mut results = Vec::new();
        
        for test_case in &self.test_cases {
            info!("Running evaluation: {}", test_case.name);
            self.client.reset_conversation();
            
            let result = self.client.evaluate_prompt(&test_case.prompt).await?;
            let success = (test_case.success_criteria)(&result);
            
            info!("Evaluation {} completed: success={}", test_case.name, success);
            results.push((test_case.clone(), result));
        }
        
        Ok(results)
    }
}

pub fn create_default_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            name: "Basic AST search".to_string(),
            prompt: "Search for all function declarations in this JavaScript code: function hello() { return 'world'; }".to_string(),
            expected_tools: vec!["execute_rule".to_string()],
            success_criteria: |result| {
                result.success && result.tool_calls_made > 0 && result.response.contains("function")
            },
        },
        TestCase {
            name: "Scope finding".to_string(),
            prompt: "Find the containing scope around line 1, column 10 in this code: function test() { const x = 1; return x; }".to_string(),
            expected_tools: vec!["find_scope".to_string()],
            success_criteria: |result| {
                result.success && result.tool_calls_made > 0
            },
        },
        TestCase {
            name: "Code refactoring".to_string(),
            prompt: "Replace all console.log statements with logger.info in this code: console.log('hello'); console.log('world');".to_string(),
            expected_tools: vec!["execute_rule".to_string()],
            success_criteria: |result| {
                result.success && result.tool_calls_made > 0 && result.response.contains("logger.info")
            },
        },
    ]
}