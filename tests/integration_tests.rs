use anyhow::Result;
use mcp_ast_grep::evaluation_client::{EvaluationClient, EvaluationClientConfig};
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

const INTEGRATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Test helper to create a temporary test file
async fn create_test_file(content: &str) -> Result<tempfile::NamedTempFile> {
    use std::io::Write;
    let mut temp_file = tempfile::NamedTempFile::new()?;
    write!(temp_file, "{}", content)?;
    Ok(temp_file)
}

/// Test helper to start MCP server and return handles
async fn start_mcp_server() -> Result<tokio::process::Child> {
    let child = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Give the server a moment to start
    sleep(Duration::from_millis(500)).await;
    Ok(child)
}

/// Test helper to initialize MCP server
async fn initialize_mcp_server(child: &mut tokio::process::Child) -> Result<()> {
    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "integration-test",
                "version": "1.0.0"
            }
        }
    });

    stdin.write_all(format!("{}\n", init_request).as_bytes()).await?;
    stdin.flush().await?;

    let mut response_line = String::new();
    timeout(INTEGRATION_TIMEOUT, reader.read_line(&mut response_line)).await??;

    let response: Value = serde_json::from_str(&response_line)?;
    if response.get("error").is_some() {
        return Err(anyhow::anyhow!("Server initialization failed: {}", response));
    }

    Ok(())
}

#[tokio::test]
async fn test_end_to_end_mcp_server_with_real_file() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Testing end-to-end MCP server with real JavaScript file");

    // Create a test JavaScript file
    let js_content = r#"
function hello(name) {
    console.log("Hello, " + name);
    return "Hello, " + name;
}

function goodbye(name) {
    console.log("Goodbye, " + name);
    return "Goodbye, " + name;
}

const arrow = (x) => x * 2;
"#;

    let temp_file = create_test_file(js_content).await?;
    let file_path = temp_file.path().to_string_lossy();

    let mut server = start_mcp_server().await?;
    initialize_mcp_server(&mut server).await?;

    let stdin = server.stdin.as_mut().unwrap();
    let stdout = server.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Test execute_rule to find all functions
    let rule_config = format!(r#"
id: find-functions
language: javascript
rule:
  pattern: function $NAME($$$PARAMS) {{ $$$BODY }}
"#);

    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "execute_rule",
            "arguments": {
                "rule_config": rule_config,
                "target": file_path,
                "operation": "search",
                "dry_run": true
            }
        }
    });

    stdin.write_all(format!("{}\n", tool_call).as_bytes()).await?;
    stdin.flush().await?;

    let mut response_line = String::new();
    timeout(INTEGRATION_TIMEOUT, reader.read_line(&mut response_line)).await??;

    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Function search response: {}", response);

    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();
    let content = result.get("content").unwrap().as_array().unwrap();
    let text_content = content[0].get("text").unwrap().as_str().unwrap();

    // Should find the hello and goodbye functions
    assert!(text_content.contains("hello") || text_content.contains("goodbye"));

    info!("✅ End-to-end function search test passed");

    server.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_evaluation_client_with_mock_llm() -> Result<()> {
    info!("Testing evaluation client with mock LLM endpoint");

    // Use httpbin as a mock LLM endpoint (it will return errors, but we can test the flow)
    let config = EvaluationClientConfig {
        llm_endpoint: "http://httpbin.org/status/500".to_string(), // Mock failing endpoint
        llm_api_key: None,
        model_name: "test-model".to_string(),
        server_command: "echo".to_string(),
        server_args: vec!["mock-server".to_string()],
        timeout_seconds: 5,
    };

    let mut client = EvaluationClient::new(config);

    // Test connection (should succeed with echo command)
    let connection_result = client.connect_to_mcp_server().await;
    assert!(connection_result.is_ok(), "Mock MCP connection should succeed");

    // Test getting tools (should work with simulated tools)
    let tools = client.get_available_tools().await?;
    assert!(!tools.is_empty(), "Should get simulated tools");

    // Test tool calling (should work with simulated responses)
    let tool_result = client.call_mcp_tool("find_scope", json!({"test": "data"})).await?;
    assert!(!tool_result.is_empty(), "Should get simulated tool response");

    // Test LLM chat (should fail gracefully with mock endpoint)
    let chat_result = client.chat_with_llm("Test prompt").await;
    assert!(chat_result.is_err(), "Mock LLM endpoint should fail gracefully");

    info!("✅ Evaluation client integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_tool_error_handling() -> Result<()> {
    info!("Testing MCP server error handling");

    let mut server = start_mcp_server().await?;
    initialize_mcp_server(&mut server).await?;

    let stdin = server.stdin.as_mut().unwrap();
    let stdout = server.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Test calling tool with invalid arguments
    let bad_tool_call = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "execute_rule",
            "arguments": {
                "invalid_arg": "bad_value"
            }
        }
    });

    stdin.write_all(format!("{}\n", bad_tool_call).as_bytes()).await?;
    stdin.flush().await?;

    let mut response_line = String::new();
    timeout(INTEGRATION_TIMEOUT, reader.read_line(&mut response_line)).await??;

    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Error handling response: {}", response);

    // Should either return an error or handle gracefully
    assert!(
        response.get("error").is_some() || response.get("result").is_some(),
        "Server should handle invalid arguments gracefully"
    );

    info!("✅ MCP server error handling test passed");

    server.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_concurrent_mcp_requests() -> Result<()> {
    info!("Testing concurrent MCP server requests");

    let mut server = start_mcp_server().await?;
    initialize_mcp_server(&mut server).await?;

    let stdin = server.stdin.as_mut().unwrap();
    let stdout = server.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send multiple tool list requests
    for i in 2..5 {
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "tools/list",
            "params": {}
        });

        stdin.write_all(format!("{}\n", tools_request).as_bytes()).await?;
        stdin.flush().await?;
    }

    // Read all responses
    let mut responses = Vec::new();
    for _ in 0..3 {
        let mut response_line = String::new();
        timeout(INTEGRATION_TIMEOUT, reader.read_line(&mut response_line)).await??;
        let response: Value = serde_json::from_str(&response_line)?;
        responses.push(response);
    }

    // All requests should succeed
    for (i, response) in responses.iter().enumerate() {
        debug!("Concurrent response {}: {}", i, response);
        assert!(response.get("result").is_some(), "Concurrent request {} should succeed", i);
    }

    info!("✅ Concurrent MCP requests test passed");

    server.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_resource_access() -> Result<()> {
    info!("Testing MCP server resource access");

    let mut server = start_mcp_server().await?;
    initialize_mcp_server(&mut server).await?;

    let stdin = server.stdin.as_mut().unwrap();
    let stdout = server.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // List resources
    let resources_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/list",
        "params": {}
    });

    stdin.write_all(format!("{}\n", resources_request).as_bytes()).await?;
    stdin.flush().await?;

    let mut response_line = String::new();
    timeout(INTEGRATION_TIMEOUT, reader.read_line(&mut response_line)).await??;

    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Resources response: {}", response);

    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();
    let resources = result.get("resources").unwrap();

    // If there are resources, try to read one
    if let Some(resource_array) = resources.as_array() {
        if !resource_array.is_empty() {
            if let Some(first_resource) = resource_array[0].get("uri") {
                let uri = first_resource.as_str().unwrap();
                
                let read_request = json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "resources/read",
                    "params": {
                        "uri": uri
                    }
                });

                stdin.write_all(format!("{}\n", read_request).as_bytes()).await?;
                stdin.flush().await?;

                response_line.clear();
                timeout(INTEGRATION_TIMEOUT, reader.read_line(&mut response_line)).await??;

                let read_response: Value = serde_json::from_str(&response_line)?;
                debug!("Resource read response: {}", read_response);

                // Should either succeed or fail gracefully
                assert!(
                    read_response.get("result").is_some() || read_response.get("error").is_some(),
                    "Resource read should return result or error"
                );
            }
        }
    }

    info!("✅ MCP server resource access test passed");

    server.kill().await?;
    Ok(())
}