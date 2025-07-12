# LLM Performance Benchmark Framework

A comprehensive statistical benchmarking system for measuring LLM performance with AST-grep MCP server operations.

## Overview

This framework provides systematic performance measurement for LLM interactions with code analysis tools, focusing on:

- **Multi-Iteration Testing**: Statistical significance through repeated measurements
- **Success Rate Analysis**: Percentage of successful command generation/execution
- **Response Time Metrics**: Latency analysis with statistical distributions
- **A/B Testing**: Compare different models, prompts, and configurations
- **Regression Detection**: Track performance changes over time

## Features

### 🎯 Core Benchmarking Capabilities

- **Criterion.rs Integration**: Professional statistical analysis with confidence intervals
- **Multi-Model Comparison**: Test multiple LLM endpoints simultaneously
- **Scenario-Based Testing**: Predefined test cases for different complexity levels
- **Configurable Parameters**: Temperature, iterations, timeouts, and more
- **Real-time Progress**: Live feedback during benchmark execution

### 📊 Statistical Analysis

- **Success Rate Calculation**: Percentage of successful tool calls
- **Response Time Distribution**: Mean, median, percentiles (P95, P99)
- **Consistency Scoring**: Measure response variation across iterations
- **Outlier Detection**: Identify and handle statistical outliers
- **Confidence Intervals**: 95% confidence bounds for all metrics

### 📈 Reporting & Visualization

- **HTML Reports**: Interactive charts and detailed analysis
- **CSV Export**: Raw data for external analysis
- **JSON Output**: Structured data for programmatic access
- **Trend Analysis**: Performance changes over time
- **Recommendations**: Automated insights and suggestions

## Quick Start

### 1. Build the Framework

```bash
cargo build --release
```

### 2. Generate Default Configuration

```bash
./target/release/benchmark-runner --generate-config
```

### 3. Configure Your LLM Endpoints

Edit `benchmark_config.yaml`:

```yaml
models:
  - name: "gpt-3.5-turbo"
    endpoint: "http://localhost:1234/v1"
    api_key: null
    parameters:
      temperature: 0.7
      max_tokens: 1000
```

### 4. Run Benchmarks

```bash
# Quick test with 5 iterations
./target/release/benchmark-runner --iterations 5

# Full benchmark suite
./target/release/benchmark-runner

# Run statistical benchmarks with Criterion
cargo bench
```

## Configuration

### Benchmark Configuration (`benchmark_config.yaml`)

```yaml
name: "Your Benchmark Name"
description: "Benchmark description"
iterations: 10                    # Number of iterations per test
timeout_seconds: 30              # Timeout for each evaluation

models:
  - name: "model-name"
    endpoint: "http://localhost:1234/v1"
    api_key: null                # Optional API key
    parameters:
      temperature: 0.7
      max_tokens: 1000

test_scenarios:
  - name: "Test Name"
    category: "Category"
    prompt: "Your test prompt"
    expected_tools: ["tool_name"]
    success_criteria: "tool_calls > 0"
    weight: 1.0                  # Weight for overall scoring

statistical_config:
  confidence_level: 0.95         # 95% confidence intervals
  min_sample_size: 10
  max_sample_size: 100
  significance_threshold: 0.05
  outlier_detection: true

export_config:
  formats: ["json", "csv", "html"]
  output_directory: "benchmark_results"
  include_raw_data: true
  include_charts: true
```

## Command Line Interface

### Basic Usage

```bash
# List available scenarios
./target/release/benchmark-runner --list-scenarios

# Validate configuration
./target/release/benchmark-runner --dry-run

# Run full benchmark
./target/release/benchmark-runner
```

### Advanced Options

```bash
# Run specific scenarios
./target/release/benchmark-runner \
  --scenario "Basic Function Search" \
  --scenario "Variable Refactoring"

# Test specific models
./target/release/benchmark-runner \
  --model "gpt-3.5-turbo" \
  --model "gpt-4"

# Override iterations
./target/release/benchmark-runner --iterations 20

# Custom output directory
./target/release/benchmark-runner --output custom_results/
```

## Test Scenarios

### Built-in Scenarios

1. **Basic Function Search**
   - Category: AST Search
   - Tests: Simple function declaration finding
   - Weight: 1.0

2. **Variable Refactoring**
   - Category: Code Transformation
   - Tests: var → const/let conversion
   - Weight: 1.5

3. **Scope Analysis**
   - Category: Structural Analysis
   - Tests: Finding containing scopes
   - Weight: 2.0

4. **Error Handling Patterns**
   - Category: Pattern Recognition
   - Tests: try-catch block detection
   - Weight: 1.5

5. **Complex Refactoring**
   - Category: Advanced Transformation
   - Tests: React class → functional component
   - Weight: 3.0

### Custom Scenarios

Add your own test scenarios in the configuration:

```yaml
test_scenarios:
  - name: "Custom Test"
    category: "Custom Category"
    prompt: "Your custom prompt here"
    expected_tools: ["execute_rule"]
    success_criteria: "tool_calls > 0 && response.contains('expected_text')"
    weight: 2.0
```

## Results & Reports

### Output Files

Results are saved in the specified output directory:

```
benchmark_results/
├── benchmark_results.json       # Complete raw data
├── benchmark_summary.csv        # Summary statistics
└── benchmark_report.html        # Interactive HTML report
```

### Key Metrics

- **Success Rate**: Percentage of successful evaluations
- **Average Duration**: Mean response time in milliseconds
- **P95/P99 Latency**: 95th and 99th percentile response times
- **Reliability Score**: Overall reliability metric
- **Consistency Score**: Response variation measurement
- **Tool Call Rate**: Average tool calls per request

### Model Rankings

Models are ranked by a composite score considering:
- Success Rate (50% weight)
- Reliability (30% weight)
- Speed (20% weight)

## Criterion Benchmarks

For advanced statistical analysis, use Criterion benchmarks:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench benchmark_llm_response_time

# Generate HTML report
cargo bench --bench llm_benchmarks
```

Available benchmarks:
- `benchmark_llm_response_time`: Response time analysis
- `benchmark_success_rate`: Success rate measurement
- `benchmark_model_comparison`: Model performance comparison
- `benchmark_temperature_effects`: Temperature parameter impact
- `benchmark_prompt_complexity`: Prompt complexity analysis
- `benchmark_concurrent_requests`: Concurrency performance

## Integration with Evaluation Client

The benchmark framework integrates with the existing evaluation client:

```rust
use mcp_ast_grep::evaluation_client::{EvaluationClient, EvaluationClientConfig};
use mcp_ast_grep::benchmark_utils::{BenchmarkRunner, BenchmarkConfiguration};

// Create configuration
let config = BenchmarkConfiguration::default();

// Run benchmarks
let mut runner = BenchmarkRunner::new(config);
let summary = runner.run_full_benchmark().await?;
```

## Performance Optimization

### LLM Server Setup

For optimal performance:

1. **Use Local LLM Server**: Reduce network latency
2. **Configure Appropriate Hardware**: GPU acceleration recommended
3. **Optimize Model Parameters**: Balance quality vs. speed
4. **Monitor Resource Usage**: CPU, memory, and GPU utilization

### Benchmark Tuning

- **Start Small**: Begin with 5-10 iterations for testing
- **Parallel Testing**: Use multiple model configurations
- **Scenario Filtering**: Focus on relevant test cases
- **Statistical Significance**: Use minimum sample sizes

## Troubleshooting

### Common Issues

1. **LLM Server Not Running**
   ```bash
   # Check if server is accessible
   curl http://localhost:1234/v1/models
   ```

2. **Configuration Errors**
   ```bash
   # Validate configuration
   ./target/release/benchmark-runner --dry-run
   ```

3. **Memory Issues**
   - Reduce iterations or concurrent requests
   - Monitor system resources during benchmarks

4. **Network Timeouts**
   - Increase timeout_seconds in configuration
   - Check network connectivity to LLM endpoint

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug ./target/release/benchmark-runner
```

## Best Practices

### Configuration

1. **Realistic Scenarios**: Use prompts representative of actual use cases
2. **Appropriate Weights**: Weight scenarios by importance
3. **Statistical Rigor**: Use sufficient iterations for significance
4. **Consistent Environment**: Run benchmarks in stable conditions

### Analysis

1. **Compare Baselines**: Establish performance baselines
2. **Monitor Trends**: Track performance over time
3. **Consider Context**: Evaluate results in context of use cases
4. **Document Changes**: Note configuration or environment changes

### Reporting

1. **Share Results**: Export results in multiple formats
2. **Provide Context**: Include configuration and environment details
3. **Regular Reviews**: Schedule periodic benchmark reviews
4. **Action Items**: Convert insights into actionable improvements

## Contributing

To add new benchmark scenarios or improve the framework:

1. Add scenarios to `benchmark_config.yaml`
2. Implement custom evaluation logic if needed
3. Update documentation
4. Test thoroughly with multiple models

## License

This benchmark framework is part of the organized-grepster-mcp project and follows the same license terms.