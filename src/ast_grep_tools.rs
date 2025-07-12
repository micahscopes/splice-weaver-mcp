use crate::binary_manager::BinaryManager;
use crate::simple_search::SimpleSearchEngine;
use anyhow::{anyhow, Result};
use rmcp::model::*;
use rust_embed::RustEmbed;
use serde_json::Value;
use serde_yaml;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::process::Command as TokioCommand;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

pub struct AstGrepTools {
    binary_manager: Arc<BinaryManager>,
    search_engine: Arc<Mutex<Option<SimpleSearchEngine>>>,
    roots: Arc<Mutex<Vec<Root>>>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct Position {
    line: u32,
    column: u32,
}

fn get_embedded_catalog() -> Result<String> {
    match Assets::get("catalog.json") {
        Some(catalog_file) => std::str::from_utf8(&catalog_file.data)
            .map(|s| s.to_string())
            .map_err(|e| anyhow!("Failed to parse embedded catalog.json as UTF-8: {}", e)),
        None => Err(anyhow!("Embedded catalog.json not found")),
    }
}

impl AstGrepTools {
    pub fn new(binary_manager: Arc<BinaryManager>) -> Self {
        Self {
            binary_manager,
            search_engine: Arc::new(Mutex::new(None)),
            roots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_roots(&self, roots: Vec<Root>) {
        *self.roots.lock().unwrap() = roots;
    }

    pub fn resolve_path(&self, target: &str) -> Result<PathBuf> {
        let target_path = Path::new(target);

        // If it's already an absolute path, return it as-is
        if target_path.is_absolute() {
            return Ok(target_path.to_path_buf());
        }

        // Get the roots
        let roots = self.roots.lock().unwrap();

        // If no roots are configured, default to current directory
        if roots.is_empty() {
            let current_dir = std::env::current_dir()
                .map_err(|e| anyhow!("Failed to get current directory: {}", e))?;
            return Ok(current_dir.join(target_path));
        }

        // Try each root in order until we find the first match
        for root in roots.iter() {
            let root_path = match root.uri.strip_prefix("file://") {
                Some(path) => PathBuf::from(path),
                None => {
                    // If it's not a file URI, try to parse it as a direct path
                    PathBuf::from(&root.uri)
                }
            };

            let resolved_path = root_path.join(target_path);

            // Check if the resolved path exists
            if resolved_path.exists() {
                return Ok(resolved_path);
            }
        }

        // If no matches found, return an error with helpful information
        let root_names: Vec<String> = roots
            .iter()
            .map(|r| r.name.clone().unwrap_or_else(|| r.uri.clone()))
            .collect();

        Err(anyhow!(
            "Path '{}' not found in any root directory. Available roots: {}",
            target,
            root_names.join(", ")
        ))
    }

    fn with_search_engine<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&SimpleSearchEngine) -> Result<R>,
    {
        let mut engine_guard = self.search_engine.lock().unwrap();

        if engine_guard.is_none() {
            // Initialize search engine with embedded catalog
            let catalog_content = get_embedded_catalog()?;
            let engine = SimpleSearchEngine::from_content(&catalog_content)?;
            *engine_guard = Some(engine);
        }

        f(engine_guard.as_ref().unwrap())
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<String> {
        match tool_name {
            "find_scope" => self.find_scope(arguments).await,
            "execute_rule" => self.execute_rule(arguments).await,
            "search_examples" => self.search_examples(arguments).await,
            "similarity_search" => self.similarity_search_tool(arguments).await,
            "suggest_examples" => self.suggest_examples(arguments).await,
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

    fn validate_rule_yaml(&self, rule_config: &str) -> Result<()> {
        // First, validate YAML syntax
        let parsed: serde_yaml::Value = serde_yaml::from_str(rule_config)
            .map_err(|e| anyhow!("Invalid YAML syntax: {}\n\nPlease check your YAML formatting. Common issues:\n- Incorrect indentation (use spaces, not tabs)\n- Missing colons after keys\n- Unmatched quotes or brackets\n\nExample of correct format:\n{}", e, self.get_example_rule()))?;

        // Ensure we have a mapping (object), not a scalar or array
        let rule_map = parsed.as_mapping()
            .ok_or_else(|| anyhow!("Rule must be a YAML object, not a scalar value or array.\n\nExample of correct format:\n{}", self.get_example_rule()))?;

        // Check for required fields
        let mut missing_fields = Vec::new();

        if !rule_map.contains_key("id") {
            missing_fields.push("id");
        }
        if !rule_map.contains_key("language") {
            missing_fields.push("language");
        }
        if !rule_map.contains_key("rule") {
            missing_fields.push("rule");
        }

        if !missing_fields.is_empty() {
            return Err(anyhow!(
                "Missing required fields: {}\n\nRequired fields for ast-grep rules:\n- id: unique identifier for the rule\n- language: programming language (e.g., 'javascript', 'rust', 'python')\n- rule: the search/replace pattern definition\n\nExample of correct format:\n{}",
                missing_fields.join(", "),
                self.get_example_rule()
            ));
        }

        // Validate language field
        if let Some(language) = rule_map.get("language").and_then(|v| v.as_str()) {
            self.validate_language(language)?;
        } else {
            return Err(anyhow!("Language field must be a string.\n\nSupported languages: javascript, typescript, rust, python, java, go, cpp, c\n\nExample of correct format:\n{}", self.get_example_rule()));
        }

        // Validate rule structure
        if let Some(rule) = rule_map.get("rule") {
            if rule.as_mapping().is_none() {
                return Err(anyhow!("Rule field must be an object containing pattern or other rule definitions.\n\nExample of correct format:\n{}", self.get_example_rule()));
            }
        }

        Ok(())
    }

    fn validate_language(&self, language: &str) -> Result<()> {
        match language {
            "javascript" | "typescript" | "rust" | "python" | "java" | "go" | "cpp" | "c++" | "c" => Ok(()),
            _ => Err(anyhow!(
                "Unsupported language: '{}'\n\nSupported languages: javascript, typescript, rust, python, java, go, cpp, c\n\nExample of correct format:\n{}",
                language,
                self.get_example_rule()
            ))
        }
    }

    fn get_example_rule(&self) -> &'static str {
        r#"id: example-rule
language: javascript
rule:
  pattern: function $NAME($$$PARAMS) { $$$BODY }
  # Optional: add constraints, transformations, etc.
  # constraints:
  #   - pattern: $NAME
  #     regex: "^[a-z]"
  # transform:
  #   $NAME: "new_$NAME""#
    }

    async fn execute_rule(&self, args: Value) -> Result<String> {
        let rule_config = args["rule_config"]
            .as_str()
            .ok_or(anyhow!("Missing rule_config"))?;
        let target = args["target"].as_str().ok_or(anyhow!("Missing target"))?;
        let operation = args["operation"].as_str().unwrap_or("search");
        let dry_run = args["dry_run"].as_bool().unwrap_or(true);

        // Validate the YAML rule configuration before processing
        self.validate_rule_yaml(rule_config)?;

        // Resolve the target path using MCP roots
        let resolved_target = self.resolve_path(target)?;

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
                    .arg(&resolved_target)
                    .arg("--json");
            }
            "replace" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(&resolved_target);

                if dry_run {
                    // When dry_run is true, add --json to get structured output without applying
                    cmd.arg("--json");
                } else {
                    // When dry_run is false, apply the fixes using -U without --json
                    cmd.arg("-U");
                }
            }
            "scan" => {
                cmd.arg("scan")
                    .arg("--rule")
                    .arg(&temp_rule_file)
                    .arg(&resolved_target)
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
            // Discovery and help resources (most important for smaller models)
            Resource::new(
                RawResource {
                    uri: "ast-grep://discover".to_string(),
                    name: "🔍 Resource Discovery".to_string(),
                    description: Some(
                        "Complete guide to all available resources and how to use them".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://languages".to_string(),
                    name: "📚 Available Languages".to_string(),
                    description: Some(
                        "List of supported programming languages with example URIs for each".to_string(),
                    ),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),

            // Core documentation resources
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

            // Quick access to popular language examples
            Resource::new(
                RawResource {
                    uri: "ast-grep://examples/javascript".to_string(),
                    name: "JavaScript Examples".to_string(),
                    description: Some("JavaScript/TypeScript pattern examples and rules".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://examples/python".to_string(),
                    name: "Python Examples".to_string(),
                    description: Some("Python pattern examples and rules".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://examples/rust".to_string(),
                    name: "Rust Examples".to_string(),
                    description: Some("Rust pattern examples and rules".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://examples/java".to_string(),
                    name: "Java Examples".to_string(),
                    description: Some("Java pattern examples and rules".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
            Resource::new(
                RawResource {
                    uri: "ast-grep://examples/go".to_string(),
                    name: "Go Examples".to_string(),
                    description: Some("Go pattern examples and rules".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                    size: None,
                },
                None,
            ),
        ];

        // Add status information about catalog loading
        let catalog_status = self.get_catalog_status();
        resources.push(Resource::new(
            RawResource {
                uri: "ast-grep://catalog-status".to_string(),
                name: "📊 Catalog Status".to_string(),
                description: Some(format!(
                    "Catalog loading status: {}",
                    catalog_status.summary
                )),
                mime_type: Some("text/markdown".to_string()),
                size: None,
            },
            None,
        ));

        // Add dynamic navigation resources with better error handling
        match self.load_navigation_resources() {
            Ok(nav_resources) => {
                resources.extend(nav_resources);
            }
            Err(e) => {
                // Add error resource instead of failing silently
                resources.push(Resource::new(
                    RawResource {
                        uri: "ast-grep://navigation-error".to_string(),
                        name: "⚠️ Navigation Error".to_string(),
                        description: Some(format!("Failed to load navigation resources: {}", e)),
                        mime_type: Some("text/plain".to_string()),
                        size: None,
                    },
                    None,
                ));
            }
        }

        // Add catalog examples with better error handling
        match self.load_catalog_resources() {
            Ok(catalog_resources) => {
                resources.extend(catalog_resources);
            }
            Err(e) => {
                // Add error resource instead of failing silently
                resources.push(Resource::new(
                    RawResource {
                        uri: "ast-grep://catalog-error".to_string(),
                        name: "⚠️ Catalog Error".to_string(),
                        description: Some(format!("Failed to load catalog resources: {}", e)),
                        mime_type: Some("text/plain".to_string()),
                        size: None,
                    },
                    None,
                ));
            }
        }

        resources
    }

    pub fn read_resource(&self, uri: &str) -> Result<String> {
        match uri {
            // Discovery and help resources
            "ast-grep://discover" => self.get_discovery_guide(),
            "ast-grep://languages" => self.get_available_languages(),
            "ast-grep://catalog-status" => self.get_catalog_status_content(),
            "ast-grep://navigation-error" => self.get_navigation_error_content(),
            "ast-grep://catalog-error" => self.get_catalog_error_content(),

            // Core resources
            "ast-grep://binary-path" => self.get_binary_path(),
            "ast-grep://cli-reference" => self.get_cli_reference(),
            "ast-grep://rule-examples" => self.get_rule_examples(),
            "ast-grep://relational-patterns" => self.get_relational_patterns(),
            "ast-grep://node-kinds" => self.get_node_kinds(),
            "ast-grep://cheatsheet/rules" => self.get_cheatsheet_rules(),
            "ast-grep://cheatsheet/yaml" => self.get_cheatsheet_yaml(),

            // Legacy static URIs (deprecated, use dynamic ones)
            "ast-grep://examples-by-language" => self.get_examples_by_language(),
            "ast-grep://pattern-syntax" => self.get_pattern_syntax(),
            "ast-grep://rule-composition" => self.get_rule_composition(),
            _ => {
                // Handle dynamic resources
                if let Some(content) = self.handle_dynamic_resource(uri)? {
                    Ok(content)
                } else if uri.starts_with("ast-grep://catalog/") {
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
        let catalog_content = get_embedded_catalog()?;

        let catalog: Value = serde_json::from_str(&catalog_content)
            .map_err(|e| anyhow!("Failed to parse catalog.json: {}", e))?;

        let mut resources = Vec::new();

        if let Some(examples) = catalog["examples"].as_array() {
            for (index, example) in examples.iter().enumerate() {
                if let Some(content) = example["content"].as_str() {
                    let default_id = format!("example-{index}");
                    let id = example["id"].as_str().unwrap_or(&default_id);
                    let language = example["language"].as_str().unwrap_or("unknown");
                    let title = example["title"].as_str().unwrap_or(id);
                    let description = example["description"].as_str().unwrap_or("No description");
                    let has_fix = example["has_fix"].as_bool().unwrap_or(false);

                    // Create more informative resource names and descriptions
                    let name = if has_fix {
                        format!("🔧 {title}")
                    } else {
                        format!("📝 {title}")
                    };

                    let full_description = format!(
                        "{} [{}{}] {}",
                        description,
                        language.to_uppercase(),
                        if has_fix { " + Fix" } else { "" },
                        if let Some(playground) = example["playground_link"].as_str() {
                            if !playground.is_empty() {
                                "• Has playground"
                            } else {
                                ""
                            }
                        } else {
                            ""
                        }
                    );

                    resources.push(Resource::new(
                        RawResource {
                            uri: format!("ast-grep://catalog/{id}"),
                            name,
                            description: Some(full_description),
                            mime_type: Some("text/yaml".to_string()),
                            size: Some(content.len() as u32),
                        },
                        None,
                    ));
                } else {
                    // Log warning for malformed examples but continue processing
                    eprintln!(
                        "Warning: Catalog example {} is missing content field",
                        index
                    );
                }
            }
        } else {
            return Err(anyhow!("Catalog file is missing 'examples' array"));
        }

        Ok(resources)
    }

    fn get_catalog_example(&self, uri: &str) -> Result<String> {
        let catalog_content = get_embedded_catalog()?;

        let catalog: Value = serde_json::from_str(&catalog_content)?;
        let rule_id = uri.strip_prefix("ast-grep://catalog/").unwrap_or("");

        if let Some(examples) = catalog["examples"].as_array() {
            for (index, example) in examples.iter().enumerate() {
                let default_id = format!("example-{index}");
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
        let catalog_content = get_embedded_catalog()?;

        let catalog: Value = serde_json::from_str(&catalog_content)
            .map_err(|e| anyhow!("Failed to parse catalog.json for navigation: {}", e))?;

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
                let count = examples
                    .iter()
                    .filter(|ex| ex["language"].as_str() == Some(&language))
                    .count();

                resources.push(Resource::new(
                    RawResource {
                        uri: format!("ast-grep://navigation/language/{language}"),
                        name: format!("Catalog: {} Examples", language.to_uppercase()),
                        description: Some(format!(
                            "All {count} catalog examples for {language} programming language"
                        )),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }

            // Add feature navigation resources
            for feature in features {
                let count = examples
                    .iter()
                    .filter(|ex| {
                        ex["features"]
                            .as_array()
                            .is_some_and(|arr| arr.iter().any(|f| f.as_str() == Some(&feature)))
                    })
                    .count();

                resources.push(Resource::new(
                    RawResource {
                        uri: format!("ast-grep://navigation/feature/{feature}"),
                        name: format!("Catalog: {feature} Feature"),
                        description: Some(format!(
                            "All {count} examples using the '{feature}' feature"
                        )),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }

            // Add examples with fixes
            let fix_count = examples
                .iter()
                .filter(|ex| ex["has_fix"].as_bool() == Some(true))
                .count();

            if fix_count > 0 {
                resources.push(Resource::new(
                    RawResource {
                        uri: "ast-grep://navigation/has-fix".to_string(),
                        name: "Catalog: Examples with Fixes".to_string(),
                        description: Some(format!(
                            "All {fix_count} examples that include code transformations/fixes"
                        )),
                        mime_type: Some("application/json".to_string()),
                        size: None,
                    },
                    None,
                ));
            }

            // Add rule type navigation
            for rule_type in rule_types {
                let count = examples
                    .iter()
                    .filter(|ex| {
                        ex["rules"]
                            .as_array()
                            .is_some_and(|arr| arr.iter().any(|r| r.as_str() == Some(&rule_type)))
                    })
                    .count();

                resources.push(Resource::new(
                    RawResource {
                        uri: format!("ast-grep://navigation/rule/{rule_type}"),
                        name: format!("Catalog: {rule_type} Rules"),
                        description: Some(format!(
                            "All {count} examples using '{rule_type}' rule type"
                        )),
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
        let catalog_content = get_embedded_catalog()?;

        let catalog: Value = serde_json::from_str(&catalog_content)?;

        if let Some(examples) = catalog["examples"].as_array() {
            if uri.starts_with("ast-grep://navigation/language/") {
                let language = uri
                    .strip_prefix("ast-grep://navigation/language/")
                    .unwrap_or("");
                let filtered: Vec<&Value> = examples
                    .iter()
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
                let feature = uri
                    .strip_prefix("ast-grep://navigation/feature/")
                    .unwrap_or("");
                let filtered: Vec<&Value> = examples
                    .iter()
                    .filter(|ex| {
                        ex["features"]
                            .as_array()
                            .is_some_and(|arr| arr.iter().any(|f| f.as_str() == Some(feature)))
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
                let filtered: Vec<&Value> = examples
                    .iter()
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
                let rule_type = uri
                    .strip_prefix("ast-grep://navigation/rule/")
                    .unwrap_or("");
                let filtered: Vec<&Value> = examples
                    .iter()
                    .filter(|ex| {
                        ex["rules"]
                            .as_array()
                            .is_some_and(|arr| arr.iter().any(|r| r.as_str() == Some(rule_type)))
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

    fn handle_dynamic_resource(&self, uri: &str) -> Result<Option<String>> {
        // Parse dynamic URI patterns
        if let Some(content) = self.parse_docs_uri(uri)? {
            return Ok(Some(content));
        }

        if let Some(content) = self.parse_examples_uri(uri)? {
            return Ok(Some(content));
        }

        if let Some(content) = self.parse_patterns_uri(uri)? {
            return Ok(Some(content));
        }

        if let Some(content) = self.parse_query_uri(uri)? {
            return Ok(Some(content));
        }

        Ok(None)
    }

    fn parse_docs_uri(&self, uri: &str) -> Result<Option<String>> {
        if let Some(doc_type) = uri.strip_prefix("ast-grep://docs/") {
            let (doc_type, params) = self.parse_uri_params(doc_type);

            match doc_type {
                "examples-by-language" => Ok(Some(self.get_examples_by_language()?)),
                "pattern-syntax" => Ok(Some(self.get_pattern_syntax()?)),
                "rule-composition" => Ok(Some(self.get_rule_composition()?)),
                _ if doc_type.starts_with("language-guide/") => {
                    let language = doc_type
                        .strip_prefix("language-guide/")
                        .unwrap_or("javascript");
                    Ok(Some(self.get_language_guide(language, &params)?))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_examples_uri(&self, uri: &str) -> Result<Option<String>> {
        if let Some(language) = uri.strip_prefix("ast-grep://examples/") {
            let (language, params) = self.parse_uri_params(language);
            Ok(Some(self.get_language_examples(language, &params)?))
        } else {
            Ok(None)
        }
    }

    fn parse_patterns_uri(&self, uri: &str) -> Result<Option<String>> {
        if let Some(category) = uri.strip_prefix("ast-grep://patterns/") {
            let (category, params) = self.parse_uri_params(category);
            Ok(Some(self.get_pattern_category(category, &params)?))
        } else {
            Ok(None)
        }
    }

    fn parse_query_uri(&self, uri: &str) -> Result<Option<String>> {
        if let Some(query_part) = uri.strip_prefix("ast-grep://query/") {
            let (query_type, params) = self.parse_uri_params(query_part);
            Ok(Some(self.handle_query_resource(query_type, &params)?))
        } else {
            Ok(None)
        }
    }

    fn parse_uri_params<'a>(
        &self,
        input: &'a str,
    ) -> (&'a str, std::collections::HashMap<String, String>) {
        let mut params = std::collections::HashMap::new();

        if let Some((base, query_string)) = input.split_once('?') {
            for param_pair in query_string.split('&') {
                if let Some((key, value)) = param_pair.split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                }
            }
            (base, params)
        } else {
            (input, params)
        }
    }

    fn get_examples_by_language(&self) -> Result<String> {
        Ok(r#"# Examples by Language

Quick reference of common ast-grep patterns organized by programming language.

## JavaScript/TypeScript

### Variable Declarations
```yaml
# Find var declarations
rule:
  pattern: "var $VAR = $VALUE"

# Find all variable declarations (var, let, const)
rule:
  any:
    - pattern: "var $VAR = $VALUE"
    - pattern: "let $VAR = $VALUE"
    - pattern: "const $VAR = $VALUE"
```

### Function Patterns
```yaml
# Function declarations
rule:
  pattern: "function $NAME($PARAMS) { $BODY }"

# Arrow functions
rule:
  pattern: "($PARAMS) => $BODY"

# Method calls
rule:
  pattern: "$OBJECT.$METHOD($ARGS)"
```

### Common Patterns
```yaml
# Console statements
rule:
  pattern: "console.$METHOD($ARGS)"

# Async/await
rule:
  pattern: "await $EXPR"

# Import statements
rule:
  pattern: "import $ITEMS from '$MODULE'"
```

## Python

### Function Definitions
```yaml
# Function definitions
rule:
  pattern: "def $NAME($PARAMS): $BODY"

# Class definitions
rule:
  pattern: "class $NAME: $BODY"

# Method definitions
rule:
  pattern: "def $NAME(self, $PARAMS): $BODY"
```

### Common Patterns
```yaml
# Print statements
rule:
  pattern: "print($ARGS)"

# If statements
rule:
  pattern: "if $CONDITION: $BODY"

# For loops
rule:
  pattern: "for $VAR in $ITER: $BODY"
```

## Rust

### Function Items
```yaml
# Function definitions
rule:
  pattern: "fn $NAME($PARAMS) -> $RET { $BODY }"

# Public functions
rule:
  pattern: "pub fn $NAME($PARAMS) -> $RET { $BODY }"

# Method definitions
rule:
  pattern: "impl $TYPE { fn $NAME($PARAMS) -> $RET { $BODY } }"
```

### Error Handling
```yaml
# Match expressions
rule:
  pattern: "match $EXPR { $ARMS }"

# Result handling
rule:
  pattern: "$EXPR?"

# Unwrap calls
rule:
  pattern: "$EXPR.unwrap()"
```

## Java

### Class Elements
```yaml
# Class declarations
rule:
  pattern: "class $NAME { $BODY }"

# Method declarations
rule:
  pattern: "public $RET $NAME($PARAMS) { $BODY }"

# Field declarations
rule:
  pattern: "private $TYPE $NAME;"
```

### Common Patterns
```yaml
# System.out.println
rule:
  pattern: "System.out.println($ARGS)"

# Try-catch blocks
rule:
  pattern: "try { $BODY } catch ($EX) { $HANDLER }"
```

## Go

### Function Definitions
```yaml
# Function declarations
rule:
  pattern: "func $NAME($PARAMS) $RET { $BODY }"

# Method declarations
rule:
  pattern: "func ($RECV) $NAME($PARAMS) $RET { $BODY }"
```

### Error Handling
```yaml
# Error checking
rule:
  pattern: "if err != nil { $BODY }"

# Function calls with error
rule:
  pattern: "$VAR, err := $CALL"
```

## Usage Tips

1. Use `$$$` for multiple items: `console.log($$$)` matches any number of arguments
2. Use `$$` for statement sequences: `{ $$ }` matches any block content
3. Combine with constraints to be more specific:
   ```yaml
   rule:
     pattern: "$VAR = $VALUE"
   constraints:
     VAR: { regex: "^[A-Z_]+$" }  # Only CONSTANTS
   ```
"#
        .to_string())
    }

    fn get_pattern_syntax(&self) -> Result<String> {
        Ok(r#"# Pattern Syntax Reference

Concise guide to ast-grep pattern syntax for quick reference.

## Meta-variables

### Single Node Variables
- `$VAR` - Matches exactly one AST node
- `$_` - Anonymous variable (matches one node, don't capture)

### Multi-Node Variables  
- `$$$VAR` - Matches zero or more nodes in sequence
- `$$$` - Anonymous multi-match

### Statement Variables
- `$$VAR` - Matches zero or more statements
- `$$` - Anonymous statement sequence

## Basic Patterns

### Exact Match
```yaml
pattern: "console.log('hello')"  # Matches exactly this code
```

### Variable Capture
```yaml
pattern: "console.log($MSG)"     # Captures the argument as $MSG
```

### Multiple Arguments
```yaml
pattern: "console.log($$$ARGS)"  # Captures all arguments
```

### Method Calls
```yaml
pattern: "$OBJ.$METHOD($$$)"     # Any method call on any object
```

## Pattern Types

### Expression Patterns
- `$VAR = $VALUE` - Assignment
- `$FUNC($$$)` - Function call
- `$OBJ.$PROP` - Property access
- `$A + $B` - Binary operation

### Statement Patterns
- `if ($COND) { $$ }` - If statement
- `for ($INIT; $COND; $UPDATE) { $$ }` - For loop
- `try { $$ } catch ($E) { $$ }` - Try-catch

### Declaration Patterns
- `function $NAME($$$) { $$ }` - Function declaration
- `class $NAME { $$ }` - Class declaration
- `const $VAR = $VALUE` - Variable declaration

## Advanced Features

### Context Patterns
When a pattern is ambiguous, use context:
```yaml
pattern:
  context: "class Foo { bar() }"
  selector: method_definition
```

### Whitespace Handling
Patterns ignore most whitespace differences:
```yaml
pattern: "if($COND){$$}"  # Matches "if (cond) { ... }"
```

### Comments
Patterns typically ignore comments:
```yaml
pattern: "$VAR = $VALUE"  # Matches with or without comments
```

## Language-Specific Notes

### JavaScript/TypeScript
- Use `function` for function declarations
- Use `=>` for arrow functions
- Template literals: `` `${}` ``

### Python
- Indentation matters in patterns
- Use `:` for statement endings
- `def` for functions, `class` for classes

### Rust
- Use `->` for return types
- Match expressions: `match $EXPR { $$ }`
- Lifetime annotations: `&'$LT $TYPE`

### Java
- Access modifiers: `public`, `private`, etc.
- Generics: `List<$TYPE>`
- Annotations: `@$ANNOTATION`

## Common Mistakes

❌ **Don't** use regex in patterns:
```yaml
pattern: "console\.log\(.*\)"  # This won't work
```

✅ **Do** use meta-variables:
```yaml
pattern: "console.log($$$)"    # This works
```

❌ **Don't** be too specific with whitespace:
```yaml
pattern: "if ( $COND ) { $$ }"  # Too restrictive
```

✅ **Do** let ast-grep handle whitespace:
```yaml
pattern: "if ($COND) { $$ }"    # More flexible
```

## Quick Reference

| Pattern | Matches | Example |
|---------|---------|---------|
| `$VAR` | Single node | Variable, literal, expression |
| `$$$` | Multiple nodes | Function arguments, array elements |
| `$$` | Statements | Block contents, statement sequences |
| `$_.method()` | Anonymous match | Any object's method call |
| `$$$_` | Anonymous multi | Any number of items (don't capture) |
"#
        .to_string())
    }

    fn get_rule_composition(&self) -> Result<String> {
        Ok(r#"# Rule Composition Guide

Learn to build complex ast-grep rules by combining simple patterns.

## Building Blocks

### 1. Atomic Rules
Start with simple, single-purpose patterns:
```yaml
# Find console.log calls
basic_console:
  pattern: "console.log($$$)"

# Find function declarations  
basic_function:
  pattern: "function $NAME($$$) { $$ }"
```

### 2. Relational Rules
Add context using relationships:
```yaml
# Console.log inside functions
console_in_function:
  all:
    - pattern: "console.log($$$)"
    - inside:
        pattern: "function $NAME($$$) { $$ }"
```

### 3. Composite Rules
Combine multiple conditions:
```yaml
# Complex condition
complex_rule:
  all:
    - pattern: "$VAR = $VALUE"
    - inside: { kind: function_declaration }
    - not: { inside: { kind: arrow_function } }
```

## Composition Patterns

### AND Logic (all)
All conditions must match:
```yaml
rule:
  all:
    - pattern: "console.log($MSG)"
    - inside: { pattern: "function $NAME($$$) { $$ }" }
    - not: { pattern: "console.log('debug')" }
# Finds console.log in functions, but not debug messages
```

### OR Logic (any)
At least one condition must match:
```yaml
rule:
  any:
    - pattern: "console.log($$$)"
    - pattern: "console.error($$$)"
    - pattern: "console.warn($$$)"
# Finds any console method call
```

### NOT Logic (not)
Exclude specific patterns:
```yaml
rule:
  all:
    - pattern: "$FUNC($$$)"
    - not: { pattern: "console.$METHOD($$$)" }
# Function calls except console methods
```

## Common Composition Recipes

### 1. Scope-Limited Search
Find patterns within specific scopes:
```yaml
# Variables declared in constructors
rule:
  all:
    - pattern: "this.$PROP = $VALUE"
    - inside:
        pattern: "constructor($$$) { $$ }"
```

### 2. Multi-Level Nesting
Navigate through multiple scope levels:
```yaml
# Methods that use console in classes
rule:
  all:
    - pattern: "console.log($$$)"
    - inside: { kind: method_definition }
    - inside: { kind: class_declaration }
```

### 3. Excluding Nested Scopes
Find patterns at exact scope level:
```yaml
# Direct class methods (not nested functions)
rule:
  all:
    - pattern: "function $NAME($$$) { $$ }"
    - inside: { kind: class_declaration }
    - not:
        inside:
          all:
            - kind: function_declaration
            - inside: { kind: class_declaration }
```

### 4. Conditional Transformations
Transform only when conditions are met:
```yaml
# Convert var to const only in functions
rule:
  all:
    - pattern: "var $VAR = $VALUE"
    - inside: { kind: function_declaration }
    - has: { pattern: "$VAR" }  # Variable is used
fix: "const $VAR = $VALUE"
```

## Advanced Compositions

### Utility Rules for Modularity
Break complex rules into reusable parts:
```yaml
utils:
  is_react_component:
    any:
      - has: { pattern: "return <$TAG" }
      - has: { pattern: "React.createElement" }
  
  is_async_function:
    any:
      - pattern: "async function $NAME($$$) { $$ }"
      - pattern: "async ($$$) => $BODY"

rules:
  - id: no-console-in-react-async
    rule:
      all:
        - pattern: "console.log($$$)"
        - inside: { matches: is_react_component }
        - inside: { matches: is_async_function }
```

### Constraint-Based Refinement
Use constraints to refine matches:
```yaml
rule:
  pattern: "$OBJ.$METHOD($$$)"
constraints:
  OBJ: { regex: "^[A-Z]" }      # Capitalized objects only
  METHOD: { regex: "^get" }     # Getter methods only
```

### Sequential Patterns
Find patterns that follow specific sequences:
```yaml
# Variable declared then immediately used
rule:
  all:
    - pattern: "const $VAR = $VALUE"
    - follows:
        pattern: "$VAR.$METHOD($$$)"
```

## Building Strategy

### 1. Start Simple
Begin with a basic pattern:
```yaml
pattern: "console.log($$$)"
```

### 2. Add Context
Specify where it should appear:
```yaml
all:
  - pattern: "console.log($$$)"
  - inside: { kind: function_declaration }
```

### 3. Refine with Constraints
Add more specific conditions:
```yaml
all:
  - pattern: "console.log($MSG)"
  - inside: { kind: function_declaration }
  - not: { pattern: "console.log('TODO')" }
```

### 4. Test and Iterate
Run the rule and refine based on results:
```bash
ast-grep scan --rule my-rule.yml test-files/
```

## Tips for Effective Composition

1. **Start broad, then narrow**: Begin with simple patterns and add restrictions
2. **Use utils for reusability**: Extract common patterns into utility rules
3. **Test incrementally**: Verify each composition step works correctly
4. **Consider performance**: Complex nested rules can be slow on large codebases
5. **Document your logic**: Use clear rule IDs and comments

## Example: Complete Rule Development

```yaml
# Goal: Find TODO comments in production code (not test files)

# Step 1: Basic pattern
pattern: "// TODO: $MESSAGE"

# Step 2: Add exclusions
all:
  - pattern: "// TODO: $MESSAGE"
  - not: { inside: { pattern: "describe($$$)" } }  # Not in tests

# Step 3: Add file constraints
all:
  - pattern: "// TODO: $MESSAGE"  
  - not: { inside: { pattern: "describe($$$)" } }
files: ["src/**/*.js"]
ignores: ["**/*.test.js", "**/*.spec.js"]

# Step 4: Add metadata
id: no-todo-in-production
message: "TODO comments should not remain in production code"
severity: warning
note: |
  Consider:
  - Creating a proper issue tracker entry
  - Implementing the feature
  - Removing the comment if no longer relevant
```
"#
        .to_string())
    }

    // Dynamic resource content generators
    fn get_language_guide(
        &self,
        language: &str,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let level = params
            .get("level")
            .map(|s| s.as_str())
            .unwrap_or("beginner");
        let focus = params
            .get("focus")
            .map(|s| s.as_str())
            .unwrap_or("patterns");

        Ok(format!(
            r#"# {language} AST-Grep Guide ({level} level)

## Focus: {focus}

This guide provides {level}-level patterns and examples for {language} development.

## Common Patterns for {language}

{patterns}

## Advanced Techniques

{advanced}

## Best Practices

{best_practices}

"#,
            language = language.to_title_case(),
            level = level,
            focus = focus,
            patterns = self.get_language_patterns(language)?,
            advanced = self.get_advanced_patterns(language, level)?,
            best_practices = self.get_best_practices(language)?
        ))
    }

    fn get_language_examples(
        &self,
        language: &str,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let category = params.get("category").map(|s| s.as_str()).unwrap_or("all");
        let complexity = params
            .get("complexity")
            .map(|s| s.as_str())
            .unwrap_or("basic");

        let examples = self.filter_examples_by_criteria(language, category, complexity)?;

        Ok(format!(
            r#"# {language} Examples

**Category**: {category}  
**Complexity**: {complexity}

{examples}

## Usage

These examples can be copied directly into your ast-grep rules or used as templates.

"#,
            language = language.to_title_case(),
            category = category,
            complexity = complexity,
            examples = examples
        ))
    }

    fn get_pattern_category(
        &self,
        category: &str,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let language = params.get("language").map(|s| s.as_str()).unwrap_or("any");
        let with_fixes = params.get("fixes").map(|s| s == "true").unwrap_or(false);

        let patterns = self.get_categorized_patterns(category, language, with_fixes)?;

        Ok(format!(
            r#"# {category} Patterns

**Language Filter**: {language}  
**Include Fixes**: {with_fixes}

{patterns}

## Related Categories

{related}

"#,
            category = category.to_title_case(),
            language = language,
            with_fixes = with_fixes,
            patterns = patterns,
            related = self.get_related_categories(category)?
        ))
    }

    fn handle_query_resource(
        &self,
        query_type: &str,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        match query_type {
            "search" => self.handle_search_query(params),
            "filter" => self.handle_filter_query(params),
            "similar" => self.handle_similar_query(params),
            _ => Err(anyhow!("Unknown query type: {}", query_type)),
        }
    }

    fn handle_search_query(
        &self,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let query = params
            .get("q")
            .ok_or_else(|| anyhow!("Missing search query"))?;
        let language = params.get("lang").map(|s| s.as_str()).unwrap_or("any");
        let limit = params
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        // Get actual results to count them
        let search_results = self.with_search_engine(|engine| {
            let lang_filter = if language == "any" {
                None
            } else {
                Some(language)
            };
            engine.search(query, lang_filter, limit)
        })?;

        // Search through catalog for matching patterns (format results)
        let results_display = self.search_catalog(query, language, limit)?;

        Ok(format!(
            r#"# Search Results for "{query}"

**Language**: {language}  
**Results**: {count} matches (limited to {limit})

{results}

"#,
            query = query,
            language = language,
            count = search_results.len(),
            limit = limit,
            results = results_display
        ))
    }

    fn handle_filter_query(
        &self,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let has_fix = params.get("has_fix").map(|s| s == "true");
        let language = params.get("language").map(|s| s.as_str());
        let features = params
            .get("features")
            .map(|s| s.split(',').collect::<Vec<_>>());

        let filtered = self.filter_catalog(has_fix, language, features.as_deref())?;

        Ok(format!(
            r#"# Filtered Catalog

**Filters Applied**:
{filters}

**Results**: {count} patterns

{results}

"#,
            filters = self.format_filters(has_fix, language, features.as_deref()),
            count = filtered.len(),
            results = filtered
        ))
    }

    fn handle_similar_query(
        &self,
        params: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let pattern = params
            .get("pattern")
            .ok_or_else(|| anyhow!("Missing pattern"))?;
        let limit = params
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let results = self.with_search_engine(|engine| engine.similarity_search(pattern, limit))?;

        if results.is_empty() {
            return Ok(format!(
                "## No Similar Patterns Found\n\nNo examples found similar to the provided pattern.\n\n**Pattern**: {}\n\n**Suggestion**: Try using simpler or more general terms in your pattern.",
                pattern
            ));
        }

        let mut output = format!("# Similar Patterns to \"{}\"\n\n", pattern);
        output.push_str(&format!(
            "**Found {} similar examples:**\n\n",
            results.len()
        ));

        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("### {}. {}\n\n", i + 1, result.title));
            output.push_str(&format!(
                "**Language**: {} | **Similarity Score**: {:.2}\n",
                result.language, result.score
            ));

            if result.has_fix {
                output.push_str("**Has Fix**: Yes ✅\n");
            }

            output.push_str(&format!("\n**Description**: {}\n\n", result.description));

            if !result.yaml_content.is_empty() {
                output.push_str("**YAML Rule**:\n```yaml\n");
                output.push_str(&result.yaml_content);
                output.push_str("\n```\n\n");
            }

            if !result.playground_link.is_empty() {
                output.push_str(&format!(
                    "**[Try it in Playground]({})**\n\n",
                    result.playground_link
                ));
            }

            output.push_str("---\n\n");
        }

        // Add caveat message for similarity search
        output.push_str("## ⚠️ Similarity Search Note\n\n");
        output.push_str(
            "These examples were found based on text similarity to your provided pattern. ",
        );
        output.push_str("**The similarity scoring is based on common terms and concepts, not exact pattern matching.** ");
        output.push_str("Review each example carefully to determine if it's relevant to your specific use case.\n\n");
        output.push_str("Consider these as inspiration and adapt them to your exact requirements.");

        Ok(output)
    }

    // Helper methods for dynamic content generation
    fn get_language_patterns(&self, language: &str) -> Result<String> {
        match language.to_lowercase().as_str() {
            "javascript" | "typescript" => Ok(r#"
### Variable Declarations
```yaml
rule:
  any:
    - pattern: "var $VAR = $VALUE"
    - pattern: "let $VAR = $VALUE"  
    - pattern: "const $VAR = $VALUE"
```

### Function Calls
```yaml
rule:
  pattern: "$OBJ.$METHOD($$$ARGS)"
```
"#
            .to_string()),
            "python" => Ok(r#"
### Function Definitions
```yaml
rule:
  pattern: "def $NAME($$$PARAMS): $$$BODY"
```

### Class Definitions
```yaml
rule:
  pattern: "class $NAME: $$$BODY"
```
"#
            .to_string()),
            "rust" => Ok(r#"
### Function Items
```yaml
rule:
  pattern: "fn $NAME($$$PARAMS) -> $RET { $$$BODY }"
```

### Match Expressions
```yaml
rule:
  pattern: "match $EXPR { $$$ARMS }"
```
"#
            .to_string()),
            _ => Ok("Generic patterns available for all languages.".to_string()),
        }
    }

    fn get_advanced_patterns(&self, language: &str, level: &str) -> Result<String> {
        if level == "advanced" {
            Ok(format!(
                "Advanced {language} patterns with complex relational rules and constraints."
            ))
        } else {
            Ok("Intermediate patterns with basic relational rules.".to_string())
        }
    }

    fn get_best_practices(&self, language: &str) -> Result<String> {
        Ok(format!(
            "Best practices for writing maintainable {language} ast-grep rules."
        ))
    }

    fn filter_examples_by_criteria(
        &self,
        language: &str,
        category: &str,
        complexity: &str,
    ) -> Result<String> {
        // This would filter the actual catalog based on criteria
        Ok(format!(
            "Filtered examples for {language}, category: {category}, complexity: {complexity}"
        ))
    }

    fn get_categorized_patterns(
        &self,
        category: &str,
        language: &str,
        with_fixes: bool,
    ) -> Result<String> {
        Ok(format!(
            "Patterns for category: {category}, language: {language}, fixes: {with_fixes}"
        ))
    }

    fn get_related_categories(&self, category: &str) -> Result<String> {
        Ok(format!("Related categories for: {category}"))
    }

    fn search_catalog(&self, query: &str, language: &str, limit: usize) -> Result<String> {
        let results = self.with_search_engine(|engine| {
            let lang_filter = if language == "any" {
                None
            } else {
                Some(language)
            };
            engine.search(query, lang_filter, limit)
        })?;

        if results.is_empty() {
            return Ok(format!(
                "## No Results Found\n\nNo examples found matching '{}' in {} language.\n\n**Suggestion**: Try:\n- Using broader search terms\n- Searching in 'any' language\n- Using different keywords related to your problem",
                query, language
            ));
        }

        let mut output = String::new();

        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("### {}. {}\n\n", i + 1, result.title));
            output.push_str(&format!("**Language**: {}\n", result.language));
            output.push_str(&format!("**Score**: {:.2}\n", result.score));

            if result.has_fix {
                output.push_str("**Has Fix**: Yes ✅\n");
            }

            if !result.features.is_empty() {
                output.push_str(&format!("**Features**: {}\n", result.features.join(", ")));
            }

            output.push_str(&format!("\n**Description**: {}\n\n", result.description));

            if !result.yaml_content.is_empty() {
                output.push_str("**YAML Rule**:\n```yaml\n");
                output.push_str(&result.yaml_content);
                output.push_str("\n```\n\n");
            }

            if !result.playground_link.is_empty() {
                output.push_str(&format!(
                    "**[Try it in Playground]({})**\n\n",
                    result.playground_link
                ));
            }

            output.push_str("---\n\n");
        }

        // Add caveat message
        output.push_str("## ⚠️ Important Note\n\n");
        output.push_str(
            "These examples are provided to help you understand similar patterns and approaches. ",
        );
        output.push_str("**Please don't assume these examples will be a perfect fit for your specific use case.** ");
        output.push_str("You may need to adapt the patterns, rules, or logic to match your exact requirements.\n\n");
        output.push_str("Consider these examples as starting points and learning resources rather than drop-in solutions.");

        Ok(output)
    }

    fn get_total_patterns(&self) -> Result<usize> {
        Ok(42) // From the catalog
    }

    fn filter_catalog(
        &self,
        has_fix: Option<bool>,
        language: Option<&str>,
        features: Option<&[&str]>,
    ) -> Result<String> {
        Ok(format!("Filtered catalog results (fix: {has_fix:?}, lang: {language:?}, features: {features:?})"))
    }

    fn format_filters(
        &self,
        has_fix: Option<bool>,
        language: Option<&str>,
        features: Option<&[&str]>,
    ) -> String {
        let mut filters = Vec::new();
        if let Some(fix) = has_fix {
            filters.push(format!("- Has fix: {fix}"));
        }
        if let Some(lang) = language {
            filters.push(format!("- Language: {lang}"));
        }
        if let Some(feats) = features {
            filters.push(format!("- Features: {}", feats.join(", ")));
        }
        filters.join("\n")
    }

    // New MCP tools for search functionality
    async fn search_examples(&self, args: Value) -> Result<String> {
        let query = args["query"].as_str().ok_or(anyhow!("Missing query"))?;
        let language = args["language"].as_str().unwrap_or("any");
        let limit = args["limit"].as_u64().unwrap_or(10) as usize;
        let offset = args["offset"].as_u64().unwrap_or(0) as usize;

        let results = self.with_search_engine(|engine| {
            let lang_filter = if language == "any" {
                None
            } else {
                Some(language)
            };
            engine.search_paginated(query, lang_filter, limit, offset)
        })?;

        if results.results.is_empty() {
            if offset > 0 {
                return Ok(format!(
                    "No more examples found at offset {} for '{}' in {} language.",
                    offset, query, language
                ));
            } else {
                return Ok(format!(
                    "No examples found matching '{}' in {} language. Try broader search terms or search in 'any' language.",
                    query, language
                ));
            }
        }

        let mut output = format!(
            "Found {} of {} total examples for '{}' in {} (page {}-{}):\n\n",
            results.results.len(),
            results.pagination.total_count,
            query,
            language,
            offset + 1,
            offset + results.results.len()
        );

        for (i, result) in results.results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({})\n",
                offset + i + 1,
                result.title,
                result.language
            ));
            output.push_str(&format!("   Description: {}\n", result.description));
            if result.has_fix {
                output.push_str("   ✅ Has Fix\n");
            }
            if !result.features.is_empty() {
                output.push_str(&format!("   Features: {}\n", result.features.join(", ")));
            }
            output.push_str("\n");
        }

        if results.pagination.has_more {
            output.push_str(&format!(
                "📄 More results available. Use offset: {} to get next page ({} more results).\n\n",
                offset + limit,
                results.pagination.total_count - offset - results.results.len()
            ));
        }

        output.push_str(
            "⚠️ Note: These are starting points - adapt them to your specific requirements.",
        );
        Ok(output)
    }

    // New discovery and status methods
    fn get_discovery_guide(&self) -> Result<String> {
        Ok(r#"# 🔍 AST-Grep MCP Resource Discovery Guide

Welcome! This guide helps you navigate all available resources in the AST-Grep MCP server.

## 🚀 Quick Start Resources

**For beginners, start with these:**
- `ast-grep://languages` - See all supported programming languages
- `ast-grep://examples/javascript` - JavaScript/TypeScript examples  
- `ast-grep://examples/python` - Python examples
- `ast-grep://rule-examples` - Basic rule configurations
- `ast-grep://cheatsheet/rules` - Comprehensive rule reference

## 📚 Core Documentation

### Essential References
- `ast-grep://cli-reference` - Complete CLI documentation
- `ast-grep://cheatsheet/rules` - Rule syntax cheat sheet
- `ast-grep://cheatsheet/yaml` - YAML configuration reference
- `ast-grep://binary-path` - Get the ast-grep executable path

### Pattern References  
- `ast-grep://relational-patterns` - Inside/has/follows examples
- `ast-grep://node-kinds` - Tree-sitter node types by language

## 🎯 Language-Specific Examples

### Direct Access (Most Popular)
- `ast-grep://examples/javascript` - JavaScript/TypeScript
- `ast-grep://examples/python` - Python  
- `ast-grep://examples/rust` - Rust
- `ast-grep://examples/java` - Java
- `ast-grep://examples/go` - Go

### Any Language
- `ast-grep://examples/{language}` - Replace {language} with: cpp, csharp, ruby, php, etc.

## 🔧 Dynamic Resources

### Pattern Categories
- `ast-grep://patterns/variables` - Variable declarations and usage
- `ast-grep://patterns/functions` - Function definitions and calls
- `ast-grep://patterns/loops` - Loop constructs
- `ast-grep://patterns/classes` - Class definitions
- `ast-grep://patterns/error-handling` - Try/catch, error patterns
- `ast-grep://patterns/imports` - Import/require statements

### Documentation Types
- `ast-grep://docs/examples-by-language` - All examples organized by language
- `ast-grep://docs/pattern-syntax` - Pattern syntax reference
- `ast-grep://docs/rule-composition` - Building complex rules
- `ast-grep://docs/language-guide/{lang}` - Detailed language guides

### Query Interface
- `ast-grep://query/search?q=console.log` - Search catalog by term
- `ast-grep://query/filter?has_fix=true` - Find rules with fixes
- `ast-grep://query/similar?pattern=function` - Find similar patterns

## 📊 Catalog Resources

### Status and Navigation
- `ast-grep://catalog-status` - Check catalog loading status
- `ast-grep://navigation/language/{lang}` - Examples by language
- `ast-grep://navigation/feature/{feature}` - Examples by feature
- `ast-grep://navigation/has-fix` - Examples with code fixes

### Individual Examples
- `ast-grep://catalog/{id}` - Specific catalog examples (browse navigation first)

## 💡 Tips for Smaller Models

1. **Start Simple**: Begin with `ast-grep://examples/{your-language}`
2. **Use Discovery**: Check `ast-grep://languages` for supported languages
3. **Clear URIs**: All resource URIs follow the pattern `ast-grep://category/specifics`
4. **Error Info**: If something fails, error resources show what went wrong
5. **Status Check**: Use `ast-grep://catalog-status` to troubleshoot catalog issues

## 📖 Usage Examples

### Finding JavaScript Patterns
```
1. Read ast-grep://examples/javascript
2. For more: ast-grep://navigation/language/javascript  
3. Specific pattern: ast-grep://patterns/functions
```

### Creating Rules
```
1. Start with: ast-grep://rule-examples
2. Learn syntax: ast-grep://cheatsheet/rules
3. Advanced: ast-grep://docs/rule-composition
```

### Language Support
```
1. Check: ast-grep://languages
2. Examples: ast-grep://examples/{your-language}
3. Guide: ast-grep://docs/language-guide/{your-language}
```

## 🆘 Troubleshooting

- **Empty results?** Check `ast-grep://catalog-status`
- **Unknown language?** See `ast-grep://languages` for supported ones
- **Need examples?** Start with `ast-grep://examples/{language}`
- **Complex rules?** Read `ast-grep://docs/rule-composition`

This MCP server provides 50+ resources to help you master AST-based code transformation!"#
            .to_string())
    }

    fn get_available_languages(&self) -> Result<String> {
        Ok(r#"# 📚 Supported Programming Languages

ast-grep supports the following programming languages through Tree-sitter grammars.

## 🔥 Most Popular (with direct example access)

| Language | URI | Description |
|----------|-----|-------------|
| **JavaScript** | `ast-grep://examples/javascript` | JavaScript and TypeScript patterns |
| **Python** | `ast-grep://examples/python` | Python 3.x patterns and idioms |
| **Rust** | `ast-grep://examples/rust` | Rust patterns and error handling |
| **Java** | `ast-grep://examples/java` | Java class and method patterns |
| **Go** | `ast-grep://examples/go` | Go function and error patterns |

## 🌐 All Supported Languages

### Systems Programming
- **C** - `ast-grep://examples/c`
- **C++** - `ast-grep://examples/cpp` 
- **Rust** - `ast-grep://examples/rust`
- **Go** - `ast-grep://examples/go`
- **Zig** - `ast-grep://examples/zig`

### Web & Application Development  
- **JavaScript** - `ast-grep://examples/javascript`
- **TypeScript** - `ast-grep://examples/typescript` (same as javascript)
- **Python** - `ast-grep://examples/python`
- **Java** - `ast-grep://examples/java`
- **C#** - `ast-grep://examples/csharp`
- **Kotlin** - `ast-grep://examples/kotlin`
- **Scala** - `ast-grep://examples/scala`

### Functional Programming
- **Haskell** - `ast-grep://examples/haskell`
- **OCaml** - `ast-grep://examples/ocaml`
- **Elm** - `ast-grep://examples/elm`
- **Clojure** - `ast-grep://examples/clojure`

### Scripting & Dynamic
- **Ruby** - `ast-grep://examples/ruby`
- **PHP** - `ast-grep://examples/php`
- **Lua** - `ast-grep://examples/lua`
- **Perl** - `ast-grep://examples/perl`

### Mobile Development
- **Swift** - `ast-grep://examples/swift`
- **Dart** - `ast-grep://examples/dart`

### Shell & Configuration
- **Bash** - `ast-grep://examples/bash`
- **YAML** - `ast-grep://examples/yaml`
- **JSON** - `ast-grep://examples/json`
- **TOML** - `ast-grep://examples/toml`

### Markup & Documentation  
- **HTML** - `ast-grep://examples/html`
- **CSS** - `ast-grep://examples/css`
- **Markdown** - `ast-grep://examples/markdown`

## 🎯 Usage Patterns

### Get Examples for Any Language
```
ast-grep://examples/{language}
```

### Check Available Patterns
```
ast-grep://navigation/language/{language}
```

### Language-Specific Guides
```
ast-grep://docs/language-guide/{language}
```

## 📝 File Extensions by Language

| Language | Extensions | Example |
|----------|------------|---------|
| JavaScript | `.js`, `.jsx`, `.mjs` | `ast-grep://examples/javascript` |
| TypeScript | `.ts`, `.tsx` | `ast-grep://examples/javascript` |
| Python | `.py`, `.pyw` | `ast-grep://examples/python` |
| Rust | `.rs` | `ast-grep://examples/rust` |
| Java | `.java` | `ast-grep://examples/java` |
| C/C++ | `.c`, `.cpp`, `.h`, `.hpp` | `ast-grep://examples/cpp` |
| Go | `.go` | `ast-grep://examples/go` |

## 💡 Quick Tips

1. **Case Sensitive**: Use exact language names (lowercase)
2. **Aliases**: `typescript` works same as `javascript`
3. **Patterns**: All languages support the same pattern syntax
4. **Node Types**: Use `ast-grep://node-kinds` for language-specific AST nodes

## 🔍 Don't See Your Language?

If your language isn't listed:
1. Check if it has a Tree-sitter grammar
2. File an issue at the ast-grep repository
3. Consider using a similar language's patterns as a starting point"#
            .to_string())
    }

    fn get_catalog_status(&self) -> CatalogStatus {
        match get_embedded_catalog() {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(catalog) => {
                    let example_count = catalog["examples"]
                        .as_array()
                        .map(|arr| arr.len())
                        .unwrap_or(0);

                    CatalogStatus {
                        loaded: true,
                        path: "embedded://catalog.json".to_string(),
                        example_count,
                        summary: format!("✅ Loaded {} catalog examples", example_count),
                        error: None,
                    }
                }
                Err(e) => CatalogStatus {
                    loaded: false,
                    path: "embedded://catalog.json".to_string(),
                    example_count: 0,
                    summary: "❌ Catalog file corrupt".to_string(),
                    error: Some(format!("JSON parse error: {}", e)),
                },
            },
            Err(e) => CatalogStatus {
                loaded: false,
                path: "embedded://catalog.json".to_string(),
                example_count: 0,
                summary: "❌ Embedded catalog not available".to_string(),
                error: Some(format!("Embedded catalog error: {}", e)),
            },
        }
    }

    fn get_catalog_status_content(&self) -> Result<String> {
        let status = self.get_catalog_status();

        Ok(format!(
            r#"# 📊 Catalog Status

## Current Status: {}

**File Location**: `{}`  
**Examples Loaded**: {}  
**Status**: {}

## Details

{}

## What This Means

{}

## Troubleshooting

{}

"#,
            if status.loaded {
                "✅ LOADED"
            } else {
                "❌ FAILED"
            },
            status.path,
            status.example_count,
            status.summary,
            if let Some(error) = &status.error {
                format!("**Error Details**: {}", error)
            } else {
                "The catalog loaded successfully and all example resources are available."
                    .to_string()
            },
            if status.loaded {
                format!(
                    "✅ **Good news!** {} catalog examples are available.\n\
                    You can browse them using:\n\
                    - `ast-grep://navigation/language/{{lang}}` for language-specific examples\n\
                    - `ast-grep://navigation/has-fix` for examples with code fixes\n\
                    - `ast-grep://catalog/{{id}}` for specific examples",
                    status.example_count
                )
            } else {
                "❌ **Issue detected.** Catalog examples are not available.\n\
                This means you won't see individual catalog resources, but all static resources still work.".to_string()
            },
            if status.loaded {
                "🎉 **No action needed!** Everything is working correctly."
            } else {
                "🔧 **To fix catalog issues:**\n\
                1. Try rebuilding the project: `cargo build`\n\
                2. Check if the build process can access GitHub (for cloning examples)\n\
                3. Verify disk space and permissions in the build directory\n\
                4. If issues persist, you can still use all static resources"
            }
        ))
    }

    fn get_navigation_error_content(&self) -> Result<String> {
        Ok("⚠️ Navigation resources failed to load. This usually means the catalog is not available. Check ast-grep://catalog-status for details.".to_string())
    }

    fn get_catalog_error_content(&self) -> Result<String> {
        Ok("⚠️ Catalog resources failed to load. This usually means the catalog file is missing or corrupt. Check ast-grep://catalog-status for details.".to_string())
    }

    async fn similarity_search_tool(&self, args: Value) -> Result<String> {
        let pattern = args["pattern"].as_str().ok_or(anyhow!("Missing pattern"))?;
        let limit = args["limit"].as_u64().unwrap_or(10) as usize;
        let offset = args["offset"].as_u64().unwrap_or(0) as usize;

        let results = self.with_search_engine(|engine| engine.similarity_search_paginated(pattern, limit, offset))?;

        if results.results.is_empty() {
            if offset > 0 {
                return Ok(format!(
                    "No more similar patterns found at offset {} for the provided pattern.",
                    offset
                ));
            } else {
                return Ok(
                    "No similar patterns found. Try using simpler or more general terms.".to_string(),
                );
            }
        }

        let mut output = format!(
            "Found {} of {} total similar patterns (page {}-{}):\n\n",
            results.results.len(),
            results.pagination.total_count,
            offset + 1,
            offset + results.results.len()
        );

        for (i, result) in results.results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({})\n",
                offset + i + 1,
                result.title,
                result.language
            ));
            output.push_str(&format!("   Similarity: {:.2}\n", result.score));
            output.push_str(&format!("   Description: {}\n", result.description));
            if !result.yaml_content.is_empty() {
                output.push_str(&format!(
                    "   YAML Rule: {}...\n",
                    result
                        .yaml_content
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(50)
                        .collect::<String>()
                ));
            }
            output.push_str("\n");
        }

        if results.pagination.has_more {
            output.push_str(&format!(
                "📄 More results available. Use offset: {} to get next page ({} more results).\n\n",
                offset + limit,
                results.pagination.total_count - offset - results.results.len()
            ));
        }

        output.push_str(
            "⚠️ Note: Similarity is based on text matching - review each example for relevance.",
        );
        Ok(output)
    }

    async fn suggest_examples(&self, args: Value) -> Result<String> {
        let problem_description = args["description"]
            .as_str()
            .ok_or(anyhow!("Missing description"))?;
        let language = args["language"].as_str().unwrap_or("any");
        let limit = args["limit"].as_u64().unwrap_or(5) as usize;
        let offset = args["offset"].as_u64().unwrap_or(0) as usize;

        // Use similarity search with the problem description
        let results = self.with_search_engine(|engine| {
            let lang_filter = if language == "any" {
                None
            } else {
                Some(language)
            };
            // First try regular search with the description
            let search_results = engine.search_paginated(problem_description, lang_filter, limit, offset);
            if search_results.as_ref().map_or(true, |r| r.results.is_empty()) && offset == 0 {
                // Fall back to similarity search only if this is the first page and no results
                engine.similarity_search_paginated(problem_description, limit, offset)
            } else {
                search_results
            }
        })?;

        if results.results.is_empty() {
            if offset > 0 {
                return Ok(format!(
                    "No more relevant examples found at offset {} for your problem description.",
                    offset
                ));
            } else {
                return Ok("No relevant examples found for your problem description. Try describing your problem with different terms or be more specific about what you're trying to achieve.".to_string());
            }
        }

        let mut output = format!(
            "Based on your problem description, here are {} of {} potentially relevant examples (page {}-{}):\n\n",
            results.results.len(),
            results.pagination.total_count,
            offset + 1,
            offset + results.results.len()
        );

        for (i, result) in results.results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({})\n",
                offset + i + 1,
                result.title,
                result.language
            ));
            output.push_str(&format!("   Relevance: {:.2}\n", result.score));
            output.push_str(&format!("   What it does: {}\n", result.description));
            if result.has_fix {
                output.push_str("   ✅ Includes code transformation\n");
            }
            if !result.playground_link.is_empty() {
                output.push_str(&format!("   Try it: {}\n", result.playground_link));
            }
            output.push_str("\n");
        }

        if results.pagination.has_more {
            output.push_str(&format!(
                "📄 More results available. Use offset: {} to get next page ({} more results).\n\n",
                offset + limit,
                results.pagination.total_count - offset - results.results.len()
            ));
        }

        output.push_str("💡 These examples might help with your problem, but you'll likely need to adapt them to your specific use case. ");
        output.push_str("Don't assume they'll work exactly as-is for your requirements.");

        Ok(output)
    }
}

#[derive(Debug)]
struct CatalogStatus {
    loaded: bool,
    path: String,
    example_count: usize,
    summary: String,
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_tools() -> AstGrepTools {
        let binary_manager =
            Arc::new(BinaryManager::new().expect("Failed to create binary manager"));
        AstGrepTools::new(binary_manager)
    }

    #[test]
    fn test_discovery_resources_included() {
        let tools = create_test_tools();
        let resources = tools.list_resources();

        // Check that discovery resources are included
        let discovery_guide = resources
            .iter()
            .find(|r| r.raw.uri == "ast-grep://discover");
        assert!(
            discovery_guide.is_some(),
            "Discovery guide should be included"
        );

        let languages_list = resources
            .iter()
            .find(|r| r.raw.uri == "ast-grep://languages");
        assert!(
            languages_list.is_some(),
            "Languages list should be included"
        );

        let catalog_status = resources
            .iter()
            .find(|r| r.raw.uri == "ast-grep://catalog-status");
        assert!(
            catalog_status.is_some(),
            "Catalog status should be included"
        );
    }

    #[test]
    fn test_popular_language_examples_included() {
        let tools = create_test_tools();
        let resources = tools.list_resources();

        // Check that popular language examples have direct access
        let js_examples = resources
            .iter()
            .find(|r| r.raw.uri == "ast-grep://examples/javascript");
        assert!(
            js_examples.is_some(),
            "JavaScript examples should be directly accessible"
        );

        let python_examples = resources
            .iter()
            .find(|r| r.raw.uri == "ast-grep://examples/python");
        assert!(
            python_examples.is_some(),
            "Python examples should be directly accessible"
        );
    }

    #[test]
    fn test_discovery_guide_content() {
        let tools = create_test_tools();
        let content = tools
            .get_discovery_guide()
            .expect("Should generate discovery guide");

        assert!(content.contains("🔍 AST-Grep MCP Resource Discovery Guide"));
        assert!(content.contains("Quick Start Resources"));
        assert!(content.contains("ast-grep://languages"));
        assert!(content.contains("Tips for Smaller Models"));
    }

    #[test]
    fn test_languages_content() {
        let tools = create_test_tools();
        let content = tools
            .get_available_languages()
            .expect("Should generate languages list");

        assert!(content.contains("📚 Supported Programming Languages"));
        assert!(content.contains("JavaScript"));
        assert!(content.contains("Python"));
        assert!(content.contains("ast-grep://examples/"));
    }

    #[test]
    fn test_error_handling_improvements() {
        let tools = create_test_tools();
        let resources = tools.list_resources();

        // Should have either catalog resources OR error resources, not empty list
        let has_catalog = resources
            .iter()
            .any(|r| r.raw.uri.starts_with("ast-grep://catalog/"));
        let has_catalog_error = resources
            .iter()
            .any(|r| r.raw.uri == "ast-grep://catalog-error");
        let _has_nav_error = resources
            .iter()
            .any(|r| r.raw.uri == "ast-grep://navigation-error");

        // Should have either working catalog OR visible error information
        assert!(
            has_catalog || has_catalog_error,
            "Should show catalog resources or catalog error"
        );
    }

    #[test]
    fn test_resource_count_increased() {
        let tools = create_test_tools();
        let resources = tools.list_resources();

        // Before improvements, there were ~11 static resources
        // After improvements, we should have significantly more
        assert!(
            resources.len() >= 15,
            "Should have at least 15 resources (was ~11 before improvements)"
        );

        // Check for improved naming with emojis
        let emoji_resources = resources
            .iter()
            .filter(|r| {
                r.raw.name.contains("🔍")
                    || r.raw.name.contains("📚")
                    || r.raw.name.contains("📊")
                    || r.raw.name.contains("🔧")
                    || r.raw.name.contains("📝")
            })
            .count();

        assert!(
            emoji_resources > 0,
            "Should have resources with emoji names for better UX"
        );
    }
}

// Helper trait for string formatting
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
            }
        }
    }
}
