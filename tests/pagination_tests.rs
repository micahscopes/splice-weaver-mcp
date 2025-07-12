use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use splice_weaver_mcp::ast_grep_tools::AstGrepTools;
use splice_weaver_mcp::binary_manager::BinaryManager;

#[tokio::test]
async fn test_search_examples_pagination_basic() -> Result<()> {
    let binary_manager = Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
    let tools = AstGrepTools::new(binary_manager);

    // Test first page
    let args = json!({
        "query": "function",
        "language": "any",
        "limit": 3,
        "offset": 0
    });

    let result = tools.call_tool("search_examples", args).await?;
    
    // Should contain pagination info
    assert!(result.contains("Found"));
    assert!(result.contains("total examples"));
    assert!(result.contains("page 1-"));
    
    // Test second page  
    let args = json!({
        "query": "function",
        "language": "any", 
        "limit": 3,
        "offset": 3
    });

    let result = tools.call_tool("search_examples", args).await?;
    
    // Should show different page numbers
    assert!(result.contains("page 4-"));
    
    Ok(())
}

#[tokio::test]
async fn test_search_examples_pagination_with_more_indicator() -> Result<()> {
    let binary_manager = Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
    let tools = AstGrepTools::new(binary_manager);

    // Test with small limit to ensure has_more indicator appears
    let args = json!({
        "query": "example",
        "language": "any",
        "limit": 2,
        "offset": 0
    });

    let result = tools.call_tool("search_examples", args).await?;
    
    // Should show more results available if there are more than 2 examples
    if result.contains("total examples") {
        let total_line = result.lines()
            .find(|line| line.contains("total examples"))
            .unwrap_or("");
        
        if total_line.contains("of ") {
            let parts: Vec<&str> = total_line.split(" of ").collect();
            if parts.len() > 1 {
                let total_str = parts[1].split(" ").next().unwrap_or("0");
                if let Ok(total) = total_str.parse::<i32>() {
                    if total > 2 {
                        assert!(result.contains("More results available"));
                        assert!(result.contains("offset: 2"));
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_similarity_search_pagination_basic() -> Result<()> {
    let binary_manager = Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
    let tools = AstGrepTools::new(binary_manager);

    // Test first page
    let args = json!({
        "pattern": "console.log",
        "limit": 3,
        "offset": 0
    });

    let result = tools.call_tool("similarity_search", args).await?;
    
    // Should contain pagination info
    assert!(result.contains("similar patterns") || result.contains("No similar patterns"));
    
    Ok(())
}

#[tokio::test]
async fn test_suggest_examples_pagination_basic() -> Result<()> {
    let binary_manager = Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
    let tools = AstGrepTools::new(binary_manager);

    // Test first page
    let args = json!({
        "description": "find all function declarations",
        "language": "any",
        "limit": 2,
        "offset": 0
    });

    let result = tools.call_tool("suggest_examples", args).await?;
    
    // Should contain pagination info or no results message
    assert!(result.contains("potentially relevant examples") || result.contains("No relevant examples"));
    
    Ok(())
}

#[tokio::test]
async fn test_pagination_edge_cases() -> Result<()> {
    let binary_manager = Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
    let tools = AstGrepTools::new(binary_manager);

    // Test offset beyond results
    let args = json!({
        "query": "nonexistentquerythatshouldhavenooutputs",
        "language": "any",
        "limit": 10,
        "offset": 1000
    });

    let result = tools.call_tool("search_examples", args).await?;
    assert!(result.contains("No") && (result.contains("more examples") || result.contains("examples found")));

    // Test limit of 0 (should still work)
    let args = json!({
        "query": "function",
        "language": "any", 
        "limit": 0,
        "offset": 0
    });

    let result = tools.call_tool("search_examples", args).await?;
    // Should handle gracefully
    assert!(!result.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_pagination_consistency_across_pages() -> Result<()> {
    let binary_manager = Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
    let tools = AstGrepTools::new(binary_manager);

    // Get first page
    let args1 = json!({
        "query": "function",
        "language": "any",
        "limit": 2,
        "offset": 0
    });

    let result1 = tools.call_tool("search_examples", args1).await?;
    
    // Get second page  
    let args2 = json!({
        "query": "function",
        "language": "any",
        "limit": 2,
        "offset": 2
    });

    let result2 = tools.call_tool("search_examples", args2).await?;

    // Results should be different (unless there are fewer than 3 total results)
    if result1.contains("total examples") && result2.contains("total examples") {
        // Extract total count from both results to verify consistency
        let extract_total = |result: &str| -> Option<i32> {
            result.lines()
                .find(|line| line.contains("total examples"))
                .and_then(|line| {
                    line.split(" of ").nth(1)
                        .and_then(|part| part.split(" ").next())
                        .and_then(|total_str| total_str.parse().ok())
                })
        };

        if let (Some(total1), Some(total2)) = (extract_total(&result1), extract_total(&result2)) {
            assert_eq!(total1, total2, "Total count should be consistent across pages");
            
            if total1 > 2 {
                // If there are more than 2 results, pages should show different content
                assert_ne!(result1, result2, "Different pages should show different results");
            }
        }
    }

    Ok(())
}