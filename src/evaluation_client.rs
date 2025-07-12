use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{debug, error, info};
use std::time::{SystemTime, UNIX_EPOCH};

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
        let initial_history_len = self.conversation_history.len();
        
        let response = self.chat_with_llm(prompt).await?;
        
        let duration = start_time.elapsed();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let tool_calls = self.extract_tool_calls_from_recent_conversation(initial_history_len);
        
        Ok(EvaluationResult {
            prompt: prompt.to_string(),
            response,
            duration_ms: duration.as_millis() as u64,
            tool_calls_made: self.count_tool_calls_in_conversation(),
            success: true,
            timestamp,
            model_name: self.config.model_name.clone(),
            tool_calls,
            conversation_length: self.conversation_history.len(),
        })
    }

    fn extract_tool_calls_from_recent_conversation(&self, start_index: usize) -> Vec<ToolCallResult> {
        let mut tool_calls = Vec::new();
        
        for message in self.conversation_history.iter().skip(start_index) {
            if let Some(calls) = &message.tool_calls {
                for call in calls {
                    let args: Value = serde_json::from_str(&call.function.arguments)
                        .unwrap_or_default();
                    
                    tool_calls.push(ToolCallResult {
                        tool_name: call.function.name.clone(),
                        arguments: args,
                        result: "Simulated result".to_string(), // In real implementation, track actual results
                        success: true,
                        duration_ms: 0, // Would need to track actual timing
                    });
                }
            }
        }
        
        tool_calls
    }

    fn count_tool_calls_in_conversation(&self) -> usize {
        self.conversation_history
            .iter()
            .filter_map(|msg| msg.tool_calls.as_ref())
            .map(|calls| calls.len())
            .sum()
    }

    pub fn analyze_response(&self, response: &str, result: &EvaluationResult) -> ResponseAnalysis {
        let contains_tool_calls = result.tool_calls_made > 0;
        let contains_code = self.detect_code_blocks(response);
        let contains_error = self.detect_error_patterns(response);
        let word_count = response.split_whitespace().count();
        let sentiment = self.analyze_sentiment(response);
        let success_indicators = self.extract_success_indicators(response);
        let failure_indicators = self.extract_failure_indicators(response);

        ResponseAnalysis {
            contains_tool_calls,
            contains_code,
            contains_error,
            word_count,
            sentiment,
            success_indicators,
            failure_indicators,
        }
    }

    fn detect_code_blocks(&self, response: &str) -> bool {
        response.contains("```") || 
        response.contains("function ") ||
        response.contains("const ") ||
        response.contains("let ") ||
        response.contains("var ") ||
        response.contains("class ") ||
        response.contains("impl ") ||
        response.contains("fn ")
    }

    fn detect_error_patterns(&self, response: &str) -> bool {
        let error_patterns = ["error", "failed", "exception", "panic", "undefined", "null reference"];
        let response_lower = response.to_lowercase();
        error_patterns.iter().any(|pattern| response_lower.contains(pattern))
    }

    fn analyze_sentiment(&self, response: &str) -> ResponseSentiment {
        let response_lower = response.to_lowercase();
        
        if response_lower.contains("sorry") || response_lower.contains("confused") || response_lower.contains("unclear") {
            ResponseSentiment::Confused
        } else if response_lower.contains("help") || response_lower.contains("assist") || response_lower.contains("guide") {
            ResponseSentiment::Helpful
        } else if response_lower.contains("success") || response_lower.contains("complete") || response_lower.contains("done") {
            ResponseSentiment::Positive
        } else if response_lower.contains("fail") || response_lower.contains("error") || response_lower.contains("problem") {
            ResponseSentiment::Negative
        } else {
            ResponseSentiment::Neutral
        }
    }

    fn extract_success_indicators(&self, response: &str) -> Vec<String> {
        let mut indicators = Vec::new();
        let response_lower = response.to_lowercase();
        
        let patterns = [
            "successfully",
            "completed",
            "found",
            "executed",
            "processed",
            "analyzed",
            "generated",
            "created",
        ];
        
        for pattern in &patterns {
            if response_lower.contains(pattern) {
                indicators.push(pattern.to_string());
            }
        }
        
        indicators
    }

    fn extract_failure_indicators(&self, response: &str) -> Vec<String> {
        let mut indicators = Vec::new();
        let response_lower = response.to_lowercase();
        
        let patterns = [
            "failed",
            "error",
            "unable",
            "cannot",
            "invalid",
            "missing",
            "not found",
            "timeout",
        ];
        
        for pattern in &patterns {
            if response_lower.contains(pattern) {
                indicators.push(pattern.to_string());
            }
        }
        
        indicators
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvaluationResult {
    pub prompt: String,
    pub response: String,
    pub duration_ms: u64,
    pub tool_calls_made: usize,
    pub success: bool,
    pub timestamp: u64,
    pub model_name: String,
    pub tool_calls: Vec<ToolCallResult>,
    pub conversation_length: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub arguments: Value,
    pub result: String,
    pub success: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotMetadata {
    pub test_name: String,
    pub model_name: String,
    pub timestamp: u64,
    pub git_commit: Option<String>,
    pub prompt_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseSnapshot {
    pub metadata: SnapshotMetadata,
    pub evaluation_result: EvaluationResult,
    pub response_analysis: ResponseAnalysis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseAnalysis {
    pub contains_tool_calls: bool,
    pub contains_code: bool,
    pub contains_error: bool,
    pub word_count: usize,
    pub sentiment: ResponseSentiment,
    pub success_indicators: Vec<String>,
    pub failure_indicators: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResponseSentiment {
    Positive,
    Negative,
    Neutral,
    Confused,
    Helpful,
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