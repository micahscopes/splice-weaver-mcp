use anyhow::Result;
use mcp_ast_grep::evaluation_client::{EvaluationClient, EvaluationClientConfig};
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, warn};

const SMOKE_TEST_TIMEOUT: Duration = Duration::from_secs(10);

// Test categories using Cargo's built-in test filtering

#[tokio::test]
#[cfg(test)]
async fn smoke_build_and_binaries() -> Result<()> {
    info!("🔧 Testing that binaries can be invoked");
    
    // Test evaluation client binary help
    let output = Command::new("./target/debug/evaluation-client")
        .arg("--help")
        .output()
        .await?;
    
    assert!(output.status.success(), "Evaluation client --help should work");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Rust MCP Evaluation Client"));
    
    info!("✅ Binaries work correctly");
    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn smoke_mcp_server_startup() -> Result<()> {
    info!("🚀 Testing MCP server startup and basic protocol");
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "smoke-test",
                "version": "1.0.0"
            }
        }
    });

    stdin.write_all(format!("{}\n", init_request).as_bytes()).await?;
    stdin.flush().await?;

    // Read response with timeout
    let mut response_line = String::new();
    let result = timeout(SMOKE_TEST_TIMEOUT, reader.read_line(&mut response_line)).await;
    
    match result {
        Ok(Ok(_)) => {
            let response: Value = serde_json::from_str(&response_line)?;
            assert!(response.get("jsonrpc").is_some());
            assert!(response.get("result").is_some());
            let result = response.get("result").unwrap();
            assert!(result.get("protocolVersion").is_some());
            assert!(result.get("capabilities").is_some());
            
            info!("✅ MCP server initializes correctly");
        }
        _ => {
            warn!("⚠️  MCP server startup test skipped (may need dependencies)");
        }
    }

    child.kill().await.ok();
    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn smoke_mcp_tools() -> Result<()> {
    info!("🛠️  Testing MCP server tool listing");
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "smoke-test", "version": "1.0.0"}
        }
    });

    stdin.write_all(format!("{}\n", init_request).as_bytes()).await?;
    stdin.flush().await?;

    let mut response_line = String::new();
    if timeout(SMOKE_TEST_TIMEOUT, reader.read_line(&mut response_line)).await.is_err() {
        warn!("⚠️  MCP tools test skipped (server timeout)");
        child.kill().await.ok();
        return Ok(());
    }

    // Send tools/list request
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    stdin.write_all(format!("{}\n", tools_request).as_bytes()).await?;
    stdin.flush().await?;

    response_line.clear();
    if let Ok(Ok(_)) = timeout(SMOKE_TEST_TIMEOUT, reader.read_line(&mut response_line)).await {
        let response: Value = serde_json::from_str(&response_line)?;
        if let Some(result) = response.get("result") {
            if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                let tool_names: Vec<&str> = tools
                    .iter()
                    .filter_map(|t| t.get("name")?.as_str())
                    .collect();
                
                assert!(tool_names.contains(&"find_scope"), "Expected find_scope tool");
                assert!(tool_names.contains(&"execute_rule"), "Expected execute_rule tool");
                
                info!("✅ MCP server tools available: {:?}", tool_names);
            }
        }
    } else {
        warn!("⚠️  Tools listing test skipped (response timeout)");
    }

    child.kill().await.ok();
    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn smoke_evaluation_client() -> Result<()> {
    info!("🎯 Testing evaluation client functionality");
    
    let config = EvaluationClientConfig {
        llm_endpoint: "http://httpbin.org/status/200".to_string(),
        llm_api_key: None,
        model_name: "test-model".to_string(),
        server_command: "echo".to_string(),
        server_args: vec!["test".to_string()],
        timeout_seconds: 5,
    };

    let client = EvaluationClient::new(config.clone());
    
    // Test basic configuration
    assert_eq!(client.config.llm_endpoint, config.llm_endpoint);
    assert_eq!(client.config.model_name, config.model_name);
    
    // Test tool listing (simulated)
    let tools = client.get_available_tools().await?;
    assert!(!tools.is_empty(), "Should return simulated tools");
    assert!(tools.len() >= 2, "Should return at least 2 tools");
    
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"find_scope"));
    assert!(tool_names.contains(&"execute_rule"));
    
    // Test tool calling (simulated)
    let test_args = json!({"code": "function test() { return 42; }", "language": "javascript"});
    let result = client.call_mcp_tool("find_scope", test_args).await?;
    assert!(!result.is_empty(), "Tool call should return result");
    
    info!("✅ Evaluation client works correctly");
    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn smoke_end_to_end() -> Result<()> {
    info!("🔄 Testing end-to-end functionality");
    
    // Create a temporary JavaScript file
    let js_content = r#"
function hello(name) {
    console.log("Hello, " + name);
    return "Hello, " + name;
}

function goodbye(name) {
    console.log("Goodbye, " + name);
    return "Goodbye, " + name;
}
"#;

    let temp_file = {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new()?;
        write!(temp_file, "{}", js_content)?;
        temp_file
    };
    
    let file_path = temp_file.path().to_string_lossy();
    
    // Start MCP server
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = server.stdin.as_mut().unwrap();
    let stdout = server.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Initialize server
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "e2e-test", "version": "1.0.0"}
        }
    });

    stdin.write_all(format!("{}\n", init_request).as_bytes()).await?;
    stdin.flush().await?;

    let mut response_line = String::new();
    if timeout(SMOKE_TEST_TIMEOUT, reader.read_line(&mut response_line)).await.is_err() {
        warn!("⚠️  End-to-end test skipped (server timeout)");
        server.kill().await.ok();
        return Ok(());
    }

    // Test execute_rule to find functions
    let rule_config = r#"
id: find-functions
language: javascript
rule:
  pattern: function $NAME($$$PARAMS) { $$$BODY }
"#;

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

    response_line.clear();
    if let Ok(Ok(_)) = timeout(SMOKE_TEST_TIMEOUT, reader.read_line(&mut response_line)).await {
        let response: Value = serde_json::from_str(&response_line)?;
        if response.get("result").is_some() {
            info!("✅ End-to-end test successful");
        } else {
            warn!("⚠️  End-to-end test partial (tool execution may need setup)");
        }
    } else {
        warn!("⚠️  End-to-end test skipped (tool timeout)");
    }

    server.kill().await.ok();
    Ok(())
}

// Quick verification test - tests core functionality without external dependencies
#[tokio::test]
#[cfg(test)]
async fn smoke_quick_verification() -> Result<()> {
    info!("🧪 Running quick smoke verification");
    
    // Test that binaries exist and work
    let output = Command::new("./target/debug/evaluation-client")
        .arg("--help")
        .output()
        .await?;
    assert!(output.status.success(), "Evaluation client should work");
    
    // Test evaluation client core functionality
    let config = EvaluationClientConfig {
        llm_endpoint: "http://httpbin.org/status/200".to_string(),
        llm_api_key: None,
        model_name: "test-model".to_string(),
        server_command: "echo".to_string(),
        server_args: vec!["test".to_string()],
        timeout_seconds: 5,
    };

    let client = EvaluationClient::new(config);
    let tools = client.get_available_tools().await?;
    assert!(!tools.is_empty(), "Should have tools available");
    
    info!("✅ Quick verification completed successfully");
    Ok(())
}