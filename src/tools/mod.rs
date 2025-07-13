// ============================================================================
// TOOLS MODULE - Sistema de Herramientas Nativas para Enjambre
// ============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Módulos de herramientas
pub mod core;
pub mod filesystem;
pub mod system;
pub mod text;
pub mod network;
pub mod data;
pub mod memory;
pub mod safla_tool;
pub mod ruv_swarm_tool;
pub mod utils;

// ============================================================================
// TRAIT PRINCIPAL: Tool
// ============================================================================

/// Trait que define la interfaz común para todas las herramientas
#[async_trait]
pub trait Tool: Send + Sync {
    /// Nombre único de la herramienta
    fn name(&self) -> &str;
    
    /// Descripción para function calling
    fn description(&self) -> &str;
    
    /// Esquema de parámetros para Gemini
    fn parameters_schema(&self) -> serde_json::Value;
    
    /// Categoría de la herramienta
    fn category(&self) -> ToolCategory;
    
    /// Ejecutar la herramienta con parámetros
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError>;
    
    /// Si requiere confirmación del usuario
    fn requires_confirmation(&self) -> bool {
        false
    }
    
    /// Nivel de riesgo de la operación
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }
}

// ============================================================================
// ENUMS Y ESTRUCTURAS
// ============================================================================

/// Categorías de herramientas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    FileSystem,
    System,
    Network,
    Text,
    Data,
    Memory,
    Utils,
    Security,
    Development,
    AI,
}

/// Niveles de riesgo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,     // Operaciones de lectura
    Medium,  // Operaciones de escritura
    High,    // Operaciones del sistema
    Critical, // Operaciones destructivas
}

/// Parámetros de entrada para herramientas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub data: HashMap<String, serde_json::Value>,
}

impl ToolParams {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn insert<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.data.insert(key.to_string(), serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
        self
    }
    
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, ToolError> {
        self.data.get(key)
            .ok_or_else(|| ToolError::MissingParameter(key.to_string()))
            .and_then(|v| serde_json::from_value(v.clone()).map_err(|e| ToolError::InvalidParameter(key.to_string(), e.to_string())))
    }
    
    pub fn get_optional<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, ToolError> {
        match self.data.get(key) {
            Some(v) if v.is_null() => Ok(None),
            Some(v) => serde_json::from_value(v.clone())
                .map(Some)
                .map_err(|e| ToolError::InvalidParameter(key.to_string(), e.to_string())),
            None => Ok(None),
        }
    }
}

/// Resultado de ejecución de herramientas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ToolResult {
    pub fn success<T: Serialize>(data: T, message: String) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            message,
            metadata: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            message,
            metadata: None,
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Errores del sistema de herramientas
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Parámetro requerido faltante: {0}")]
    MissingParameter(String),
    
    #[error("Parámetro inválido '{0}': {1}")]
    InvalidParameter(String, String),
    
    #[error("Herramienta no encontrada: {0}")]
    ToolNotFound(String),
    
    #[error("Operación no permitida: {0}")]
    PermissionDenied(String),
    
    #[error("Error de E/O: {0}")]
    IoError(String),
    
    #[error("Error de red: {0}")]
    NetworkError(String),
    
    #[error("Error de validación: {0}")]
    ValidationError(String),
    
    #[error("Error interno: {0}")]
    InternalError(String),
    
    #[error("Error de ejecución: {0}")]
    ExecutionError(String),
    
    #[error("Respuesta inválida: {0}")]
    InvalidResponse(String),
}

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::IoError(err.to_string())
    }
}

// ============================================================================
// REGISTRY DE HERRAMIENTAS
// ============================================================================

/// Registro global de herramientas
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    categories: HashMap<ToolCategory, Vec<String>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
        }
    }
    
    /// Registra una nueva herramienta
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        let category = tool.category();
        
        // Agregar a categoría
        self.categories.entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());
        
        // Registrar herramienta
        self.tools.insert(name, Box::new(tool));
    }
    
    /// Obtiene una herramienta por nombre
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }
    
    /// Lista todas las herramientas
    pub fn list_all(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
    
    /// Lista herramientas por categoría
    pub fn list_by_category(&self, category: &ToolCategory) -> Vec<&str> {
        self.categories.get(category)
            .map(|tools| tools.iter().map(|s| s.as_str()).collect())
            .unwrap_or_else(Vec::new)
    }
    
    /// Obtiene esquemas de función para Gemini
    pub fn get_function_schemas(&self) -> Vec<serde_json::Value> {
        self.tools.values().map(|tool| {
            serde_json::json!({
                "name": tool.name(),
                "description": tool.description(),
                "parameters": tool.parameters_schema()
            })
        }).collect()
    }
    
    /// Ejecuta una herramienta
    pub async fn execute(&self, name: &str, params: ToolParams) -> Result<ToolResult, ToolError> {
        let tool = self.get(name)
            .ok_or_else(|| ToolError::ToolNotFound(name.to_string()))?;
        
        // Verificar si requiere confirmación
        if tool.requires_confirmation() {
            // TODO: Implementar sistema de confirmación
            println!("⚠️  La herramienta '{}' requiere confirmación del usuario", name);
        }
        
        // Ejecutar
        tool.execute(params).await
    }
}

// ============================================================================
// INSTANCIA GLOBAL SIMPLIFICADA
// ============================================================================

/// Inicializa el registro global de herramientas
pub fn initialize_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    // Registrar herramientas de filesystem
    registry.register(filesystem::ListFilesTool::new());
    registry.register(filesystem::ReadFileTool::new());
    registry.register(filesystem::WriteFileTool::new());
    
    // Registrar herramientas de memoria
    registry.register(memory::MemoryStoreTool::new());
    registry.register(memory::MemoryRetrieveTool::new());
    registry.register(memory::MemoryListTool::new());
    
    // Registrar herramientas de utilidades
    registry.register(utils::Base64Tool::new());
    registry.register(utils::HashTool::new());
    registry.register(utils::UrlTool::new());
    registry.register(utils::JsonTool::new());
    
    // Registrar herramientas de AI
    registry.register(safla_tool::SaflaTool::new());
    registry.register(ruv_swarm_tool::RuvSwarmTool::new());
    
    registry
}

/// Obtiene un registro inicializado
pub fn get_registry() -> ToolRegistry {
    initialize_registry()
}

/// Obtiene un registro inicializado mutable
pub fn get_registry_mut() -> ToolRegistry {
    initialize_registry()
}

// ============================================================================
// UTILIDADES
// ============================================================================

/// Crea un esquema de parámetros estándar
pub fn create_parameters_schema(properties: serde_json::Value, required: Vec<&str>) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Valida parámetros contra un esquema
pub fn validate_parameters(_params: &ToolParams, _schema: &serde_json::Value) -> Result<(), ToolError> {
    // TODO: Implementar validación real usando jsonschema
    Ok(())
} 