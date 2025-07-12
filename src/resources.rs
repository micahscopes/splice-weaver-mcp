use anyhow::Result;
use rmcp::model::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct ResourceProvider {
    pattern_library: PatternLibrary,
    language_references: LanguageReferences,
    rule_templates: RuleTemplates,
}

impl ResourceProvider {
    pub fn new() -> Self {
        Self {
            pattern_library: PatternLibrary::new(),
            language_references: LanguageReferences::new(),
            rule_templates: RuleTemplates::new(),
        }
    }

    pub fn list_resources(&self) -> Vec<Resource> {
        let mut resources = Vec::new();
        
        // Pattern Library Resources
        resources.extend(self.pattern_library.get_resources());
        
        // Language Reference Resources
        resources.extend(self.language_references.get_resources());
        
        // Rule Template Resources
        resources.extend(self.rule_templates.get_resources());
        
        resources
    }

    pub fn get_resource(&self, uri: &str) -> Result<Value> {
        if uri.starts_with("patterns://") {
            self.pattern_library.get_resource(uri)
        } else if uri.starts_with("lang://") {
            self.language_references.get_resource(uri)
        } else if uri.starts_with("rules://") {
            self.rule_templates.get_resource(uri)
        } else {
            Err(anyhow::anyhow!("Unknown resource URI: {}", uri))
        }
    }
}

struct PatternLibrary {
    patterns: HashMap<String, Value>,
}

impl PatternLibrary {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // JavaScript patterns
        patterns.insert("patterns://javascript/refactoring/extract-function".to_string(), serde_json::json!({
            "title": "Extract Function Patterns",
            "description": "Common patterns for extracting functions in JavaScript",
            "patterns": [
                {
                    "name": "Extract arrow function",
                    "pattern": "($A) => { $$$ }",
                    "description": "Extract arrow function expression",
                    "example": "const handler = (event) => { console.log(event); };"
                },
                {
                    "name": "Extract function declaration",
                    "pattern": "function $NAME($PARAMS) { $$$ }",
                    "description": "Extract function declaration",
                    "example": "function processData(data) { return data.map(item => item.value); }"
                }
            ]
        }));

        patterns.insert("patterns://javascript/security/common-vulnerabilities".to_string(), serde_json::json!({
            "title": "JavaScript Security Patterns",
            "description": "Patterns for detecting common security vulnerabilities",
            "patterns": [
                {
                    "name": "Hardcoded secrets",
                    "pattern": "const $VAR = \"$_\"",
                    "description": "Detect potential hardcoded secrets",
                    "example": "const API_KEY = \"sk-1234567890abcdef\";"
                },
                {
                    "name": "Eval usage",
                    "pattern": "eval($ARG)",
                    "description": "Detect dangerous eval() usage",
                    "example": "eval(userInput);"
                }
            ]
        }));

        // Python patterns
        patterns.insert("patterns://python/refactoring/extract-function".to_string(), serde_json::json!({
            "title": "Python Function Extraction",
            "description": "Patterns for extracting functions in Python",
            "patterns": [
                {
                    "name": "Extract function",
                    "pattern": "def $NAME($PARAMS):\n    $$$",
                    "description": "Extract function definition",
                    "example": "def process_data(data):\n    return [item.value for item in data]"
                }
            ]
        }));

        // Rust patterns
        patterns.insert("patterns://rust/refactoring/extract-function".to_string(), serde_json::json!({
            "title": "Rust Function Extraction",
            "description": "Patterns for extracting functions in Rust",
            "patterns": [
                {
                    "name": "Extract function",
                    "pattern": "fn $NAME($PARAMS) -> $RET { $$$ }",
                    "description": "Extract function definition",
                    "example": "fn process_data(data: Vec<Item>) -> Vec<Value> { data.into_iter().map(|item| item.value).collect() }"
                }
            ]
        }));

        Self { patterns }
    }

    fn get_resources(&self) -> Vec<Resource> {
        self.patterns.keys().map(|uri| {
            Resource::new(RawResource::new(uri.clone(), "Pattern Library".to_string()), None)
        }).collect()
    }

    fn get_resource(&self, uri: &str) -> Result<Value> {
        self.patterns.get(uri)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Pattern resource not found: {}", uri))
    }
}

struct LanguageReferences {
    references: HashMap<String, Value>,
}

impl LanguageReferences {
    fn new() -> Self {
        let mut references = HashMap::new();
        
        references.insert("lang://javascript/ast-nodes".to_string(), serde_json::json!({
            "title": "JavaScript AST Node Types",
            "description": "Common AST node types in JavaScript for ast-grep patterns",
            "nodes": [
                {
                    "name": "function_declaration",
                    "description": "Function declaration node",
                    "example": "function name() {}"
                },
                {
                    "name": "arrow_function",
                    "description": "Arrow function expression",
                    "example": "() => {}"
                },
                {
                    "name": "call_expression",
                    "description": "Function call expression",
                    "example": "func(args)"
                },
                {
                    "name": "identifier",
                    "description": "Variable or function name",
                    "example": "variableName"
                }
            ]
        }));

        references.insert("lang://python/ast-nodes".to_string(), serde_json::json!({
            "title": "Python AST Node Types",
            "description": "Common AST node types in Python for ast-grep patterns",
            "nodes": [
                {
                    "name": "function_definition",
                    "description": "Function definition node",
                    "example": "def name(): pass"
                },
                {
                    "name": "call",
                    "description": "Function call expression",
                    "example": "func(args)"
                },
                {
                    "name": "name",
                    "description": "Variable or function name",
                    "example": "variable_name"
                }
            ]
        }));

        references.insert("lang://rust/ast-nodes".to_string(), serde_json::json!({
            "title": "Rust AST Node Types",
            "description": "Common AST node types in Rust for ast-grep patterns",
            "nodes": [
                {
                    "name": "function_item",
                    "description": "Function definition node",
                    "example": "fn name() {}"
                },
                {
                    "name": "call_expression",
                    "description": "Function call expression",
                    "example": "func(args)"
                },
                {
                    "name": "identifier",
                    "description": "Variable or function name",
                    "example": "variable_name"
                }
            ]
        }));

        Self { references }
    }

    fn get_resources(&self) -> Vec<Resource> {
        self.references.keys().map(|uri| {
            Resource::new(RawResource::new(uri.clone(), "Language Reference".to_string()), None)
        }).collect()
    }

    fn get_resource(&self, uri: &str) -> Result<Value> {
        self.references.get(uri)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Language reference not found: {}", uri))
    }
}

struct RuleTemplates {
    templates: HashMap<String, Value>,
}

impl RuleTemplates {
    fn new() -> Self {
        let mut templates = HashMap::new();
        
        templates.insert("rules://security/detect-hardcoded-secrets".to_string(), serde_json::json!({
            "title": "Detect Hardcoded Secrets",
            "description": "Rule template for detecting hardcoded secrets",
            "rule": {
                "id": "hardcoded-secrets",
                "message": "Potential hardcoded secret detected",
                "severity": "error",
                "language": "javascript",
                "rule": {
                    "pattern": "const $VAR = \"$SECRET\"",
                    "constraints": {
                        "SECRET": {
                            "regex": "^(sk-|key-|token-|secret-|api-)"
                        }
                    }
                }
            }
        }));

        templates.insert("rules://performance/unused-variables".to_string(), serde_json::json!({
            "title": "Unused Variables",
            "description": "Rule template for detecting unused variables",
            "rule": {
                "id": "unused-variables",
                "message": "Unused variable detected",
                "severity": "warning",
                "language": "javascript",
                "rule": {
                    "pattern": "let $VAR = $VALUE",
                    "not": {
                        "pattern": "$VAR"
                    }
                }
            }
        }));

        Self { templates }
    }

    fn get_resources(&self) -> Vec<Resource> {
        self.templates.keys().map(|uri| {
            Resource::new(RawResource::new(uri.clone(), "Rule Template".to_string()), None)
        }).collect()
    }

    fn get_resource(&self, uri: &str) -> Result<Value> {
        self.templates.get(uri)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Rule template not found: {}", uri))
    }
}