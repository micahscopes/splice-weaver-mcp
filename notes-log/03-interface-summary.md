# MCP AST-Grep Interface - Implementation Summary

## Core Architecture

A **dual-approach MCP interface** for ast-grep that supports both direct CLI access and guided workflows for small LLMs.

## Interface Components

### Tools (2)
- **find_scope**: Navigate to containing scopes using relational rules
- **execute_rule**: Execute ast-grep operations with full CLI power

### Resources (5)
- `ast-grep://binary-path`: Direct CLI access
- `ast-grep://cli-reference`: Command documentation  
- `ast-grep://rule-examples`: Pattern library
- `ast-grep://relational-patterns`: Scope navigation examples
- `ast-grep://node-kinds`: Language node types

### Prompts (2)
- **scope_navigation_rule**: Generate scope-finding rules
- **transform_in_scope**: Generate transformation rules

## Dual Approach Benefits

### Direct Access (Large LLMs)
- Full CLI power via `execute_rule`
- Complete documentation access
- No abstraction overhead

### Guided Workflows (Small LLMs)
- Template-based rule generation
- Pre-built pattern libraries
- Structured scope navigation

## Key Innovation: Relational Rules

Leverages ast-grep's relational rule system for precise scope-based refactoring:

```yaml
# Find patterns within specific scopes
rule:
  all:
    - pattern: "console.log($$$)"
    - inside:
        pattern: "function $NAME($$$) { $$$ }"
    - not:
        inside:
          pattern: "function $INNER($$$) { $$$ }"
```

## Status

- ✅ Compiles successfully
- ✅ MCP protocol compliant  
- ✅ Minimal interface (2 tools, 5 resources, 2 prompts)
- ✅ Ready for evaluation with small LLMs

For complete ast-grep documentation: https://ast-grep.github.io/