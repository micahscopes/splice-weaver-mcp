use anyhow::Result;
use rmcp::model::Root;
use std::path::Path;

// Test the path resolution functionality
#[tokio::test]
async fn test_path_resolution() -> Result<()> {
    // Create a test instance
    let binary_manager = std::sync::Arc::new(
        mcp_ast_grep::binary_manager::BinaryManager::new()
            .expect("Failed to create binary manager")
    );
    let tools = mcp_ast_grep::ast_grep_tools::AstGrepTools::new(binary_manager);
    
    // Test 1: Absolute path should work as-is
    let absolute_path = "/tmp/test.rs";
    // Create a temporary file for testing
    tokio::fs::write(absolute_path, "fn main() {}").await?;
    
    let result = tools.resolve_path(absolute_path)?;
    assert_eq!(result, Path::new(absolute_path));
    
    // Clean up
    tokio::fs::remove_file(absolute_path).await.ok();
    
    // Test 2: Relative path with no roots should resolve to current directory
    let relative_path = "src/main.rs";
    let result = tools.resolve_path(relative_path)?;
    let expected = std::env::current_dir()?.join(relative_path);
    assert_eq!(result, expected);
    
    // Test 3: Relative path with roots should resolve to first matching root
    let root1 = Root {
        uri: format!("file://{}", std::env::current_dir()?.display()),
        name: Some("workspace".to_string()),
    };
    let root2 = Root {
        uri: "file:///tmp".to_string(),
        name: Some("temp".to_string()),
    };
    
    tools.set_roots(vec![root1, root2]);
    
    let result = tools.resolve_path(relative_path)?;
    let expected = std::env::current_dir()?.join(relative_path);
    assert_eq!(result, expected);
    
    println!("✅ Path resolution tests passed!");
    Ok(())
}