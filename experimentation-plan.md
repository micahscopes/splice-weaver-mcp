# AST-Grep MCP Server Experimentation Plan

## Goal
Thoroughly test the existing ast-grep-mcp server to understand:
- What works well and feels intuitive
- What's confusing or challenging
- Edge cases and limitations
- Comparison with direct ast-grep CLI usage

## Experimentation Categories

### 1. Basic Pattern Matching
- Simple function detection
- Variable finding
- Control flow patterns
- Cross-language consistency

### 2. Advanced Pattern Features
- Variable capture (`$VAR`)
- Multiple capture (`$$$`)
- Complex nested patterns
- Pattern specificity vs generality

### 3. File Operations
- Single file vs directory scanning
- Large file handling
- File type filtering
- Path pattern matching

### 4. Replacement Operations
- Simple text replacement
- Pattern-based transformation
- Dry-run vs actual changes
- Bulk operations

### 5. Rule-Based Operations
- YAML rule creation
- Rule validation
- Complex rule logic
- Rule management

### 6. AST Understanding
- AST generation
- Node type discovery
- Language-specific differences
- Pattern debugging

### 7. Error Handling & Edge Cases
- Invalid patterns
- Unsupported languages
- File permission issues
- Large scale operations

### 8. Usability & Developer Experience
- Learning curve
- Error messages
- Documentation clarity
- Workflow efficiency

## Test Cases to Try

### Basic Patterns
1. Find all function declarations
2. Find console.log statements
3. Find variable assignments
4. Find specific method calls

### Advanced Patterns
1. Capture function names and parameters
2. Find nested patterns (functions inside objects)
3. Find patterns with conditions
4. Cross-language pattern consistency

### Edge Cases
1. Very large files
2. Binary files
3. Empty files
4. Files with syntax errors
5. Non-standard code formatting

### Comparison Tests
1. MCP server vs direct CLI
2. Different pattern approaches for same goal
3. Performance differences
4. Feature availability

## Success Metrics
- Time to accomplish common tasks
- Number of attempts needed for correct patterns
- Clarity of error messages
- Discoverability of features
- Overall developer satisfaction