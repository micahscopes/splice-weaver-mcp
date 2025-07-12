use crate::evaluation_client::ResponseSnapshot;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotComparison {
    pub previous: ResponseSnapshot,
    pub current: ResponseSnapshot,
    pub differences: Vec<SnapshotDifference>,
    pub similarity_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotDifference {
    pub field: String,
    pub previous_value: String,
    pub current_value: String,
    pub diff_type: DifferenceType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DifferenceType {
    ResponseContent,
    ToolCalls,
    Duration,
    Analysis,
    Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotSummary {
    pub total_snapshots: usize,
    pub test_coverage: HashMap<String, usize>,
    pub model_coverage: HashMap<String, usize>,
    pub avg_response_time: f64,
    pub success_rate: f64,
    pub most_common_tools: Vec<(String, usize)>,
}

pub struct SnapshotManager {
    snapshots_dir: String,
}

impl SnapshotManager {
    pub fn new(snapshots_dir: &str) -> Self {
        Self {
            snapshots_dir: snapshots_dir.to_string(),
        }
    }

    /// Load all snapshots from the snapshots directory
    pub fn load_all_snapshots(&self) -> Result<Vec<ResponseSnapshot>> {
        let mut snapshots = Vec::new();
        let snapshots_path = Path::new(&self.snapshots_dir);

        if !snapshots_path.exists() {
            return Ok(snapshots);
        }

        for entry in fs::read_dir(snapshots_path)? {
            let entry = entry?;
            let path = entry.path();

            if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "yml")
            {
                match self.load_snapshot(&path) {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(e) => eprintln!("Failed to load snapshot {:?}: {}", path, e),
                }
            }
        }

        Ok(snapshots)
    }

    /// Load a specific snapshot file
    pub fn load_snapshot(&self, path: &Path) -> Result<ResponseSnapshot> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read snapshot file: {:?}", path))?;

        serde_yaml::from_str(&content).context(format!("Failed to parse snapshot YAML: {:?}", path))
    }

    /// Compare two snapshots and identify differences
    pub fn compare_snapshots(
        &self,
        previous: &ResponseSnapshot,
        current: &ResponseSnapshot,
    ) -> SnapshotComparison {
        let mut differences = Vec::new();

        // Compare response content
        if previous.evaluation_result.response != current.evaluation_result.response {
            differences.push(SnapshotDifference {
                field: "response_content".to_string(),
                previous_value: previous.evaluation_result.response.clone(),
                current_value: current.evaluation_result.response.clone(),
                diff_type: DifferenceType::ResponseContent,
            });
        }

        // Compare tool calls
        if previous.evaluation_result.tool_calls_made != current.evaluation_result.tool_calls_made {
            differences.push(SnapshotDifference {
                field: "tool_calls_count".to_string(),
                previous_value: previous.evaluation_result.tool_calls_made.to_string(),
                current_value: current.evaluation_result.tool_calls_made.to_string(),
                diff_type: DifferenceType::ToolCalls,
            });
        }

        // Compare duration (with tolerance)
        let duration_diff = (previous.evaluation_result.duration_ms as i64
            - current.evaluation_result.duration_ms as i64)
            .abs();
        if duration_diff > 1000 {
            // More than 1 second difference
            differences.push(SnapshotDifference {
                field: "duration_ms".to_string(),
                previous_value: previous.evaluation_result.duration_ms.to_string(),
                current_value: current.evaluation_result.duration_ms.to_string(),
                diff_type: DifferenceType::Duration,
            });
        }

        // Compare analysis results
        if previous.response_analysis.contains_error != current.response_analysis.contains_error {
            differences.push(SnapshotDifference {
                field: "contains_error".to_string(),
                previous_value: previous.response_analysis.contains_error.to_string(),
                current_value: current.response_analysis.contains_error.to_string(),
                diff_type: DifferenceType::Analysis,
            });
        }

        if previous.response_analysis.word_count != current.response_analysis.word_count {
            differences.push(SnapshotDifference {
                field: "word_count".to_string(),
                previous_value: previous.response_analysis.word_count.to_string(),
                current_value: current.response_analysis.word_count.to_string(),
                diff_type: DifferenceType::Analysis,
            });
        }

        let similarity_score = self.calculate_similarity_score(&previous, &current, &differences);

        SnapshotComparison {
            previous: previous.clone(),
            current: current.clone(),
            differences,
            similarity_score,
        }
    }

    /// Calculate similarity score between two snapshots
    fn calculate_similarity_score(
        &self,
        _previous: &ResponseSnapshot,
        _current: &ResponseSnapshot,
        differences: &[SnapshotDifference],
    ) -> f64 {
        let base_score = 1.0;
        let penalty_per_difference = 0.1;

        // Additional penalties for critical differences
        let mut total_penalty = differences.len() as f64 * penalty_per_difference;

        for diff in differences {
            match diff.diff_type {
                DifferenceType::ResponseContent => {
                    // Heavy penalty for response content changes
                    total_penalty += 0.3;
                }
                DifferenceType::ToolCalls => {
                    // Medium penalty for tool call changes
                    total_penalty += 0.2;
                }
                DifferenceType::Analysis => {
                    // Light penalty for analysis changes
                    total_penalty += 0.1;
                }
                _ => {}
            }
        }

        (base_score - total_penalty).max(0.0)
    }

    /// Generate a summary report of all snapshots
    pub fn generate_summary(&self) -> Result<SnapshotSummary> {
        let snapshots = self.load_all_snapshots()?;

        let total_snapshots = snapshots.len();
        let mut test_coverage = HashMap::new();
        let mut model_coverage = HashMap::new();
        let mut total_duration = 0u64;
        let mut success_count = 0;
        let mut tool_usage = HashMap::new();

        for snapshot in &snapshots {
            // Test coverage
            *test_coverage
                .entry(snapshot.metadata.test_name.clone())
                .or_insert(0) += 1;

            // Model coverage
            *model_coverage
                .entry(snapshot.metadata.model_name.clone())
                .or_insert(0) += 1;

            // Duration stats
            total_duration += snapshot.evaluation_result.duration_ms;

            // Success rate
            if snapshot.evaluation_result.success {
                success_count += 1;
            }

            // Tool usage
            for tool_call in &snapshot.evaluation_result.tool_calls {
                *tool_usage.entry(tool_call.tool_name.clone()).or_insert(0) += 1;
            }
        }

        let avg_response_time = if total_snapshots > 0 {
            total_duration as f64 / total_snapshots as f64
        } else {
            0.0
        };

        let success_rate = if total_snapshots > 0 {
            success_count as f64 / total_snapshots as f64
        } else {
            0.0
        };

        let mut most_common_tools: Vec<(String, usize)> = tool_usage.into_iter().collect();
        most_common_tools.sort_by(|a, b| b.1.cmp(&a.1));
        most_common_tools.truncate(10); // Top 10 tools

        Ok(SnapshotSummary {
            total_snapshots,
            test_coverage,
            model_coverage,
            avg_response_time,
            success_rate,
            most_common_tools,
        })
    }

    /// Find regression candidates by comparing recent snapshots with baseline
    pub fn find_regressions(&self, baseline_days: u64) -> Result<Vec<SnapshotComparison>> {
        let snapshots = self.load_all_snapshots()?;
        let mut regressions = Vec::new();

        let baseline_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (baseline_days * 24 * 60 * 60);

        // Group snapshots by test name
        let mut test_groups: HashMap<String, Vec<&ResponseSnapshot>> = HashMap::new();
        for snapshot in &snapshots {
            test_groups
                .entry(snapshot.metadata.test_name.clone())
                .or_insert_with(Vec::new)
                .push(snapshot);
        }

        for (_test_name, mut test_snapshots) in test_groups {
            // Sort by timestamp
            test_snapshots.sort_by_key(|s| s.metadata.timestamp);

            // Find baseline (oldest snapshot within baseline period)
            let baseline = test_snapshots
                .iter()
                .find(|s| s.metadata.timestamp >= baseline_timestamp);

            // Find most recent snapshot
            let recent = test_snapshots.last();

            if let (Some(baseline), Some(recent)) = (baseline, recent) {
                if baseline.metadata.timestamp != recent.metadata.timestamp {
                    let comparison = self.compare_snapshots(baseline, recent);

                    // Consider it a regression if similarity is below threshold
                    if comparison.similarity_score < 0.7 {
                        regressions.push(comparison);
                    }
                }
            }
        }

        Ok(regressions)
    }

    /// Export snapshots in different formats for analysis
    pub fn export_snapshots(&self, format: ExportFormat, output_path: &str) -> Result<()> {
        let snapshots = self.load_all_snapshots()?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&snapshots)?;
                fs::write(output_path, json)?;
            }
            ExportFormat::Csv => {
                self.export_to_csv(&snapshots, output_path)?;
            }
            ExportFormat::Summary => {
                let summary = self.generate_summary()?;
                let yaml = serde_yaml::to_string(&summary)?;
                fs::write(output_path, yaml)?;
            }
        }

        Ok(())
    }

    fn export_to_csv(&self, snapshots: &[ResponseSnapshot], output_path: &str) -> Result<()> {
        let mut csv_content = String::new();
        csv_content.push_str("test_name,model_name,timestamp,prompt_hash,duration_ms,tool_calls_made,success,word_count,contains_error,sentiment\n");

        for snapshot in snapshots {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{:?}\n",
                snapshot.metadata.test_name,
                snapshot.metadata.model_name,
                snapshot.metadata.timestamp,
                snapshot.metadata.prompt_hash,
                snapshot.evaluation_result.duration_ms,
                snapshot.evaluation_result.tool_calls_made,
                snapshot.evaluation_result.success,
                snapshot.response_analysis.word_count,
                snapshot.response_analysis.contains_error,
                snapshot.response_analysis.sentiment
            ));
        }

        fs::write(output_path, csv_content)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ExportFormat {
    Json,
    Csv,
    Summary,
}

/// Utility function to clean up old snapshots
pub fn cleanup_old_snapshots(snapshots_dir: &str, days_to_keep: u64) -> Result<usize> {
    let cutoff_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (days_to_keep * 24 * 60 * 60);

    let manager = SnapshotManager::new(snapshots_dir);
    let snapshots = manager.load_all_snapshots()?;
    let mut removed_count = 0;

    for snapshot in snapshots {
        if snapshot.metadata.timestamp < cutoff_timestamp {
            // In a real implementation, we'd need to track the file path
            // and remove the actual file. For now, just count.
            removed_count += 1;
        }
    }

    Ok(removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation_client::*;

    fn create_test_snapshot(test_name: &str, response: &str) -> ResponseSnapshot {
        ResponseSnapshot {
            metadata: SnapshotMetadata {
                test_name: test_name.to_string(),
                model_name: "test-model".to_string(),
                timestamp: 1234567890,
                git_commit: Some("abc123".to_string()),
                prompt_hash: "hash123".to_string(),
            },
            evaluation_result: EvaluationResult {
                prompt: "test prompt".to_string(),
                response: response.to_string(),
                duration_ms: 100,
                tool_calls_made: 1,
                success: true,
                timestamp: 1234567890,
                model_name: "test-model".to_string(),
                tool_calls: vec![],
                conversation_length: 2,
            },
            response_analysis: ResponseAnalysis {
                contains_tool_calls: true,
                contains_code: false,
                contains_error: false,
                word_count: response.split_whitespace().count(),
                sentiment: ResponseSentiment::Neutral,
                success_indicators: vec![],
                failure_indicators: vec![],
            },
        }
    }

    #[test]
    fn test_snapshot_comparison() {
        let manager = SnapshotManager::new("test_snapshots");

        let snapshot1 = create_test_snapshot("test1", "original response");
        let snapshot2 = create_test_snapshot("test1", "modified response");

        let comparison = manager.compare_snapshots(&snapshot1, &snapshot2);

        assert!(!comparison.differences.is_empty());
        assert!(comparison.similarity_score < 1.0);
    }

    #[test]
    fn test_similarity_calculation() {
        let manager = SnapshotManager::new("test_snapshots");

        let snapshot1 = create_test_snapshot("test1", "same response");
        let snapshot2 = create_test_snapshot("test1", "same response");

        let comparison = manager.compare_snapshots(&snapshot1, &snapshot2);

        assert!(comparison.differences.is_empty());
        assert_eq!(comparison.similarity_score, 1.0);
    }
}
