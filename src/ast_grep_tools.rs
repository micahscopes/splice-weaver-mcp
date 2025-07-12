use crate::binary_manager::BinaryManager;
use anyhow::{anyhow, Result};
use rmcp::model::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;

pub struct AstGrepTools {
    binary_manager: Arc<BinaryManager>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct Position {
    line: u32,
    column: u32,
}

impl AstGrepTools {
    pub fn new(binary_manager: Arc<BinaryManager>) -> Self {
        Self { binary_manager }
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<String> {
        match tool_name {
            "find_scope" => self.find_scope(arguments).await,
            "execute_rule" => self.execute_rule(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    async fn find_scope(&self, args: Value) -> Result<String> {
        let code = args["code"].as_str().ok_or(anyhow!("Missing code"))?;
        let language = args["language"]
            .as_str()
            .ok_or(anyhow!("Missing language"))?;
        let _position: Position = serde_json::from_value(args["position"].clone())
            .map_err(|_| anyhow!("Missing or invalid position"))?;
        let scope_rule = args["scope_rule"]
            .as_str()
            .ok_or(anyhow!("Missing scope_rule"))?;

        // Create a temporary rule file with position-aware matching
        let temp_rule_file = std::env::temp_dir().join("find_scope_rule.yml");

        // For now, we'll use the provided rule directly
        // In a more sophisticated implementation, we'd inject position constraints
        tokio::fs::write(&temp_rule_file, scope_rule).await?;

        // Write code to temporary file for processing
        let temp_code_file =
            std::env::temp_dir().join(format!("code.{}", self.get_file_extension(language)?));
        tokio::fs::write(&temp_code_file, code).await?;

        let binary_path = self.binary_manager.ensure_binary().await?;
        let output = TokioCommand::new(&binary_path)
            .arg("scan")
            .arg("--rule")
            .arg(&temp_rule_file)
            .arg(&temp_code_file)
            .arg("--json")
            .output()
            .await?;

        // Cleanup
        tokio::fs::remove_file(temp_rule_file).await.ok();
        tokio::fs::remove_file(temp_code_file).await.ok();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ast-grep failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    async fn execute_rule(&self, args: Value) -> Result<String> {
        let rule_config = args["rule_config"]
            .as_str()
            .ok_or(anyhow!("Missing rule_config"))?;
        let target = args["target"].as_str().ok_or(anyhow!("Missing target"))?;
        let operation = args["operation"].as_str().unwrap_or("search");
        let dry_run = args["dry_run"].as_bool().unwrap_or(true);

        // Create temporary rule file
        let temp_rule_file = std::env::temp_dir().join("execute_rule.yml");
        tokio::fs::write(&temp_rule_file, rule_config).await?;

        let binary_path = self.binary_manager.ensure_binary().await?;
        let mut cmd = TokioCommand::new(&binary_path);

        match operation {
            "search" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(target)
                    .arg("--json");
            }
            "replace" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(target)
                    .arg("--json");

                if dry_run {
                    cmd.arg("--dry-run");
                }
            }
            "scan" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(target)
                    .arg("--json");
            }
            _ => return Err(anyhow!("Unknown operation: {}", operation)),
        }

        let output = cmd.output().await?;

        tokio::fs::remove_file(temp_rule_file).await.ok();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ast-grep failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    fn get_file_extension(&self, language: &str) -> Result<&str> {
        match language {
            "javascript" => Ok("js"),
            "typescript" => Ok("ts"),
            "rust" => Ok("rs"),
            "python" => Ok("py"),
            "java" => Ok("java"),
            "go" => Ok("go"),
            "cpp" | "c++" => Ok("cpp"),
            "c" => Ok("c"),
            _ => Err(anyhow!("Unsupported language: {}", language)),
        }
    }

    pub fn list_resources(&self) -> Vec<Resource> {
        let mut resources = vec![
            Resource::new(
                RawResource {
                    uri: "ast-grep://binary-path".to_string(),
                    name: "AST-Grep Binary Path".to_string(),
                    description: Some(
                        "Path to the ast-grep executable for direct CLI access".to_string(),
                    ),
                    mime_type: Some("text/plain".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://cli-reference".to_string(),
                    name: "CLI Reference".to_string(),
                    description: Some(
                        "Complete ast-grep CLI documentation and command reference".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://rule-examples".to_string(),
                    name: "Rule Configuration Examples".to_string(),
                    description: Some(
                        "Examples of YAML rule configurations for common use cases".to_string(),
                    ),
                    mime_type: Some("text/yaml".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://relational-patterns".to_string(),
                    name: "Relational Pattern Examples".to_string(),
                    description: Some(
                        "Examples of inside/has/follows relational rules for scope navigation"
                            .to_string(),
                    ),
                    mime_type: Some("text/yaml".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://node-kinds".to_string(),
                    name: "Tree-sitter Node Kinds".to_string(),
                    description: Some(
                        "Tree-sitter node types by language for kind-based rules".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://cheatsheet/rules".to_string(),
                    name: "Rule Cheat Sheet".to_string(),
                    description: Some("Comprehensive cheat sheet covering Atomic, Relational, and Composite rules with examples".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://cheatsheet/yaml".to_string(),
                    name: "YAML Configuration Cheat Sheet".to_string(),
                    description: Some("Complete reference for ast-grep YAML configuration including linting, patching, and metadata".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
        ];

        // Add dynamic navigation resources
        if let Ok(nav_resources) = self.load_navigation_resources() {
            resources.extend(nav_resources);
        }

        // Add catalog examples from the generated JSON
        if let Ok(catalog_resources) = self.load_catalog_resources() {
            resources.extend(catalog_resources);
        }

        resources
    }

    pub fn read_resource(&self, uri: &str) -> Result<String> {
        match uri {
            "ast-grep://binary-path" => self.get_binary_path(),
            "ast-grep://cli-reference" => self.get_cli_reference(),
            "ast-grep://rule-examples" => self.get_rule_examples(),
            "ast-grep://relational-patterns" => self.get_relational_patterns(),
            "ast-grep://node-kinds" => self.get_node_kinds(),
            "ast-grep://cheatsheet/rules" => self.get_cheatsheet_rules(),
            "ast-grep://cheatsheet/yaml" => self.get_cheatsheet_yaml(),
            _ => {
                // Check if it's a catalog resource
                if uri.starts_with("ast-grep://catalog/") {
                    self.get_catalog_example(uri)
                } else if uri.starts_with("ast-grep://navigation/") {
                    self.get_navigation_content(uri)
                } else {
                    Err(anyhow!("Unknown resource: {}", uri))
                }
            }
        }
    }

    pub fn list_prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt::new(
                "scope_navigation_rule",
                Some("Generate YAML rule to find specific scope types containing target patterns"),
                Some(vec![
                    PromptArgument {
                        name: "scope_type".to_string(),
                        description: Some("Type of scope: 'function', 'class', 'loop', 'block'".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "target_pattern".to_string(),
                        description: Some("Pattern to find within scope: 'console.log', 'await', '$VAR = $VALUE'".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "language".to_string(),
                        description: Some("Programming language".to_string()),
                        required: Some(true),
                    },
                ]),
            ),
            Prompt::new(
                "transform_in_scope",
                Some("Generate YAML rule to transform code within specific scope types"),
                Some(vec![
                    PromptArgument {
                        name: "what".to_string(),
                        description: Some("What to transform: 'var to const', 'function to arrow', 'callback to async'".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "scope_type".to_string(),
                        description: Some("Scope type: 'function', 'class', 'module', 'loop'".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "language".to_string(),
                        description: Some("Programming language".to_string()),
                        required: Some(true),
                    },
                ]),
            ),
        ]
    }

    pub fn get_prompt(&self, name: &str, arguments: HashMap<String, Value>) -> Result<String> {
        match name {
            "scope_navigation_rule" => self.scope_navigation_rule_prompt(arguments),
            "transform_in_scope" => self.transform_in_scope_prompt(arguments),
            _ => Err(anyhow!("Unknown prompt: {}", name)),
        }
    }

    // Resource Content Methods
    fn get_binary_path(&self) -> Result<String> {
        self.binary_manager.get_binary_path()
    }

    fn get_cli_reference(&self) -> Result<String> {
        let binary_path = self
            .get_binary_path()
            .unwrap_or_else(|_| "ast-grep".to_string());

        Ok(format!(
            r#"# AST-Grep CLI Reference

## Binary Information
- **Bundled Binary Path**: `{binary_path}`
- **Version**: 0.38.7
- **Auto-download**: Binary is automatically downloaded if not found

## Core Commands
- `ast-grep run` - One-time search/rewrite (default command)
- `ast-grep scan` - Rule-based code checking
- `ast-grep test` - Rule testing framework
- `ast-grep new` - Project scaffolding
- `ast-grep lsp` - Language server

## Basic Usage
```bash
# Search for patterns
{binary_path} run --pattern 'console.log($$$)' --lang javascript src/

# Replace patterns
{binary_path} run --pattern 'var $VAR = $VALUE' --rewrite 'const $VAR = $VALUE' --lang javascript src/

# Scan with rules
{binary_path} scan --rule rule.yml src/

# Interactive mode
{binary_path} run --pattern 'old()' --rewrite 'new()' --interactive src/

# Get help
{binary_path} --help
{binary_path} run --help
{binary_path} scan --help
```

## Advanced Options
```bash
# JSON output
{binary_path} scan --rule rule.yml --json src/

# Dry run (preview changes)
{binary_path} run --pattern 'old' --rewrite 'new' --dry-run src/

# Specify language explicitly
{binary_path} run --pattern '$VAR = $VALUE' --lang typescript src/

# Multiple files/directories
{binary_path} scan --rule rule.yml src/ tests/ lib/

# Ignore patterns
{binary_path} run --pattern 'debug($$$)' --ignore '**/node_modules/**' src/
```

## Rule Format
```yaml
id: rule-name
language: javascript
message: "Description"
severity: warning
rule:
  pattern: "console.log($$$)"
  # OR kind: call_expression
  # OR relational rules (inside, has, follows, precedes)
fix: "console.debug($$$)"  # Optional
```

## File Extensions by Language
- JavaScript: `.js`, `.jsx`, `.mjs`
- TypeScript: `.ts`, `.tsx`
- Python: `.py`
- Rust: `.rs`
- Java: `.java`
- Go: `.go`
- C/C++: `.c`, `.cpp`, `.h`, `.hpp`

## Configuration
ast-grep uses `sgconfig.yml` for project configuration:
```yaml
ruleDirs: ["rules"]
testConfigs:
  - testDir: tests
    snapshots: __snapshots__
```

## Resources
- Official Documentation: https://ast-grep.github.io/
- CLI Reference: https://ast-grep.github.io/reference/cli.html
- Rule Reference: https://ast-grep.github.io/reference/rule.html
- Pattern Syntax: https://ast-grep.github.io/reference/pattern.html
"#
        ))
    }

    fn get_rule_examples(&self) -> Result<String> {
        Ok(r#"# Common Rule Examples

## Simple Pattern Rule
```yaml
id: find-console-log
language: javascript
message: "Console log found"
severity: info
rule:
  pattern: "console.log($$$)"
fix: "console.debug($$$)"
```

## Kind-Based Rule
```yaml
id: find-functions
language: javascript
message: "Function declaration"
rule:
  kind: function_declaration
```

## Variable Assignment
```yaml
id: modernize-var
language: javascript
message: "Use const instead of var"
rule:
  pattern: "var $VAR = $VALUE"
fix: "const $VAR = $VALUE"
```

## Error Handling
```yaml
id: find-try-catch
language: javascript
rule:
  pattern: "try { $$$ } catch ($ERROR) { $$$ }"
```
"#
        .to_string())
    }

    fn get_relational_patterns(&self) -> Result<String> {
        Ok(r#"# Relational Pattern Examples

## Inside Pattern
```yaml
# Find console.log inside functions
id: console-in-function
language: javascript
rule:
  all:
    - pattern: "console.log($$$)"
    - inside:
        pattern: "function $NAME($$$) { $$$ }"
```

## Has Pattern
```yaml
# Find functions that contain await
id: async-functions
language: javascript
rule:
  all:
    - pattern: "function $NAME($$$) { $$$ }"
    - has:
        pattern: "await $EXPR"
```

## Excluding Nested Scopes
```yaml
# Find patterns at exact level (not nested)
id: exact-level
language: javascript
rule:
  all:
    - pattern: "console.log($$$)"
    - inside:
        pattern: "function $NAME($$$) { $$$ }"
    - not:
        inside:
          all:
            - pattern: "function $INNER($$$) { $$$ }"
            - inside:
                pattern: "function $NAME($$$) { $$$ }"
```

## Multiple Boundaries
```yaml
# Find patterns within multiple scopes
id: multi-boundary
language: javascript
rule:
  all:
    - pattern: "$VAR = $VALUE"
    - inside:
        pattern: "class $CLASS { $$$ }"
    - inside:
        pattern: "function $METHOD($$$) { $$$ }"
```
"#
        .to_string())
    }

    fn get_node_kinds(&self) -> Result<String> {
        Ok(r#"# Tree-sitter Node Kinds by Language

## JavaScript/TypeScript
- `function_declaration`
- `arrow_function`
- `call_expression`
- `variable_declaration`
- `class_declaration`
- `method_definition`
- `if_statement`
- `for_statement`
- `while_statement`
- `try_statement`
- `catch_clause`
- `identifier`
- `string`
- `number`

## Rust
- `function_item`
- `impl_item`
- `struct_item`
- `enum_item`
- `call_expression`
- `match_expression`
- `if_expression`
- `loop_expression`
- `for_expression`
- `while_expression`
- `let_declaration`
- `identifier`

## Python
- `function_definition`
- `class_definition`
- `call`
- `assignment`
- `if_statement`
- `for_statement`
- `while_statement`
- `try_statement`
- `except_clause`
- `identifier`

## To discover all node kinds for a language:
```bash
ast-grep run --pattern '$ANY' --lang <language> <file> --debug-query
```

Or use the `generate_ast` MCP tool with sample code.
"#
        .to_string())
    }

    fn get_cheatsheet_rules(&self) -> Result<String> {
        Ok(r#"# Rule Cheat Sheet

This cheat sheet provides a concise overview of ast-grep's rule object configuration, covering Atomic, Relational, and Composite rules, along with notes on Utility rules.

## ⚛️ Atomic Rules

These are your precision tools, matching individual AST nodes based on their inherent properties.

### Pattern Matching
```yaml
pattern: console.log($ARG)
```
🧩 Match a node by code structure. e.g. `console.log` call with a single `$ARG`

### Pattern with Context
```yaml
pattern:
  context: '{ key: value }'
  selector: pair
```
🧩 To parse ambiguous patterns, use `context` and specify `selector` AST to search.

### Kind Matching
```yaml
kind: if_statement
```
🏷️ Match an AST node by its `kind` name

### Regex Matching
```yaml
regex: ^regex.+$
```
🔍 Matches node text content against a Rust regular expression

### Positional Matching
```yaml
nthChild: 1
```
🔢 Find a node by its **1-based index** among its _named siblings_

### Advanced Positional
```yaml
nthChild:
  position: 2
  reverse: true
  ofRule: { kind: argument_list }
```
🔢 Advanced positional control: `position`, `reverse` (count from end), or filter siblings using `ofRule`

### Range Matching
```yaml
range:
  start: { line: 0, column: 0 }
  end: { line: 0, column: 13 }
```
🎯 Matches a node based on its character span: 0-based, inclusive start, exclusive end

## 🔗 Relational Rules

These powerful rules define how nodes relate to each other structurally. Think of them as your AST GPS!

### Inside
```yaml
inside:
  kind: function_declaration
```
🏠 Target node must appear **inside** its _parent/ancestor_ node matching the sub-rule

### Has
```yaml
has:
  kind: method_definition
```
🌳 Target node must **have** a _child/descendant_ node matching the sub-rule

### Has with Field
```yaml
has:
  kind: statement_block
  field: body
```
🌳 `field` makes `has`/`inside` match nodes by their semantic role

### Precedes
```yaml
precedes:
  pattern: function $FUNC() { $$ }
```
◀️ Target node must appear _before_ another node matching the sub-rule

### Follows
```yaml
follows:
  pattern: let x = 10;
```
▶️ Target node must appear _after_ another node matching the sub-rule

### StopBy
```yaml
inside:
  kind: function_declaration
  stopBy: end
```
🏠 `stopBy` makes relational rules search all the way to the end, not just immediate neighbors

## 🧠 Composite Rules

Combine multiple rules using Boolean logic. Crucially, these operations apply to a single target node!

### All
```yaml
all:
  - pattern: const $VAR = $VALUE
  - has: { kind: string_literal }
```
✅ Node must satisfy **ALL** the rules in the list

### Any
```yaml
any:
  - pattern: let $X = $Y
  - pattern: const $X = $Y
```
🧡 Node must satisfy **AT LEAST ONE** of the rules in the list

### Not
```yaml
not:
  pattern: console.log($$)
```
🚫 Node must **NOT** satisfy the specified sub-rule

### Matches
```yaml
matches: is-function-call
```
🔄 Matches the node if that utility rule matches it. Your gateway to modularity!

## 📦 Utility Rules

Define reusable rule definitions to cut down on duplication and build complex, maintainable rule sets.

### Local Utils
```yaml
rules:
  - id: find-my-pattern
    rule:
      matches: my-local-check
utils:
  my-local-check:
    kind: identifier
    regex: '^my'
```
🏡 Defined within the `utils` field of your current config file. Only accessible within that file.

### Global Utils
```yaml
# In utils/my-global-check.yml
id: my-global-check
language: javascript
rule:
  kind: variable_declarator
  has:
    kind: number_literal
```
🌍 Defined in separate YAML files in global `utilsDirs` folders, accessible across your entire project.
"#.to_string())
    }

    fn get_cheatsheet_yaml(&self) -> Result<String> {
        Ok(r#"# YAML Configuration Cheat Sheet

This cheat sheet provides a concise overview of ast-grep's linter rule YAML configuration.

## ℹ️ Basic Information

Core details that identify and define your rule and miscellaneous keys for documentation and custom data.

### Rule ID
```yaml
id: no-console-log
```
🆔 A unique, descriptive identifier for the rule.

### Language
```yaml
language: JavaScript
```
🌐 The programming language the rule applies to.

### Documentation URL
```yaml
url: 'https://doc.link/'
```
🔗 A URL to the rule's documentation.

### Metadata
```yaml
metadata: { author: 'John Doe' }
```
📓 A dictionary for custom data related to the rule.

## 🔍 Finding Code

Keys for specifying what code to search for.

### Rule
```yaml
rule:
  pattern: 'console.log($$$ARGS)'
```
🎯 The core `rule` to find matching AST nodes.

### Constraints
```yaml
constraints:
  ARG: { kind: 'string' }
```
⚙️ Additional `constraints` rules to filter meta-variable matches.

### Utils
```yaml
utils:
  is-react:
    kind: function_declaration
    has: { kind: jsx_element }
```
🛠️ A dictionary of reusable utility rules. Use them in `matches` to modularize your rules.

## 🛠️ Patching Code

Keys for defining how to automatically fix the found code.

### Transform (Object Form)
```yaml
transform:
  NEW_VAR:
    substring: {endChar: 1, source: $V}
```
🎩 `transform` meta-variables before they are used in `fix`.

### Transform (String Form)
```yaml
transform:
  NEW_VAR: substring($V, endChar=1)
```
🎩 `transform` also accepts string form.

### Fix (Simple)
```yaml
fix: "logger.log($$$ARGS)"
```
🔧 A `fix` string to auto-fix the matched code.

### Fix (Object Form)
```yaml
fix:
  template: "logger.log($$$ARGS)"
  expandEnd: rule
```
🔧 Fix also accepts `FixConfig` object.

### Rewriters
```yaml
rewriters:
- id: remove-quotes
  rule: { pattern: "'$A'" }
  fix: "$A"
```
✍️ A list of `rewriters` for complex transformations.

## 🚦 Linting

Keys for configuring the messages and severity of reported issues.

### Severity
```yaml
severity: warning
```
⚠️ The `severity` level of the linting message.

### Message
```yaml
message: "Avoid using $MATCH in production."
```
💬 A concise `message` explaining the rule. Matched $VAR can be used.

### Note
```yaml
note:
  Use a _logger_ instead of `console`
```
📌 More detailed `note`. It supports Markdown format.

### Labels
```yaml
labels:
  ARG:
    style: 'primary'
    message: 'The argument to log'
```
🎨 Customized `labels` for highlighting parts of the matched code.

### File Patterns
```yaml
files: ['src/**/*.js']
```
✅ Glob `files` patterns to include files for the rule.

### Ignore Patterns
```yaml
ignores: ['test/**/*.js']
```
❌ Glob patterns to exclude files from the rule.
"#.to_string())
    }

    // Prompt Methods - Mad Libs Style Templates
    fn scope_navigation_rule_prompt(&self, args: HashMap<String, Value>) -> Result<String> {
        let scope_type = args
            .get("scope_type")
            .and_then(|v| v.as_str())
            .unwrap_or("function");
        let target_pattern = args
            .get("target_pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("console.log");
        let language = args
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("javascript");

        let scope_pattern = self.get_scope_pattern(scope_type, language)?;

        let template = format!(
            r#"To find {scope_type} containing {target_pattern} in {language}:

```yaml
id: find-{scope_type}-with-{}
language: {language}
message: "Found {scope_type} containing {target_pattern}"
severity: info
rule:
  all:
    - {scope_pattern}
    - has:
        pattern: "{target_pattern}"
```

Use this rule with the execute_rule tool:
- rule_config: (paste the YAML above)
- target: (your file or directory path)
- operation: "search"
"#,
            target_pattern.replace(' ', "-"),
            scope_pattern = scope_pattern
        );

        Ok(template)
    }

    fn transform_in_scope_prompt(&self, args: HashMap<String, Value>) -> Result<String> {
        let what = args
            .get("what")
            .and_then(|v| v.as_str())
            .unwrap_or("var to const");
        let scope_type = args
            .get("scope_type")
            .and_then(|v| v.as_str())
            .unwrap_or("function");
        let language = args
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("javascript");

        let scope_pattern = self.get_scope_pattern(scope_type, language)?;
        let (target_pattern, fix_pattern) = self.get_transform_patterns(what, language)?;

        let template = format!(
            r#"To transform {what} within {scope_type} in {language}:

```yaml
id: transform-{}-in-{}
language: {language}
message: "Transform {what} within {scope_type}"
severity: info
rule:
  all:
    - pattern: "{target_pattern}"
    - inside:
        {scope_pattern}
fix: "{fix_pattern}"
```

Use this rule with the execute_rule tool:
- rule_config: (paste the YAML above)
- target: (your file or directory path)
- operation: "replace"
- dry_run: true (to preview changes)
"#,
            what.replace(' ', "-"),
            scope_type.replace(' ', "-")
        );

        Ok(template)
    }

    fn get_scope_pattern(&self, scope_type: &str, language: &str) -> Result<String> {
        let pattern = match (scope_type, language) {
            ("function", "javascript" | "typescript") => "pattern: \"function $NAME($$$) { $$$ }\"",
            ("function", "rust") => "pattern: \"fn $NAME($$$) -> $RET { $$$ }\"",
            ("function", "python") => "pattern: \"def $NAME($$$): $$$\"",
            ("class", "javascript" | "typescript") => "pattern: \"class $NAME { $$$ }\"",
            ("class", "rust") => "pattern: \"impl $TYPE { $$$ }\"",
            ("class", "python") => "pattern: \"class $NAME: $$$\"",
            ("loop", "javascript" | "typescript") => "pattern: \"for ($$$) { $$$ }\"",
            ("loop", "rust") => "pattern: \"for $VAR in $ITER { $$$ }\"",
            ("loop", "python") => "pattern: \"for $VAR in $ITER: $$$\"",
            ("block", _) => "pattern: \"{ $$$ }\"",
            _ => {
                return Err(anyhow!(
                    "Unsupported scope type '{}' for language '{}'",
                    scope_type,
                    language
                ))
            }
        };
        Ok(pattern.to_string())
    }

    fn get_transform_patterns(&self, what: &str, language: &str) -> Result<(String, String)> {
        let (target, fix) = match (what, language) {
            ("var to const", "javascript" | "typescript") => {
                ("var $VAR = $VALUE", "const $VAR = $VALUE")
            }
            ("function to arrow", "javascript" | "typescript") => {
                ("function($$$) { $$$ }", "($$$) => { $$$ }")
            }
            ("callback to async", "javascript" | "typescript") => {
                ("function($$$) { $$$ }", "async function($$$) { $$$ }")
            }
            ("print to print function", "python") => ("print $ARGS", "print($ARGS)"),
            _ => {
                return Err(anyhow!(
                    "Unsupported transformation '{}' for language '{}'",
                    what,
                    language
                ))
            }
        };
        Ok((target.to_string(), fix.to_string()))
    }

    // Catalog-related methods
    fn load_catalog_resources(&self) -> Result<Vec<Resource>> {
        let catalog_path = std::env::var("OUT_DIR")
            .map(|out_dir| std::path::Path::new(&out_dir).join("catalog.json"))
            .or_else(|_| Ok::<_, anyhow::Error>(std::path::PathBuf::from("target/catalog.json")))
            .unwrap_or_else(|_| std::path::PathBuf::from("catalog.json"));
        
        let catalog_content = std::fs::read_to_string(&catalog_path)
            .map_err(|_| anyhow!("Failed to read catalog.json"))?;
        
        let catalog: Value = serde_json::from_str(&catalog_content)?;
        let mut resources = Vec::new();

        if let Some(examples) = catalog["examples"].as_array() {
            for (index, example) in examples.iter().enumerate() {
                if let Some(content) = example["content"].as_str() {
                    let default_id = format!("example-{}", index);
                    let id = example["id"].as_str().unwrap_or(&default_id);
                    let language = example["language"].as_str().unwrap_or("unknown");
                    let message = example["message"].as_str().unwrap_or("No description");
                    let source_file = example["source_file"].as_str().unwrap_or("unknown");
                    
                    resources.push(Resource::new(
                        RawResource {
                            uri: format!("ast-grep://catalog/{}", id),
                            name: format!("Catalog Rule: {}", id),
                            description: Some(format!("{} (Language: {}, Source: {})", message, language, source_file)),
                            mime_type: Some("text/yaml".to_string()),
                            size: Some(content.len() as u32),
                        },
                        None,
                    ));
                }
            }
        }

        Ok(resources)
    }

    fn get_catalog_example(&self, uri: &str) -> Result<String> {
        let catalog_path = std::env::var("OUT_DIR")
            .map(|out_dir| std::path::Path::new(&out_dir).join("catalog.json"))
            .or_else(|_| Ok::<_, anyhow::Error>(std::path::PathBuf::from("target/catalog.json")))
            .unwrap_or_else(|_| std::path::PathBuf::from("catalog.json"));
        
        let catalog_content = std::fs::read_to_string(&catalog_path)
            .map_err(|_| anyhow!("Failed to read catalog.json"))?;
        
        let catalog: Value = serde_json::from_str(&catalog_content)?;
        let rule_id = uri.strip_prefix("ast-grep://catalog/").unwrap_or("");

        if let Some(examples) = catalog["examples"].as_array() {
            for (index, example) in examples.iter().enumerate() {
                let default_id = format!("example-{}", index);
                let id = example["id"].as_str().unwrap_or(&default_id);
                if id == rule_id {
                    if let Some(content) = example["content"].as_str() {
                        return Ok(content.to_string());
                    }
                }
            }
        }

        Err(anyhow!("Catalog example not found: {}", rule_id))
    }

    // Navigation-related methods
    fn load_navigation_resources(&self) -> Result<Vec<Resource>> {
        let catalog_path = std::env::var("OUT_DIR")
            .map(|out_dir| std::path::Path::new(&out_dir).join("catalog.json"))
            .or_else(|_| Ok::<_, anyhow::Error>(std::path::PathBuf::from("target/catalog.json")))
            .unwrap_or_else(|_| std::path::PathBuf::from("catalog.json"));
        
        let catalog_content = std::fs::read_to_string(&catalog_path)
            .map_err(|_| anyhow!("Failed to read catalog.json"))?;
        
        let catalog: Value = serde_json::from_str(&catalog_content)?;
        let mut resources = Vec::new();

        if let Some(examples) = catalog["examples"].as_array() {
            // Collect unique languages
            let mut languages = std::collections::HashSet::new();
            let mut features = std::collections::HashSet::new();
            let mut rule_types = std::collections::HashSet::new();
            
            for example in examples {
                if let Some(lang) = example["language"].as_str() {
                    languages.insert(lang.to_string());
                }
                if let Some(feature_array) = example["features"].as_array() {
                    for feature in feature_array {
                        if let Some(feature_str) = feature.as_str() {
                            features.insert(feature_str.to_string());
                        }
                    }
                }
                if let Some(rule_array) = example["rules"].as_array() {
                    for rule in rule_array {
                        if let Some(rule_str) = rule.as_str() {
                            rule_types.insert(rule_str.to_string());
                        }
                    }
                }
            }

            // Add language navigation resources
            for language in languages {
                let count = examples.iter()
                    .filter(|ex| ex["language"].as_str() == Some(&language))
                    .count();
                
                resources.push(Resource::new(
                    RawResource {
                        uri: format!("ast-grep://navigation/language/{}", language),
                        name: format!("Catalog: {} Examples", language.to_uppercase()),
                        description: Some(format!("All {} catalog examples for {} programming language", count, language)),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }

            // Add feature navigation resources
            for feature in features {
                let count = examples.iter()
                    .filter(|ex| {
                        ex["features"].as_array()
                            .map_or(false, |arr| arr.iter().any(|f| f.as_str() == Some(&feature)))
                    })
                    .count();
                
                resources.push(Resource::new(
                    RawResource {
                        uri: format!("ast-grep://navigation/feature/{}", feature),
                        name: format!("Catalog: {} Feature", feature),
                        description: Some(format!("All {} examples using the '{}' feature", count, feature)),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }

            // Add examples with fixes
            let fix_count = examples.iter()
                .filter(|ex| ex["has_fix"].as_bool() == Some(true))
                .count();
            
            if fix_count > 0 {
                resources.push(Resource::new(
                    RawResource {
                        uri: "ast-grep://navigation/has-fix".to_string(),
                        name: "Catalog: Examples with Fixes".to_string(),
                        description: Some(format!("All {} examples that include code transformations/fixes", fix_count)),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }

            // Add rule type navigation
            for rule_type in rule_types {
                let count = examples.iter()
                    .filter(|ex| {
                        ex["rules"].as_array()
                            .map_or(false, |arr| arr.iter().any(|r| r.as_str() == Some(&rule_type)))
                    })
                    .count();
                
                resources.push(Resource::new(
                    RawResource {
                        uri: format!("ast-grep://navigation/rule/{}", rule_type),
                        name: format!("Catalog: {} Rules", rule_type),
                        description: Some(format!("All {} examples using '{}' rule type", count, rule_type)),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }
        }

        Ok(resources)
    }

    fn get_navigation_content(&self, uri: &str) -> Result<String> {
        let catalog_path = std::env::var("OUT_DIR")
            .map(|out_dir| std::path::Path::new(&out_dir).join("catalog.json"))
            .or_else(|_| Ok::<_, anyhow::Error>(std::path::PathBuf::from("target/catalog.json")))
            .unwrap_or_else(|_| std::path::PathBuf::from("catalog.json"));
        
        let catalog_content = std::fs::read_to_string(&catalog_path)
            .map_err(|_| anyhow!("Failed to read catalog.json"))?;
        
        let catalog: Value = serde_json::from_str(&catalog_content)?;
        
        if let Some(examples) = catalog["examples"].as_array() {
            if uri.starts_with("ast-grep://navigation/language/") {
                let language = uri.strip_prefix("ast-grep://navigation/language/").unwrap_or("");
                let filtered: Vec<&Value> = examples.iter()
                    .filter(|ex| ex["language"].as_str() == Some(language))
                    .collect();
                
                return Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "language": language,
                    "total_examples": filtered.len(),
                    "examples": filtered.iter().map(|ex| serde_json::json!({
                        "id": ex["id"],
                        "title": ex["title"],
                        "description": ex["description"],
                        "has_fix": ex["has_fix"],
                        "features": ex["features"],
                        "playground_link": ex["playground_link"],
                        "yaml_content": ex["yaml_content"]
                    })).collect::<Vec<_>>()
                }))?);
            }
            
            if uri.starts_with("ast-grep://navigation/feature/") {
                let feature = uri.strip_prefix("ast-grep://navigation/feature/").unwrap_or("");
                let filtered: Vec<&Value> = examples.iter()
                    .filter(|ex| {
                        ex["features"].as_array()
                            .map_or(false, |arr| arr.iter().any(|f| f.as_str() == Some(feature)))
                    })
                    .collect();
                
                return Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "feature": feature,
                    "total_examples": filtered.len(),
                    "examples": filtered.iter().map(|ex| serde_json::json!({
                        "id": ex["id"],
                        "title": ex["title"],
                        "language": ex["language"],
                        "description": ex["description"],
                        "yaml_content": ex["yaml_content"]
                    })).collect::<Vec<_>>()
                }))?);
            }
            
            if uri == "ast-grep://navigation/has-fix" {
                let filtered: Vec<&Value> = examples.iter()
                    .filter(|ex| ex["has_fix"].as_bool() == Some(true))
                    .collect();
                
                return Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "filter": "has_fix",
                    "total_examples": filtered.len(),
                    "examples": filtered.iter().map(|ex| serde_json::json!({
                        "id": ex["id"],
                        "title": ex["title"],
                        "language": ex["language"],
                        "description": ex["description"],
                        "yaml_content": ex["yaml_content"]
                    })).collect::<Vec<_>>()
                }))?);
            }

            if uri.starts_with("ast-grep://navigation/rule/") {
                let rule_type = uri.strip_prefix("ast-grep://navigation/rule/").unwrap_or("");
                let filtered: Vec<&Value> = examples.iter()
                    .filter(|ex| {
                        ex["rules"].as_array()
                            .map_or(false, |arr| arr.iter().any(|r| r.as_str() == Some(rule_type)))
                    })
                    .collect();
                
                return Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "rule_type": rule_type,
                    "total_examples": filtered.len(),
                    "examples": filtered.iter().map(|ex| serde_json::json!({
                        "id": ex["id"],
                        "title": ex["title"],
                        "language": ex["language"],
                        "description": ex["description"],
                        "yaml_content": ex["yaml_content"]
                    })).collect::<Vec<_>>()
                }))?);
            }
        }

        Err(anyhow!("Navigation resource not found: {}", uri))
    }
}
