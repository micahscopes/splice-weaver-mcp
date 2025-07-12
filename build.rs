use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use yaml_rust::{YamlLoader, Yaml};
use serde_json::{json, Value};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let catalog_path = Path::new(&out_dir).join("catalog.json");
    
    // Extract catalog examples and write to JSON
    match extract_catalog_examples() {
        Ok(catalog) => {
            fs::write(&catalog_path, serde_json::to_string_pretty(&catalog).unwrap())
                .expect("Failed to write catalog.json");
            println!("cargo:warning=Generated catalog.json with {} examples", 
                     catalog["examples"].as_array().map_or(0, |arr| arr.len()));
        }
        Err(e) => {
            println!("cargo:warning=Failed to extract catalog examples: {}", e);
            // Create empty catalog on failure
            let empty_catalog = json!({
                "version": "1.0",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "examples": []
            });
            fs::write(&catalog_path, serde_json::to_string_pretty(&empty_catalog).unwrap())
                .expect("Failed to write empty catalog.json");
        }
    }
}

fn extract_catalog_examples() -> Result<Value, Box<dyn std::error::Error>> {
    let mut examples = Vec::new();
    
    let out_dir = env::var("OUT_DIR")?;
    let website_path = Path::new(&out_dir).join("ast-grep.github.io");
    
    // Clone the repository if it doesn't exist
    if !website_path.exists() {
        println!("cargo:warning=Cloning ast-grep.github.io repository...");
        let output = std::process::Command::new("git")
            .args(&["clone", "https://github.com/ast-grep/ast-grep.github.io.git"])
            .arg(&website_path)
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to clone repository: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
    }
    
    println!("cargo:warning=Found ast-grep website at: {}", website_path.display());
    
    // Walk through the website directory looking for catalog/rule examples
    for entry in WalkDir::new(website_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Look for YAML/YML files that might contain rules
        if let Some(ext) = path.extension() {
            if ext == "yml" || ext == "yaml" {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(extracted) = extract_rule_from_content(&content, path) {
                        examples.push(extracted);
                    }
                }
            }
        }
        
        // Also look for markdown files with embedded YAML
        if let Some(ext) = path.extension() {
            if ext == "md" {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(mut extracted) = extract_rules_from_markdown(&content, path) {
                        examples.append(&mut extracted);
                    }
                }
            }
        }
    }
    
    Ok(json!({
        "version": "1.0",
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "source": "ast-grep.github.io",
        "examples": examples
    }))
}

fn extract_rule_from_content(content: &str, file_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let docs = YamlLoader::load_from_str(content)?;
    
    if let Some(doc) = docs.first() {
        if let Some(rule) = doc.as_hash() {
            let mut example = json!({
                "source_file": file_path.to_string_lossy(),
                "type": "rule",
                "content": content.trim()
            });
            
            // Extract metadata if available
            if let Some(id) = rule.get(&Yaml::String("id".to_string())) {
                example["id"] = json!(id.as_str().unwrap_or("unknown"));
            }
            
            if let Some(message) = rule.get(&Yaml::String("message".to_string())) {
                example["message"] = json!(message.as_str().unwrap_or(""));
            }
            
            if let Some(language) = rule.get(&Yaml::String("language".to_string())) {
                example["language"] = json!(language.as_str().unwrap_or(""));
            }
            
            return Ok(example);
        }
    }
    
    Err("No valid rule found in content".into())
}

fn extract_rules_from_markdown(content: &str, file_path: &Path) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut examples = Vec::new();
    
    // Regex to find YAML code blocks
    let yaml_block_re = Regex::new(r"```(?:yaml|yml)\n(.*?)\n```")?;
    
    for cap in yaml_block_re.captures_iter(content) {
        if let Some(yaml_content) = cap.get(1) {
            if let Ok(example) = extract_rule_from_content(yaml_content.as_str(), file_path) {
                examples.push(example);
            }
        }
    }
    
    Ok(examples)
}