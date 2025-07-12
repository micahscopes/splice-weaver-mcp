use anyhow::Result;
use insta::{assert_yaml_snapshot, with_settings};
use mcp_ast_grep::evaluation_client::{
    create_default_test_cases, EvaluationClient, EvaluationClientConfig, ResponseSnapshot,
    SnapshotMetadata, TestCase,
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Creates a snapshot test configuration with reproducible settings
fn create_test_config() -> EvaluationClientConfig {
    EvaluationClientConfig {
        llm_endpoint: "http://localhost:1234/v1".to_string(),
        llm_api_key: None,
        model_name: "test-model".to_string(),
        server_command: "echo".to_string(),
        server_args: vec!["mock-server".to_string()],
        timeout_seconds: 10,
    }
}

/// Generates a hash for a prompt to ensure consistent snapshot naming
fn hash_prompt(prompt: &str) -> String {
    let mut hasher = DefaultHasher::new();
    prompt.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Creates snapshot metadata for a test case
fn create_snapshot_metadata(test_name: &str, model_name: &str, prompt: &str) -> SnapshotMetadata {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    SnapshotMetadata {
        test_name: test_name.to_string(),
        model_name: model_name.to_string(),
        timestamp,
        git_commit: std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string()),
        prompt_hash: hash_prompt(prompt),
    }
}

/// Core snapshot testing function
async fn test_llm_response_snapshot(test_case: &TestCase) -> Result<()> {
    let config = create_test_config();
    let mut client = EvaluationClient::new(config.clone());
    
    // Skip actual server connection for testing
    // client.connect_to_mcp_server().await?;
    
    let result = client.evaluate_prompt(&test_case.prompt).await?;
    let analysis = client.analyze_response(&result.response, &result);
    
    let metadata = create_snapshot_metadata(
        &test_case.name,
        &config.model_name,
        &test_case.prompt,
    );
    
    let snapshot = ResponseSnapshot {
        metadata,
        evaluation_result: result,
        response_analysis: analysis,
    };
    
    // Use insta to create/compare snapshots
    let snapshot_name = format!("{}_{}", test_case.name.replace(" ", "_"), hash_prompt(&test_case.prompt));
    
    with_settings!({
        snapshot_path => "snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_yaml_snapshot!(snapshot_name, snapshot);
    });
    
    Ok(())
}

#[tokio::test]
async fn test_basic_ast_search_snapshot() -> Result<()> {
    let test_cases = create_default_test_cases();
    let basic_search = test_cases.iter()
        .find(|tc| tc.name == "Basic AST search")
        .expect("Basic AST search test case should exist");
    
    test_llm_response_snapshot(basic_search).await
}

#[tokio::test]
async fn test_scope_finding_snapshot() -> Result<()> {
    let test_cases = create_default_test_cases();
    let scope_finding = test_cases.iter()
        .find(|tc| tc.name == "Scope finding")
        .expect("Scope finding test case should exist");
    
    test_llm_response_snapshot(scope_finding).await
}

#[tokio::test]
async fn test_code_refactoring_snapshot() -> Result<()> {
    let test_cases = create_default_test_cases();
    let code_refactoring = test_cases.iter()
        .find(|tc| tc.name == "Code refactoring")
        .expect("Code refactoring test case should exist");
    
    test_llm_response_snapshot(code_refactoring).await
}

/// Test custom prompts that aren't in the default test suite
#[tokio::test]
async fn test_error_handling_snapshot() -> Result<()> {
    let test_case = TestCase {
        name: "Error handling detection".to_string(),
        prompt: "Find all try-catch blocks and error handling patterns in this JavaScript code: try { riskyOperation(); } catch (e) { console.error(e); }".to_string(),
        expected_tools: vec!["execute_rule".to_string()],
        success_criteria: |result| {
            result.success && result.response.contains("try") && result.response.contains("catch")
        },
    };
    
    test_llm_response_snapshot(&test_case).await
}

#[tokio::test] 
async fn test_complex_query_snapshot() -> Result<()> {
    let test_case = TestCase {
        name: "Complex multi-step query".to_string(),
        prompt: "First find all function declarations, then analyze their parameters, and finally suggest refactoring opportunities for this code: function calculateTotal(items) { let sum = 0; for (let item of items) { sum += item.price; } return sum; }".to_string(),
        expected_tools: vec!["execute_rule".to_string(), "find_scope".to_string()],
        success_criteria: |result| {
            result.success && result.tool_calls_made >= 2
        },
    };
    
    test_llm_response_snapshot(&test_case).await
}

/// Utility function to run all snapshot tests in batch
#[tokio::test] 
async fn test_batch_snapshot_generation() -> Result<()> {
    let test_cases = create_default_test_cases();
    
    for test_case in &test_cases {
        println!("Processing snapshot for: {}", test_case.name);
        if let Err(e) = test_llm_response_snapshot(test_case).await {
            eprintln!("Failed to create snapshot for {}: {}", test_case.name, e);
        }
    }
    
    Ok(())
}

/// Test for detecting response variations across multiple runs
#[tokio::test]
async fn test_response_consistency() -> Result<()> {
    let test_case = TestCase {
        name: "Consistency check".to_string(),
        prompt: "Count the number of functions in this code: function a() {} function b() {} function c() {}".to_string(),
        expected_tools: vec!["execute_rule".to_string()],
        success_criteria: |result| result.success,
    };
    
    let config = create_test_config();
    let mut results = Vec::new();
    
    // Run the same prompt multiple times to check for consistency
    for i in 0..3 {
        let mut client = EvaluationClient::new(config.clone());
        let result = client.evaluate_prompt(&test_case.prompt).await?;
        let analysis = client.analyze_response(&result.response, &result);
        
        let metadata = create_snapshot_metadata(
            &format!("{}_run_{}", test_case.name, i),
            &config.model_name,
            &test_case.prompt,
        );
        
        let snapshot = ResponseSnapshot {
            metadata,
            evaluation_result: result,
            response_analysis: analysis,
        };
        
        results.push(snapshot);
    }
    
    // Create a comparison snapshot showing all runs
    let snapshot_name = format!("consistency_check_{}", hash_prompt(&test_case.prompt));
    
    with_settings!({
        snapshot_path => "snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_yaml_snapshot!(snapshot_name, results);
    });
    
    Ok(())
}

/// Integration test that validates snapshot structure
#[test]
fn test_snapshot_structure_validation() {
    use mcp_ast_grep::evaluation_client::*;
    
    // Create a sample snapshot to validate structure
    let metadata = SnapshotMetadata {
        test_name: "structure_test".to_string(),
        model_name: "test-model".to_string(),
        timestamp: 1234567890,
        git_commit: Some("abc123".to_string()),
        prompt_hash: "def456".to_string(),
    };
    
    let result = EvaluationResult {
        prompt: "test prompt".to_string(),
        response: "test response".to_string(),
        duration_ms: 100,
        tool_calls_made: 1,
        success: true,
        timestamp: 1234567890,
        model_name: "test-model".to_string(),
        tool_calls: vec![ToolCallResult {
            tool_name: "test_tool".to_string(),
            arguments: serde_json::json!({"test": "value"}),
            result: "test result".to_string(),
            success: true,
            duration_ms: 50,
        }],
        conversation_length: 3,
    };
    
    let analysis = ResponseAnalysis {
        contains_tool_calls: true,
        contains_code: false,
        contains_error: false,
        word_count: 2,
        sentiment: ResponseSentiment::Neutral,
        success_indicators: vec!["test".to_string()],
        failure_indicators: vec![],
    };
    
    let snapshot = ResponseSnapshot {
        metadata,
        evaluation_result: result,
        response_analysis: analysis,
    };
    
    // Validate that the snapshot can be serialized/deserialized
    let yaml = serde_yaml::to_string(&snapshot).expect("Should serialize to YAML");
    let _deserialized: ResponseSnapshot = serde_yaml::from_str(&yaml)
        .expect("Should deserialize from YAML");
    
    println!("Snapshot structure validation passed");
}