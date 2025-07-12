# Major CLI Features Missing from MCP Server

## 1. `scan` Command - The Most Powerful Missing Feature

**What it does**: Runs multiple rules simultaneously with rich configuration options.

### Key capabilities NOT in MCP server:
- **Inline rules**: `--inline-rules` - define rules without files
- **Rule filtering**: `--filter REGEX` - run subset of rules
- **Severity controls**: `--error`, `--warning`, `--info`, `--hint`, `--off`
- **Multiple output formats**: GitHub Actions, rich/medium/short styles
- **Interactive editing**: `--interactive` for guided editing
- **Rule metadata**: `--include-metadata` in JSON output
- **Context lines**: `-A`, `-B`, `-C` for showing surrounding code
- **Performance monitoring**: `--inspect` for rule/file discovery tracing

## 2. `test` Command - Rule Testing Framework

**What it does**: Comprehensive testing system for ast-grep rules.

### Missing capabilities:
- **Snapshot testing**: Automatic test generation and comparison
- **Interactive updates**: `--interactive` for selective snapshot updates
- **Test filtering**: `--filter REGEX` to run specific tests
- **Rule validation**: Check if rules are syntactically correct

## 3. `new` Command - Project Scaffolding

**What it does**: Creates ast-grep projects, rules, tests, and utilities.

### Missing capabilities:
- **Project creation**: `new project` - full project scaffolding
- **Rule generation**: `new rule` - creates rule templates
- **Test creation**: `new test` - creates test cases
- **Utility creation**: `new util` - creates reusable utilities

## 4. Enhanced `run` Command Features

### Missing from MCP server:
- **Debug query**: `--debug-query` - shows AST representation
- **Strictness levels**: `--strictness` (cst, smart, ast, relaxed, signature)
- **Context lines**: `-A`, `-B`, `-C` for showing surrounding code
- **Performance monitoring**: `--inspect` for performance analysis
- **Interactive editing**: `--interactive` for guided transformations