use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info};

const MCP_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::test]
async fn test_mcp_server_starts_and_responds() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Testing MCP server startup and basic response");

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

    let request_line = format!("{}\n", init_request);
    stdin.write_all(request_line.as_bytes()).await?;
    stdin.flush().await?;

    // Read response with timeout
    let mut response_line = String::new();
    let result = timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await;
    
    match result {
        Ok(Ok(_)) => {
            debug!("Got response: {}", response_line);
            let response: Value = serde_json::from_str(&response_line)?;
            
            // Verify it's a valid JSON-RPC response
            assert!(response.get("jsonrpc").is_some());
            assert!(response.get("id").is_some());
            assert!(response.get("result").is_some());
            
            let result = response.get("result").unwrap();
            assert!(result.get("protocolVersion").is_some());
            assert!(result.get("capabilities").is_some());
            assert!(result.get("serverInfo").is_some());
            
            info!("✅ MCP server initialized successfully");
        }
        Ok(Err(e)) => {
            panic!("IO error reading from server: {}", e);
        }
        Err(_) => {
            panic!("Timeout waiting for server response");
        }
    }

    // Clean shutdown
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_lists_tools() -> Result<()> {
    info!("Testing MCP server tool listing");

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
            "clientInfo": {
                "name": "smoke-test",
                "version": "1.0.0"
            }
        }
    });

    stdin.write_all(format!("{}\n", init_request).as_bytes()).await?;
    stdin.flush().await?;

    // Read init response
    let mut response_line = String::new();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;

    // Send tools/list request
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    stdin.write_all(format!("{}\n", tools_request).as_bytes()).await?;
    stdin.flush().await?;

    // Read tools response
    response_line.clear();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;
    
    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Tools response: {}", response);
    
    let result = response.get("result").unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();
    
    // Verify we have the expected tools
    assert!(tools.len() >= 2, "Expected at least 2 tools");
    
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name")?.as_str())
        .collect();
    
    assert!(tool_names.contains(&"find_scope"), "Expected find_scope tool");
    assert!(tool_names.contains(&"execute_rule"), "Expected execute_rule tool");
    
    info!("✅ MCP server tools listed successfully: {:?}", tool_names);

    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_call_tool() -> Result<()> {
    info!("Testing MCP server tool execution");

    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Initialize
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

    let mut response_line = String::new();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;

    // Call execute_rule tool
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "execute_rule",
            "arguments": {
                "rule_config": "id: test-rule\nlanguage: javascript\nrule:\n  pattern: function $NAME() { $$$ }",
                "target": "/tmp/test.js",
                "operation": "search",
                "dry_run": true
            }
        }
    });

    stdin.write_all(format!("{}\n", tool_call).as_bytes()).await?;
    stdin.flush().await?;

    response_line.clear();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;
    
    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Tool call response: {}", response);
    
    // Verify the tool call succeeded (even if no matches found)
    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();
    assert!(result.get("content").is_some());
    
    info!("✅ MCP server tool call executed successfully");

    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_resources() -> Result<()> {
    info!("Testing MCP server resource listing");

    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Initialize
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

    let mut response_line = String::new();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;

    // List resources
    let resources_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/list",
        "params": {}
    });

    stdin.write_all(format!("{}\n", resources_request).as_bytes()).await?;
    stdin.flush().await?;

    response_line.clear();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;
    
    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Resources response: {}", response);
    
    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();
    assert!(result.get("resources").is_some());
    
    info!("✅ MCP server resources listed successfully");

    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_prompts() -> Result<()> {
    info!("Testing MCP server prompt listing");

    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "mcp-ast-grep"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(stdout);

    // Initialize
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

    let mut response_line = String::new();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;

    // List prompts
    let prompts_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "prompts/list",
        "params": {}
    });

    stdin.write_all(format!("{}\n", prompts_request).as_bytes()).await?;
    stdin.flush().await?;

    response_line.clear();
    timeout(MCP_TIMEOUT, reader.read_line(&mut response_line)).await??;
    
    let response: Value = serde_json::from_str(&response_line)?;
    debug!("Prompts response: {}", response);
    
    assert!(response.get("result").is_some());
    let result = response.get("result").unwrap();
    assert!(result.get("prompts").is_some());
    
    info!("✅ MCP server prompts listed successfully");

    child.kill().await?;
    Ok(())
}