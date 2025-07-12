# MCP ast-grep Server

An MCP (Model Context Protocol) server that provides ast-grep functionality to language models with **automatic binary bundling** and dual support for both direct CLI access and guided workflows.

## Features

### Core Tools
- **find_scope**: Universal scope navigation using relational rules
- **execute_rule**: Rule-based operations (search, replace, scan)

### Binary Bundling
- **Automatic Download**: ast-grep binary (v0.38.7) is automatically downloaded if not found
- **Multi-Platform Support**: Supports Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), Windows
- **System Binary Detection**: Uses system-installed ast-grep if available, otherwise downloads bundled version

### MCP Resources
- **ast-grep://binary-path**: Get the path to the ast-grep executable
- **ast-grep://cli-reference**: Complete CLI documentation with bundled binary path
- **ast-grep://rule-examples**: YAML rule configuration examples
- **ast-grep://relational-patterns**: Examples of relational rules for scope navigation
- **ast-grep://node-kinds**: Tree-sitter node types by language

### MCP Prompts
- **scope_navigation_rule**: Generate YAML rules to find specific scope types
- **transform_in_scope**: Generate YAML rules to transform code within scopes

## Prerequisites

- Rust toolchain
- **No ast-grep installation required** - the server bundles the binary automatically

## Installation

```bash
# From crates.io (once published)
cargo install mcp-ast-grep

# Or from Git
cargo install --git https://github.com/your-org/mcp-ast-grep

# For development
cargo build --release
```

## Usage

### As MCP Server

Add to your MCP client configuration:
```json
{
  "mcp": {
    "servers": {
      "ast-grep": {
        "command": "mcp-ast-grep",
        "args": []
      }
    }
  }
}
```

### Direct Usage

```bash
# If installed via cargo install
mcp-ast-grep

# If built manually
./target/release/mcp-ast-grep
```

### Binary Management

The server automatically manages the ast-grep binary:

1. **System Check**: First checks if ast-grep is installed on the system
2. **Auto-Download**: If not found, downloads the appropriate binary for your platform
3. **Bundled Storage**: Stores the binary in `./bundled_binaries/` relative to the executable

Supported platforms:
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- Linux ARM64 (`aarch64-unknown-linux-gnu`) 
- macOS Intel (`x86_64-apple-darwin`)
- macOS Apple Silicon (`aarch64-apple-darwin`)
- Windows x64 (`x86_64-pc-windows-msvc`)
- Windows x86 (`i686-pc-windows-msvc`)

## Design Philosophy

This server implements a **dual approach** to ast-grep access:

1. **Direct Access**: Full CLI power for experienced users and large LLMs
2. **Guided Workflows**: Structured templates and examples for small LLMs

For complete ast-grep documentation, visit: https://ast-grep.github.io/

## License

MIT