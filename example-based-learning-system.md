# Example-Based Learning System for Pattern Construction

## Core Principle

Research shows that **knowledge distillation** and **example-based learning** are highly effective for small LLMs. Our system leverages this by providing rich, structured examples that demonstrate pattern construction from rough context.

## Example Database Structure

### Learning Example Format

```json
{
  "id": "js-async-functions-001",
  "description": "find all async functions",
  "language": "javascript",
  "difficulty": "beginner",
  "intent_category": "function_detection",
  "learning_progression": {
    "step_1": {
      "rough_description": "find functions that are async",
      "reasoning": "async functions have 'async' keyword before function declaration",
      "ast_concept": "function_declaration with async modifier",
      "base_pattern": "async function $NAME($ARGS) { $$$ }",
      "explanation": "The 'async' keyword modifies the function declaration"
    },
    "step_2": {
      "refinement": "include arrow functions",
      "reasoning": "async can also apply to arrow functions",
      "extended_pattern": "async ($ARGS) => { $$$ }",
      "explanation": "Arrow functions can also be async"
    },
    "step_3": {
      "final_pattern": "async function $NAME($ARGS) { $$$ } | async ($ARGS) => { $$$ }",
      "validation": {
        "test_code": [
          "async function fetchData() { return await api.get('/data'); }",
          "const getData = async () => { return await fetch('/api'); }",
          "function normalFunction() { return 'not async'; }"
        ],
        "expected_matches": 2,
        "false_positives": 0
      }
    }
  },
  "variations": [
    "async function expressions",
    "async methods in classes",
    "async generators"
  ],
  "related_patterns": ["promise-handling", "await-expressions"],
  "common_mistakes": [
    {
      "mistake": "forgetting arrow functions",
      "incorrect_pattern": "async function $NAME($ARGS) { $$$ }",
      "why_wrong": "misses async arrow functions",
      "fix": "add alternative pattern for arrow functions"
    }
  ]
}
```

## Learning Categories

### 1. Function Detection Patterns
- Basic functions
- Arrow functions
- Async functions
- Generator functions
- Method definitions
- Constructors

### 2. Control Flow Patterns
- Conditional statements
- Loops (for, while, do-while)
- Switch statements
- Try-catch blocks
- Break/continue statements

### 3. Variable and Data Patterns
- Variable declarations
- Assignments
- Destructuring
- Property access
- Array/object literals

### 4. Class and Object Patterns
- Class declarations
- Inheritance patterns
- Method definitions
- Property definitions
- Static members

### 5. Import/Export Patterns
- ES6 imports/exports
- CommonJS requires
- Dynamic imports
- Default exports
- Named exports

### 6. Error Handling Patterns
- Try-catch blocks
- Throw statements
- Error constructors
- Promise rejections
- Async error handling

### 7. Performance and Security Patterns
- Inefficient loops
- Memory leaks
- Security vulnerabilities
- Code smells
- Best practice violations

## Progressive Learning Paths

### Beginner Path: Simple Pattern Construction
1. **Basic Function Matching**: `function $NAME() { $$$ }`
2. **With Parameters**: `function $NAME($ARGS) { $$$ }`
3. **With Return**: `function $NAME($ARGS) { return $VALUE; }`
4. **Multiple Variations**: Include arrow functions

### Intermediate Path: Conditional Pattern Construction
1. **Basic Conditionals**: `if ($CONDITION) { $$$ }`
2. **With Else**: `if ($CONDITION) { $$$ } else { $$$ }`
3. **Complex Conditions**: `if ($A && $B) { $$$ }`
4. **Switch Statements**: `switch ($VALUE) { $$$ }`

### Advanced Path: Context-Aware Pattern Construction
1. **Scoped Patterns**: Functions within classes
2. **Nested Patterns**: Functions within functions
3. **Conditional Patterns**: Based on surrounding code
4. **Multi-file Patterns**: Cross-file dependencies

## Example Matching Algorithm

### 1. Intent Classification
```rust
fn classify_intent(description: &str) -> IntentCategory {
    // Keywords → Category mapping
    if description.contains("function") || description.contains("method") {
        IntentCategory::FunctionDetection
    } else if description.contains("error") || description.contains("exception") {
        IntentCategory::ErrorHandling
    } else if description.contains("loop") || description.contains("iterate") {
        IntentCategory::ControlFlow
    }
    // ... more classifications
}
```

### 2. Similarity Scoring
```rust
fn find_similar_examples(description: &str, category: IntentCategory) -> Vec<ScoredExample> {
    // 1. Keyword similarity
    // 2. AST concept overlap
    // 3. Complexity level match
    // 4. Language specificity
    // 5. Success rate history
}
```

### 3. Example Selection
```rust
fn select_learning_examples(scored_examples: Vec<ScoredExample>) -> LearningSet {
    // 1. Best match (highest similarity)
    // 2. Progression examples (simpler to complex)
    // 3. Variation examples (different approaches)
    // 4. Common mistake examples (what not to do)
}
```

## Resource Implementation

### Example Database Resource
```
examples://pattern-construction/javascript/
├── function-detection/
│   ├── basic-functions.json
│   ├── arrow-functions.json
│   ├── async-functions.json
│   └── generator-functions.json
├── control-flow/
│   ├── conditionals.json
│   ├── loops.json
│   └── switch-statements.json
├── error-handling/
│   ├── try-catch.json
│   ├── throw-statements.json
│   └── promise-errors.json
└── security/
    ├── injection-patterns.json
    ├── xss-patterns.json
    └── unsafe-operations.json
```

### Learning Progression Resource
```
progressions://pattern-construction/
├── beginner/
│   ├── simple-matching.json
│   ├── basic-wildcards.json
│   └── parameter-matching.json
├── intermediate/
│   ├── conditional-patterns.json
│   ├── nested-structures.json
│   └── multi-pattern-rules.json
└── advanced/
    ├── context-aware.json
    ├── performance-patterns.json
    └── security-patterns.json
```

## Learning Workflow Integration

### Pattern Construction Prompt with Examples
```yaml
name: "build_pattern_with_examples"
description: "Build pattern using example-based learning"
arguments:
  - name: "description"
    description: "Rough description of what to find"
    required: true
  - name: "language"
    description: "Programming language"
    required: true
  - name: "difficulty"
    description: "Desired complexity level"
    required: false
```

### Generated Learning Workflow
```
Building pattern for: "{description}" in {language}

Step 1: Intent Analysis
Based on your description, this appears to be a {intent_category} pattern.

Step 2: Similar Examples
Here are 3 similar patterns that have been successful:

Example 1: {example_description}
Pattern: {pattern}
Explanation: {explanation}
Success rate: {success_rate}

Example 2: {example_description}
Pattern: {pattern}  
Explanation: {explanation}
Success rate: {success_rate}

Example 3: {example_description}
Pattern: {pattern}
Explanation: {explanation}
Success rate: {success_rate}

Step 3: Pattern Construction
Based on these examples, I'll construct your pattern:

Initial pattern: {base_pattern}
Reasoning: {reasoning}

Step 4: Refinement
Let me refine this pattern based on common variations:
- Variation 1: {variation_pattern}
- Variation 2: {variation_pattern}

Final pattern: {final_pattern}

Step 5: Validation
Testing against sample code:
✓ Matches: {matches}
✗ Misses: {misses}
⚠ False positives: {false_positives}

Step 6: Usage Examples
Here's how to use this pattern:
{usage_examples}

Common mistakes to avoid:
- {mistake_1}
- {mistake_2}
```

## Feedback Integration

### Success Tracking
```json
{
  "pattern_id": "js-async-functions-001",
  "usage_stats": {
    "total_uses": 156,
    "successful_uses": 142,
    "success_rate": 0.91,
    "common_failures": [
      "missed async arrow functions",
      "matched non-async functions with 'async' in name"
    ]
  },
  "user_feedback": {
    "helpful_votes": 89,
    "total_votes": 98,
    "comments": [
      "Great explanation of async patterns",
      "Helped me understand arrow function syntax"
    ]
  }
}
```

### Continuous Improvement
```rust
fn update_example_quality(pattern_id: &str, success: bool, feedback: &str) {
    // 1. Update success rates
    // 2. Identify common failure patterns
    // 3. Suggest example improvements
    // 4. Flag examples needing updates
}
```

## Success Metrics

### Learning Effectiveness
- **Pattern Construction Success**: >85% correct patterns from examples
- **Learning Speed**: 50% faster pattern construction vs. trial-and-error
- **Retention**: Users remember pattern principles between sessions
- **Confidence**: Higher success rates increase user confidence

### Example Quality
- **Accuracy**: >95% of examples produce working patterns
- **Completeness**: Examples cover 90% of common use cases
- **Clarity**: >90% of users understand example explanations
- **Relevance**: Examples match user intent >80% of the time

## Implementation Priority

### Phase 1: Core Examples (High Priority)
- Function detection patterns
- Basic control flow patterns
- Variable patterns
- Error handling patterns

### Phase 2: Advanced Examples (Medium Priority)
- Class and object patterns
- Import/export patterns
- Security patterns
- Performance patterns

### Phase 3: Specialization (Low Priority)
- Framework-specific patterns
- Library-specific patterns
- Domain-specific patterns
- Custom pattern categories

## Integration with ast-grep Server

### New Resource Types
```rust
struct ExampleDatabase {
    examples: HashMap<String, LearningExample>,
    progressions: HashMap<String, LearningProgression>,
    similarity_index: SimilarityIndex,
}

struct LearningExample {
    id: String,
    description: String,
    language: String,
    difficulty: Difficulty,
    intent_category: IntentCategory,
    learning_progression: Vec<LearningStep>,
    variations: Vec<String>,
    related_patterns: Vec<String>,
    common_mistakes: Vec<CommonMistake>,
}
```

### Enhanced Pattern Building
```rust
async fn build_pattern_with_examples(
    &self,
    description: String,
    language: String,
    difficulty: Option<Difficulty>
) -> Result<PatternWithExamples> {
    // 1. Classify intent
    // 2. Find similar examples
    // 3. Select learning progression
    // 4. Build pattern with explanation
    // 5. Validate against examples
    // 6. Return with success metrics
}
```

This example-based learning system transforms pattern construction from a trial-and-error process into a guided learning experience, dramatically improving success rates for small LLMs while building understanding of ast-grep concepts.