# MCP ast-grep Server

An MCP (Model Context Protocol) server that provides ast-grep functionality to language models.

## Features

- **ast_grep_search**: Search for AST patterns in code
- **ast_grep_replace**: Replace AST patterns with new code (supports dry-run mode)
- **ast_grep_scan**: Scan code for potential issues using custom rules

## Prerequisites

- Rust toolchain
- ast-grep installed (`cargo install ast-grep`)

## Installation

```bash
cargo build --release
```

## Usage

The server communicates via JSON-RPC over stdin/stdout:

```bash
./target/release/mcp-ast-grep
```

## Tools

### ast_grep_search
Search for AST patterns in code.

Parameters:
- `pattern`: The AST pattern to search for
- `language`: Programming language (e.g., 'javascript', 'python', 'rust')
- `path`: Path to search in (file or directory)

### ast_grep_replace
Replace AST patterns in code.

Parameters:
- `pattern`: The AST pattern to search for
- `replacement`: The replacement pattern
- `language`: Programming language
- `path`: Path to search in
- `dry_run`: If true, show what would be changed without applying changes (default: true)

### ast_grep_scan
Scan code for potential issues using ast-grep rules.

Parameters:
- `rule`: The ast-grep rule to apply (YAML format)
- `path`: Path to scan (file or directory)

## License

MIT