# Major CLI Features Missing from MCP Server - Experiment Results

## Summary of Critical Missing Features

After thorough CLI exploration, here are the most important capabilities that the MCP server lacks:

## 🚀 **Top Priority: `scan` Command with Inline Rules**

**What it does**: Run multiple rules simultaneously without creating files.

**Why it's game-changing for LLMs**:
- **No file management**: Rules defined inline, no disk I/O
- **Multiple rules at once**: Batch analysis with different severity levels
- **Rich formatted output**: Beautiful error reports with line numbers and context
- **Exit codes**: Perfect for CI/CD integration

**Tested example**:
```bash
ast-grep scan --inline-rules 'id: find-unwrap
language: rust
message: "Avoid unwrap() calls"
severity: error
rule:
  pattern: "$VAR.unwrap()"
---
id: find-let-bindings
language: rust
message: "Let binding found"
severity: info
rule:
  pattern: "let $VAR = $VALUE"' /path/to/file.rs
```

**Output**: Rich formatted warnings/errors with exact line numbers and severity colors.

## 🎯 **Second Priority: Debug Query (`--debug-query`)**

**What it does**: Shows the AST representation of patterns to help debug why they don't match.

**Why it's crucial for LLMs**:
- **Pattern debugging**: See exactly why a pattern fails
- **AST education**: Learn how patterns map to AST structures
- **Error diagnosis**: Understand parsing errors

**Tested example**:
```bash
ast-grep run --pattern 'println!($$$)' --lang rust --debug-query
```

**Output**:
```
Debug Pattern:
call_expression
  identifier println
  ERROR
    escape_sequence \!  # Shows why the pattern fails!
  arguments
    (
    MetaVar $$$
    )
```

## 🔍 **Third Priority: Context Lines (`-A`, `-B`, `-C`)**

**What it does**: Shows surrounding code lines around matches.

**Why it's valuable for LLMs**:
- **Code context**: Understanding surrounding code structure
- **Boundary analysis**: See what's around the match
- **Better decision making**: Context helps decide if transformation is safe

**Tested example**:
```bash
ast-grep run --pattern '$VAR.unwrap()' --lang rust -C 2 file.rs
```

**Output**:
```
file.rs:9:fn risky_function() -> Result<i32, &'static str> {
file.rs:10:    let value = some_operation()?;
file.rs:11:    Ok(value.unwrap()) // This is bad - double unwrap!
file.rs:12:}
```

## 🏗️ **Fourth Priority: Project Scaffolding (`new`)**

**What it does**: Creates complete project structure with rules, tests, and configuration.

**Why it's helpful for LLMs**:
- **Template generation**: Creates proper rule templates
- **Testing setup**: Automatic test case generation
- **Best practices**: Follows ast-grep conventions

**Tested results**:
- ✅ Created complete project structure
- ✅ Generated rule template with schema validation
- ✅ Created test template with valid/invalid examples
- ✅ Set up proper configuration files

## 🧪 **Fifth Priority: Testing Framework (`test`)**

**What it does**: Comprehensive snapshot-based testing for rules.

**Why it's important for LLMs**:
- **Rule validation**: Ensure rules work as expected
- **Regression testing**: Detect when rule behavior changes
- **Confidence building**: LLMs can verify their rule creation

**Tested results**:
- ✅ Snapshot testing with precise match positions
- ✅ Automatic baseline generation
- ✅ Interactive updates for changed behavior
- ✅ Clear pass/fail reporting

## 📊 **Additional Missing Features**

### Severity Controls
- `--error`, `--warning`, `--info`, `--hint`, `--off` flags
- Control rule severity levels dynamically
- Perfect for different CI/CD environments

### Output Formats
- `--json=pretty|stream|compact` for machine processing
- `--format=github` for GitHub Actions integration
- `--report-style=rich|medium|short` for different verbosity

### Performance Monitoring
- `--inspect=summary|entity` for rule/file discovery tracing
- Thread control with `--threads`
- Performance analysis for large codebases

### Rule Management
- `--filter REGEX` to run subset of rules
- Multiple rule execution from configuration
- Rule metadata inclusion in output

## 💡 **Key Insights for LLM Enhancement**

1. **Inline Rules are Game-Changing**: No file management, just pure pattern execution
2. **Debug Query is Essential**: LLMs need to understand why patterns fail
3. **Context is Critical**: Surrounding code helps make better transformation decisions
4. **Batch Operations**: Multiple rules at once is much more efficient
5. **Rich Output**: Formatted errors with severity levels improve UX dramatically

## 🎯 **Recommended MCP Server Enhancements**

### High Priority
1. Add `scan_inline` tool with inline rules support
2. Add `debug_pattern` tool for pattern debugging
3. Add context line options to all search tools
4. Add severity and formatting controls

### Medium Priority
1. Add project scaffolding tools
2. Add rule testing capabilities
3. Add performance monitoring
4. Add rule filtering and batch execution

### Low Priority
1. Add interactive editing support
2. Add shell completion generation
3. Add documentation generation

## 🔬 **Experiment Conclusions**

The CLI has significantly more capabilities than exposed by the MCP server. The most impactful missing features for LLMs are:

1. **Inline rules** - eliminate file management complexity
2. **Pattern debugging** - help LLMs understand failures
3. **Context lines** - provide code understanding
4. **Batch operations** - improve efficiency
5. **Rich formatting** - better user experience

These features would transform the MCP server from a basic pattern wrapper into a sophisticated code analysis and transformation platform perfectly suited for LLM interaction.