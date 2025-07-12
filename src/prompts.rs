use anyhow::Result;
use rmcp::model::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct PromptManager {
    refactoring_prompts: RefactoringPrompts,
    analysis_prompts: AnalysisPrompts,
    educational_prompts: EducationalPrompts,
}

impl PromptManager {
    pub fn new() -> Self {
        Self {
            refactoring_prompts: RefactoringPrompts::new(),
            analysis_prompts: AnalysisPrompts::new(),
            educational_prompts: EducationalPrompts::new(),
        }
    }

    pub fn list_prompts(&self) -> Vec<Prompt> {
        let mut prompts = Vec::new();
        
        prompts.extend(self.refactoring_prompts.get_prompts());
        prompts.extend(self.analysis_prompts.get_prompts());
        prompts.extend(self.educational_prompts.get_prompts());
        
        prompts
    }

    pub fn get_prompt(&self, name: &str, arguments: HashMap<String, Value>) -> Result<String> {
        if name.starts_with("refactor_") {
            self.refactoring_prompts.get_prompt(name, arguments)
        } else if name.starts_with("security_") || name.starts_with("performance_") || name.starts_with("quality_") {
            self.analysis_prompts.get_prompt(name, arguments)
        } else if name.starts_with("explain_") || name.starts_with("debug_") {
            self.educational_prompts.get_prompt(name, arguments)
        } else {
            Err(anyhow::anyhow!("Unknown prompt: {}", name))
        }
    }
}

struct RefactoringPrompts {
    prompts: HashMap<String, PromptTemplate>,
}

impl RefactoringPrompts {
    fn new() -> Self {
        let mut prompts = HashMap::new();
        
        prompts.insert("refactor_function".to_string(), PromptTemplate {
            name: "refactor_function".to_string(),
            description: "Extract or refactor functions with guided workflow".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "function_name".to_string(),
                    description: Some("Name of the function to refactor".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "operation".to_string(),
                    description: Some("Type of refactoring: extract, inline, rename".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "file_path".to_string(),
                    description: Some("Path to the file containing the function".to_string()),
                    required: Some(true),
                },
            ],
            template: r#"
# Function Refactoring Workflow

You are helping refactor a function in a {language} codebase.

## Task Details
- Function: {function_name}
- Operation: {operation}
- File: {file_path}

## Step-by-Step Process

1. **Analyze Current Function**
   - Use ast_grep_search to find the function definition
   - Pattern: Based on language-specific patterns from resources

2. **Validate Refactoring**
   - Check for dependencies and side effects
   - Verify the refactoring is safe

3. **Apply Refactoring**
   - Use ast_grep_replace with appropriate patterns
   - Apply changes incrementally

4. **Verify Results**
   - Check syntax and functionality
   - Ensure no breaking changes

## Available Resources
- patterns://{language}/refactoring/extract-function
- lang://{language}/ast-nodes

Let's start by finding the function definition.
"#.to_string(),
        });

        prompts.insert("refactor_variable".to_string(), PromptTemplate {
            name: "refactor_variable".to_string(),
            description: "Rename or extract variables with guided workflow".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "variable_name".to_string(),
                    description: Some("Name of the variable to refactor".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "new_name".to_string(),
                    description: Some("New name for the variable (for rename operation)".to_string()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "scope".to_string(),
                    description: Some("Scope to refactor (file path or directory)".to_string()),
                    required: Some(true),
                },
            ],
            template: r#"
# Variable Refactoring Workflow

You are helping refactor a variable in a {language} codebase.

## Task Details
- Variable: {variable_name}
- New Name: {new_name}
- Language: {language}
- Scope: {scope}

## Step-by-Step Process

1. **Find All Occurrences**
   - Use ast_grep_search to find all usages of the variable
   - Pattern: Language-specific variable patterns

2. **Analyze Context**
   - Check scope and dependencies
   - Verify rename safety

3. **Apply Changes**
   - Use ast_grep_replace to rename all occurrences
   - Process files systematically

4. **Validate Results**
   - Check for syntax errors
   - Ensure all references are updated

## Available Resources
- lang://{language}/ast-nodes

Let's start by finding all occurrences of the variable.
"#.to_string(),
        });

        Self { prompts }
    }

    fn get_prompts(&self) -> Vec<Prompt> {
        self.prompts.values().map(|template| {
            Prompt {
                name: template.name.clone(),
                description: Some(template.description.clone()),
                arguments: Some(template.arguments.clone()),
            }
        }).collect()
    }

    fn get_prompt(&self, name: &str, arguments: HashMap<String, Value>) -> Result<String> {
        let template = self.prompts.get(name)
            .ok_or_else(|| anyhow::anyhow!("Refactoring prompt not found: {}", name))?;
        
        let mut result = template.template.clone();
        for (key, value) in arguments {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s,
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        Ok(result)
    }
}

struct AnalysisPrompts {
    prompts: HashMap<String, PromptTemplate>,
}

impl AnalysisPrompts {
    fn new() -> Self {
        let mut prompts = HashMap::new();
        
        prompts.insert("security_scan".to_string(), PromptTemplate {
            name: "security_scan".to_string(),
            description: "Comprehensive security analysis workflow".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language to analyze".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "scope".to_string(),
                    description: Some("Path to analyze (file or directory)".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "severity".to_string(),
                    description: Some("Minimum severity level: low, medium, high".to_string()),
                    required: Some(false),
                },
            ],
            template: r#"
# Security Analysis Workflow

You are performing a comprehensive security analysis on a {language} codebase.

## Analysis Scope
- Language: {language}
- Target: {scope}
- Severity: {severity}

## Step-by-Step Security Analysis

1. **Hardcoded Secrets Detection**
   - Use rule: rules://security/detect-hardcoded-secrets
   - Check for API keys, tokens, passwords

2. **Injection Vulnerabilities**
   - Scan for SQL injection patterns
   - Check for XSS vulnerabilities
   - Validate input sanitization

3. **Authentication & Authorization**
   - Review access control patterns
   - Check for privilege escalation risks

4. **Data Protection**
   - Verify encryption usage
   - Check for data leakage patterns

## Available Resources
- rules://security/detect-hardcoded-secrets
- patterns://{language}/security/common-vulnerabilities

Let's start with detecting hardcoded secrets.
"#.to_string(),
        });

        prompts.insert("performance_audit".to_string(), PromptTemplate {
            name: "performance_audit".to_string(),
            description: "Performance analysis and optimization workflow".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language to analyze".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "scope".to_string(),
                    description: Some("Path to analyze (file or directory)".to_string()),
                    required: Some(true),
                },
            ],
            template: r#"
# Performance Audit Workflow

You are performing a performance analysis on a {language} codebase.

## Analysis Scope
- Language: {language}
- Target: {scope}

## Step-by-Step Performance Analysis

1. **Unused Code Detection**
   - Use rule: rules://performance/unused-variables
   - Find unused functions and variables

2. **Algorithm Complexity**
   - Identify nested loops and recursive patterns
   - Check for inefficient data structures

3. **Resource Usage**
   - Memory allocation patterns
   - I/O operation optimization

4. **Language-Specific Optimizations**
   - Check for language-specific anti-patterns
   - Identify optimization opportunities

## Available Resources
- rules://performance/unused-variables
- patterns://{language}/performance/optimizations

Let's start by detecting unused code.
"#.to_string(),
        });

        Self { prompts }
    }

    fn get_prompts(&self) -> Vec<Prompt> {
        self.prompts.values().map(|template| {
            Prompt {
                name: template.name.clone(),
                description: Some(template.description.clone()),
                arguments: Some(template.arguments.clone()),
            }
        }).collect()
    }

    fn get_prompt(&self, name: &str, arguments: HashMap<String, Value>) -> Result<String> {
        let template = self.prompts.get(name)
            .ok_or_else(|| anyhow::anyhow!("Analysis prompt not found: {}", name))?;
        
        let mut result = template.template.clone();
        for (key, value) in arguments {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s,
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        Ok(result)
    }
}

struct EducationalPrompts {
    prompts: HashMap<String, PromptTemplate>,
}

impl EducationalPrompts {
    fn new() -> Self {
        let mut prompts = HashMap::new();
        
        prompts.insert("explain_pattern".to_string(), PromptTemplate {
            name: "explain_pattern".to_string(),
            description: "Explain ast-grep patterns with examples".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "pattern".to_string(),
                    description: Some("The ast-grep pattern to explain".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language context".to_string()),
                    required: Some(true),
                },
            ],
            template: r#"
# Pattern Explanation

You are explaining an ast-grep pattern to help someone understand how it works.

## Pattern to Explain
```
{pattern}
```

## Language Context
- Language: {language}

## Explanation Structure

1. **Pattern Breakdown**
   - Break down each part of the pattern
   - Explain special symbols and syntax

2. **What It Matches**
   - Show examples of code that matches
   - Explain why it matches

3. **What It Doesn't Match**
   - Show examples of code that doesn't match
   - Explain the differences

4. **Common Use Cases**
   - When you would use this pattern
   - Similar patterns for related tasks

## Available Resources
- patterns://{language}/refactoring/extract-function
- lang://{language}/ast-nodes

Let's break down this pattern step by step.
"#.to_string(),
        });

        prompts.insert("debug_pattern".to_string(), PromptTemplate {
            name: "debug_pattern".to_string(),
            description: "Debug ast-grep patterns that aren't working".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "pattern".to_string(),
                    description: Some("The ast-grep pattern that isn't working".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "expected_match".to_string(),
                    description: Some("Code that should match but doesn't".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language context".to_string()),
                    required: Some(true),
                },
            ],
            template: r#"
# Pattern Debugging

You are helping debug an ast-grep pattern that isn't working as expected.

## Problem Details
- Pattern: `{pattern}`
- Expected Match: `{expected_match}`
- Language: {language}

## Debugging Process

1. **Analyze the Pattern**
   - Check pattern syntax
   - Verify language compatibility

2. **Test with Simple Cases**
   - Start with minimal examples
   - Gradually increase complexity

3. **Check AST Structure**
   - Examine the AST for the expected match
   - Compare with pattern expectations

4. **Suggest Corrections**
   - Provide corrected pattern
   - Explain the changes

## Available Resources
- lang://{language}/ast-nodes
- patterns://{language}/refactoring/extract-function

Let's start by analyzing the pattern and the expected match.
"#.to_string(),
        });

        Self { prompts }
    }

    fn get_prompts(&self) -> Vec<Prompt> {
        self.prompts.values().map(|template| {
            Prompt {
                name: template.name.clone(),
                description: Some(template.description.clone()),
                arguments: Some(template.arguments.clone()),
            }
        }).collect()
    }

    fn get_prompt(&self, name: &str, arguments: HashMap<String, Value>) -> Result<String> {
        let template = self.prompts.get(name)
            .ok_or_else(|| anyhow::anyhow!("Educational prompt not found: {}", name))?;
        
        let mut result = template.template.clone();
        for (key, value) in arguments {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s,
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        Ok(result)
    }
}

struct PromptTemplate {
    name: String,
    description: String,
    arguments: Vec<PromptArgument>,
    template: String,
}