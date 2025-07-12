# MCP Best Practices for Small LLMs: Design Guide for Usable MCP Servers

## Executive Summary

This document provides research findings and recommendations for designing Model Context Protocol (MCP) servers that are usable by small local language models. The research identifies key challenges faced by small LLMs and provides actionable design patterns to improve usability.

## Key Challenges for Small LLMs with MCP Servers

### 1. Context Window Limitations
- **Problem**: Small models have limited context windows, making complex tool schemas overwhelming
- **Impact**: Reduced comprehension and incorrect tool usage

### 2. Instruction Following Capacity
- **Problem**: Small models struggle with complex, multi-step instructions
- **Impact**: Difficulty parsing complex tool specifications and function calls

### 3. JSON Schema Complexity
- **Problem**: Complex JSON schemas with nested structures confuse small models
- **Impact**: Malformed function calls and parameter errors

### 4. Error Recovery
- **Problem**: Limited ability to recover from errors or understand error messages
- **Impact**: System failures and poor user experience

## Design Principles for Small LLM-Friendly MCP Servers

### 1. Simplicity First
- **Single Purpose**: Each server should focus on one clear domain or capability
- **Minimal Tool Count**: Limit to 3-5 tools per server maximum
- **Clear Naming**: Use descriptive, unambiguous function names

### 2. Schema Optimization
- **Flat Structure**: Avoid nested JSON objects where possible
- **Required vs Optional**: Minimize required parameters
- **Meaningful Defaults**: Provide sensible default values
- **Type Simplicity**: Use basic types (string, number, boolean) over complex objects

### 3. Context Efficiency
- **Compact Descriptions**: Keep tool descriptions under 100 words
- **Essential Parameters Only**: Only expose parameters that are absolutely necessary
- **Clear Examples**: Provide concrete usage examples in descriptions

## Recommended Design Patterns

### Pattern 1: Atomic Operations
```json
{
  "name": "send_message",
  "description": "Send a simple message",
  "parameters": {
    "type": "object",
    "properties": {
      "message": {"type": "string", "description": "The message to send"},
      "recipient": {"type": "string", "description": "Who to send it to"}
    },
    "required": ["message", "recipient"]
  }
}
```

### Pattern 2: Enumerated Choices
```json
{
  "name": "set_status",
  "description": "Update status",
  "parameters": {
    "type": "object",
    "properties": {
      "status": {
        "type": "string", 
        "enum": ["active", "inactive", "pending"],
        "description": "Current status"
      }
    },
    "required": ["status"]
  }
}
```

### Pattern 3: Resource Templates
```json
{
  "name": "create_document",
  "description": "Create a new document",
  "parameters": {
    "type": "object",
    "properties": {
      "title": {"type": "string", "description": "Document title"},
      "template": {
        "type": "string",
        "enum": ["blank", "report", "letter"],
        "default": "blank"
      }
    },
    "required": ["title"]
  }
}
```

## Implementation Guidelines

### 1. Schema Generation
- Use automatic schema generation from function signatures
- Leverage Pydantic models for type safety
- Implement schema validation before exposing to models

### 2. Error Handling
- Provide clear, actionable error messages
- Use standardized error codes
- Include recovery suggestions in error responses

### 3. Documentation
- Include usage examples for each tool
- Provide parameter format specifications
- Document expected response formats

### 4. Testing with Small Models
- Test with models like Llama 3.1 8B, Gemma 2 9B, and similar
- Validate tool calling accuracy across different model sizes
- Measure success rates for complex vs simple schemas

## Architecture Recommendations

### Layered Approach
1. **Core Layer**: Essential, simple tools
2. **Enhanced Layer**: More complex operations (optional)
3. **Expert Layer**: Advanced features (for larger models only)

### Progressive Disclosure
- Start with basic functionality
- Add complexity only when model demonstrates competence
- Provide fallback mechanisms for failed operations

### Caching Strategy
- Cache schema information to reduce context usage
- Pre-compute common responses
- Implement smart defaults based on usage patterns

## Specific Recommendations by Use Case

### Database Operations
- **Good**: `get_user(id: string)` → user object
- **Bad**: `query_database(sql: string, params: object, options: object)`

### File Operations
- **Good**: `read_file(path: string)` → file content
- **Bad**: `file_operation(action: string, path: string, mode: string, options: object)`

### API Integrations
- **Good**: `get_weather(city: string)` → weather data
- **Bad**: `api_call(endpoint: string, method: string, headers: object, body: object)`

## Success Metrics

### Quantitative Metrics
- Tool calling success rate (target: >90% for small models)
- Parameter accuracy (target: >95%)
- Error rate (target: <5%)

### Qualitative Metrics
- User satisfaction with tool responses
- Ease of integration for developers
- Maintenance overhead

## Common Anti-Patterns to Avoid

### 1. Swiss Army Knife Tools
- Don't create tools that do everything
- Avoid tools with >5 parameters
- Don't combine unrelated operations

### 2. Complex Nested Structures
- Avoid deeply nested JSON objects
- Don't use arrays of objects as parameters
- Minimize optional parameter chains

### 3. Ambiguous Naming
- Don't use generic names like "process" or "handle"
- Avoid abbreviations and acronyms
- Don't use similar names for different operations

## Future Considerations

### Model Evolution
- Monitor small model improvements in tool calling
- Adapt complexity based on model capabilities
- Prepare for fine-tuned models with better tool calling

### Protocol Evolution
- Track MCP specification updates
- Implement new features gradually
- Maintain backward compatibility

### Ecosystem Integration
- Consider integration with existing MCP servers
- Plan for server composition and chaining
- Design for interoperability

## Conclusion

Designing MCP servers for small LLMs requires a fundamentally different approach than designing for large models. The key is to prioritize simplicity, clarity, and efficiency while maintaining the power and flexibility that makes MCP valuable. By following these guidelines, developers can create MCP servers that are accessible to a broader range of models and use cases.

---

*This document is based on research conducted in July 2025 and should be updated as the MCP ecosystem and small model capabilities evolve.*