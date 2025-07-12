use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::evaluation_client::{EvaluationClient, EvaluationClientConfig, EvaluationResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfiguration {
    pub name: String,
    pub description: String,
    pub iterations: usize,
    pub timeout_seconds: u64,
    pub models: Vec<ModelConfig>,
    pub test_scenarios: Vec<TestScenario>,
    pub statistical_config: StatisticalConfig,
    pub export_config: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub name: String,
    pub category: String,
    pub prompt: String,
    pub expected_tools: Vec<String>,
    pub success_criteria: String, // JSON-serializable criteria
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    pub confidence_level: f64,
    pub min_sample_size: usize,
    pub max_sample_size: usize,
    pub significance_threshold: f64,
    pub outlier_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub formats: Vec<String>, // ["csv", "json", "html"]
    pub output_directory: String,
    pub include_raw_data: bool,
    pub include_charts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSession {
    pub configuration: BenchmarkConfiguration,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub results: Vec<ModelBenchmarkResult>,
    pub summary: Option<BenchmarkSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBenchmarkResult {
    pub model_name: String,
    pub scenario_results: Vec<ScenarioResult>,
    pub overall_metrics: OverallMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub evaluations: Vec<EvaluationResult>,
    pub metrics: ScenarioMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMetrics {
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub std_deviation_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
    pub tool_calls_per_request: f64,
    pub error_rate: f64,
    pub consistency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallMetrics {
    pub weighted_success_rate: f64,
    pub total_duration_ms: u64,
    pub avg_scenario_duration_ms: f64,
    pub total_evaluations: usize,
    pub total_errors: usize,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_duration_ms: u64,
    pub total_evaluations: usize,
    pub model_rankings: Vec<ModelRanking>,
    pub scenario_difficulty: Vec<ScenarioDifficulty>,
    pub performance_trends: Vec<PerformanceTrend>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRanking {
    pub model_name: String,
    pub overall_score: f64,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub reliability_score: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDifficulty {
    pub scenario_name: String,
    pub difficulty_score: f64,
    pub avg_success_rate: f64,
    pub avg_duration_ms: f64,
    pub model_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub metric_name: String,
    pub trend_direction: String, // "improving", "declining", "stable"
    pub slope: f64,
    pub r_squared: f64,
}

impl Default for BenchmarkConfiguration {
    fn default() -> Self {
        Self {
            name: "Default Benchmark".to_string(),
            description: "Comprehensive LLM performance evaluation".to_string(),
            iterations: 10,
            timeout_seconds: 30,
            models: vec![ModelConfig {
                name: "gpt-3.5-turbo".to_string(),
                endpoint: "http://localhost:1234/v1".to_string(),
                api_key: None,
                parameters: HashMap::new(),
            }],
            test_scenarios: create_default_scenarios(),
            statistical_config: StatisticalConfig {
                confidence_level: 0.95,
                min_sample_size: 10,
                max_sample_size: 100,
                significance_threshold: 0.05,
                outlier_detection: true,
            },
            export_config: ExportConfig {
                formats: vec!["json".to_string(), "csv".to_string(), "html".to_string()],
                output_directory: "benchmark_results".to_string(),
                include_raw_data: true,
                include_charts: true,
            },
        }
    }
}

pub fn create_default_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            name: "Basic Function Search".to_string(),
            category: "AST Search".to_string(),
            prompt: "Find all function declarations in this JavaScript code: function hello() { return 'world'; } const greet = () => 'hi';".to_string(),
            expected_tools: vec!["execute_rule".to_string()],
            success_criteria: "tool_calls > 0 && response.contains('function')".to_string(),
            weight: 1.0,
        },
        TestScenario {
            name: "Variable Refactoring".to_string(),
            category: "Code Transformation".to_string(),
            prompt: "Replace all var declarations with const/let in: var x = 1; var y = 2; function test() { var z = 3; }".to_string(),
            expected_tools: vec!["execute_rule".to_string()],
            success_criteria: "tool_calls > 0 && response.contains('const') || response.contains('let')".to_string(),
            weight: 1.5,
        },
        TestScenario {
            name: "Scope Analysis".to_string(),
            category: "Structural Analysis".to_string(),
            prompt: "Find the containing scope around line 1, column 15 in: function outer() { function inner() { const x = 1; } }".to_string(),
            expected_tools: vec!["find_scope".to_string()],
            success_criteria: "tool_calls > 0 && success == true".to_string(),
            weight: 2.0,
        },
        TestScenario {
            name: "Error Handling Patterns".to_string(),
            category: "Pattern Recognition".to_string(),
            prompt: "Find all try-catch blocks and identify error handling patterns in: try { riskyOperation(); } catch (e) { console.error(e); } finally { cleanup(); }".to_string(),
            expected_tools: vec!["execute_rule".to_string()],
            success_criteria: "tool_calls > 0 && response.contains('try') && response.contains('catch')".to_string(),
            weight: 1.5,
        },
        TestScenario {
            name: "Complex Refactoring".to_string(),
            category: "Advanced Transformation".to_string(),
            prompt: "Modernize this React class component to functional component with hooks: class Counter extends React.Component { constructor(props) { super(props); this.state = { count: 0 }; } render() { return <div>{this.state.count}</div>; } }".to_string(),
            expected_tools: vec!["execute_rule".to_string()],
            success_criteria: "tool_calls > 0 && response.contains('useState') || response.contains('function')".to_string(),
            weight: 3.0,
        },
    ]
}

pub struct BenchmarkRunner {
    pub config: BenchmarkConfiguration,
    pub session: BenchmarkSession,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfiguration) -> Self {
        let session = BenchmarkSession {
            configuration: config.clone(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            end_time: None,
            results: Vec::new(),
            summary: None,
        };

        Self { config, session }
    }

    pub async fn run_full_benchmark(&mut self) -> Result<BenchmarkSummary> {
        let mut all_results = Vec::new();

        for model in &self.config.models {
            println!("Running benchmark for model: {}", model.name);
            let model_result = self.run_model_benchmark(model).await?;
            all_results.push(model_result);
        }

        self.session.results = all_results;
        self.session.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        let summary = self.generate_summary();
        self.session.summary = Some(summary.clone());

        Ok(summary)
    }

    async fn run_model_benchmark(&self, model: &ModelConfig) -> Result<ModelBenchmarkResult> {
        let eval_config = EvaluationClientConfig {
            llm_endpoint: model.endpoint.clone(),
            llm_api_key: model.api_key.clone(),
            model_name: model.name.clone(),
            server_command: "echo".to_string(), // Mock for benchmarking
            server_args: vec!["mock".to_string()],
            timeout_seconds: self.config.timeout_seconds,
        };

        let mut client = EvaluationClient::new(eval_config);
        let mut scenario_results = Vec::new();

        for scenario in &self.config.test_scenarios {
            println!("  Running scenario: {}", scenario.name);
            let scenario_result = self.run_scenario_benchmark(&mut client, scenario).await?;
            scenario_results.push(scenario_result);
        }

        let overall_metrics = self.calculate_overall_metrics(&scenario_results);

        Ok(ModelBenchmarkResult {
            model_name: model.name.clone(),
            scenario_results,
            overall_metrics,
        })
    }

    async fn run_scenario_benchmark(
        &self,
        client: &mut EvaluationClient,
        scenario: &TestScenario,
    ) -> Result<ScenarioResult> {
        let mut evaluations = Vec::new();

        for i in 0..self.config.iterations {
            client.reset_conversation();

            // Add small delay to avoid overwhelming the API
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            match client.evaluate_prompt(&scenario.prompt).await {
                Ok(evaluation) => evaluations.push(evaluation),
                Err(e) => {
                    eprintln!("Evaluation failed: {}", e);
                    // Create a failed evaluation record
                    evaluations.push(EvaluationResult {
                        prompt: scenario.prompt.clone(),
                        response: format!("Error: {}", e),
                        duration_ms: 0,
                        tool_calls_made: 0,
                        success: false,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        model_name: client.config.model_name.clone(),
                        tool_calls: vec![],
                        conversation_length: 0,
                    });
                }
            }
        }

        let metrics = self.calculate_scenario_metrics(&evaluations);

        Ok(ScenarioResult {
            scenario_name: scenario.name.clone(),
            evaluations,
            metrics,
        })
    }

    fn calculate_scenario_metrics(&self, evaluations: &[EvaluationResult]) -> ScenarioMetrics {
        let success_count = evaluations.iter().filter(|e| e.success).count();
        let success_rate = success_count as f64 / evaluations.len() as f64;

        let durations: Vec<u64> = evaluations.iter().map(|e| e.duration_ms).collect();
        let avg_duration_ms = durations.iter().sum::<u64>() as f64 / durations.len() as f64;

        let variance = durations
            .iter()
            .map(|d| (*d as f64 - avg_duration_ms).powi(2))
            .sum::<f64>()
            / durations.len() as f64;
        let std_deviation_ms = variance.sqrt();

        let min_duration_ms = *durations.iter().min().unwrap_or(&0);
        let max_duration_ms = *durations.iter().max().unwrap_or(&0);

        let mut sorted_durations = durations.clone();
        sorted_durations.sort_unstable();
        let p95_duration_ms = Self::percentile(&sorted_durations, 0.95);
        let p99_duration_ms = Self::percentile(&sorted_durations, 0.99);

        let total_tool_calls: usize = evaluations.iter().map(|e| e.tool_calls_made).sum();
        let tool_calls_per_request = total_tool_calls as f64 / evaluations.len() as f64;

        let error_count = evaluations.iter().filter(|e| !e.success).count();
        let error_rate = error_count as f64 / evaluations.len() as f64;

        let consistency_score = self.calculate_consistency_score(evaluations);

        ScenarioMetrics {
            success_rate,
            avg_duration_ms,
            std_deviation_ms,
            min_duration_ms,
            max_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            tool_calls_per_request,
            error_rate,
            consistency_score,
        }
    }

    fn percentile(sorted_data: &[u64], percentile: f64) -> f64 {
        let index = (percentile * (sorted_data.len() - 1) as f64).round() as usize;
        sorted_data.get(index).copied().unwrap_or(0) as f64
    }

    fn calculate_consistency_score(&self, evaluations: &[EvaluationResult]) -> f64 {
        // Calculate consistency based on response similarity and tool usage patterns
        let _responses: Vec<&str> = evaluations.iter().map(|e| e.response.as_str()).collect();
        let tool_counts: Vec<usize> = evaluations.iter().map(|e| e.tool_calls_made).collect();

        // Simple consistency metric based on tool usage variance
        if tool_counts.is_empty() {
            return 0.0;
        }

        let avg_tools = tool_counts.iter().sum::<usize>() as f64 / tool_counts.len() as f64;
        let tool_variance = tool_counts
            .iter()
            .map(|&c| (c as f64 - avg_tools).powi(2))
            .sum::<f64>()
            / tool_counts.len() as f64;

        // Higher consistency score for lower variance
        1.0 / (1.0 + tool_variance)
    }

    fn calculate_overall_metrics(&self, scenario_results: &[ScenarioResult]) -> OverallMetrics {
        let total_evaluations: usize = scenario_results.iter().map(|sr| sr.evaluations.len()).sum();

        let total_errors: usize = scenario_results
            .iter()
            .map(|sr| sr.evaluations.iter().filter(|e| !e.success).count())
            .sum();

        let total_duration_ms: u64 = scenario_results
            .iter()
            .map(|sr| sr.evaluations.iter().map(|e| e.duration_ms).sum::<u64>())
            .sum();

        let weighted_success_rate = scenario_results
            .iter()
            .zip(&self.config.test_scenarios)
            .map(|(result, scenario)| result.metrics.success_rate * scenario.weight)
            .sum::<f64>()
            / self
                .config
                .test_scenarios
                .iter()
                .map(|s| s.weight)
                .sum::<f64>();

        let avg_scenario_duration_ms = total_duration_ms as f64 / scenario_results.len() as f64;

        let reliability_score = 1.0 - (total_errors as f64 / total_evaluations as f64);

        OverallMetrics {
            weighted_success_rate,
            total_duration_ms,
            avg_scenario_duration_ms,
            total_evaluations,
            total_errors,
            reliability_score,
        }
    }

    fn generate_summary(&self) -> BenchmarkSummary {
        let total_duration_ms = self.session.end_time.unwrap_or(0) - self.session.start_time;
        let total_evaluations = self
            .session
            .results
            .iter()
            .map(|r| r.overall_metrics.total_evaluations)
            .sum();

        let model_rankings = self.generate_model_rankings();
        let scenario_difficulty = self.generate_scenario_difficulty();
        let performance_trends = self.generate_performance_trends();
        let recommendations = self.generate_recommendations();

        BenchmarkSummary {
            total_duration_ms: total_duration_ms * 1000, // Convert to milliseconds
            total_evaluations,
            model_rankings,
            scenario_difficulty,
            performance_trends,
            recommendations,
        }
    }

    fn generate_model_rankings(&self) -> Vec<ModelRanking> {
        let mut rankings: Vec<ModelRanking> = self
            .session
            .results
            .iter()
            .map(|result| {
                let overall_score = result.overall_metrics.weighted_success_rate * 0.5
                    + result.overall_metrics.reliability_score * 0.3
                    + (1.0 / (1.0 + result.overall_metrics.avg_scenario_duration_ms / 1000.0))
                        * 0.2;

                ModelRanking {
                    model_name: result.model_name.clone(),
                    overall_score,
                    success_rate: result.overall_metrics.weighted_success_rate,
                    avg_duration_ms: result.overall_metrics.avg_scenario_duration_ms,
                    reliability_score: result.overall_metrics.reliability_score,
                    rank: 0, // Will be set after sorting
                }
            })
            .collect();

        rankings.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

        for (i, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = i + 1;
        }

        rankings
    }

    fn generate_scenario_difficulty(&self) -> Vec<ScenarioDifficulty> {
        self.config
            .test_scenarios
            .iter()
            .map(|scenario| {
                let scenario_results: Vec<_> = self
                    .session
                    .results
                    .iter()
                    .filter_map(|r| {
                        r.scenario_results
                            .iter()
                            .find(|sr| sr.scenario_name == scenario.name)
                    })
                    .collect();

                let avg_success_rate = scenario_results
                    .iter()
                    .map(|sr| sr.metrics.success_rate)
                    .sum::<f64>()
                    / scenario_results.len() as f64;

                let avg_duration_ms = scenario_results
                    .iter()
                    .map(|sr| sr.metrics.avg_duration_ms)
                    .sum::<f64>()
                    / scenario_results.len() as f64;

                let success_rates: Vec<f64> = scenario_results
                    .iter()
                    .map(|sr| sr.metrics.success_rate)
                    .collect();

                let variance = success_rates
                    .iter()
                    .map(|&sr| (sr - avg_success_rate).powi(2))
                    .sum::<f64>()
                    / success_rates.len() as f64;

                let difficulty_score =
                    (1.0 - avg_success_rate) + (avg_duration_ms / 10000.0) + variance;

                ScenarioDifficulty {
                    scenario_name: scenario.name.clone(),
                    difficulty_score,
                    avg_success_rate,
                    avg_duration_ms,
                    model_variance: variance,
                }
            })
            .collect()
    }

    fn generate_performance_trends(&self) -> Vec<PerformanceTrend> {
        // For now, return placeholder trends
        // In a real implementation, this would analyze historical data
        vec![
            PerformanceTrend {
                metric_name: "Success Rate".to_string(),
                trend_direction: "stable".to_string(),
                slope: 0.0,
                r_squared: 0.0,
            },
            PerformanceTrend {
                metric_name: "Response Time".to_string(),
                trend_direction: "stable".to_string(),
                slope: 0.0,
                r_squared: 0.0,
            },
        ]
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze results and generate recommendations
        for result in &self.session.results {
            if result.overall_metrics.reliability_score < 0.8 {
                recommendations.push(
                    format!("Model '{}' has low reliability ({}). Consider adjusting parameters or switching models.",
                            result.model_name, result.overall_metrics.reliability_score)
                );
            }

            if result.overall_metrics.avg_scenario_duration_ms > 5000.0 {
                recommendations.push(
                    format!("Model '{}' has high average response time ({}ms). Consider optimizing prompts or using a faster model.",
                            result.model_name, result.overall_metrics.avg_scenario_duration_ms)
                );
            }
        }

        if recommendations.is_empty() {
            recommendations.push("All models performed within acceptable parameters.".to_string());
        }

        recommendations
    }

    pub fn save_results(&self, output_dir: &str) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        // Save detailed results as JSON
        let json_path = format!("{}/benchmark_results.json", output_dir);
        let json_data = serde_json::to_string_pretty(&self.session)?;
        std::fs::write(&json_path, json_data)?;

        // Save summary as CSV
        if self
            .config
            .export_config
            .formats
            .contains(&"csv".to_string())
        {
            self.save_csv_summary(output_dir)?;
        }

        // Save HTML report
        if self
            .config
            .export_config
            .formats
            .contains(&"html".to_string())
        {
            self.save_html_report(output_dir)?;
        }

        println!("Benchmark results saved to: {}", output_dir);
        Ok(())
    }

    fn save_csv_summary(&self, output_dir: &str) -> Result<()> {
        let csv_path = format!("{}/benchmark_summary.csv", output_dir);
        let mut csv_content = String::new();

        // Header
        csv_content.push_str("Model,Success Rate,Avg Duration (ms),Reliability Score,Rank\n");

        // Data rows
        if let Some(summary) = &self.session.summary {
            for ranking in &summary.model_rankings {
                csv_content.push_str(&format!(
                    "{},{:.3},{:.1},{:.3},{}\n",
                    ranking.model_name,
                    ranking.success_rate,
                    ranking.avg_duration_ms,
                    ranking.reliability_score,
                    ranking.rank
                ));
            }
        }

        std::fs::write(&csv_path, csv_content)?;
        Ok(())
    }

    fn save_html_report(&self, output_dir: &str) -> Result<()> {
        let html_path = format!("{}/benchmark_report.html", output_dir);
        let html_content = self.generate_html_report();
        std::fs::write(&html_path, html_content)?;
        Ok(())
    }

    fn generate_html_report(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html><head><title>LLM Benchmark Report</title>");
        html.push_str(
            "<style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            table { border-collapse: collapse; width: 100%; margin: 20px 0; }
            th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
            th { background-color: #f2f2f2; }
            .metric { background-color: #f9f9f9; padding: 10px; margin: 10px 0; }
            .good { color: green; }
            .warning { color: orange; }
            .bad { color: red; }
        </style></head><body>",
        );

        html.push_str(&format!("<h1>LLM Benchmark Report</h1>"));
        html.push_str(&format!("<h2>Configuration: {}</h2>", self.config.name));
        html.push_str(&format!("<p>{}</p>", self.config.description));

        if let Some(summary) = &self.session.summary {
            html.push_str("<h2>Summary</h2>");
            html.push_str(&format!(
                "<div class='metric'>Total Evaluations: {}</div>",
                summary.total_evaluations
            ));
            html.push_str(&format!(
                "<div class='metric'>Total Duration: {:.2}s</div>",
                summary.total_duration_ms as f64 / 1000.0
            ));

            html.push_str("<h2>Model Rankings</h2>");
            html.push_str("<table>");
            html.push_str("<tr><th>Rank</th><th>Model</th><th>Overall Score</th><th>Success Rate</th><th>Avg Duration (ms)</th><th>Reliability</th></tr>");

            for ranking in &summary.model_rankings {
                let class = if ranking.success_rate > 0.8 {
                    "good"
                } else if ranking.success_rate > 0.6 {
                    "warning"
                } else {
                    "bad"
                };
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td class='{}'>{:.3}</td><td class='{}'>{:.1}%</td><td>{:.1}</td><td class='{}'>{:.1}%</td></tr>",
                    ranking.rank,
                    ranking.model_name,
                    class,
                    ranking.overall_score,
                    class,
                    ranking.success_rate * 100.0,
                    ranking.avg_duration_ms,
                    class,
                    ranking.reliability_score * 100.0
                ));
            }
            html.push_str("</table>");

            html.push_str("<h2>Recommendations</h2>");
            html.push_str("<ul>");
            for recommendation in &summary.recommendations {
                html.push_str(&format!("<li>{}</li>", recommendation));
            }
            html.push_str("</ul>");
        }

        html.push_str("</body></html>");
        html
    }
}

pub fn load_benchmark_config(path: &Path) -> Result<BenchmarkConfiguration> {
    let contents = fs::read_to_string(path)?;
    let config: BenchmarkConfiguration = serde_yaml::from_str(&contents)?;
    Ok(config)
}

pub fn save_benchmark_config(config: &BenchmarkConfiguration, path: &Path) -> Result<()> {
    let yaml_content = serde_yaml::to_string(config)?;
    fs::write(path, yaml_content)?;
    Ok(())
}
