# MCP Non-Tool Features for ast-grep Server: Resources and Prompts Design

## Overview

This document outlines how to enhance the ast-grep MCP server with Resources and Prompts capabilities to better support small LLMs. The current implementation only provides Tools, but Resources and Prompts can significantly improve usability for small models.

## Current State Analysis

Your ast-grep MCP server currently supports:
- `ast_grep_search` - Search for AST patterns
- `ast_grep_replace` - Replace AST patterns
- `ast_grep_scan` - Scan code for issues

**Gap**: No Resources or Prompts support, making it harder for small LLMs to use effectively.

## MCP Resources Design for ast-grep

### 1. Pattern Library Resources

**Purpose**: Provide small LLMs with pre-built, tested ast-grep patterns for common use cases.

**Implementation**:
```rust
// Resource URIs
patterns://javascript/common-bugs
patterns://python/security-issues
patterns://rust/performance-antipatterns
```

**Benefits for Small LLMs**:
- No need to construct complex patterns from scratch
- Reduced cognitive load
- Higher success rates with proven patterns

### 2. Language Reference Resources

**Purpose**: Provide AST node types and syntax references for each language.

**Implementation**:
```rust
// Resource URIs
lang://javascript/ast-nodes
lang://python/syntax-reference
lang://rust/node-types
```

**Content Example**:
```json
{
  "language": "javascript",
  "common_nodes": {
    "function_declaration": {
      "pattern": "function $NAME($ARGS) { $$$ }",
      "description": "Function declaration pattern",
      "examples": ["function add(a, b) { return a + b; }"]
    }
  }
}
```

### 3. Code Context Resources

**Purpose**: Provide file contents and project structure for informed pattern matching.

**Implementation**:
```rust
// Resource URIs
file://src/main.rs
project://package.json
context://recently-modified-files
```

**Benefits for Small LLMs**:
- Better understanding of codebase structure
- Contextual pattern suggestions
- Informed replacement decisions

### 4. Rule Templates Resources

**Purpose**: Provide pre-configured ast-grep rules for common tasks.

**Implementation**:
```rust
// Resource URIs
rules://security/detect-sql-injection
rules://performance/unused-variables
rules://style/naming-conventions
```

## MCP Prompts Design for ast-grep

### 1. Guided Refactoring Prompts

**Purpose**: Provide structured workflows for common refactoring tasks.

**Prompt Template**:
```yaml
name: "refactor_function"
description: "Guide through function refactoring workflow"
arguments:
  - name: "function_name"
    description: "Name of function to refactor"
    required: true
  - name: "refactor_type"
    description: "Type of refactoring (extract, rename, optimize)"
    required: true
```

**Generated Prompt**:
```
You are helping refactor the function '{function_name}' using {refactor_type}.

Step 1: First, search for the function using ast_grep_search with pattern "function {function_name}($ARGS) { $$$ }"
Step 2: Analyze the function structure and identify refactoring opportunities
Step 3: Apply the refactoring using ast_grep_replace with appropriate patterns
Step 4: Verify the changes don't break existing functionality

Use the provided resources for pattern examples and best practices.
```

### 2. Code Quality Improvement Prompts

**Purpose**: Structure code quality analysis and improvement workflows.

**Prompt Template**:
```yaml
name: "improve_code_quality"
description: "Systematic code quality improvement workflow"
arguments:
  - name: "file_path"
    description: "Path to file to improve"
    required: true
  - name: "focus_area"
    description: "Focus area (performance, security, style, bugs)"
    required: true
```

### 3. Security Scanning Prompts

**Purpose**: Guide security-focused code analysis.

**Prompt Template**:
```yaml
name: "security_scan"
description: "Security-focused code scanning workflow"
arguments:
  - name: "language"
    description: "Programming language"
    required: true
  - name: "scope"
    description: "Scan scope (file, directory, project)"
    required: true
```

### 4. Pattern Explanation Prompts

**Purpose**: Help understand complex ast-grep patterns.

**Prompt Template**:
```yaml
name: "explain_pattern"
description: "Explain how an ast-grep pattern works"
arguments:
  - name: "pattern"
    description: "The ast-grep pattern to explain"
    required: true
  - name: "language"
    description: "Programming language"
    required: true
```

## Implementation Strategy

### Phase 1: Add Resources Support

1. **Update server capabilities**:
```rust
"capabilities": {
    "tools": {},
    "resources": {
        "subscribe": true,
        "listChanged": true
    }
}
```

2. **Implement resource handlers**:
```rust
"resources/list" => self.list_resources(),
"resources/read" => self.read_resource(params),
"resources/subscribe" => self.subscribe_resource(params),
```

3. **Create resource providers**:
```rust
struct ResourceProvider {
    pattern_library: PatternLibrary,
    language_references: LanguageReferences,
    rule_templates: RuleTemplates,
}
```

### Phase 2: Add Prompts Support

1. **Update server capabilities**:
```rust
"capabilities": {
    "tools": {},
    "resources": { "subscribe": true, "listChanged": true },
    "prompts": {}
}
```

2. **Implement prompt handlers**:
```rust
"prompts/list" => self.list_prompts(),
"prompts/get" => self.get_prompt(params),
```

3. **Create prompt templates**:
```rust
struct PromptTemplate {
    name: String,
    description: String,
    arguments: Vec<PromptArgument>,
    template: String,
}
```

## Benefits for Small LLMs

### 1. Reduced Cognitive Load

**Before (Tools only)**:
- Small LLM needs to understand ast-grep syntax
- Must construct patterns from scratch
- No guidance on common use cases

**After (with Resources + Prompts)**:
- Pre-built patterns available as resources
- Structured workflows via prompts
- Context-aware suggestions

### 2. Higher Success Rates

**Resources provide**:
- Proven, tested patterns
- Language-specific guidance
- Project context

**Prompts provide**:
- Step-by-step workflows
- Error prevention
- Best practice guidance

### 3. Better Error Recovery

**Resources help with**:
- Alternative patterns when one fails
- Syntax references for corrections
- Context for understanding errors

**Prompts help with**:
- Structured debugging workflows
- Fallback strategies
- Recovery procedures

## Usage Examples

### Example 1: Security Scan with Small LLM

1. **LLM uses prompt**: `/security_scan language:javascript scope:src/`
2. **Prompt provides workflow**: Step-by-step security scanning process
3. **LLM accesses resources**: `rules://security/detect-sql-injection`
4. **LLM applies tools**: `ast_grep_scan` with pre-built rules

### Example 2: Code Refactoring

1. **LLM uses prompt**: `/refactor_function function_name:calculateTotal refactor_type:extract`
2. **Prompt guides process**: Structured refactoring workflow
3. **LLM reads context**: `file://src/calculator.js` resource
4. **LLM applies patterns**: Using `patterns://javascript/function-extraction`

## Resource URI Scheme

```
patterns://<language>/<category>/<pattern-name>
rules://<category>/<rule-name>
lang://<language>/<reference-type>
file://<file-path>
project://<project-file>
context://<context-type>
```

## Prompt Categories

```
/refactor_* - Refactoring workflows
/scan_* - Code scanning workflows
/explain_* - Pattern explanation workflows
/fix_* - Bug fixing workflows
/optimize_* - Performance optimization workflows
```

## Implementation Priorities

### High Priority
1. Pattern library resources (immediate value)
2. Basic refactoring prompts (guided workflows)
3. Security scanning prompts (structured analysis)

### Medium Priority
1. Language reference resources
2. Code context resources
3. Pattern explanation prompts

### Low Priority
1. Advanced optimization prompts
2. Custom rule generation
3. Dynamic resource updates

## Conclusion

Adding Resources and Prompts to your ast-grep MCP server will significantly improve its usability for small LLMs by:

1. **Reducing complexity** through pre-built patterns and structured workflows
2. **Improving success rates** via proven patterns and guided processes
3. **Enhancing understanding** through context and explanations
4. **Enabling better error recovery** with alternatives and debugging guidance

This transforms your server from a raw tool interface into a comprehensive, small-LLM-friendly development assistant.