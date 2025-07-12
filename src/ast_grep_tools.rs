use anyhow::{Result, anyhow};
use serde_json::Value;
use tokio::process::Command as TokioCommand;
use rmcp::model::*;
use std::collections::HashMap;

pub struct AstGrepTools;

#[derive(serde::Deserialize)]
struct Position {
    line: u32,
    column: u32,
}

impl AstGrepTools {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<String> {
        match tool_name {
            "find_scope" => self.find_scope(arguments).await,
            "execute_rule" => self.execute_rule(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name))
        }
    }
    
    async fn find_scope(&self, args: Value) -> Result<String> {
        let code = args["code"].as_str().ok_or(anyhow!("Missing code"))?;
        let language = args["language"].as_str().ok_or(anyhow!("Missing language"))?;
        let _position: Position = serde_json::from_value(args["position"].clone())
            .map_err(|_| anyhow!("Missing or invalid position"))?;
        let scope_rule = args["scope_rule"].as_str().ok_or(anyhow!("Missing scope_rule"))?;
        
        // Create a temporary rule file with position-aware matching
        let temp_rule_file = std::env::temp_dir().join("find_scope_rule.yml");
        
        // For now, we'll use the provided rule directly
        // In a more sophisticated implementation, we'd inject position constraints
        tokio::fs::write(&temp_rule_file, scope_rule).await?;
        
        // Write code to temporary file for processing
        let temp_code_file = std::env::temp_dir().join(format!("code.{}", self.get_file_extension(language)?));
        tokio::fs::write(&temp_code_file, code).await?;
        
        let output = TokioCommand::new("ast-grep")
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
        let rule_config = args["rule_config"].as_str().ok_or(anyhow!("Missing rule_config"))?;
        let target = args["target"].as_str().ok_or(anyhow!("Missing target"))?;
        let operation = args["operation"].as_str().unwrap_or("search");
        let dry_run = args["dry_run"].as_bool().unwrap_or(true);
        
        // Create temporary rule file
        let temp_rule_file = std::env::temp_dir().join("execute_rule.yml");
        tokio::fs::write(&temp_rule_file, rule_config).await?;
        
        let mut cmd = TokioCommand::new("ast-grep");
        
        match operation {
            "search" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(target)
                    .arg("--json");
            },
            "replace" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(target)
                    .arg("--json");
                
                if dry_run {
                    cmd.arg("--dry-run");
                }
            },
            "scan" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(target)
                    .arg("--json");
            },
            _ => return Err(anyhow!("Unknown operation: {}", operation))
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
            _ => Err(anyhow!("Unsupported language: {}", language))
        }
    }

    pub fn list_resources(&self) -> Vec<Resource> {
        vec![
            Resource::new(
                RawResource {
                    uri: "ast-grep://binary-path".to_string(),
                    name: "AST-Grep Binary Path".to_string(),
                    description: Some("Path to the ast-grep executable for direct CLI access".to_string()),
                    mime_type: Some("text/plain".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://cli-reference".to_string(),
                    name: "CLI Reference".to_string(),
                    description: Some("Complete ast-grep CLI documentation and command reference".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://rule-examples".to_string(),
                    name: "Rule Configuration Examples".to_string(),
                    description: Some("Examples of YAML rule configurations for common use cases".to_string()),
                    mime_type: Some("text/yaml".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://relational-patterns".to_string(),
                    name: "Relational Pattern Examples".to_string(),
                    description: Some("Examples of inside/has/follows relational rules for scope navigation".to_string()),
                    mime_type: Some("text/yaml".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://node-kinds".to_string(),
                    name: "Tree-sitter Node Kinds".to_string(),
                    description: Some("Tree-sitter node types by language for kind-based rules".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
        ]
    }

    pub fn read_resource(&self, uri: &str) -> Result<String> {
        match uri {
            "ast-grep://binary-path" => self.get_binary_path(),
            "ast-grep://cli-reference" => self.get_cli_reference(),
            "ast-grep://rule-examples" => self.get_rule_examples(),
            "ast-grep://relational-patterns" => self.get_relational_patterns(),
            "ast-grep://node-kinds" => self.get_node_kinds(),
            _ => Err(anyhow!("Unknown resource: {}", uri)),
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
        // Try to find ast-grep in PATH
        match std::process::Command::new("which").arg("ast-grep").output() {
            Ok(output) if output.status.success() => {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            },
            _ => {
                // Fallback to common locations
                let common_paths = [
                    "/usr/local/bin/ast-grep",
                    "/usr/bin/ast-grep",
                    "/opt/homebrew/bin/ast-grep",
                    "ast-grep", // Assume it's in PATH
                ];
                for path in &common_paths {
                    if std::path::Path::new(path).exists() || path == &"ast-grep" {
                        return Ok(path.to_string());
                    }
                }
                Err(anyhow!("ast-grep binary not found"))
            }
        }
    }

    fn get_cli_reference(&self) -> Result<String> {
        Ok(r#"# AST-Grep CLI Reference

## Core Commands
- `ast-grep run` - One-time search/rewrite (default command)
- `ast-grep scan` - Rule-based code checking
- `ast-grep test` - Rule testing framework
- `ast-grep new` - Project scaffolding
- `ast-grep lsp` - Language server

## Basic Usage
```bash
# Search for patterns
ast-grep run --pattern 'console.log($$$)' --lang javascript src/

# Replace patterns
ast-grep run --pattern 'var $VAR = $VALUE' --rewrite 'const $VAR = $VALUE' --lang javascript src/

# Scan with rules
ast-grep scan --rule rule.yml src/

# Interactive mode
ast-grep run --pattern 'old()' --rewrite 'new()' --interactive src/
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
  # OR relational rules
fix: "console.debug($$$)"  # Optional
```

See official docs: https://ast-grep.github.io/reference/cli.html
"#.to_string())
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
"#.to_string())
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
"#.to_string())
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
"#.to_string())
    }

    // Prompt Methods - Mad Libs Style Templates
    fn scope_navigation_rule_prompt(&self, args: HashMap<String, Value>) -> Result<String> {
        let scope_type = args.get("scope_type").and_then(|v| v.as_str()).unwrap_or("function");
        let target_pattern = args.get("target_pattern").and_then(|v| v.as_str()).unwrap_or("console.log");
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("javascript");
        
        let scope_pattern = self.get_scope_pattern(scope_type, language)?;
        
        let template = format!(r#"To find {scope_type} containing {target_pattern} in {language}:

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
"#, target_pattern.replace(' ', "-"), scope_pattern = scope_pattern);
        
        Ok(template)
    }
    
    fn transform_in_scope_prompt(&self, args: HashMap<String, Value>) -> Result<String> {
        let what = args.get("what").and_then(|v| v.as_str()).unwrap_or("var to const");
        let scope_type = args.get("scope_type").and_then(|v| v.as_str()).unwrap_or("function");
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("javascript");
        
        let scope_pattern = self.get_scope_pattern(scope_type, language)?;
        let (target_pattern, fix_pattern) = self.get_transform_patterns(what, language)?;
        
        let template = format!(r#"To transform {what} within {scope_type} in {language}:

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
"#, what.replace(' ', "-"), scope_type.replace(' ', "-"));
        
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
            _ => return Err(anyhow!("Unsupported scope type '{}' for language '{}'", scope_type, language))
        };
        Ok(pattern.to_string())
    }
    
    fn get_transform_patterns(&self, what: &str, language: &str) -> Result<(String, String)> {
        let (target, fix) = match (what, language) {
            ("var to const", "javascript" | "typescript") => ("var $VAR = $VALUE", "const $VAR = $VALUE"),
            ("function to arrow", "javascript" | "typescript") => ("function($$$) { $$$ }", "($$$) => { $$$ }"),
            ("callback to async", "javascript" | "typescript") => ("function($$$) { $$$ }", "async function($$$) { $$$ }"),
            ("print to print function", "python") => ("print $ARGS", "print($ARGS)"),
            _ => return Err(anyhow!("Unsupported transformation '{}' for language '{}'", what, language))
        };
        Ok((target.to_string(), fix.to_string()))
    }
}