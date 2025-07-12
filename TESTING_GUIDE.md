# AST-Grep Testing Guide

This guide provides comprehensive documentation for testing ast-grep patterns and transformations using the fixtures and framework created in this project.

## Overview

This testing setup includes:
- **Test Fixtures**: Code samples in multiple languages
- **Pattern Examples**: Common ast-grep patterns for different use cases
- **Testing Framework**: Python-based framework for automated testing
- **Rule Examples**: YAML rule configurations for complex patterns
- **MCP Server Integration**: Direct testing with ast-grep-mcp server

## Directory Structure

```
test-fixtures/
├── README.md                    # Overview of test fixtures
├── test-framework.py           # Python testing framework
├── test-config.json           # Test case configurations
├── TESTING_GUIDE.md           # This guide
├── javascript/                # JavaScript test files
│   ├── basic-functions.js
│   ├── variables-and-scope.js
│   └── control-flow.js
├── typescript/                # TypeScript test files
│   └── classes-and-interfaces.ts
├── rust/                      # Rust test files
│   └── basic-functions.rs
├── python/                    # Python test files
│   └── basic-functions.py
├── go/                        # Go test files
│   └── basic-functions.go
├── java/                      # Java test files
│   └── BasicFunctions.java
├── patterns/                  # Pattern documentation
│   └── common-patterns.md
└── rules/                     # Rule configurations
    ├── javascript-modernization.yaml
    ├── console-cleanup.yaml
    └── rust-error-handling.yaml
```

## Quick Start

### 1. Basic Pattern Testing

Test a simple pattern with the MCP server:

```python
# Search for JavaScript functions
mcp__ast-grep-mcp__search(
    code="function hello() { return 'world'; }",
    pattern="function $NAME($PARAMS) { $BODY }",
    language="javascript"
)
```

### 2. File-based Testing

Search patterns across multiple files:

```python
# Find all console.log statements
mcp__ast-grep-mcp__file_search(
    path_pattern="test-fixtures/**/*.js",
    pattern="console.log($ARGS)",
    language="javascript"
)
```

### 3. Rule-based Testing

Test complex rules with validation:

```python
# Validate and test a rule
mcp__ast-grep-mcp__validate_rule(
    rule_config="""
id: test-rule
language: javascript
rule:
  pattern: "console.log($ARGS)"
fix: ""
""",
    test_code="console.log('debug');"
)
```

## Test Categories

### 1. Basic Pattern Matching

**Purpose**: Test fundamental ast-grep pattern syntax
**Examples**:
- Function declarations: `function $NAME($PARAMS) { $BODY }`
- Variable assignments: `const $NAME = $VALUE`
- Method calls: `$OBJ.$METHOD($ARGS)`

**Test Files**: All fixture files contain basic patterns

### 2. Language-Specific Patterns

**JavaScript/TypeScript**:
- Arrow functions: `const $NAME = ($PARAMS) => $BODY`
- Class methods: `$NAME($PARAMS) { $BODY }`
- Async functions: `async function $NAME($PARAMS) { $BODY }`

**Rust**:
- Function signatures: `fn $NAME($PARAMS) -> $RETURN { $BODY }`
- Struct definitions: `struct $NAME { $FIELDS }`
- Match expressions: `match $EXPR { $ARMS }`

**Python**:
- Function definitions: `def $NAME($PARAMS): $$$`
- Class definitions: `class $NAME: $$$`
- Decorators: `@$DECORATOR`

**Go**:
- Function definitions: `func $NAME($PARAMS) $RETURN { $BODY }`
- Method definitions: `func ($RECEIVER) $NAME($PARAMS) $RETURN { $BODY }`
- Struct definitions: `type $NAME struct { $FIELDS }`

**Java**:
- Method definitions: `public $RETURN $NAME($PARAMS) { $BODY }`
- Class definitions: `public class $NAME { $BODY }`
- Static methods: `public static $RETURN $NAME($PARAMS) { $BODY }`

### 3. Transformation Testing

**Purpose**: Test pattern replacement and code transformation
**Examples**:
- Modernization: `var $NAME = $VALUE` → `const $NAME = $VALUE`
- Cleanup: `console.log($ARGS)` → `` (empty)
- Error handling: `$EXPR.unwrap()` → `$EXPR?`

### 4. Rule-based Testing

**Purpose**: Test complex YAML rule configurations
**Features**:
- Multiple pattern matching (`any`, `all`, `not`)
- Conditional logic
- Fix transformations
- Severity levels

## Testing Framework Usage

### Running the Python Framework

```bash
python test-fixtures/test-framework.py
```

### Custom Test Cases

Create custom test cases by modifying `test-config.json`:

```json
{
  "name": "custom_test",
  "language": "javascript",
  "code": "your code here",
  "pattern": "your pattern",
  "expected_matches": 1,
  "description": "Test description"
}
```

### Test Case Structure

```python
@dataclass
class TestCase:
    name: str                    # Test identifier
    language: str               # Programming language
    code: str                   # Source code to test
    pattern: str                # AST-grep pattern
    expected_matches: int       # Expected number of matches
    expected_vars: Dict         # Expected captured variables
    replacement: str            # Replacement pattern (optional)
    expected_replacement: str   # Expected transformed code
    description: str            # Test description
```

## Advanced Testing Scenarios

### 1. Multi-file Testing

Test patterns across multiple files:

```python
# Test all JavaScript files
mcp__ast-grep-mcp__file_search(
    path_pattern="test-fixtures/**/*.js",
    pattern="function $NAME($PARAMS) { $BODY }",
    language="javascript",
    max_results=50
)
```

### 2. Bulk Transformation Testing

Test large-scale code transformations:

```python
# Preview changes before applying
mcp__ast-grep-mcp__file_replace(
    path_pattern="test-fixtures/**/*.js",
    pattern="var $NAME = $VALUE",
    replacement="const $NAME = $VALUE",
    language="javascript",
    dry_run=True,
    summary_only=True
)
```

### 3. AST Structure Analysis

Understand AST structure for complex patterns:

```python
# Generate AST to understand structure
mcp__ast-grep-mcp__generate_ast(
    code="function test() { return 42; }",
    language="javascript"
)
```

### 4. Rule Management Testing

Test rule storage and retrieval:

```python
# Create and test rules
mcp__ast-grep-mcp__create_rule(rule_config="...")
mcp__ast-grep-mcp__list_rules()
mcp__ast-grep-mcp__get_rule(rule_id="my-rule")
```

## Best Practices

### 1. Pattern Development

1. **Start Simple**: Begin with basic patterns
2. **Test Incrementally**: Add complexity gradually
3. **Use AST Generation**: Understand node structure
4. **Test Edge Cases**: Include boundary conditions

### 2. Test Organization

1. **Group by Language**: Organize tests by programming language
2. **Categorize by Purpose**: Group similar transformations
3. **Document Patterns**: Include clear descriptions
4. **Version Control**: Track pattern evolution

### 3. Performance Considerations

1. **Limit File Size**: Use `max_file_size` parameter
2. **Batch Operations**: Use `summary_only` for large changes
3. **Pagination**: Handle large result sets
4. **Specific Patterns**: Avoid overly broad patterns

## Common Patterns Reference

### JavaScript Patterns

```javascript
// Function declarations
function $NAME($PARAMS) { $BODY }

// Arrow functions
const $NAME = ($PARAMS) => $BODY

// Object methods
$OBJ.$METHOD($ARGS)

// Variable declarations
const $NAME = $VALUE
let $NAME = $VALUE
var $NAME = $VALUE

// Control flow
if ($CONDITION) { $BODY }
for ($INIT; $CONDITION; $UPDATE) { $BODY }
while ($CONDITION) { $BODY }

// Error handling
try { $BODY } catch ($ERROR) { $HANDLER }
```

### TypeScript Patterns

```typescript
// Interface definitions
interface $NAME { $BODY }

// Class definitions
class $NAME { $BODY }

// Method definitions
$NAME($PARAMS): $RETURN { $BODY }

// Generic types
$NAME<$TYPES>

// Type annotations
$NAME: $TYPE
```

### Rust Patterns

```rust
// Function definitions
fn $NAME($PARAMS) -> $RETURN { $BODY }

// Struct definitions
struct $NAME { $FIELDS }

// Impl blocks
impl $NAME { $METHODS }

// Match expressions
match $EXPR { $ARMS }

// Error handling
$EXPR.unwrap()
$EXPR?
```

### Python Patterns

```python
# Function definitions
def $NAME($PARAMS): $$$

# Class definitions
class $NAME: $$$
class $NAME($BASE): $$$

# Method definitions
def $NAME(self, $PARAMS): $$$

# Import statements
import $MODULE
from $MODULE import $NAME

# Decorators
@$DECORATOR
```

## Troubleshooting

### Common Issues

1. **Pattern Not Matching**:
   - Check whitespace and indentation
   - Use `generate_ast` to understand structure
   - Test with simpler patterns first

2. **Replacement Not Working**:
   - Ensure all captured variables are used
   - Check pattern syntax
   - Test with `dry_run=True`

3. **Performance Issues**:
   - Use specific glob patterns
   - Limit file size and results
   - Enable pagination for large operations

### Debug Steps

1. **Pattern Validation**:
   ```python
   mcp__ast-grep-mcp__validate_rule(rule_config="...", test_code="...")
   ```

2. **AST Analysis**:
   ```python
   mcp__ast-grep-mcp__generate_ast(code="...", language="...")
   ```

3. **Incremental Testing**:
   ```python
   mcp__ast-grep-mcp__search(code="...", pattern="...", language="...")
   ```

## Contributing

### Adding New Test Cases

1. Create test files in appropriate language directories
2. Update `test-config.json` with new test cases
3. Document patterns in `common-patterns.md`
4. Test with the MCP server
5. Update this guide with new examples

### Adding New Languages

1. Create new language directory
2. Add comprehensive test files
3. Document language-specific patterns
4. Update test framework language mappings
5. Add examples to this guide

## Resources

- [ast-grep documentation](https://ast-grep.github.io/)
- [ast-grep catalog](https://ast-grep.github.io/catalog/)
- [Tree-sitter documentation](https://tree-sitter.github.io/tree-sitter/)
- [MCP server documentation](https://mcp.dev/)

## Summary

This testing setup provides a comprehensive framework for exploring and testing ast-grep patterns across multiple programming languages. The combination of fixtures, patterns, rules, and automated testing enables robust development and validation of code transformation patterns.

Key features:
- **Multi-language support**: JavaScript, TypeScript, Rust, Python, Go, Java
- **Pattern library**: Common patterns for each language
- **Rule configurations**: Complex transformation rules
- **MCP integration**: Direct testing with ast-grep-mcp server
- **Automated testing**: Python framework for systematic testing
- **Comprehensive documentation**: Detailed guides and examples

Use this framework to develop, test, and validate ast-grep patterns for your specific use cases.