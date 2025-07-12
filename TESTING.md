# Testing Guide for MCP ast-grep Server and Evaluation Client

This document describes the comprehensive testing strategy for our MCP ast-grep server and Rust evaluation client.

## Test Structure

```
tests/
├── mcp_server_smoke_tests.rs      # MCP server functionality tests
├── evaluation_client_smoke_tests.rs # Evaluation client unit tests  
├── integration_tests.rs           # End-to-end integration tests
└── test_helpers.rs                # Test utilities and helpers

scripts/
├── quick_smoke_test.sh            # Fast smoke test (< 30 seconds)
└── run_smoke_tests.sh             # Comprehensive test suite
```

## Quick Smoke Tests

Run the quick smoke test for rapid feedback:

```bash
./scripts/quick_smoke_test.sh
```

This tests:
- ✅ Project builds successfully
- ✅ Evaluation client library works
- ✅ MCP server binary is valid
- ✅ Evaluation client binary works
- ✅ Tool simulation functions

## MCP Server Smoke Tests

Test the core MCP server functionality:

```bash
cargo test --test mcp_server_smoke_tests
```

### Test Coverage:
- **Server Startup**: MCP server starts and responds to initialize
- **Tool Listing**: Server lists available tools (find_scope, execute_rule)
- **Tool Execution**: Server executes ast-grep tools with real arguments
- **Resource Access**: Server provides resource listing and reading
- **Prompt Support**: Server lists and provides prompts

### Sample Test Output:
```
✅ MCP server initialized successfully
✅ MCP server tools listed successfully: ["find_scope", "execute_rule"]
✅ MCP server tool call executed successfully
✅ MCP server resources listed successfully
✅ MCP server prompts listed successfully
```

## Evaluation Client Smoke Tests

Test the evaluation client functionality:

```bash
cargo test --test evaluation_client_smoke_tests
```

### Test Coverage:
- **Client Creation**: Configuration and initialization
- **MCP Connection**: Connection to MCP server process
- **Tool Discovery**: Getting available tools from server
- **Tool Calling**: Executing tools with various arguments
- **Conversation Management**: History tracking and reset
- **Message Serialization**: OpenAI API format handling
- **Test Framework**: Default test cases and success criteria

### Sample Test Output:
```
✅ Evaluation client created successfully
✅ Tool listing test passed with 2 tools
✅ Tool calling test passed
✅ Default test cases validation passed
```

## Integration Tests

Test end-to-end functionality:

```bash
cargo test --test integration_tests
```

### Test Coverage:
- **Real File Processing**: MCP server with actual JavaScript files
- **Mock LLM Integration**: Evaluation client with simulated LLM endpoints
- **Error Handling**: Server graceful error responses
- **Concurrent Requests**: Multiple simultaneous MCP requests
- **Resource Access**: File reading through MCP protocol

### Sample Test Output:
```
✅ End-to-end function search test passed
✅ Evaluation client integration test passed
✅ MCP server error handling test passed
✅ Concurrent MCP requests test passed
```

## Test Helpers and Utilities

The `test_helpers.rs` module provides:

### McpServerHandle
```rust
let mut server = McpServerHandle::start().await?;
server.initialize().await?;
let tools = server.list_tools().await?;
let result = server.call_tool("execute_rule", args).await?;
server.shutdown().await?;
```

### TestFile Creation
```rust
let js_file = TestFile::javascript("function test() {}")?;
let py_file = TestFile::python("def test(): pass")?;
let rs_file = TestFile::rust("fn test() {}")?;
```

### Sample Code Snippets
```rust
use test_helpers::sample_code;
let code = sample_code::JAVASCRIPT_FUNCTIONS;
```

### Test Assertions
```rust
use test_helpers::assertions;
assertions::assert_success(&response);
assertions::assert_has_tools(&tools, &["find_scope", "execute_rule"]);
```

## Manual Testing

### Interactive Mode Testing
```bash
./target/debug/evaluation-client --interactive
```

Example interaction:
```
> Find all functions in this code: function hello() { return 'world'; }
🤖: I found 1 function declaration in your JavaScript code...

> tools
Available tools:
  - find_scope: Find containing scope around a position using relational rules
  - execute_rule: Execute ast-grep rule for search, replace, or scan operations
```

### Test Suite Execution
```bash
./target/debug/evaluation-client --run-tests
```

### Single Prompt Testing
```bash
./target/debug/evaluation-client --prompt "Find functions in: function test() {}"
```

## Performance Testing

### Test Response Times
All MCP operations should complete within 10 seconds:
- Server initialization: < 2 seconds
- Tool listing: < 1 second  
- Tool execution: < 5 seconds
- Resource access: < 3 seconds

### Concurrent Load Testing
Integration tests verify the server handles multiple concurrent requests correctly.

## Troubleshooting Tests

### Common Issues:

1. **Server fails to start**
   ```bash
   # Check if port is available
   # Verify cargo build succeeded
   # Check dependencies are installed
   ```

2. **Tool execution timeouts**
   ```bash
   # Increase timeout in test configuration
   # Check ast-grep binary is available
   # Verify file permissions
   ```

3. **Integration test failures**
   ```bash
   # Check network connectivity for mock endpoints
   # Verify temporary file creation permissions
   # Ensure proper cleanup between tests
   ```

## Test Configuration

### Environment Variables
```bash
export RUST_LOG=debug          # Enable debug logging
export MCP_TIMEOUT=30          # Set MCP operation timeout
export TEST_ENDPOINT=localhost # Override test LLM endpoint
```

### Test-only Features
The evaluation client includes simulation modes for testing without real LLM APIs:
- Mock tool responses for isolated testing
- Configurable endpoints for integration testing
- Timeout controls for CI/CD environments

## Continuous Integration

The test suite is designed for CI/CD with:
- Fast smoke tests (< 30 seconds)
- Comprehensive tests (< 5 minutes)
- Clear pass/fail indicators
- Detailed error reporting
- No external dependencies required

## Success Metrics

Tests verify:
- ✅ **Functionality**: All core features work as expected
- ✅ **Reliability**: Error conditions are handled gracefully  
- ✅ **Performance**: Operations complete within expected timeframes
- ✅ **Compatibility**: Works with OpenAI-compatible LLM endpoints
- ✅ **Maintainability**: Test code is clear and well-documented

Run `./scripts/quick_smoke_test.sh` for immediate verification that everything is working correctly.