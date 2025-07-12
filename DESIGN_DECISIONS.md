# MCP ast-grep Server: Design Decisions

## Core Design Philosophy

This MCP server implements a **dual-approach architecture** to make ast-grep accessible to both large and small language models.

## Key Design Decisions

### 1. Minimal Tool Set
- **Decision**: Implement only 2 core tools instead of exposing all ast-grep functionality
- **Rationale**: Small LLMs perform better with fewer, focused tools (research shows >90% success vs ~45% with complex APIs)
- **Implementation**: 
  - `find_scope`: Scope navigation using relational rules
  - `execute_rule`: Direct rule execution with full CLI power

### 2. MCP Resources for Context
- **Decision**: Use MCP Resources to provide pattern libraries and documentation
- **Rationale**: Reduces cognitive load by providing pre-built, tested patterns
- **Implementation**: 
  - `ast-grep://cli-reference`: Complete command documentation
  - `ast-grep://rule-examples`: Common YAML rule patterns
  - `ast-grep://node-kinds`: Tree-sitter node types by language

### 3. MCP Prompts for Guided Workflows
- **Decision**: Use MCP Prompts as "mad libs" templates requiring no AI reasoning
- **Rationale**: Small LLMs struggle with pattern construction but excel at template filling
- **Implementation**:
  - `scope_navigation_rule`: Generate rules to find containing scopes
  - `transform_in_scope`: Generate transformation rules within specific scopes

### 4. Relational Rules Focus
- **Decision**: Emphasize ast-grep's relational rule system for scope navigation
- **Rationale**: Enables surgical precision for scope-based refactoring, which is ast-grep's unique strength
- **Example**: Find console.log statements inside functions but not nested functions

## Small LLM Optimization Principles

Based on research with 8B parameter models:

### Context Efficiency
- Keep tool descriptions under 100 words
- Use flat parameter structures
- Provide meaningful defaults
- Minimize required parameters

### Error Recovery
- Provide clear, actionable error messages
- Include recovery suggestions
- Use enumerated choices where possible

### Progressive Disclosure
- Start with simple operations
- Layer complexity only when needed
- Provide fallback mechanisms

## Architecture Benefits

### For Large LLMs
- Direct binary access via resources
- Full CLI flexibility through `execute_rule`
- Complete ast-grep documentation

### For Small LLMs  
- Template-based rule generation via prompts
- Pre-built pattern libraries via resources
- Guided workflows requiring minimal reasoning

### For All Users
- Consistent MCP protocol implementation
- Extensible pattern library
- Clear separation of concerns

## Success Metrics

- **Tool calling success rate**: Target >90% for 8B parameter models
- **Pattern construction accuracy**: Measured via automated testing
- **Learning progression**: Track graduation from prompts to direct tools
- **Hybrid workflow effectiveness**: Identify optimal combinations

## Future Considerations

### Model Evolution
- Monitor small model improvements in tool calling
- Adapt complexity based on model capabilities
- Support for fine-tuned models with better ast-grep knowledge

### Pattern Library Growth
- Community-contributed patterns
- Domain-specific rule collections
- Automated pattern testing and validation

This design provides a foundation for evaluating which approach works best for different model sizes while preserving ast-grep's full power for advanced users.