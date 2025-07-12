use anyhow::{Result, anyhow};
use serde_json::Value;
use tokio::process::Command as TokioCommand;

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
}