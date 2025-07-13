// src/tools/ruv_swarm_tool.rs
use crate::mcp_client::McpClient;
use crate::tools::{
    async_trait, Tool, ToolCategory, ToolError, ToolParams, ToolResult,
    create_parameters_schema, RiskLevel,
};
use serde_json;

/// Una herramienta para delegar tareas complejas al orquestador `ruv-swarm` vía MCP.
pub struct RuvSwarmTool {
    mcp_client: McpClient,
}

impl RuvSwarmTool {
    pub fn new() -> Self {
        // En una implementación real, esto vendría de un archivo de configuración.
        let server_url = "http://localhost:8081"; // Puerto diferente para ruv-swarm
        Self {
            mcp_client: McpClient::new(server_url),
        }
    }
}

#[async_trait]
impl Tool for RuvSwarmTool {
    fn name(&self) -> &str {
        "ruv_swarm_orchestrate"
    }

    fn description(&self) -> &str {
        "Delega un objetivo complejo a un enjambre de agentes especializados para su ejecución."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "objective": {
                    "type": "string",
                    "description": "El objetivo de alto nivel a ser ejecutado por el enjambre."
                },
                 "context": {
                    "type": "string",
                    "description": "Cualquier contexto o datos adicionales requeridos para la tarea."
                }
            }),
            vec!["objective"],
        )
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::AI
    }
    
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High // Coordina la ejecución de otras herramientas
    }

    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        // La documentación indica que la operación principal es 'task_orchestrate'
        let response = self.mcp_client.execute_tool("task_orchestrate", &params).await?;

        if response.success {
            Ok(ToolResult::success(response.output, "Orquestación de ruv-swarm completada.".to_string()))
        } else {
            Err(ToolError::ExecutionError(
                response.error.unwrap_or_else(|| "Error desconocido del MCP de ruv-swarm.".to_string()),
            ))
        }
    }
} 