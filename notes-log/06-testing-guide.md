# MCP AST-Grep Server Testing Guide

This guide provides instructions for testing the MCP ast-grep server implementation, focusing on both direct CLI access and guided workflow functionality.

## Overview

Testing covers:
- **MCP Protocol Compliance**: Tools, resources, and prompts
- **Small LLM Workflows**: Template-based rule generation
- **Direct CLI Access**: Full ast-grep functionality
- **Pattern Libraries**: Pre-built rule examples

## Test Structure

```
test-fixtures/
├── README.md                    # Test overview
├── javascript/                 # JS test files
├── typescript/                 # TS test files  
├── rust/                       # Rust test files
├── patterns/                   # Pattern examples
└── rules/                      # YAML rule configs
```

## Testing MCP Interface

### 1. Tool Testing

Test the two core tools:

```javascript
// Test find_scope tool
{
  "code": "function test() { console.log('hello'); }",
  "language": "javascript", 
  "position": {"line": 1, "column": 20},
  "scope_rule": "pattern: 'function $NAME($$$) { $$$ }'"
}

// Test execute_rule tool  
{
  "rule_config": "rule:\n  pattern: 'console.log($$$)'",
  "target": "test-fixtures/javascript/",
  "operation": "search",
  "dry_run": true
}
```

### 2. Resource Testing

Access pattern libraries and documentation:

```
ast-grep://binary-path        # CLI access path
ast-grep://rule-examples      # Pattern library
ast-grep://node-kinds         # Language node types
```

### 3. Prompt Testing

Generate rules using templates:

```javascript
// Generate scope navigation rule
scope_navigation_rule(
  scope_type: "function",
  target_pattern: "console.log($$$)",
  language: "javascript"
)
```

## Test Categories

### 1. MCP Protocol Compliance
- Verify all tools, resources, and prompts work correctly
- Test error handling and response formats
- Validate JSON-RPC communication

### 2. Small LLM Workflow Testing
- Test template-based rule generation via prompts
- Verify pattern library access via resources
- Measure success rates with 8B parameter models

### 3. Direct CLI Access Testing
- Test full ast-grep functionality via execute_rule
- Verify binary path access via resources
- Test complex rule configurations

## Usage Examples

See test-fixtures/ directory for comprehensive examples across multiple languages.

For complete ast-grep pattern documentation: https://ast-grep.github.io/

## Key Test Areas

1. **Scope Navigation**: Test find_scope tool with various code structures
2. **Rule Execution**: Test execute_rule with search, replace, and scan operations  
3. **Template Generation**: Test prompts for automatic rule creation
4. **Pattern Libraries**: Verify resource access and content quality