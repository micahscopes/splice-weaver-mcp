# Pattern Construction Strategy for Small LLMs

## The Challenge: From Rough Context to Precise Patterns

Small LLMs struggle with pattern construction because they need to:
1. **Parse ambiguous descriptions** ("find all the error handling code")
2. **Map natural language to AST concepts** (function → function_declaration)
3. **Handle incomplete information** ("something like a callback")
4. **Construct valid ast-grep syntax** (proper wildcards and captures)

## Research-Backed Approach

Based on research showing that **fine-tuned 7-8B LLMs can outperform GPT-3.5 by 15-20 F1 points** on specific tasks, and that **natural language boosts LLM performance in coding**, our strategy focuses on:

1. **Example-Based Learning**: Extensive pattern examples with explanations
2. **Progressive Refinement**: Start simple, add complexity iteratively
3. **Natural Language Bridging**: Map descriptions to AST concepts
4. **Validation Loops**: Test patterns before finalizing

## Core Implementation Strategy

### 1. Intent Recognition Resources

**Resource**: `patterns://intent-mapping/javascript`

**Content Structure**:
```json
{
  "intent_patterns": {
    "find_functions": {
      "keywords": ["function", "method", "procedure", "handler"],
      "ast_concepts": ["function_declaration", "arrow_function", "method_definition"],
      "base_patterns": [
        "function $NAME($ARGS) { $$$ }",
        "const $NAME = ($ARGS) => { $$$ }",
        "$NAME($ARGS) { $$$ }"
      ]
    },
    "find_error_handling": {
      "keywords": ["error", "exception", "catch", "try", "throw"],
      "ast_concepts": ["try_statement", "catch_clause", "throw_statement"],
      "base_patterns": [
        "try { $$$ } catch ($ERR) { $$$ }",
        "throw $ERROR",
        "if ($CONDITION) { throw $$$ }"
      ]
    }
  }
}
```

### 2. Pattern Builder Prompts

**Prompt**: `/build_pattern description:"{rough_description}" language:"{lang}"`

**Workflow**:
```
Step 1: Intent Analysis
- Analyze description for keywords
- Map to AST concepts using intent resources
- Suggest base pattern templates

Step 2: Example Matching
- Find similar patterns in pattern library
- Show successful examples with explanations
- Highlight pattern components

Step 3: Pattern Construction
- Start with base template
- Add specificity based on description
- Use wildcards appropriately

Step 4: Validation
- Test pattern against sample code
- Refine based on results
- Provide usage examples
```

### 3. Example-Based Learning System

**Resource**: `examples://pattern-construction/javascript`

**Content Structure**:
```json
{
  "learning_examples": [
    {
      "description": "find all async functions",
      "reasoning": "async functions have 'async' keyword before function declaration",
      "ast_concept": "function_declaration with async modifier",
      "pattern": "async function $NAME($ARGS) { $$$ }",
      "variations": [
        "async ($ARGS) => { $$$ }",
        "async function($ARGS) { $$$ }"
      ],
      "test_code": "async function fetchData() { return await api.get('/data'); }",
      "matches": true
    }
  ]
}
```

### 4. Progressive Refinement Process

**Initial Pattern**: Start with broadest possible match
**Refinement Steps**:
1. **Scope Narrowing**: Add constraints to reduce false positives
2. **Specificity Addition**: Add required elements based on context
3. **Edge Case Handling**: Account for variations and special cases
4. **Performance Optimization**: Simplify pattern for efficiency

### 5. Natural Language to AST Mapping

**Resource**: `mappings://natural-to-ast/javascript`

**Mapping Examples**:
```json
{
  "natural_to_ast": {
    "function": ["function_declaration", "arrow_function", "method_definition"],
    "loop": ["for_statement", "while_statement", "do_statement"],
    "conditional": ["if_statement", "conditional_expression", "switch_statement"],
    "variable": ["variable_declaration", "identifier", "assignment_expression"],
    "class": ["class_declaration", "class_expression"],
    "import": ["import_statement", "import_declaration"],
    "callback": ["arrow_function", "function_expression", "identifier"],
    "async": ["async_function", "await_expression", "promise"]
  }
}
```

## Implementation in Your ast-grep Server

### New MCP Capabilities

1. **Pattern Builder Tool**:
```rust
async fn build_pattern(&self, description: String, language: String) -> Result<String> {
    // 1. Analyze intent using intent-mapping resource
    // 2. Find similar examples using example-matching resource
    // 3. Construct base pattern using templates
    // 4. Refine iteratively using validation
    // 5. Return final pattern with explanation
}
```

2. **Pattern Explain Tool**:
```rust
async fn explain_pattern(&self, pattern: String, language: String) -> Result<String> {
    // 1. Parse pattern components
    // 2. Map to natural language concepts
    // 3. Provide usage examples
    // 4. Suggest variations
}
```

3. **Pattern Validate Tool**:
```rust
async fn validate_pattern(&self, pattern: String, test_code: String) -> Result<ValidationResult> {
    // 1. Test pattern against sample code
    // 2. Report matches/misses
    // 3. Suggest improvements
    // 4. Provide confidence score
}
```

### Resource Architecture

```
resources://
├── intent-mapping/
│   ├── javascript/
│   ├── python/
│   └── rust/
├── examples/
│   ├── pattern-construction/
│   │   ├── javascript/
│   │   ├── python/
│   │   └── rust/
│   └── successful-patterns/
├── mappings/
│   ├── natural-to-ast/
│   └── ast-to-natural/
└── templates/
    ├── base-patterns/
    └── refinement-rules/
```

## Usage Flow for Small LLMs

### Example: "Find all callback functions"

1. **Intent Analysis**:
   - Keywords: ["callback", "function"]
   - AST concepts: ["arrow_function", "function_expression", "identifier"]
   - Base patterns: ["($ARGS) => { $$$ }", "function($ARGS) { $$$ }"]

2. **Example Matching**:
   - Find similar: "find all event handlers", "find all promise callbacks"
   - Show patterns: `addEventListener($EVENT, $CALLBACK)`, `promise.then($CALLBACK)`

3. **Pattern Construction**:
   - Start broad: `function $NAME($ARGS) { $$$ }`
   - Add context: Consider where callbacks appear (parameters, assignments)
   - Refine: `$FUNC($ARGS, $CALLBACK)` where `$CALLBACK` is function-like

4. **Validation**:
   - Test against sample code
   - Check for false positives/negatives
   - Refine based on results

### Example: "Find security vulnerabilities"

1. **Intent Analysis**:
   - Keywords: ["security", "vulnerability", "dangerous", "unsafe"]
   - AST concepts: ["call_expression", "member_expression", "string_literal"]
   - Base patterns: Focus on dangerous function calls

2. **Example Matching**:
   - Find similar: "find SQL injection", "find XSS vulnerabilities"
   - Show patterns: `eval($CODE)`, `innerHTML = $HTML`

3. **Pattern Construction**:
   - Multiple patterns for different vulnerability types
   - Use rule-based approach with severity ratings

## Success Metrics

### Quantitative Metrics
- **Pattern Accuracy**: >85% correct patterns from descriptions
- **Construction Speed**: <30 seconds from description to pattern
- **Validation Success**: >90% patterns work on first try
- **False Positive Rate**: <15% for constructed patterns

### Qualitative Improvements
- **Learning Curve**: Faster pattern construction learning
- **Confidence**: Higher success rates reduce frustration
- **Discoverability**: Examples help understand capabilities
- **Maintainability**: Structured approach enables improvements

## Key Design Principles

### 1. Start Simple, Add Complexity
- Begin with broad patterns
- Progressively narrow scope
- Add constraints iteratively

### 2. Example-Driven Learning
- Show similar successful patterns
- Explain reasoning behind choices
- Provide test cases and validation

### 3. Natural Language First
- Map descriptions to AST concepts
- Use familiar terminology
- Explain AST concepts in plain language

### 4. Validation-Centric
- Test patterns immediately
- Provide feedback on accuracy
- Suggest improvements automatically

### 5. Context-Aware
- Consider file types and project structure
- Use codebase knowledge for better patterns
- Adapt to specific languages and frameworks

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- Intent mapping resources
- Basic pattern templates
- Natural language to AST mapping

### Phase 2: Pattern Builder (Week 3-4)
- Pattern construction prompts
- Example-based learning resources
- Basic validation tools

### Phase 3: Refinement (Week 5-6)
- Progressive refinement system
- Advanced validation
- Success metrics tracking

### Phase 4: Enhancement (Week 7-8)
- Context-aware construction
- Performance optimization
- User feedback integration

## Conclusion

By combining research insights on small LLM capabilities with practical pattern construction needs, this strategy provides a systematic approach to building ast-grep patterns from rough context. The key is leveraging small LLMs' strengths in example-based learning and structured workflows while compensating for their limitations in complex reasoning through progressive refinement and extensive validation.

The result is a system that can turn vague descriptions like "find all the error handling code" into precise, working ast-grep patterns that small LLMs can confidently construct and use.