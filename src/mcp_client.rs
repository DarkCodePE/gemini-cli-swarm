// src/mcp_client.rs
use crate::tools::{ToolError, ToolParams};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct McpRequest<'a> {
    tool_name: &'a str,
    arguments: &'a ToolParams,
}

#[derive(Deserialize, Debug)]
pub struct McpResponse {
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct McpClient {
    base_url: String,
    client: Client,
}

impl McpClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            base_url: server_url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: &ToolParams,
    ) -> Result<McpResponse, ToolError> {
        let request_url = format!("{}/execute_tool", self.base_url);
        let payload = McpRequest {
            tool_name,
            arguments: params,
        };

        let response = self
            .client
            .post(&request_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let err_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Error ilegible del servidor MCP".to_string());
            return Err(ToolError::ExecutionError(format!(
                "El servidor MCP respondió con error: {}",
                err_text
            )));
        }

        response
            .json::<McpResponse>()
            .await
            .map_err(|e| ToolError::InvalidResponse(e.to_string()))
    }
} 