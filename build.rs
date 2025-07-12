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
    
    // Focus specifically on catalog directory
    let catalog_path = website_path.join("website/catalog");
    if catalog_path.exists() {
        for entry in WalkDir::new(catalog_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // Only process markdown files that are actual examples (not index.md or rule-template.md)
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if filename != "index.md" && filename != "rule-template.md" {
                        if let Ok(content) = fs::read_to_string(path) {
                            if let Ok(extracted) = extract_catalog_example(&content, path) {
                                examples.push(extracted);
                            }
                        }
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

fn extract_catalog_example(content: &str, file_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    // Extract ID from filename
    let id = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Extract language from path (catalog/language/example.md)
    let language = file_path.parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Extract title from first heading
    let title_re = Regex::new(r"##\s*([^<\n]+)")?;
    let title = title_re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim())
        .unwrap_or(&id)
        .to_string();
    
    // Check if it has a fix
    let has_fix = content.contains("<Badge") && content.contains("Has Fix");
    
    // Extract playground link
    let playground_re = Regex::new(r"\[Playground Link\]\((.+?)\)")?;
    let playground_link = playground_re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    
    // Extract description (look for ### Description section)
    let desc_re = Regex::new(r"(?s)### Description\s*\n\n(.*?)(?:\n###|\n```|$)")?;
    let description = desc_re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "No description available".to_string());
    
    // Extract YAML rule content
    let yaml_re = Regex::new(r"(?s)```ya?ml\n(.*?)\n```")?;
    let yaml_content = yaml_re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    
    // Determine type
    let rule_type = if yaml_content.is_empty() { "Pattern" } else { "YAML" };
    
    // Extract features and rules from YAML if available
    let mut features = Vec::new();
    let mut rules = Vec::new();
    
    if !yaml_content.is_empty() {
        if let Ok(docs) = YamlLoader::load_from_str(&yaml_content) {
            if let Some(doc) = docs.first() {
                if let Some(yaml_obj) = doc.as_hash() {
                    // Check for advanced features
                    if yaml_obj.contains_key(&Yaml::String("utils".to_string())) {
                        features.push("utils".to_string());
                    }
                    if yaml_obj.contains_key(&Yaml::String("constraints".to_string())) {
                        features.push("constraints".to_string());
                    }
                    if yaml_obj.contains_key(&Yaml::String("transform".to_string())) {
                        features.push("transform".to_string());
                    }
                    if yaml_obj.contains_key(&Yaml::String("rewriters".to_string())) {
                        features.push("rewriters".to_string());
                    }
                    
                    // Extract rule types used
                    if let Some(rule_obj) = yaml_obj.get(&Yaml::String("rule".to_string())) {
                        extract_rule_types(rule_obj, &mut rules);
                    }
                }
            }
        }
    }
    
    Ok(json!({
        "id": id,
        "title": title,
        "description": description,
        "language": language,
        "type": rule_type,
        "has_fix": has_fix,
        "playground_link": playground_link,
        "features": features,
        "rules": rules,
        "yaml_content": yaml_content,
        "source_file": file_path.to_string_lossy(),
        "content": content
    }))
}

fn extract_rule_types(yaml_value: &Yaml, rules: &mut Vec<String>) {
    if let Some(hash) = yaml_value.as_hash() {
        for key in hash.keys() {
            if let Some(key_str) = key.as_str() {
                rules.push(key_str.to_string());
                
                // Recursively check nested rules
                if key_str == "any" || key_str == "all" {
                    if let Some(arr) = hash.get(key).and_then(|v| v.as_vec()) {
                        for item in arr {
                            extract_rule_types(item, rules);
                        }
                    }
                }
            }
        }
    }
}