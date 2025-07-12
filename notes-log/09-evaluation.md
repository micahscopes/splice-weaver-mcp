# Rust MCP Evaluation Client

A Rust-based evaluation client that connects to the ast-grep MCP server and provides LLMs with real tool access for evaluation and benchmarking.

## Architecture

- **MCP Client**: Rust client that connects to our MCP server using child process transport
- **LLM Integration**: OpenAI-compatible API calls to any endpoint (LM Studio, vLLM, etc.)
- **Tool Routing**: Routes LLM tool requests through MCP protocol to our server
- **Evaluation Framework**: Enables benchmarking and snapshot testing

## Usage

### Build the project
```bash
cargo build
```

### Run help
```bash
./target/debug/evaluation-client --help
```

### Interactive mode
```bash
./target/debug/evaluation-client --interactive
```

### Run built-in test suite
```bash
./target/debug/evaluation-client --run-tests
```

### Single prompt evaluation
```bash
./target/debug/evaluation-client --prompt "Find all functions in this code: function test() {}"
```

### Custom LLM endpoint
```bash
./target/debug/evaluation-client --endpoint http://localhost:8080/v1 --model "custom-model" --interactive
```

## Configuration

The evaluation client supports these configuration options:

- `--endpoint`: LLM API endpoint (default: http://localhost:1234/v1)
- `--api-key`: API key for LLM endpoint (optional for local endpoints)
- `--model`: Model name to use (default: gpt-3.5-turbo)
- `--server-cmd`: Command to start MCP server (default: cargo)
- `--server-args`: Arguments for MCP server command (default: "run --bin mcp-ast-grep")

## Available Tools

The evaluation client provides access to these ast-grep tools through the MCP interface:

1. **find_scope**: Find containing scope around a position using relational rules
2. **execute_rule**: Execute ast-grep rule for search, replace, or scan operations

## Test Cases

Built-in test cases include:

1. **Basic AST search**: Search for function declarations in JavaScript code
2. **Scope finding**: Find containing scope around a specific position
3. **Code refactoring**: Replace console.log statements with logger.info

## Example Workflow

1. Start the evaluation client in interactive mode
2. Type prompts that require code analysis
3. The client will:
   - Send your prompt to the LLM
   - LLM decides which tools to call
   - Client executes the tools via MCP
   - Results are sent back to the LLM
   - LLM provides final response

## Dependencies

- `rmcp`: MCP protocol implementation
- `reqwest`: HTTP client for LLM API calls
- `tokio`: Async runtime
- `clap`: Command-line argument parsing
- `serde_json`: JSON serialization

## Future Enhancements

- [ ] Real MCP client connection (currently using simulated tools)
- [ ] Benchmark result persistence
- [ ] Custom test case configuration
- [ ] Performance metrics collection
- [ ] Support for more LLM providers