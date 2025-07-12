use anyhow::Result;
use clap::{Arg, Command};
use mcp_ast_grep::evaluation_client::{
    create_default_test_cases, EvaluationClient, EvaluationClientConfig, EvaluationSuite,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("evaluation-client")
        .version("0.1.0")
        .about("Rust MCP Evaluation Client for ast-grep server")
        .arg(
            Arg::new("endpoint")
                .long("endpoint")
                .value_name("URL")
                .help("LLM API endpoint (OpenAI compatible)")
                .default_value("http://localhost:1234/v1"),
        )
        .arg(
            Arg::new("api-key")
                .long("api-key")
                .value_name("KEY")
                .help("API key for LLM endpoint (optional for local endpoints)"),
        )
        .arg(
            Arg::new("model")
                .long("model")
                .value_name("MODEL")
                .help("Model name to use")
                .default_value("gpt-3.5-turbo"),
        )
        .arg(
            Arg::new("server-cmd")
                .long("server-cmd")
                .value_name("COMMAND")
                .help("Command to start MCP server")
                .default_value("cargo"),
        )
        .arg(
            Arg::new("server-args")
                .long("server-args")
                .value_name("ARGS")
                .help("Arguments for MCP server command")
                .default_value("run --bin mcp-ast-grep"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .short('i')
                .help("Run in interactive mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("run-tests")
                .long("run-tests")
                .help("Run built-in test suite")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("prompt")
                .long("prompt")
                .value_name("TEXT")
                .help("Single prompt to evaluate"),
        )
        .get_matches();

    let config = EvaluationClientConfig {
        llm_endpoint: matches.get_one::<String>("endpoint").unwrap().clone(),
        llm_api_key: matches.get_one::<String>("api-key").cloned(),
        model_name: matches.get_one::<String>("model").unwrap().clone(),
        server_command: matches.get_one::<String>("server-cmd").unwrap().clone(),
        server_args: matches
            .get_one::<String>("server-args")
            .unwrap()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
        timeout_seconds: 30,
    };

    info!("Starting Rust MCP Evaluation Client");
    info!("LLM Endpoint: {}", config.llm_endpoint);
    info!("Model: {}", config.model_name);
    info!("Server Command: {} {:?}", config.server_command, config.server_args);

    if matches.get_flag("run-tests") {
        run_test_suite(config).await?;
    } else if let Some(prompt) = matches.get_one::<String>("prompt") {
        run_single_prompt(config, prompt).await?;
    } else if matches.get_flag("interactive") {
        run_interactive_mode(config).await?;
    } else {
        println!("Use --help to see available options");
        println!("Examples:");
        println!("  {} --run-tests", env!("CARGO_PKG_NAME"));
        println!("  {} --interactive", env!("CARGO_PKG_NAME"));
        println!("  {} --prompt \"Find all functions in this code: function test() {{}}\"", env!("CARGO_PKG_NAME"));
    }

    Ok(())
}

async fn run_test_suite(config: EvaluationClientConfig) -> Result<()> {
    info!("Running built-in test suite");
    
    let mut suite = EvaluationSuite::new(config);
    suite.initialize().await?;
    
    for test_case in create_default_test_cases() {
        suite.add_test_case(test_case);
    }
    
    let results = suite.run_evaluations().await?;
    
    println!("\n=== Evaluation Results ===");
    for (test_case, result) in results {
        let success = (test_case.success_criteria)(&result);
        println!("Test: {}", test_case.name);
        println!("Status: {}", if success { "✅ PASS" } else { "❌ FAIL" });
        println!("Duration: {}ms", result.duration_ms);
        println!("Tool calls: {}", result.tool_calls_made);
        println!("Response: {}\n", result.response);
    }
    
    Ok(())
}

async fn run_single_prompt(config: EvaluationClientConfig, prompt: &str) -> Result<()> {
    info!("Evaluating single prompt");
    
    let mut client = EvaluationClient::new(config);
    client.connect_to_mcp_server().await?;
    
    let result = client.evaluate_prompt(prompt).await?;
    
    println!("Prompt: {}", prompt);
    println!("Response: {}", result.response);
    println!("Duration: {}ms", result.duration_ms);
    println!("Tool calls made: {}", result.tool_calls_made);
    
    Ok(())
}

async fn run_interactive_mode(config: EvaluationClientConfig) -> Result<()> {
    info!("Starting interactive mode");
    
    let mut client = EvaluationClient::new(config);
    client.connect_to_mcp_server().await?;
    
    println!("🤖 MCP Evaluation Client Interactive Mode");
    println!("Connected to ast-grep MCP server");
    println!("Type your prompts below (type 'quit' to exit, 'reset' to clear conversation):\n");
    
    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("Goodbye! 👋");
            break;
        }
        
        if input == "reset" {
            client.reset_conversation();
            println!("🔄 Conversation reset\n");
            continue;
        }
        
        if input == "tools" {
            match client.get_available_tools().await {
                Ok(tools) => {
                    println!("Available tools:");
                    for tool in tools {
                        println!("  - {}: {}", tool.name, tool.description);
                    }
                    println!();
                }
                Err(e) => {
                    error!("Failed to get tools: {}", e);
                }
            }
            continue;
        }
        
        match client.chat_with_llm(input).await {
            Ok(response) => {
                println!("🤖: {}\n", response);
            }
            Err(e) => {
                error!("Error: {}\n", e);
            }
        }
    }
    
    Ok(())
}