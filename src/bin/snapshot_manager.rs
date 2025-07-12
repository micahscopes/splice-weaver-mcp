use anyhow::Result;
use clap::{Arg, Command};
use splice_weaver_mcp::evaluation_client::{EvaluationClient, EvaluationClientConfig, TestCase};
use splice_weaver_mcp::snapshot_utils::{cleanup_old_snapshots, ExportFormat, SnapshotManager};
use std::fs;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("snapshot-manager")
        .version("0.1.0")
        .about("Manage LLM response snapshots for regression testing")
        .subcommand(
            Command::new("capture")
                .about("Capture new snapshots")
                .arg(
                    Arg::new("endpoint")
                        .long("endpoint")
                        .value_name("URL")
                        .help("LLM API endpoint")
                        .default_value("http://localhost:1234/v1"),
                )
                .arg(
                    Arg::new("model")
                        .long("model")
                        .value_name("MODEL")
                        .help("Model name")
                        .default_value("test-model"),
                )
                .arg(
                    Arg::new("test-name")
                        .long("test-name")
                        .value_name("NAME")
                        .help("Specific test to run"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .value_name("DIR")
                        .help("Output directory for snapshots")
                        .default_value("tests/snapshots"),
                ),
        )
        .subcommand(
            Command::new("compare")
                .about("Compare snapshots for regressions")
                .arg(
                    Arg::new("baseline-days")
                        .long("baseline-days")
                        .value_name("DAYS")
                        .help("Days to look back for baseline")
                        .default_value("7"),
                )
                .arg(
                    Arg::new("snapshots-dir")
                        .long("snapshots-dir")
                        .value_name("DIR")
                        .help("Directory containing snapshots")
                        .default_value("tests/snapshots"),
                ),
        )
        .subcommand(
            Command::new("summary")
                .about("Generate summary report")
                .arg(
                    Arg::new("snapshots-dir")
                        .long("snapshots-dir")
                        .value_name("DIR")
                        .help("Directory containing snapshots")
                        .default_value("tests/snapshots"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export snapshots in different formats")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_name("FORMAT")
                        .help("Export format (json, csv, summary)")
                        .default_value("json"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .required(true),
                )
                .arg(
                    Arg::new("snapshots-dir")
                        .long("snapshots-dir")
                        .value_name("DIR")
                        .help("Directory containing snapshots")
                        .default_value("tests/snapshots"),
                ),
        )
        .subcommand(
            Command::new("cleanup")
                .about("Clean up old snapshots")
                .arg(
                    Arg::new("days")
                        .long("days")
                        .value_name("DAYS")
                        .help("Keep snapshots newer than this many days")
                        .default_value("30"),
                )
                .arg(
                    Arg::new("snapshots-dir")
                        .long("snapshots-dir")
                        .value_name("DIR")
                        .help("Directory containing snapshots")
                        .default_value("tests/snapshots"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("capture", sub_matches)) => {
            capture_snapshots(sub_matches).await?;
        }
        Some(("compare", sub_matches)) => {
            compare_snapshots(sub_matches).await?;
        }
        Some(("summary", sub_matches)) => {
            generate_summary(sub_matches).await?;
        }
        Some(("export", sub_matches)) => {
            export_snapshots(sub_matches).await?;
        }
        Some(("cleanup", sub_matches)) => {
            cleanup_snapshots(sub_matches).await?;
        }
        _ => {
            println!("Use --help to see available commands");
            println!("Examples:");
            println!("  snapshot-manager capture --model gpt-4");
            println!("  snapshot-manager compare --baseline-days 7");
            println!("  snapshot-manager summary");
            println!("  snapshot-manager export --format csv --output snapshots.csv");
        }
    }

    Ok(())
}

async fn capture_snapshots(matches: &clap::ArgMatches) -> Result<()> {
    info!("Capturing new snapshots");

    let config = EvaluationClientConfig {
        llm_endpoint: matches.get_one::<String>("endpoint").unwrap().clone(),
        llm_api_key: None,
        model_name: matches.get_one::<String>("model").unwrap().clone(),
        server_command: "cargo".to_string(),
        server_args: vec!["run".to_string()],
        timeout_seconds: 30,
    };

    let output_dir = matches.get_one::<String>("output").unwrap();
    fs::create_dir_all(output_dir)?;

    let mut client = EvaluationClient::new(config.clone());

    // Create test cases
    let test_cases = if let Some(test_name) = matches.get_one::<String>("test-name") {
        vec![TestCase {
            name: test_name.clone(),
            prompt: format!("Test case: {}", test_name),
            expected_tools: vec![],
            success_criteria: |_| true,
        }]
    } else {
        splice_weaver_mcp::evaluation_client::create_default_test_cases()
    };

    for test_case in test_cases {
        info!("Capturing snapshot for: {}", test_case.name);

        match client.evaluate_prompt(&test_case.prompt).await {
            Ok(result) => {
                let analysis = client.analyze_response(&result.response, &result);

                let metadata = splice_weaver_mcp::evaluation_client::SnapshotMetadata {
                    test_name: test_case.name.clone(),
                    model_name: config.model_name.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    git_commit: get_git_commit(),
                    prompt_hash: format!("{:x}", hash_string(&test_case.prompt)),
                };

                let snapshot = splice_weaver_mcp::evaluation_client::ResponseSnapshot {
                    metadata,
                    evaluation_result: result,
                    response_analysis: analysis,
                };

                // Save snapshot
                let filename = format!("{}/{}.yaml", output_dir, test_case.name.replace(" ", "_"));
                let yaml_content = serde_yaml::to_string(&snapshot)?;
                fs::write(&filename, yaml_content)?;

                info!("Saved snapshot: {}", filename);
            }
            Err(e) => {
                error!("Failed to capture snapshot for {}: {}", test_case.name, e);
            }
        }

        client.reset_conversation();
    }

    info!("Snapshot capture completed");
    Ok(())
}

async fn compare_snapshots(matches: &clap::ArgMatches) -> Result<()> {
    info!("Comparing snapshots for regressions");

    let snapshots_dir = matches.get_one::<String>("snapshots-dir").unwrap();
    let baseline_days: u64 = matches
        .get_one::<String>("baseline-days")
        .unwrap()
        .parse()
        .unwrap_or(7);

    let manager = SnapshotManager::new(snapshots_dir);
    let regressions = manager.find_regressions(baseline_days)?;

    if regressions.is_empty() {
        println!("✅ No regressions detected!");
    } else {
        println!("⚠️  Found {} potential regressions:", regressions.len());

        for (i, regression) in regressions.iter().enumerate() {
            println!(
                "\n{}. Test: {}",
                i + 1,
                regression.previous.metadata.test_name
            );
            println!("   Similarity Score: {:.2}", regression.similarity_score);
            println!("   Differences: {}", regression.differences.len());

            for diff in &regression.differences {
                println!(
                    "     - {}: {} -> {}",
                    diff.field,
                    truncate_string(&diff.previous_value, 50),
                    truncate_string(&diff.current_value, 50)
                );
            }
        }
    }

    Ok(())
}

async fn generate_summary(matches: &clap::ArgMatches) -> Result<()> {
    info!("Generating summary report");

    let snapshots_dir = matches.get_one::<String>("snapshots-dir").unwrap();
    let manager = SnapshotManager::new(snapshots_dir);
    let summary = manager.generate_summary()?;

    println!("📊 Snapshot Summary Report");
    println!("========================");
    println!("Total Snapshots: {}", summary.total_snapshots);
    println!("Average Response Time: {:.2}ms", summary.avg_response_time);
    println!("Success Rate: {:.1}%", summary.success_rate * 100.0);

    println!("\n🧪 Test Coverage:");
    for (test_name, count) in &summary.test_coverage {
        println!("  {}: {} snapshots", test_name, count);
    }

    println!("\n🤖 Model Coverage:");
    for (model_name, count) in &summary.model_coverage {
        println!("  {}: {} snapshots", model_name, count);
    }

    println!("\n🔧 Most Used Tools:");
    for (tool_name, count) in &summary.most_common_tools {
        println!("  {}: {} calls", tool_name, count);
    }

    Ok(())
}

async fn export_snapshots(matches: &clap::ArgMatches) -> Result<()> {
    info!("Exporting snapshots");

    let snapshots_dir = matches.get_one::<String>("snapshots-dir").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let format_str = matches.get_one::<String>("format").unwrap();

    let format = match format_str.as_str() {
        "json" => ExportFormat::Json,
        "csv" => ExportFormat::Csv,
        "summary" => ExportFormat::Summary,
        _ => {
            error!("Unknown format: {}", format_str);
            return Ok(());
        }
    };

    let manager = SnapshotManager::new(snapshots_dir);
    manager.export_snapshots(format, output_file)?;

    info!("Exported snapshots to: {}", output_file);
    Ok(())
}

async fn cleanup_snapshots(matches: &clap::ArgMatches) -> Result<()> {
    info!("Cleaning up old snapshots");

    let snapshots_dir = matches.get_one::<String>("snapshots-dir").unwrap();
    let days: u64 = matches
        .get_one::<String>("days")
        .unwrap()
        .parse()
        .unwrap_or(30);

    let removed_count = cleanup_old_snapshots(snapshots_dir, days)?;

    info!(
        "Cleanup completed. Would remove {} old snapshots",
        removed_count
    );
    println!("Note: Actual file removal not implemented in this demo");

    Ok(())
}

fn get_git_commit() -> Option<String> {
    std::process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
