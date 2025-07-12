use anyhow::Result;
use rmcp::model::*;
use serde_json::Value;

pub struct ResourceProvider {
    // TODO: Add actual resource providers when implementing features
}

impl ResourceProvider {
    pub fn new() -> Self {
        Self {
            // TODO: Initialize resource providers
        }
    }

    pub fn list_resources(&self) -> Vec<Resource> {
        // TODO: Return actual resources when implemented
        vec![]
    }

    pub fn get_resource(&self, uri: &str) -> Result<Value> {
        // TODO: Implement resource retrieval
        Err(anyhow::anyhow!("Resource not implemented: {}", uri))
    }
}