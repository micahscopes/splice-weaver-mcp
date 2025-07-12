# Snapshot Testing for LLM Responses

## Overview

This snapshot testing infrastructure captures and analyzes LLM responses to detect regressions and track model behavior changes over time. It builds on top of the existing `evaluation_client` and uses the `insta` crate for snapshot management.

## Features

- **Automated Snapshot Capture**: Captures LLM responses with metadata
- **Response Analysis**: Extracts patterns, tools used, and success indicators
- **Regression Detection**: Compares responses across time to detect behavioral changes
- **Multiple Export Formats**: JSON, CSV, and summary reports
- **CLI Management**: Command-line tools for snapshot operations

## Quick Start

### Running Snapshot Tests

```bash
# Run all snapshot tests
cargo test snapshot

# Run specific snapshot test
cargo test test_basic_ast_search_snapshot

# Review snapshot changes
cargo insta review

# Accept all snapshot changes
cargo insta accept
```

### Capturing New Snapshots

```bash
# Capture snapshots for all test cases
cargo run --bin snapshot-manager capture --model gpt-4

# Capture snapshot for specific test
cargo run --bin snapshot-manager capture --test-name "Basic AST search"

# Use custom LLM endpoint
cargo run --bin snapshot-manager capture --endpoint http://localhost:8080/v1
```

## Snapshot Structure

Each snapshot contains:

```yaml
metadata:
  test_name: "Basic AST search"
  model_name: "gpt-4"
  timestamp: 1642684800
  git_commit: "abc123def456"
  prompt_hash: "7f8e9a1b2c3d"

evaluation_result:
  prompt: "Search for all function declarations..."
  response: "I'll help you search for function declarations..."
  duration_ms: 1250
  tool_calls_made: 2
  success: true
  tool_calls:
    - tool_name: "execute_rule"
      arguments: {...}
      result: "Found 3 function declarations"
      success: true

response_analysis:
  contains_tool_calls: true
  contains_code: true
  contains_error: false
  word_count: 45
  sentiment: "Helpful"
  success_indicators: ["found", "successfully"]
  failure_indicators: []
```

## Management Commands

### Compare Snapshots for Regressions

```bash
# Check for regressions in the last 7 days
cargo run --bin snapshot-manager compare --baseline-days 7

# Check specific snapshot directory
cargo run --bin snapshot-manager compare --snapshots-dir custom/snapshots
```

### Generate Summary Reports

```bash
# Generate comprehensive summary
cargo run --bin snapshot-manager summary

# Summary shows:
# - Total snapshots count
# - Test coverage by name
# - Model coverage
# - Average response times
# - Success rates
# - Most used tools
```

### Export Data

```bash
# Export as JSON
cargo run --bin snapshot-manager export --format json --output snapshots.json

# Export as CSV for analysis
cargo run --bin snapshot-manager export --format csv --output snapshots.csv

# Export summary only
cargo run --bin snapshot-manager export --format summary --output summary.yaml
```

### Cleanup Old Snapshots

```bash
# Keep snapshots from last 30 days
cargo run --bin snapshot-manager cleanup --days 30
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Snapshot Tests

on: [push, pull_request]

jobs:
  snapshot-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run snapshot tests
        run: cargo test snapshot
        
      - name: Check for regressions
        run: cargo run --bin snapshot-manager compare --baseline-days 30
        
      - name: Upload snapshots
        uses: actions/upload-artifact@v3
        if: failure()
        with:
          name: snapshot-diffs
          path: tests/snapshots/
```

## Test Cases

The framework includes several built-in test cases:

### Default Test Cases

1. **Basic AST Search**: Tests simple pattern matching
2. **Scope Finding**: Tests scope detection around cursor positions  
3. **Code Refactoring**: Tests code transformation capabilities

### Custom Test Cases

```rust
#[tokio::test]
async fn test_custom_scenario_snapshot() -> Result<()> {
    let test_case = TestCase {
        name: "Custom scenario".to_string(),
        prompt: "Your custom prompt here".to_string(),
        expected_tools: vec!["execute_rule".to_string()],
        success_criteria: |result| {
            result.success && result.response.contains("expected_pattern")
        },
    };
    
    test_llm_response_snapshot(&test_case).await
}
```

## Response Analysis

The system automatically analyzes responses for:

### Content Detection
- **Code blocks**: Detects code patterns and syntax
- **Tool calls**: Tracks which tools were invoked
- **Errors**: Identifies error patterns and failures

### Sentiment Analysis
- **Helpful**: Contains assistance language
- **Positive**: Indicates success or completion
- **Negative**: Contains error or failure language
- **Confused**: Indicates uncertainty or clarification needs
- **Neutral**: No strong indicators

### Success/Failure Indicators
- **Success patterns**: "successfully", "completed", "found", "executed"
- **Failure patterns**: "failed", "error", "unable", "cannot"

## Regression Detection

### Similarity Scoring

Snapshots are compared using a similarity score (0.0 - 1.0):

- **1.0**: Identical responses
- **0.8-0.9**: Minor differences (timing, formatting)
- **0.6-0.7**: Moderate changes (tool usage, response length)
- **<0.6**: Significant changes (potential regression)

### Difference Types

- **ResponseContent**: Core response text changed
- **ToolCalls**: Different tools used or call counts
- **Duration**: Significant timing differences (>1s)
- **Analysis**: Response characteristics changed

## Best Practices

### Writing Effective Tests

1. **Specific Prompts**: Use clear, deterministic prompts
2. **Success Criteria**: Define clear success conditions  
3. **Tool Expectations**: Specify expected tool usage
4. **Error Handling**: Test both success and failure cases

### Managing Snapshots

1. **Regular Reviews**: Review snapshot changes during code reviews
2. **Baseline Management**: Update baselines when behavior intentionally changes
3. **Cleanup**: Remove old snapshots periodically
4. **Documentation**: Document intentional behavior changes

### Monitoring Regressions

1. **Automated Checks**: Run regression detection in CI
2. **Threshold Tuning**: Adjust similarity thresholds based on your needs
3. **Alert Setup**: Set up notifications for detected regressions
4. **Analysis**: Investigate root causes of behavioral changes

## Troubleshooting

### Common Issues

**Snapshots not generated**: 
- Check LLM endpoint connectivity
- Verify model availability
- Check snapshot directory permissions

**Too many false positives**:
- Adjust similarity threshold
- Filter out timing-based differences
- Use more specific test cases

**Missing test coverage**:
- Add more test cases for edge scenarios
- Include error condition testing
- Test different prompt patterns

### Debugging

```bash
# Check snapshot directory
ls -la tests/snapshots/

# Validate snapshot format
cargo run --bin snapshot-manager summary

# Test single evaluation
cargo run --bin evaluation-client --prompt "test prompt"
```

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Cases    │────│ Evaluation Client │────│ LLM Responses   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Snapshot Tests  │────│ Response Analysis │────│ Snapshot Storage│
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Insta Framework │────│ Regression Detection │──│ Reports & Export│
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Future Enhancements

- [ ] Real-time MCP tool integration
- [ ] Advanced similarity algorithms
- [ ] ML-based regression detection
- [ ] Performance benchmarking
- [ ] Multi-model comparison
- [ ] Automated snapshot generation triggers