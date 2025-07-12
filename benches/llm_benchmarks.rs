use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mcp_ast_grep::evaluation_client::{
    EvaluationClient, EvaluationClientConfig, EvaluationResult, create_default_test_cases
};
use std::time::Duration;
use tokio::runtime::Runtime;
use futures::future;

mod benchmark_suite {
    use super::*;
    
    pub struct BenchmarkConfig {
        pub name: String,
        pub iterations: usize,
        pub endpoint: String,
        pub model_name: String,
        pub temperature: Option<f64>,
        pub max_tokens: Option<u32>,
    }
    
    impl Default for BenchmarkConfig {
        fn default() -> Self {
            Self {
                name: "default".to_string(),
                iterations: 10,
                endpoint: "http://localhost:1234/v1".to_string(),
                model_name: "gpt-3.5-turbo".to_string(),
                temperature: None,
                max_tokens: None,
            }
        }
    }
    
    pub struct BenchmarkResult {
        pub config: BenchmarkConfig,
        pub test_results: Vec<EvaluationResult>,
        pub success_rate: f64,
        pub avg_duration_ms: f64,
        pub std_deviation_ms: f64,
        pub min_duration_ms: u64,
        pub max_duration_ms: u64,
        pub tool_calls_per_request: f64,
        pub error_count: usize,
    }
    
    impl BenchmarkResult {
        pub fn from_results(config: BenchmarkConfig, results: Vec<EvaluationResult>) -> Self {
            let success_count = results.iter().filter(|r| r.success).count();
            let success_rate = success_count as f64 / results.len() as f64;
            
            let durations: Vec<u64> = results.iter().map(|r| r.duration_ms).collect();
            let avg_duration_ms = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
            
            let variance = durations.iter()
                .map(|d| (*d as f64 - avg_duration_ms).powi(2))
                .sum::<f64>() / durations.len() as f64;
            let std_deviation_ms = variance.sqrt();
            
            let min_duration_ms = *durations.iter().min().unwrap_or(&0);
            let max_duration_ms = *durations.iter().max().unwrap_or(&0);
            
            let total_tool_calls: usize = results.iter().map(|r| r.tool_calls_made).sum();
            let tool_calls_per_request = total_tool_calls as f64 / results.len() as f64;
            
            let error_count = results.iter().filter(|r| !r.success).count();
            
            Self {
                config,
                test_results: results,
                success_rate,
                avg_duration_ms,
                std_deviation_ms,
                min_duration_ms,
                max_duration_ms,
                tool_calls_per_request,
                error_count,
            }
        }
    }
    
    pub async fn run_benchmark_suite(config: BenchmarkConfig, prompt: &str) -> anyhow::Result<BenchmarkResult> {
        let eval_config = EvaluationClientConfig {
            llm_endpoint: config.endpoint.clone(),
            llm_api_key: None,
            model_name: config.model_name.clone(),
            server_command: "echo".to_string(), // Mock server for benchmarking
            server_args: vec!["mock".to_string()],
            timeout_seconds: 30,
        };
        
        let mut client = EvaluationClient::new(eval_config);
        let mut results = Vec::new();
        
        for i in 0..config.iterations {
            client.reset_conversation();
            
            // Add jitter to avoid thundering herd
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            match client.evaluate_prompt(prompt).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Benchmark iteration {} failed: {}", i, e);
                    // Create a failed result for consistency
                    results.push(EvaluationResult {
                        prompt: prompt.to_string(),
                        response: format!("Error: {}", e),
                        duration_ms: 0,
                        tool_calls_made: 0,
                        success: false,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        model_name: config.model_name.clone(),
                        tool_calls: vec![],
                        conversation_length: 0,
                    });
                }
            }
        }
        
        Ok(BenchmarkResult::from_results(config, results))
    }
}

use benchmark_suite::*;

fn benchmark_llm_response_time(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("llm_response_time");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));
    
    let test_prompts = vec![
        "Find all function declarations in this code: function hello() { return 'world'; }",
        "Replace console.log with logger.info in: console.log('test');",
        "Find the scope containing line 1, column 5 in: function test() { const x = 1; }",
    ];
    
    for prompt in test_prompts {
        group.bench_with_input(
            BenchmarkId::new("basic_prompt", prompt.len()),
            &prompt,
            |b, prompt| {
                b.iter(|| {
                    let config = BenchmarkConfig {
                        name: "response_time_test".to_string(),
                        iterations: 1,
                        ..Default::default()
                    };
                    
                    let result = rt.block_on(run_benchmark_suite(config, prompt)).unwrap();
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_success_rate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("success_rate");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(120));
    
    let test_cases = create_default_test_cases();
    
    for test_case in test_cases {
        group.bench_with_input(
            BenchmarkId::new("success_rate", &test_case.name),
            &test_case.prompt,
            |b, prompt| {
                b.iter(|| {
                    let config = BenchmarkConfig {
                        name: "success_rate_test".to_string(),
                        iterations: 10,
                        ..Default::default()
                    };
                    
                    let result = rt.block_on(run_benchmark_suite(config, prompt)).unwrap();
                    black_box(result.success_rate)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_model_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("model_comparison");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(180));
    
    let models = vec![
        "gpt-3.5-turbo",
        "gpt-4",
        "claude-3-sonnet",
        "llama-2-7b",
    ];
    
    let test_prompt = "Find all function declarations in this JavaScript code: function test() { return 42; }";
    
    for model in models {
        group.bench_with_input(
            BenchmarkId::new("model_comparison", model),
            &model,
            |b, model_name| {
                b.iter(|| {
                    let config = BenchmarkConfig {
                        name: format!("model_comparison_{}", model_name),
                        iterations: 5,
                        model_name: model_name.to_string(),
                        ..Default::default()
                    };
                    
                    let result = rt.block_on(run_benchmark_suite(config, test_prompt)).unwrap();
                    black_box((result.avg_duration_ms, result.success_rate))
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_temperature_effects(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("temperature_effects");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(120));
    
    let temperatures = vec![0.0, 0.3, 0.7, 1.0];
    let test_prompt = "Refactor this code to use modern JavaScript: var x = 1; var y = 2;";
    
    for temp in temperatures {
        group.bench_with_input(
            BenchmarkId::new("temperature", format!("{:.1}", temp)),
            &temp,
            |b, temperature| {
                b.iter(|| {
                    let config = BenchmarkConfig {
                        name: format!("temperature_{}", temperature),
                        iterations: 3,
                        temperature: Some(*temperature),
                        ..Default::default()
                    };
                    
                    let result = rt.block_on(run_benchmark_suite(config, test_prompt)).unwrap();
                    black_box((result.avg_duration_ms, result.success_rate, result.tool_calls_per_request))
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_prompt_complexity(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("prompt_complexity");
    group.sample_size(25);
    group.measurement_time(Duration::from_secs(150));
    
    let complexity_tests = vec![
        ("simple", "Find functions in: function test() {}"),
        ("medium", "Replace all console.log statements with logger.info and add error handling in this code: function process() { console.log('start'); try { doWork(); } catch(e) { console.log('error:', e); } }"),
        ("complex", "Analyze this React component and modernize it by converting class components to functional components with hooks, replacing var with const/let, and adding TypeScript types: var MyComponent = React.createClass({ getInitialState: function() { return {count: 0}; }, render: function() { return React.createElement('div', null, 'Count: ' + this.state.count); } });"),
    ];
    
    for (complexity, prompt) in complexity_tests {
        group.throughput(Throughput::Bytes(prompt.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("complexity", complexity),
            &prompt,
            |b, prompt| {
                b.iter(|| {
                    let config = BenchmarkConfig {
                        name: format!("complexity_{}", complexity),
                        iterations: 3,
                        ..Default::default()
                    };
                    
                    let result = rt.block_on(run_benchmark_suite(config, prompt)).unwrap();
                    black_box((result.avg_duration_ms, result.success_rate, result.tool_calls_per_request))
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_concurrent_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_requests");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(90));
    
    let concurrency_levels = vec![1, 2, 4, 8];
    let test_prompt = "Find all variable declarations in: let x = 1; const y = 2; var z = 3;";
    
    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrency", concurrency),
            &concurrency,
            |b, &concurrency_level| {
                b.iter(|| {
                    let results = rt.block_on(async {
                        let mut handles = Vec::new();
                        
                        for _ in 0..concurrency_level {
                            let config = BenchmarkConfig {
                                name: format!("concurrent_{}", concurrency_level),
                                iterations: 2,
                                ..Default::default()
                            };
                            
                            let prompt = test_prompt.to_string();
                            let handle = tokio::spawn(async move {
                                run_benchmark_suite(config, &prompt).await.unwrap()
                            });
                            handles.push(handle);
                        }
                        
                        let results = futures::future::join_all(handles).await;
                        results.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>()
                    });
                    
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_llm_response_time,
    benchmark_success_rate,
    benchmark_model_comparison,
    benchmark_temperature_effects,
    benchmark_prompt_complexity,
    benchmark_concurrent_requests
);

criterion_main!(benches);