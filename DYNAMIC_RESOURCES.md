# Dynamic MCP Resources for AST-Grep

This implementation provides a powerful dynamic resource system for ast-grep documentation and examples.

## Dynamic Resource URIs

### 1. Documentation Generator
```
ast-grep://docs/{type}
```

Examples:
- `ast-grep://docs/examples-by-language` - Complete language examples
- `ast-grep://docs/pattern-syntax` - Pattern syntax reference  
- `ast-grep://docs/rule-composition` - Rule composition guide
- `ast-grep://docs/language-guide/javascript?level=advanced&focus=patterns` - Advanced JavaScript guide

### 2. Language-Specific Examples
```
ast-grep://examples/{language}
```

Examples:
- `ast-grep://examples/javascript` - JavaScript examples
- `ast-grep://examples/python?category=functions&complexity=advanced` - Advanced Python function examples
- `ast-grep://examples/rust?category=error-handling` - Rust error handling patterns

### 3. Pattern Categories
```
ast-grep://patterns/{category}
```

Examples:
- `ast-grep://patterns/variables` - Variable-related patterns
- `ast-grep://patterns/functions?language=typescript&fixes=true` - TypeScript function patterns with fixes
- `ast-grep://patterns/loops?language=rust` - Rust loop patterns

### 4. Dynamic Queries
```
ast-grep://query/{type}
```

Examples:
- `ast-grep://query/search?q=console.log&lang=javascript&limit=5` - Search for console.log patterns
- `ast-grep://query/filter?has_fix=true&language=python` - Filter patterns with fixes for Python
- `ast-grep://query/similar?pattern=function $NAME() {}` - Find similar function patterns

## Query Parameters

Dynamic resources support various query parameters:

### Common Parameters
- `language` or `lang` - Filter by programming language
- `category` - Filter by pattern category
- `complexity` - basic, intermediate, advanced
- `level` - beginner, intermediate, advanced
- `focus` - patterns, syntax, examples, rules

### Query-Specific Parameters
- `q` - Search query string
- `limit` - Maximum number of results
- `has_fix` - true/false, filter by fix availability
- `features` - Comma-separated list of features
- `fixes` - true/false, include transformation examples

## Key Advantages

1. **Parameterized Content**: Generate content based on user needs
2. **No Static Lists**: Resources are resolved on-demand
3. **Query Support**: Search and filter capabilities
4. **Flexible URIs**: Template-based URI patterns
5. **Small-LLM Optimized**: Content generated for specific contexts

## Implementation Details

The dynamic resource system:
- Parses URI templates with `{parameter}` syntax
- Supports query string parameters with `?key=value&key2=value2`
- Generates content based on parameters
- Falls back to static resources when dynamic patterns don't match
- Provides backwards compatibility with existing static URIs

## Example Usage

```python
# Request language-specific examples with filters
uri = "ast-grep://examples/typescript?category=classes&complexity=intermediate"

# Search for specific patterns
uri = "ast-grep://query/search?q=async&lang=javascript&limit=10"

# Get advanced language guide
uri = "ast-grep://docs/language-guide/rust?level=advanced&focus=error-handling"
```

This dynamic approach allows MCP clients to request exactly the documentation they need, when they need it, with the specific level of detail and focus area that's most helpful.