use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub score: f32,
    pub has_fix: bool,
    pub features: Vec<String>,
    pub yaml_content: String,
    pub playground_link: String,
}

#[derive(Debug, Clone)]
pub struct CatalogExample {
    pub id: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub has_fix: bool,
    pub features: Vec<String>,
    pub yaml_content: String,
    pub playground_link: String,
    pub content: String, // Combined searchable text
}

pub struct SimpleSearchEngine {
    examples: Vec<CatalogExample>,
}

impl SimpleSearchEngine {
    pub fn new(catalog_path: &str) -> Result<Self> {
        let catalog_content = std::fs::read_to_string(catalog_path)
            .map_err(|e| anyhow!("Failed to read catalog at {}: {}", catalog_path, e))?;
        
        let catalog: Value = serde_json::from_str(&catalog_content)?;
        let examples_json = catalog["examples"].as_array()
            .ok_or_else(|| anyhow!("No examples found in catalog"))?;

        let mut examples = Vec::new();
        
        for example in examples_json {
            let id = example["id"].as_str().unwrap_or("").to_string();
            let title = example["title"].as_str().unwrap_or("").to_string();
            let description = example["description"].as_str().unwrap_or("").to_string();
            let language = example["language"].as_str().unwrap_or("").to_string();
            let has_fix = example["has_fix"].as_bool().unwrap_or(false);
            let yaml_content = example["yaml_content"].as_str().unwrap_or("").to_string();
            let playground_link = example["playground_link"].as_str().unwrap_or("").to_string();
            
            let features: Vec<String> = example["features"].as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            // Create searchable content combining multiple fields
            let content = format!("{} {} {} {} {}", 
                title, description, yaml_content, features.join(" "), language);

            examples.push(CatalogExample {
                id,
                title,
                description,
                language,
                has_fix,
                features,
                yaml_content,
                playground_link,
                content,
            });
        }

        Ok(SimpleSearchEngine { examples })
    }

    pub fn search(&self, query: &str, language_filter: Option<&str>, limit: usize) -> Result<Vec<SearchResult>> {
        let normalized_query = query.nfc().collect::<String>().to_lowercase();
        let query_terms: Vec<&str> = normalized_query.split_whitespace().collect();
        
        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut scored_results = Vec::new();

        for example in &self.examples {
            // Apply language filter
            if let Some(lang) = language_filter {
                if lang != "any" && !example.language.eq_ignore_ascii_case(lang) {
                    continue;
                }
            }

            let score = self.calculate_score(example, &query_terms);
            if score > 0.0 {
                scored_results.push((example, score));
            }
        }

        // Sort by score (highest first)
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert to SearchResult and limit
        let results: Vec<SearchResult> = scored_results
            .into_iter()
            .take(limit)
            .map(|(example, score)| SearchResult {
                id: example.id.clone(),
                title: example.title.clone(),
                description: example.description.clone(),
                language: example.language.clone(),
                score,
                has_fix: example.has_fix,
                features: example.features.clone(),
                yaml_content: example.yaml_content.clone(),
                playground_link: example.playground_link.clone(),
            })
            .collect();

        Ok(results)
    }

    /// Simple similarity search based on term overlap
    pub fn similarity_search(&self, example_text: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let terms = extract_key_terms(example_text);
        let query = terms.join(" ");
        
        self.search(&query, None, limit)
    }

    fn calculate_score(&self, example: &CatalogExample, query_terms: &[&str]) -> f32 {
        let content_lower = example.content.to_lowercase();
        let title_lower = example.title.to_lowercase();
        let description_lower = example.description.to_lowercase();
        
        let mut score = 0.0;

        // Check for exact phrase matches (highest weight)
        let full_query = query_terms.join(" ");
        if content_lower.contains(&full_query) {
            score += 10.0;
        }

        // Check for individual term matches with different weights
        for term in query_terms {
            if term.len() <= 2 {
                continue; // Skip very short terms
            }

            // Title matches (high weight)
            if title_lower.contains(term) {
                score += 5.0;
            }

            // Description matches (medium weight)  
            if description_lower.contains(term) {
                score += 3.0;
            }

            // YAML content matches (medium weight)
            if example.yaml_content.to_lowercase().contains(term) {
                score += 2.0;
            }

            // Language matches (low weight)
            if example.language.to_lowercase().contains(term) {
                score += 1.0;
            }

            // Features matches (medium weight)
            for feature in &example.features {
                if feature.to_lowercase().contains(term) {
                    score += 2.5;
                }
            }
        }

        // Bonus for examples with fixes
        if example.has_fix && query_terms.iter().any(|&term| 
            term.contains("fix") || term.contains("replace") || term.contains("rewrite")
        ) {
            score += 1.0;
        }

        score
    }
}

/// Extract key terms from text for similarity matching
fn extract_key_terms(text: &str) -> Vec<String> {
    let text = text.to_lowercase();
    let words: Vec<&str> = text
        .split_whitespace()
        .filter(|word| {
            word.len() > 2 && 
            !is_stop_word(word) &&
            word.chars().any(|c| c.is_alphabetic())
        })
        .collect();

    // Get unique terms with basic frequency filtering
    let mut term_counts = HashMap::new();
    for word in words {
        *term_counts.entry(word).or_insert(0) += 1;
    }

    // Return terms that appear at least once, sorted by frequency
    let mut terms: Vec<(String, usize)> = term_counts
        .into_iter()
        .map(|(word, count)| (word.to_string(), count))
        .collect();
    
    terms.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    
    // Take top terms, limit to reasonable number
    terms.into_iter()
        .take(10)
        .map(|(term, _)| term)
        .collect()
}

fn is_stop_word(word: &str) -> bool {
    matches!(word, 
        "the" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for" | 
        "of" | "with" | "by" | "is" | "are" | "was" | "were" | "be" | "been" |
        "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" |
        "should" | "could" | "can" | "may" | "might" | "must" | "shall" |
        "this" | "that" | "these" | "those" | "a" | "an" | "as" | "if" |
        "then" | "else" | "when" | "where" | "why" | "how" | "what" | "which" |
        "who" | "whom" | "whose" | "i" | "you" | "he" | "she" | "it" | "we" | "they"
    )
}