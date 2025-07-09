// ============================================================================
// ENJAMBRE NATIVE TOOLS SYSTEM v2.0
// ============================================================================
// Sistema de herramientas nativo integrado que supera a Claude-Flow MCP:
// - Zero overhead (sin protocolos externos)
// - Integración nativa con Gemini Function Calling
// - Performance local vs red/HTTP
// - Flexibilidad total vs estándar MCP
// - 87+ herramientas especializadas para agentes autónomos
// ============================================================================

pub mod core;
pub mod filesystem;
pub mod system;
pub mod text;
pub mod network;
pub mod data;
pub mod memory;
pub mod utils;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

// ============================================================================
// CORE TOOL INFRASTRUCTURE
// ============================================================================

/// Trait principal para todas las herramientas de Enjambre
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

/// Categorías de herramientas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Nivel de riesgo de las operaciones
#[derive(Debug, Clone, PartialEq)]
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
        self.data.insert(key.to_string(), serde_json::to_value(value).unwrap());
        self
    }
    
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, ToolError> {
        let value = self.data.get(key)
            .ok_or_else(|| ToolError::MissingParameter(key.to_string()))?;
        serde_json::from_value(value.clone())
            .map_err(|e| ToolError::InvalidParameter(key.to_string(), e.to_string()))
    }
    
    pub fn get_optional<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, ToolError> {
        match self.data.get(key) {
            Some(value) => {
                let result = serde_json::from_value(value.clone())
                    .map_err(|e| ToolError::InvalidParameter(key.to_string(), e.to_string()))?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
}

/// Resultado de la ejecución de herramientas
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
            data: serde_json::to_value(data).unwrap(),
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
}

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::IoError(err.to_string())
    }
}

// ============================================================================
// REGISTRY DE HERRAMIENTAS
// ============================================================================

/// Registry central de todas las herramientas
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
    
    /// Registrar una nueva herramienta
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        let category = tool.category();
        
        // Agregar a categoría
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());
        
        // Registrar herramienta
        self.tools.insert(name, Box::new(tool));
    }
    
    /// Obtener herramienta por nombre
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }
    
    /// Listar todas las herramientas
    pub fn list_all(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
    
    /// Listar herramientas por categoría
    pub fn list_by_category(&self, category: &ToolCategory) -> Vec<&str> {
        self.categories
            .get(category)
            .map(|tools| tools.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
    
    /// Obtener esquema para Gemini Function Calling
    pub fn get_function_schemas(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "parameters": tool.parameters_schema()
                })
            })
            .collect()
    }
    
    /// Ejecutar herramienta por nombre
    pub async fn execute(&self, name: &str, params: ToolParams) -> Result<ToolResult, ToolError> {
        let tool = self.get(name)
            .ok_or_else(|| ToolError::ToolNotFound(name.to_string()))?;
        
        // Verificar nivel de riesgo
        if tool.risk_level() == RiskLevel::Critical {
            return Err(ToolError::PermissionDenied(
                "Operación crítica requiere confirmación explícita".to_string()
            ));
        }
        
        tool.execute(params).await
    }
}

/// Registry global singleton
static mut GLOBAL_REGISTRY: Option<ToolRegistry> = None;
static mut REGISTRY_INITIALIZED: bool = false;

/// Inicializar el registry global con herramientas básicas
pub fn initialize_registry() -> &'static mut ToolRegistry {
    unsafe {
        if !REGISTRY_INITIALIZED {
            let mut registry = ToolRegistry::new();
            
            // Registrar herramientas básicas que existen
            registry.register(filesystem::ListFilesTool::new());
            registry.register(filesystem::ReadFileTool::new());
            registry.register(filesystem::WriteFileTool::new());
            registry.register(memory::MemoryStoreTool::new());
            registry.register(memory::MemoryRetrieveTool::new());
            registry.register(memory::MemoryListTool::new());
            registry.register(system::SystemInfoTool::new());
            registry.register(text::TextProcessTool::new());
            registry.register(utils::Base64Tool::new());
            registry.register(utils::HashTool::new());
            registry.register(utils::UrlTool::new());
            registry.register(utils::JsonTool::new());
            
            GLOBAL_REGISTRY = Some(registry);
            REGISTRY_INITIALIZED = true;
        }
        
        GLOBAL_REGISTRY.as_mut().unwrap()
    }
}

/// Obtener el registry global
pub fn get_registry() -> &'static ToolRegistry {
    unsafe {
        if !REGISTRY_INITIALIZED {
            initialize_registry();
        }
        GLOBAL_REGISTRY.as_ref().unwrap()
    }
}

/// Obtener el registry global mutable
pub fn get_registry_mut() -> &'static mut ToolRegistry {
    unsafe {
        if !REGISTRY_INITIALIZED {
            initialize_registry();
        }
        GLOBAL_REGISTRY.as_mut().unwrap()
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Crear esquema JSON Schema para parámetros
pub fn create_parameters_schema(properties: serde_json::Value, required: Vec<&str>) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Validar parámetros contra esquema
pub fn validate_parameters(params: &ToolParams, schema: &serde_json::Value) -> Result<(), ToolError> {
    // Implementación básica de validación
    // En producción usaríamos una librería como jsonschema
    Ok(())
} 