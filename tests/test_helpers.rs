use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use tracing::{debug, info};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Helper struct for managing MCP server test instances
pub struct McpServerHandle {
    pub process: Child,
    pub initialized: bool,
}

impl McpServerHandle {
    /// Start a new MCP server instance
    pub async fn start() -> Result<Self> {
        info!("Starting MCP server for testing");

        let process = Command::new("cargo")
            .args(&["run", "--bin", "mcp-ast-grep"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Give the server a moment to start
        sleep(Duration::from_millis(500)).await;

        Ok(Self {
            process,
            initialized: false,
        })
    }

    /// Initialize the MCP server
    pub async fn initialize(&mut self) -> Result<Value> {
        if self.initialized {
            return Err(anyhow::anyhow!("Server already initialized"));
        }

        let stdin = self.process.stdin.as_mut().unwrap();
        let stdout = self.process.stdout.as_mut().unwrap();
        let mut reader = BufReader::new(stdout);

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        stdin
            .write_all(format!("{}\n", init_request).as_bytes())
            .await?;
        stdin.flush().await?;

        let mut response_line = String::new();
        timeout(DEFAULT_TIMEOUT, reader.read_line(&mut response_line)).await??;

        let response: Value = serde_json::from_str(&response_line)?;

        if response.get("error").is_some() {
            return Err(anyhow::anyhow!(
                "Server initialization failed: {}",
                response
            ));
        }

        self.initialized = true;
        debug!("MCP server initialized successfully");
        Ok(response)
    }

    /// Send a JSON-RPC request to the server
    pub async fn send_request(&mut self, request: Value) -> Result<Value> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Server not initialized"));
        }

        let stdin = self.process.stdin.as_mut().unwrap();
        let stdout = self.process.stdout.as_mut().unwrap();
        let mut reader = BufReader::new(stdout);

        stdin.write_all(format!("{}\n", request).as_bytes()).await?;
        stdin.flush().await?;

        let mut response_line = String::new();
        timeout(DEFAULT_TIMEOUT, reader.read_line(&mut response_line)).await??;

        let response: Value = serde_json::from_str(&response_line)?;
        debug!("Server response: {}", response);
        Ok(response)
    }

    /// List available tools
    pub async fn list_tools(&mut self) -> Result<Vec<Value>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let response = self.send_request(request).await?;

        let result = response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in tools/list response"))?;

        let tools = result
            .get("tools")
            .ok_or_else(|| anyhow::anyhow!("No tools in result"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Tools is not an array"))?;

        Ok(tools.clone())
    }

    /// Call a tool
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        self.send_request(request).await
    }

    /// List resources
    pub async fn list_resources(&mut self) -> Result<Vec<Value>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/list",
            "params": {}
        });

        let response = self.send_request(request).await?;

        let result = response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in resources/list response"))?;

        let resources = result
            .get("resources")
            .ok_or_else(|| anyhow::anyhow!("No resources in result"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Resources is not an array"))?;

        Ok(resources.clone())
    }

    /// Read a resource
    pub async fn read_resource(&mut self, uri: &str) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": uri
            }
        });

        self.send_request(request).await
    }

    /// List prompts
    pub async fn list_prompts(&mut self) -> Result<Vec<Value>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/list",
            "params": {}
        });

        let response = self.send_request(request).await?;

        let result = response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result in prompts/list response"))?;

        let prompts = result
            .get("prompts")
            .ok_or_else(|| anyhow::anyhow!("No prompts in result"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Prompts is not an array"))?;

        Ok(prompts.clone())
    }

    /// Shutdown the server
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down MCP server");
        self.process.kill().await?;
        Ok(())
    }
}

/// Helper to create temporary test files
pub struct TestFile {
    pub temp_file: tempfile::NamedTempFile,
    pub path: String,
}

impl TestFile {
    /// Create a new test file with the given content
    pub fn new(content: &str) -> Result<Self> {
        use std::io::Write;

        let mut temp_file = tempfile::NamedTempFile::new()?;
        write!(temp_file, "{}", content)?;

        let path = temp_file.path().to_string_lossy().to_string();

        Ok(Self { temp_file, path })
    }

    /// Create a JavaScript test file
    pub fn javascript(content: &str) -> Result<Self> {
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".js")?;
        use std::io::Write;
        write!(temp_file, "{}", content)?;

        let path = temp_file.path().to_string_lossy().to_string();

        Ok(Self { temp_file, path })
    }

    /// Create a Python test file
    pub fn python(content: &str) -> Result<Self> {
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".py")?;
        use std::io::Write;
        write!(temp_file, "{}", content)?;

        let path = temp_file.path().to_string_lossy().to_string();

        Ok(Self { temp_file, path })
    }

    /// Create a Rust test file
    pub fn rust(content: &str) -> Result<Self> {
        let mut temp_file = tempfile::NamedTempFile::with_suffix(".rs")?;
        use std::io::Write;
        write!(temp_file, "{}", content)?;

        let path = temp_file.path().to_string_lossy().to_string();

        Ok(Self { temp_file, path })
    }
}

/// Sample code snippets for testing
pub mod sample_code {
    pub const JAVASCRIPT_FUNCTIONS: &str = r#"
function hello(name) {
    console.log("Hello, " + name);
    return "Hello, " + name;
}

function goodbye(name) {
    console.log("Goodbye, " + name);
    return "Goodbye, " + name;
}

const arrow = (x) => x * 2;

class Calculator {
    add(a, b) {
        return a + b;
    }
    
    multiply(a, b) {
        return a * b;
    }
}
"#;

    pub const PYTHON_FUNCTIONS: &str = r#"
def hello(name):
    print(f"Hello, {name}")
    return f"Hello, {name}"

def goodbye(name):
    print(f"Goodbye, {name}")
    return f"Goodbye, {name}"

class Calculator:
    def add(self, a, b):
        return a + b
    
    def multiply(self, a, b):
        return a * b

lambda_func = lambda x: x * 2
"#;

    pub const RUST_FUNCTIONS: &str = r#"
fn hello(name: &str) -> String {
    println!("Hello, {}", name);
    format!("Hello, {}", name)
}

fn goodbye(name: &str) -> String {
    println!("Goodbye, {}", name);
    format!("Goodbye, {}", name)
}

struct Calculator;

impl Calculator {
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    
    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

fn main() {
    let calc = Calculator;
    println!("{}", calc.add(2, 3));
}
"#;
}

/// Common test assertions
pub mod assertions {
    use serde_json::Value;

    /// Assert that a JSON-RPC response is successful
    pub fn assert_success(response: &Value) {
        assert!(
            response.get("error").is_none(),
            "Response should not contain error: {}",
            response
        );
        assert!(
            response.get("result").is_some(),
            "Response should contain result: {}",
            response
        );
    }

    /// Assert that a JSON-RPC response contains an error
    pub fn assert_error(response: &Value) {
        assert!(
            response.get("error").is_some(),
            "Response should contain error: {}",
            response
        );
    }

    /// Assert that tools list contains expected tools
    pub fn assert_has_tools(tools: &[Value], expected: &[&str]) {
        let tool_names: Vec<String> = tools
            .iter()
            .filter_map(|t| t.get("name")?.as_str().map(|s| s.to_string()))
            .collect();

        for expected_tool in expected {
            assert!(
                tool_names.contains(&expected_tool.to_string()),
                "Expected tool '{}' not found in: {:?}",
                expected_tool,
                tool_names
            );
        }
    }

    /// Assert that a tool has required schema properties
    pub fn assert_tool_schema(tool: &Value, required_props: &[&str]) {
        let schema = tool
            .get("inputSchema")
            .or_else(|| tool.get("input_schema"))
            .expect("Tool should have input schema");

        let properties = schema
            .get("properties")
            .expect("Schema should have properties")
            .as_object()
            .expect("Properties should be an object");

        for prop in required_props {
            assert!(
                properties.contains_key(*prop),
                "Tool schema should contain property '{}': {}",
                prop,
                tool
            );
        }
    }
}

/// Test configuration helpers
pub mod config {
    use splice_weaver_mcp::evaluation_client::EvaluationClientConfig;

    /// Create a test configuration for evaluation client
    pub fn test_evaluation_config() -> EvaluationClientConfig {
        EvaluationClientConfig {
            llm_endpoint: "http://httpbin.org/status/200".to_string(),
            llm_api_key: None,
            model_name: "test-model".to_string(),
            server_command: "echo".to_string(),
            server_args: vec!["test".to_string()],
            timeout_seconds: 5,
        }
    }

    /// Create a configuration for testing with real MCP server
    pub fn real_mcp_config() -> EvaluationClientConfig {
        EvaluationClientConfig {
            llm_endpoint: "http://localhost:1234/v1".to_string(),
            llm_api_key: None,
            model_name: "test-model".to_string(),
            server_command: "cargo".to_string(),
            server_args: vec![
                "run".to_string(),
                "--bin".to_string(),
                "mcp-ast-grep".to_string(),
            ],
            timeout_seconds: 10,
        }
    }
}
