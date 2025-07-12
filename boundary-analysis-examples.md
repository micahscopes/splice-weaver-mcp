# AST-Grep Syntactic Boundary Analysis for Refactoring

## The Power of Boundary Analysis

Your insight about syntactic boundaries is spot-on! ast-grep's ability to capture and work with nested structures makes it incredibly powerful for refactoring at specific levels of nesting.

## Real-World Examples

### 1. **Multi-Level Iterator Refactoring**

**Problem**: You want to refactor just the middle level of a nested iterator chain.

```javascript
// Original nested iterators
function processData(items) {
  return items
    .filter(item => item.active)
    .map(item => {
      const processed = item.data
        .filter(data => data.valid)    // ← Want to refactor this level
        .map(data => ({
          id: data.id,
          value: data.value * 2,
          nested: data.children
            .filter(child => child.enabled)
            .map(child => child.name)
        }));
      return { ...item, processed };
    })
    .sort((a, b) => a.id - b.id);
}
```

**ast-grep Solution**: Find the exact boundary and refactor just that level:

```yaml
# Find 2nd-level map operations for refactoring
id: refactor-middle-map
language: javascript
rule:
  all:
    - pattern: "$VAR.map($CALLBACK)"
    - inside:
        pattern: "$OUTER.map($OUTER_CALLBACK)"
    - not:
        inside:
          all:
            - pattern: "$INNER.map($INNER_CALLBACK)"
            - inside:
                pattern: "$VAR.map($CALLBACK)"
```

### 2. **Scope-Aware Variable Extraction**

**Problem**: Extract variables only within specific syntactic boundaries.

```javascript
function complexCalculation() {
  const result = items.map(item => {
    // Want to extract just the calculation logic within this scope
    const intermediate = item.value * 2 + item.bonus;
    const adjusted = intermediate > 100 ? intermediate * 0.9 : intermediate;
    return adjusted;
  });
}
```

**ast-grep Solution**: Find the exact scope and extract variables:

```yaml
id: extract-calculation-logic
language: javascript
rule:
  all:
    - pattern: "$CALC"
    - inside:
        pattern: "$VAR.map($CALLBACK)"
    - not:
        inside:
          pattern: "$NESTED.map($NESTED_CALLBACK)"
```

### 3. **Nested Loop Level Detection**

**Problem**: Refactor specific levels of nested loops.

```javascript
for (let i = 0; i < items.length; i++) {
  for (let j = 0; j < items[i].data.length; j++) {
    for (let k = 0; k < items[i].data[j].children.length; k++) {
      // Want to refactor this innermost level to use forEach
      console.log(items[i].data[j].children[k]);
    }
  }
}
```

**ast-grep Solution**: Target specific nesting levels:

```yaml
id: innermost-loop
language: javascript
rule:
  all:
    - kind: for_statement
    - inside:
        kind: for_statement
    - inside:
        all:
          - kind: for_statement
          - inside:
              kind: for_statement
```

## Boundary Analysis Patterns

### 1. **Finding Containing Structures**

```yaml
# Find the function that contains a specific pattern
id: find-containing-function
language: javascript
rule:
  all:
    - pattern: "function $NAME($$$) { $$$ }"
    - has:
        pattern: "specificCode()"
```

### 2. **Excluding Inner Boundaries**

```yaml
# Find patterns at exactly this level (not deeper)
id: exact-level-only
language: javascript
rule:
  all:
    - pattern: "$TARGET_PATTERN"
    - inside:
        pattern: "$BOUNDARY_PATTERN"
    - not:
        inside:
          all:
            - pattern: "$INNER_BOUNDARY"
            - inside:
                pattern: "$BOUNDARY_PATTERN"
```

### 3. **Multi-Boundary Analysis**

```yaml
# Find patterns within multiple boundary types
id: multi-boundary
language: javascript
rule:
  all:
    - pattern: "$PATTERN"
    - inside:
        pattern: "class $CLASS { $$$ }"
    - inside:
        pattern: "function $METHOD($$$) { $$$ }"
    - inside:
        pattern: "if ($CONDITION) { $$$ }"
```

## Practical Refactoring Workflows

### 1. **Progressive Boundary Discovery**

```bash
# Step 1: Find all instances of the pattern
ast-grep search --pattern 'console.log($VAR)' --lang javascript

# Step 2: Find which are inside functions
ast-grep rule --config 'rule: { all: [{ pattern: "console.log($VAR)" }, { inside: { pattern: "function $NAME($$$) { $$$ }" }}]}'

# Step 3: Find which are inside nested functions
ast-grep rule --config 'rule: { all: [{ pattern: "console.log($VAR)" }, { inside: { all: [{ pattern: "function $INNER($$$) { $$$ }" }, { inside: { pattern: "function $OUTER($$$) { $$$ }" }}]}}]}'
```

### 2. **Boundary-Aware Replacement**

```yaml
# Replace only at specific syntactic boundaries
id: boundary-aware-replace
language: javascript
rule:
  all:
    - pattern: "var $NAME = $VALUE"
    - inside:
        pattern: "function $FUNC($$$) { $$$ }"
    - not:
        inside:
          pattern: "function $INNER($$$) { $$$ }"
fix: "const $NAME = $VALUE"
```

### 3. **Context-Sensitive Transformations**

```yaml
# Different transformations based on syntactic context
id: context-sensitive
language: javascript
rule:
  any:
    - all:
        - pattern: "$EXPR"
        - inside:
            pattern: "async function $NAME($$$) { $$$ }"
      # Apply async-specific transformations
    - all:
        - pattern: "$EXPR"
        - inside:
            pattern: "function $NAME($$$) { $$$ }"
        - not:
            inside:
              pattern: "async function $NAME($$$) { $$$ }"
      # Apply sync-specific transformations
```

## Advanced Boundary Techniques

### 1. **Scope Chain Analysis**

```yaml
# Track variable usage across scope boundaries
id: scope-chain
language: javascript
rule:
  all:
    - pattern: "$VAR"
    - inside:
        pattern: "function $INNER($$$) { $$$ }"
    - not:
        inside:
          any:
            - pattern: "const $VAR = $$$"
            - pattern: "let $VAR = $$$"
            - pattern: "var $VAR = $$$"
            - pattern: "function $VAR($$$) { $$$ }"
```

### 2. **Nested Component Detection**

```yaml
# Find React components at specific nesting levels
id: nested-components
language: javascript
rule:
  all:
    - pattern: "function $COMPONENT($PROPS) { return $JSX }"
    - inside:
        pattern: "function $PARENT($PARENT_PROPS) { return $PARENT_JSX }"
```

### 3. **Error Boundary Mapping**

```yaml
# Map error handling patterns to their boundaries
id: error-boundaries
language: javascript
rule:
  all:
    - pattern: "catch ($ERROR) { $HANDLER }"
    - inside:
        pattern: "try { $$$ } catch ($ERROR) { $HANDLER }"
    - inside:
        any:
          - pattern: "function $FUNC($$$) { $$$ }"
          - pattern: "async function $FUNC($$$) { $$$ }"
```

## Why This Is Powerful for LLMs

1. **Precise Targeting**: LLMs can specify exactly which level of nesting to refactor
2. **Context Preservation**: Maintain surrounding code structure while transforming specific parts
3. **Progressive Refinement**: Start broad, then narrow down to specific boundaries
4. **Pattern Composition**: Combine simple patterns to handle complex nested structures
5. **Safe Refactoring**: Ensure transformations happen only in the intended scope

## Implementation Strategy

For an MCP server enhancement, you could provide:

1. **Boundary Discovery Prompts**: Help LLMs identify syntactic boundaries
2. **Level-Specific Patterns**: Pre-built patterns for common nesting scenarios
3. **Progressive Refinement Tools**: Start with broad matches, then narrow down
4. **Context-Aware Suggestions**: Recommend patterns based on surrounding code structure
5. **Boundary Visualization**: Show nesting levels and their relationships

This would make ast-grep incredibly powerful for complex refactoring tasks that require surgical precision at specific syntactic boundaries!