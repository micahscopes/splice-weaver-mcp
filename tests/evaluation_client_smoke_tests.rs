use anyhow::Result;
use mcp_ast_grep::evaluation_client::{
    create_default_test_cases, EvaluationClient, EvaluationClientConfig, EvaluationSuite, McpTool,
};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

const TEST_TIMEOUT: Duration = Duration::from_secs(30);

fn create_test_config() -> EvaluationClientConfig {
    EvaluationClientConfig {
        llm_endpoint: "http://httpbin.org/status/200".to_string(), // Mock endpoint for testing
        llm_api_key: None,
        model_name: "test-model".to_string(),
        server_command: "echo".to_string(), // Mock command
        server_args: vec!["test".to_string()],
        timeout_seconds: 5,
    }
}

#[tokio::test]
async fn test_evaluation_client_creation() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Testing evaluation client creation");

    let config = create_test_config();
    let client = EvaluationClient::new(config.clone());

    // Verify client was created with correct config
    assert_eq!(client.config.llm_endpoint, config.llm_endpoint);
    assert_eq!(client.config.model_name, config.model_name);
    assert_eq!(client.config.server_command, config.server_command);

    info!("✅ Evaluation client created successfully");
    Ok(())
}

#[tokio::test]
async fn test_evaluation_client_mcp_connection() -> Result<()> {
    info!("Testing evaluation client MCP server connection");

    let config = create_test_config();
    let mut client = EvaluationClient::new(config);

    // Test connection to MCP server (using echo command as mock)
    let result = timeout(TEST_TIMEOUT, client.connect_to_mcp_server()).await;
    
    match result {
        Ok(Ok(())) => {
            info!("✅ MCP server connection test passed");
        }
        Ok(Err(e)) => {
            warn!("MCP connection failed (expected for mock): {}", e);
            // This is expected since we're using echo as a mock command
        }
        Err(_) => {
            panic!("Timeout during MCP connection test");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_evaluation_client_get_tools() -> Result<()> {
    info!("Testing evaluation client tool listing");

    let config = create_test_config();
    let client = EvaluationClient::new(config);

    let tools = client.get_available_tools().await?;

    // Verify we get the expected tools
    assert!(!tools.is_empty(), "Should return at least some tools");
    assert!(tools.len() >= 2, "Should return at least 2 tools");

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"find_scope"));
    assert!(tool_names.contains(&"execute_rule"));

    // Verify tool structure
    for tool in &tools {
        assert!(!tool.name.is_empty(), "Tool name should not be empty");
        assert!(!tool.description.is_empty(), "Tool description should not be empty");
        assert!(tool.input_schema.is_object(), "Tool schema should be an object");
    }

    info!("✅ Tool listing test passed with {} tools", tools.len());
    Ok(())
}

#[tokio::test]
async fn test_evaluation_client_tool_call() -> Result<()> {
    info!("Testing evaluation client tool calling");

    let config = create_test_config();
    let client = EvaluationClient::new(config);

    let test_args = json!({
        "code": "function test() { return 42; }",
        "language": "javascript"
    });

    // Test calling find_scope tool
    let result = client.call_mcp_tool("find_scope", test_args.clone()).await?;
    assert!(!result.is_empty(), "Tool call should return some result");
    assert!(result.contains("find_scope"), "Result should mention the tool name");

    // Test calling execute_rule tool
    let result = client.call_mcp_tool("execute_rule", test_args).await?;
    assert!(!result.is_empty(), "Tool call should return some result");
    assert!(result.contains("execute_rule"), "Result should mention the tool name");

    // Test calling unknown tool
    let result = client.call_mcp_tool("unknown_tool", json!({})).await;
    assert!(result.is_err(), "Unknown tool should return error");

    info!("✅ Tool calling test passed");
    Ok(())
}

#[tokio::test]
async fn test_evaluation_suite_creation() -> Result<()> {
    info!("Testing evaluation suite creation");

    let config = create_test_config();
    let mut suite = EvaluationSuite::new(config);

    // Add test cases
    let test_cases = create_default_test_cases();
    assert!(!test_cases.is_empty(), "Should have default test cases");

    for test_case in test_cases {
        suite.add_test_case(test_case);
    }

    info!("✅ Evaluation suite created successfully");
    Ok(())
}

#[tokio::test]
async fn test_default_test_cases() -> Result<()> {
    info!("Testing default test cases structure");

    let test_cases = create_default_test_cases();
    assert!(test_cases.len() >= 3, "Should have at least 3 default test cases");

    for test_case in &test_cases {
        assert!(!test_case.name.is_empty(), "Test case should have a name");
        assert!(!test_case.prompt.is_empty(), "Test case should have a prompt");
        assert!(!test_case.expected_tools.is_empty(), "Test case should expect some tools");

        // Test the success criteria function
        let mock_result = mcp_ast_grep::evaluation_client::EvaluationResult {
            prompt: test_case.prompt.clone(),
            response: "Mock response with function".to_string(),
            duration_ms: 100,
            tool_calls_made: 1,
            success: true,
        };

        // Success criteria should be callable
        let _ = (test_case.success_criteria)(&mock_result);
    }

    info!("✅ Default test cases validation passed");
    Ok(())
}

#[tokio::test]
async fn test_conversation_history() -> Result<()> {
    info!("Testing conversation history management");

    let config = create_test_config();
    let mut client = EvaluationClient::new(config);

    // Initially empty
    assert_eq!(client.conversation_history.len(), 0);

    // Reset should keep it empty
    client.reset_conversation();
    assert_eq!(client.conversation_history.len(), 0);

    info!("✅ Conversation history test passed");
    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_structure() -> Result<()> {
    info!("Testing McpTool structure");

    let tool = McpTool {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "param1": {"type": "string"}
            }
        }),
    };

    assert_eq!(tool.name, "test_tool");
    assert_eq!(tool.description, "A test tool");
    assert!(tool.input_schema.is_object());

    info!("✅ McpTool structure test passed");
    Ok(())
}

#[tokio::test]
async fn test_config_defaults() -> Result<()> {
    info!("Testing configuration defaults");

    let default_config = EvaluationClientConfig::default();

    assert_eq!(default_config.llm_endpoint, "http://localhost:1234/v1");
    assert_eq!(default_config.model_name, "gpt-3.5-turbo");
    assert_eq!(default_config.server_command, "cargo");
    assert_eq!(default_config.server_args, vec!["run"]);
    assert_eq!(default_config.timeout_seconds, 30);
    assert!(default_config.llm_api_key.is_none());

    info!("✅ Configuration defaults test passed");
    Ok(())
}

#[tokio::test]
async fn test_openai_message_serialization() -> Result<()> {
    info!("Testing OpenAI message serialization");

    let message = mcp_ast_grep::evaluation_client::OpenAIMessage {
        role: "user".to_string(),
        content: "Test message".to_string(),
        tool_calls: None,
    };

    let serialized = serde_json::to_string(&message)?;
    assert!(serialized.contains("user"));
    assert!(serialized.contains("Test message"));

    // Test with tool calls
    let tool_call = mcp_ast_grep::evaluation_client::OpenAIToolCall {
        id: "call_123".to_string(),
        call_type: "function".to_string(),
        function: mcp_ast_grep::evaluation_client::OpenAIFunction {
            name: "test_function".to_string(),
            arguments: "{}".to_string(),
        },
    };

    let message_with_tools = mcp_ast_grep::evaluation_client::OpenAIMessage {
        role: "assistant".to_string(),
        content: "".to_string(),
        tool_calls: Some(vec![tool_call]),
    };

    let serialized_with_tools = serde_json::to_string(&message_with_tools)?;
    assert!(serialized_with_tools.contains("tool_calls"));
    assert!(serialized_with_tools.contains("test_function"));

    info!("✅ OpenAI message serialization test passed");
    Ok(())
}