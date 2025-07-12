# MCP ast-grep Server

An MCP (Model Context Protocol) server that provides ast-grep functionality to language models, with dual support for both direct CLI access and guided workflows for small LLMs.

## Features

### Tools
- **find_scope**: Universal scope navigation using relational rules
- **execute_rule**: Rule-based operations (search, replace, scan)

### Resources  
- Binary path access for direct CLI usage
- Rule examples and pattern libraries
- Language-specific node type references

### Prompts
- Guided scope navigation rule generation
- Template-based transformation workflows

## Prerequisites

- Rust toolchain
- ast-grep installed (`cargo install ast-grep`)

## Installation

```bash
cargo build --release
```

## Usage

```bash
./target/release/mcp-ast-grep
```

## Design Philosophy

This server implements a **dual approach** to ast-grep access:

1. **Direct Access**: Full CLI power for experienced users and large LLMs
2. **Guided Workflows**: Structured templates and examples for small LLMs

For complete ast-grep documentation, visit: https://ast-grep.github.io/

## License

MIT