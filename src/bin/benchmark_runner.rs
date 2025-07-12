use anyhow::Result;
use clap::{Arg, Command};
use splice_weaver_mcp::benchmark_utils::{
    load_benchmark_config, save_benchmark_config, BenchmarkConfiguration, BenchmarkRunner,
};
use std::path::Path;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("benchmark-runner")
        .version("0.1.0")
        .about("LLM Performance Benchmark Runner")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Path to benchmark configuration file")
                .default_value("benchmark_config.yaml"),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("DIR")
                .help("Output directory for results")
                .default_value("benchmark_results"),
        )
        .arg(
            Arg::new("generate-config")
                .long("generate-config")
                .help("Generate default configuration file")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list-scenarios")
                .long("list-scenarios")
                .help("List available test scenarios")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Run configuration validation without executing benchmarks")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("iterations")
                .long("iterations")
                .short('i')
                .value_name("COUNT")
                .help("Override number of iterations per test")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("scenario")
                .long("scenario")
                .short('s')
                .value_name("NAME")
                .help("Run only specific scenario")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("model")
                .long("model")
                .short('m')
                .value_name("NAME")
                .help("Run only specific model")
                .action(clap::ArgAction::Append),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let output_dir = matches.get_one::<String>("output").unwrap();

    if matches.get_flag("generate-config") {
        generate_default_config(config_path)?;
        return Ok(());
    }

    if matches.get_flag("list-scenarios") {
        list_scenarios(config_path)?;
        return Ok(());
    }

    info!("Loading benchmark configuration from: {}", config_path);
    let mut config = load_benchmark_config(Path::new(config_path))?;

    // Apply command-line overrides
    if let Some(iterations) = matches.get_one::<usize>("iterations") {
        config.iterations = *iterations;
        info!("Overriding iterations to: {}", iterations);
    }

    if let Some(scenario_names) = matches.get_many::<String>("scenario") {
        let scenario_names: Vec<String> = scenario_names.cloned().collect();
        config
            .test_scenarios
            .retain(|s| scenario_names.contains(&s.name));
        info!("Filtering to scenarios: {:?}", scenario_names);
    }

    if let Some(model_names) = matches.get_many::<String>("model") {
        let model_names: Vec<String> = model_names.cloned().collect();
        config.models.retain(|m| model_names.contains(&m.name));
        info!("Filtering to models: {:?}", model_names);
    }

    if matches.get_flag("dry-run") {
        validate_configuration(&config)?;
        return Ok(());
    }

    info!("Starting benchmark suite: {}", config.name);
    info!("Running {} iterations per scenario", config.iterations);
    info!(
        "Testing {} models against {} scenarios",
        config.models.len(),
        config.test_scenarios.len()
    );

    let mut runner = BenchmarkRunner::new(config);

    match runner.run_full_benchmark().await {
        Ok(summary) => {
            info!("Benchmark completed successfully!");
            print_summary(&summary);

            runner.save_results(output_dir)?;
            info!("Results saved to: {}", output_dir);
        }
        Err(e) => {
            error!("Benchmark failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn generate_default_config(path: &str) -> Result<()> {
    let config = BenchmarkConfiguration::default();
    save_benchmark_config(&config, Path::new(path))?;
    println!("Generated default configuration at: {}", path);
    println!("Edit this file to customize your benchmark settings.");
    Ok(())
}

fn list_scenarios(config_path: &str) -> Result<()> {
    let config = if Path::new(config_path).exists() {
        load_benchmark_config(Path::new(config_path))?
    } else {
        BenchmarkConfiguration::default()
    };

    println!("Available test scenarios:");
    println!(
        "{:<25} {:<20} {:<10} {}",
        "Name", "Category", "Weight", "Description"
    );
    println!("{}", "-".repeat(80));

    for scenario in &config.test_scenarios {
        let description = if scenario.prompt.len() > 40 {
            format!("{}...", &scenario.prompt[..37])
        } else {
            scenario.prompt.clone()
        };

        println!(
            "{:<25} {:<20} {:<10.1} {}",
            scenario.name, scenario.category, scenario.weight, description
        );
    }

    Ok(())
}

fn validate_configuration(config: &BenchmarkConfiguration) -> Result<()> {
    println!("Validating benchmark configuration...");

    // Check models
    if config.models.is_empty() {
        return Err(anyhow::anyhow!("No models configured"));
    }

    for model in &config.models {
        println!("Model: {} - Endpoint: {}", model.name, model.endpoint);

        // Validate endpoint format
        if !model.endpoint.starts_with("http://") && !model.endpoint.starts_with("https://") {
            return Err(anyhow::anyhow!(
                "Invalid endpoint format for model {}: {}",
                model.name,
                model.endpoint
            ));
        }
    }

    // Check scenarios
    if config.test_scenarios.is_empty() {
        return Err(anyhow::anyhow!("No test scenarios configured"));
    }

    for scenario in &config.test_scenarios {
        println!(
            "Scenario: {} - Category: {}",
            scenario.name, scenario.category
        );

        if scenario.prompt.is_empty() {
            return Err(anyhow::anyhow!(
                "Empty prompt for scenario: {}",
                scenario.name
            ));
        }

        if scenario.weight <= 0.0 {
            return Err(anyhow::anyhow!(
                "Invalid weight for scenario {}: {}",
                scenario.name,
                scenario.weight
            ));
        }
    }

    // Check statistical configuration
    if config.statistical_config.confidence_level <= 0.0
        || config.statistical_config.confidence_level >= 1.0
    {
        return Err(anyhow::anyhow!(
            "Invalid confidence level: {}",
            config.statistical_config.confidence_level
        ));
    }

    if config.iterations < config.statistical_config.min_sample_size {
        return Err(anyhow::anyhow!(
            "Iterations ({}) less than minimum sample size ({})",
            config.iterations,
            config.statistical_config.min_sample_size
        ));
    }

    println!("✅ Configuration validation passed!");
    println!(
        "Total test combinations: {}",
        config.models.len() * config.test_scenarios.len()
    );
    println!(
        "Total evaluations: {}",
        config.models.len() * config.test_scenarios.len() * config.iterations
    );

    Ok(())
}

fn print_summary(summary: &splice_weaver_mcp::benchmark_utils::BenchmarkSummary) {
    println!("\n🎯 Benchmark Summary");
    println!("==================");
    println!("Total Evaluations: {}", summary.total_evaluations);
    println!(
        "Total Duration: {:.2}s",
        summary.total_duration_ms as f64 / 1000.0
    );

    println!("\n🏆 Model Rankings:");
    for ranking in &summary.model_rankings {
        let score_emoji = if ranking.overall_score > 0.8 {
            "🟢"
        } else if ranking.overall_score > 0.6 {
            "🟡"
        } else {
            "🔴"
        };

        println!(
            "{} {}. {} - Score: {:.3}, Success: {:.1}%, Avg Time: {:.1}ms",
            score_emoji,
            ranking.rank,
            ranking.model_name,
            ranking.overall_score,
            ranking.success_rate * 100.0,
            ranking.avg_duration_ms
        );
    }

    println!("\n📊 Scenario Difficulty:");
    for difficulty in &summary.scenario_difficulty {
        let difficulty_emoji = if difficulty.difficulty_score < 0.3 {
            "🟢"
        } else if difficulty.difficulty_score < 0.7 {
            "🟡"
        } else {
            "🔴"
        };

        println!(
            "{} {} - Difficulty: {:.2}, Success: {:.1}%, Avg Time: {:.1}ms",
            difficulty_emoji,
            difficulty.scenario_name,
            difficulty.difficulty_score,
            difficulty.avg_success_rate * 100.0,
            difficulty.avg_duration_ms
        );
    }

    println!("\n💡 Recommendations:");
    for recommendation in &summary.recommendations {
        println!("  • {}", recommendation);
    }
}
