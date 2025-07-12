use anyhow::{Result, anyhow};
use serde_json::Value;
use tokio::process::Command as TokioCommand;
use rmcp::model::*;
use std::collections::HashMap;

pub struct AstGrepTools;

impl AstGrepTools {
    pub fn new() -> Self {
        Self
    }
    
    
    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<String> {
        match tool_name {
            "ast_grep_search" => self.search(arguments).await,
            "ast_grep_replace" => self.replace(arguments).await,
            "ast_grep_scan" => self.scan(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name))
        }
    }
    
    async fn search(&self, args: Value) -> Result<String> {
        let pattern = args["pattern"].as_str().ok_or(anyhow!("Missing pattern"))?;
        let language = args["language"].as_str().ok_or(anyhow!("Missing language"))?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing path"))?;
        
        let output = TokioCommand::new("ast-grep")
            .arg("run")
            .arg("--pattern")
            .arg(pattern)
            .arg("--lang")
            .arg(language)
            .arg(path)
            .arg("--json")
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ast-grep failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
    
    async fn replace(&self, args: Value) -> Result<String> {
        let pattern = args["pattern"].as_str().ok_or(anyhow!("Missing pattern"))?;
        let replacement = args["replacement"].as_str().ok_or(anyhow!("Missing replacement"))?;
        let language = args["language"].as_str().ok_or(anyhow!("Missing language"))?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing path"))?;
        let dry_run = args["dry_run"].as_bool().unwrap_or(true);
        
        let mut cmd = TokioCommand::new("ast-grep");
        cmd.arg("run")
            .arg("--pattern")
            .arg(pattern)
            .arg("--rewrite")
            .arg(replacement)
            .arg("--lang")
            .arg(language)
            .arg(path);
        
        if dry_run {
            cmd.arg("--dry-run");
        }
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ast-grep failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
    
    async fn scan(&self, args: Value) -> Result<String> {
        let rule = args["rule"].as_str().ok_or(anyhow!("Missing rule"))?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing path"))?;
        
        let temp_rule_file = std::env::temp_dir().join("ast_grep_rule.yml");
        tokio::fs::write(&temp_rule_file, rule).await?;
        
        let output = TokioCommand::new("ast-grep")
            .arg("scan")
            .arg("--rule")
            .arg(&temp_rule_file)
            .arg(path)
            .arg("--json")
            .output()
            .await?;
        
        tokio::fs::remove_file(temp_rule_file).await.ok();
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ast-grep failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    pub async fn list_resources(&self) -> Vec<Resource> {
        vec![
            Resource::new(
                "ast-grep://pattern-library",
                "Pattern Library",
                "Comprehensive library of common ast-grep patterns for various programming languages",
                "text/markdown",
            ),
            Resource::new(
                "ast-grep://language-reference",
                "Language Reference",
                "Reference guide for AST-grep supported languages and their syntax",
                "text/markdown",
            ),
            Resource::new(
                "ast-grep://code-context",
                "Code Context Guide",
                "Understanding AST structures and node types for effective pattern construction",
                "text/markdown",
            ),
            Resource::new(
                "ast-grep://examples-database",
                "Examples Database",
                "Rich collection of examples demonstrating pattern construction from simple to complex",
                "text/markdown",
            ),
            Resource::new(
                "ast-grep://quick-reference",
                "Quick Reference",
                "Quick reference card for common patterns and syntax",
                "text/markdown",
            ),
        ]
    }

    pub async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>> {
        match uri {
            "ast-grep://pattern-library" => Ok(vec![ResourceContent::text(self.get_pattern_library().await)]),
            "ast-grep://language-reference" => Ok(vec![ResourceContent::text(self.get_language_reference().await)]),
            "ast-grep://code-context" => Ok(vec![ResourceContent::text(self.get_code_context().await)]),
            "ast-grep://examples-database" => Ok(vec![ResourceContent::text(self.get_examples_database().await)]),
            "ast-grep://quick-reference" => Ok(vec![ResourceContent::text(self.get_quick_reference().await)]),
            _ => Err(anyhow!("Unknown resource: {}", uri)),
        }
    }

    pub async fn list_prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt::new(
                "construct-pattern",
                "Pattern Construction Assistant",
                "Guided workflow to construct ast-grep patterns from natural language descriptions",
                vec![
                    PromptArgument::new("intent", "string", "Natural language description of what you want to find or change"),
                    PromptArgument::new("language", "string", "Programming language (e.g., 'javascript', 'python', 'rust')"),
                    PromptArgument::new("code_example", "string", "Optional: Example code snippet to work with"),
                ],
            ),
            Prompt::new(
                "refactor-code",
                "Code Refactoring Assistant",
                "Step-by-step guidance for refactoring code patterns",
                vec![
                    PromptArgument::new("refactor_type", "string", "Type of refactoring (e.g., 'modernize', 'cleanup', 'optimize')"),
                    PromptArgument::new("language", "string", "Programming language"),
                    PromptArgument::new("code_path", "string", "Path to code to refactor"),
                ],
            ),
            Prompt::new(
                "security-scan",
                "Security Scanning Assistant",
                "Guided security vulnerability scanning using ast-grep patterns",
                vec![
                    PromptArgument::new("security_concern", "string", "Type of security issue to scan for"),
                    PromptArgument::new("language", "string", "Programming language"),
                    PromptArgument::new("scan_path", "string", "Path to scan for security issues"),
                ],
            ),
            Prompt::new(
                "analyze-code",
                "Code Analysis Assistant",
                "Comprehensive code analysis and pattern documentation",
                vec![
                    PromptArgument::new("analysis_type", "string", "Type of analysis (e.g., 'structure', 'complexity', 'patterns')"),
                    PromptArgument::new("language", "string", "Programming language"),
                    PromptArgument::new("code_path", "string", "Path to code to analyze"),
                ],
            ),
        ]
    }

    pub async fn get_prompt(&self, name: &str, arguments: Option<HashMap<String, Value>>) -> Result<Vec<PromptMessage>> {
        let args = arguments.unwrap_or_default();
        
        match name {
            "construct-pattern" => self.construct_pattern_prompt(args).await,
            "refactor-code" => self.refactor_code_prompt(args).await,
            "security-scan" => self.security_scan_prompt(args).await,
            "analyze-code" => self.analyze_code_prompt(args).await,
            _ => Err(anyhow!("Unknown prompt: {}", name)),
        }
    }

    // Resource Content Methods
    async fn get_pattern_library(&self) -> String {
        include_str!("../resources/pattern-library.md").to_string()
    }

    async fn get_language_reference(&self) -> String {
        include_str!("../resources/language-reference.md").to_string()
    }

    async fn get_code_context(&self) -> String {
        include_str!("../resources/code-context.md").to_string()
    }

    async fn get_examples_database(&self) -> String {
        include_str!("../resources/examples-database.md").to_string()
    }

    async fn get_quick_reference(&self) -> String {
        include_str!("../resources/quick-reference.md").to_string()
    }

    // Prompt Methods
    async fn construct_pattern_prompt(&self, args: HashMap<String, Value>) -> Result<Vec<PromptMessage>> {
        let intent = args.get("intent").and_then(|v| v.as_str()).unwrap_or("unknown");
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("javascript");
        let code_example = args.get("code_example").and_then(|v| v.as_str()).unwrap_or("");

        let system_prompt = format!(
            include_str!("../prompts/construct-pattern-system.md"),
            language = language
        );

        let user_prompt = format!(
            include_str!("../prompts/construct-pattern-user.md"),
            intent = intent,
            language = language,
            code_example = if code_example.is_empty() { "No code example provided" } else { code_example }
        );

        Ok(vec![
            PromptMessage::new(Role::System, Content::text(system_prompt)),
            PromptMessage::new(Role::User, Content::text(user_prompt)),
        ])
    }

    async fn refactor_code_prompt(&self, args: HashMap<String, Value>) -> Result<Vec<PromptMessage>> {
        let refactor_type = args.get("refactor_type").and_then(|v| v.as_str()).unwrap_or("modernize");
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("javascript");
        let code_path = args.get("code_path").and_then(|v| v.as_str()).unwrap_or("");

        let system_prompt = format!(
            include_str!("../prompts/refactor-code-system.md"),
            refactor_type = refactor_type,
            language = language
        );

        let user_prompt = format!(
            include_str!("../prompts/refactor-code-user.md"),
            refactor_type = refactor_type,
            language = language,
            code_path = code_path
        );

        Ok(vec![
            PromptMessage::new(Role::System, Content::text(system_prompt)),
            PromptMessage::new(Role::User, Content::text(user_prompt)),
        ])
    }

    async fn security_scan_prompt(&self, args: HashMap<String, Value>) -> Result<Vec<PromptMessage>> {
        let security_concern = args.get("security_concern").and_then(|v| v.as_str()).unwrap_or("general");
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("javascript");
        let scan_path = args.get("scan_path").and_then(|v| v.as_str()).unwrap_or("");

        let system_prompt = format!(
            include_str!("../prompts/security-scan-system.md"),
            security_concern = security_concern,
            language = language
        );

        let user_prompt = format!(
            include_str!("../prompts/security-scan-user.md"),
            security_concern = security_concern,
            language = language,
            scan_path = scan_path
        );

        Ok(vec![
            PromptMessage::new(Role::System, Content::text(system_prompt)),
            PromptMessage::new(Role::User, Content::text(user_prompt)),
        ])
    }

    async fn analyze_code_prompt(&self, args: HashMap<String, Value>) -> Result<Vec<PromptMessage>> {
        let analysis_type = args.get("analysis_type").and_then(|v| v.as_str()).unwrap_or("structure");
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("javascript");
        let code_path = args.get("code_path").and_then(|v| v.as_str()).unwrap_or("");

        let system_prompt = format!(
            include_str!("../prompts/analyze-code-system.md"),
            analysis_type = analysis_type,
            language = language
        );

        let user_prompt = format!(
            include_str!("../prompts/analyze-code-user.md"),
            analysis_type = analysis_type,
            language = language,
            code_path = code_path
        );

        Ok(vec![
            PromptMessage::new(Role::System, Content::text(system_prompt)),
            PromptMessage::new(Role::User, Content::text(user_prompt)),
        ])
    }
}