use anyhow::Result;
use mcp_ast_grep::evaluation_client::{EvaluationClient, EvaluationClientConfig, EvaluationSuite, create_default_test_cases};
use tracing::info;

#[tokio::test]
async fn test_evaluation_suite_creation() -> Result<()> {
    info!("Testing evaluation suite with default test cases");
    
    let config = EvaluationClientConfig::default();
    let mut suite = EvaluationSuite::new(config);
    
    // Add all default test cases
    for test_case in create_default_test_cases() {
        suite.add_test_case(test_case);
    }
    
    info!("✅ Evaluation suite created with test cases");
    Ok(())
}

#[tokio::test]
async fn test_evaluation_client_full_workflow() -> Result<()> {
    info!("Testing evaluation client full workflow");
    
    let config = EvaluationClientConfig {
        llm_endpoint: "http://httpbin.org/status/200".to_string(),
        llm_api_key: None,
        model_name: "test-model".to_string(),
        server_command: "echo".to_string(),
        server_args: vec!["mock".to_string()],
        timeout_seconds: 5,
    };
    
    let mut client = EvaluationClient::new(config);
    
    // Test connection
    let connection_result = client.connect_to_mcp_server().await;
    assert!(connection_result.is_ok(), "Should connect to mock server");
    
    // Test tool discovery
    let tools = client.get_available_tools().await?;
    assert!(!tools.is_empty(), "Should discover tools");
    
    // Test tool execution
    let result = client.call_mcp_tool("find_scope", serde_json::json!({
        "code": "function test() {}",
        "language": "javascript"
    })).await?;
    assert!(!result.is_empty(), "Should get tool result");
    
    // Test conversation management
    assert_eq!(client.conversation_history.len(), 0, "Should start with empty history");
    client.reset_conversation();
    assert_eq!(client.conversation_history.len(), 0, "Should remain empty after reset");
    
    info!("✅ Full workflow test completed");
    Ok(())
}

#[tokio::test]
async fn test_default_test_cases_structure() -> Result<()> {
    info!("Testing default test cases are well-formed");
    
    let test_cases = create_default_test_cases();
    assert!(test_cases.len() >= 3, "Should have at least 3 test cases");
    
    for (i, test_case) in test_cases.iter().enumerate() {
        assert!(!test_case.name.is_empty(), "Test case {} should have name", i);
        assert!(!test_case.prompt.is_empty(), "Test case {} should have prompt", i);
        assert!(!test_case.expected_tools.is_empty(), "Test case {} should expect tools", i);
        
        // Test success criteria function works
        let mock_result = mcp_ast_grep::evaluation_client::EvaluationResult {
            prompt: test_case.prompt.clone(),
            response: "Mock response".to_string(),
            duration_ms: 100,
            tool_calls_made: 1,
            success: true,
        };
        
        // Success criteria should be callable without panicking
        let _ = (test_case.success_criteria)(&mock_result);
    }
    
    info!("✅ Default test cases are well-formed");
    Ok(())
}