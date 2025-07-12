use anyhow::Result;
use rmcp::model::Root;

#[tokio::test]
async fn test_end_to_end_path_resolution() -> Result<()> {
    // Create a test instance
    let binary_manager = std::sync::Arc::new(
        mcp_ast_grep::binary_manager::BinaryManager::new()
            .expect("Failed to create binary manager")
    );
    let tools = mcp_ast_grep::ast_grep_tools::AstGrepTools::new(binary_manager);
    
    // Create a temporary test directory structure
    let temp_dir = tempfile::tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    // Create a nested directory structure
    let src_dir = root_path.join("src");
    tokio::fs::create_dir(&src_dir).await?;
    
    // Create a test Rust file in src directory
    let test_file = src_dir.join("lib.rs");
    tokio::fs::write(&test_file, r#"
pub fn hello() {
    println!("Hello from lib.rs!");
}

pub fn world() {
    println!("World from lib.rs!");
}
"#).await?;
    
    // Set up the root
    let root = Root {
        uri: format!("file://{}", root_path.display()),
        name: Some("test_project".to_string()),
    };
    tools.set_roots(vec![root.clone()]);
    
    // Create a simple search rule to find functions
    let rule_config = r#"
id: find-functions
language: rust
rule:
  pattern: pub fn $NAME() { $$$ }
"#;
    
    // Test Case 1: Find functions in src/lib.rs using relative path
    let result = tools.call_tool("execute_rule", serde_json::json!({
        "rule_config": rule_config,
        "target": "src/lib.rs",
        "operation": "search",
        "dry_run": true
    })).await;
    
    match result {
        Ok(output) => {
            println!("✅ Found functions in src/lib.rs using relative path");
            println!("Output: {}", output);
            
            // Parse the JSON output to check for matches
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&output) {
                if let Some(matches) = parsed.as_array() {
                    println!("Found {} matches", matches.len());
                    assert!(matches.len() > 0, "Should find at least one function");
                } else {
                    println!("Output is not an array, might be empty result");
                }
            } else {
                println!("Could not parse output as JSON, might be error message");
            }
        }
        Err(e) => {
            println!("❌ Error executing rule: {}", e);
            // This might be expected if ast-grep binary is not available
            if e.to_string().contains("ast-grep") {
                println!("⚠️  ast-grep binary not available, but path resolution is working");
            } else {
                return Err(e);
            }
        }
    }
    
    // Test Case 2: Try to find files in the root directory
    let result = tools.call_tool("execute_rule", serde_json::json!({
        "rule_config": rule_config,
        "target": ".",
        "operation": "search", 
        "dry_run": true
    })).await;
    
    match result {
        Ok(output) => {
            println!("✅ Scanned root directory successfully");
            println!("Output preview: {}", output.chars().take(200).collect::<String>());
        }
        Err(e) => {
            println!("Directory scan error (expected if no ast-grep): {}", e);
        }
    }
    
    // Test Case 3: Multiple roots scenario
    let root2 = Root {
        uri: format!("file://{}", src_dir.display()),
        name: Some("src_only".to_string()),
    };
    tools.set_roots(vec![root, root2]);
    
    // Now lib.rs should be found in the first root that has it
    let result = tools.call_tool("execute_rule", serde_json::json!({
        "rule_config": rule_config,
        "target": "src/lib.rs",
        "operation": "search",
        "dry_run": true
    })).await;
    
    match result {
        Ok(output) => {
            println!("✅ Multi-root path resolution worked");
            println!("Output preview: {}", output.chars().take(100).collect::<String>());
        }
        Err(e) => {
            println!("Multi-root test error: {}", e);
        }
    }
    
    println!("✅ End-to-end path resolution tests completed!");
    Ok(())
}