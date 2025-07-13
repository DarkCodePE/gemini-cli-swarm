// src/tools/safla_tool.rs
use crate::mcp_client::McpClient;
use crate::tools::{
    async_trait, Tool, ToolCategory, ToolError, ToolParams, ToolResult,
    create_parameters_schema, RiskLevel,
};
use serde_json;

/// Una herramienta para interactuar con el sistema de memoria SAFLA a través de MCP.
pub struct SaflaTool {
    mcp_client: McpClient,
}

impl SaflaTool {
    pub fn new() -> Self {
        // En una implementación real, esto vendría de un archivo de configuración.
        let server_url = "http://localhost:8080"; 
        Self {
            mcp_client: McpClient::new(server_url),
        }
    }
}

#[async_trait]
impl Tool for SaflaTool {
    fn name(&self) -> &str {
        "safla_memory"
    }

    fn description(&self) -> &str {
        "Interactúa con el sistema de memoria a largo plazo de SAFLA. Operaciones: 'store_memory', 'retrieve_memories'."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "operation": {
                    "type": "string",
                    "description": "La operación a realizar: 'store_memory' o 'retrieve_memories'."
                },
                "content": {
                    "type": "string",
                    "description": "El contenido para 'store_memory'."
                },
                "query": {
                    "type": "string",
                    "description": "La consulta para 'retrieve_memories'."
                }
            }),
            vec!["operation"],
        )
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium // Escribe en un sistema externo
    }

    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let operation = params.get::<String>("operation")?;

        let response = self.mcp_client.execute_tool(&operation, &params).await?;

        if response.success {
            Ok(ToolResult::success(response.output, "Operación SAFLA MCP exitosa.".to_string()))
        } else {
            Err(ToolError::ExecutionError(
                response.error.unwrap_or_else(|| "Error desconocido del MCP de SAFLA.".to_string()),
            ))
        }
    }
} 