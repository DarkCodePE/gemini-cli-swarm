// ============================================================================
// ENJAMBRE LIB v2.0 - Sistema de Agentes Autónomos con Herramientas Nativas
// ============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

// ============================================================================
// TRAIT UNIVERSAL: CodeGenerationFlow
// ============================================================================
// Este es el "contrato" que cualquier adaptador de LLM debe cumplir.
// Siguiendo el patrón SAFLA, permite intercambiar diferentes LLMs sin
// cambiar el código principal del swarm.

#[async_trait]
pub trait CodeGenerationFlow: Send + Sync {
    /// Ejecuta el flujo completo: Generar -> Verificar -> Refinar
    async fn execute(&self, problem_description: &str) -> Result<CodeGenerationResult, FlowError>;
    
    /// Verifica si el código generado cumple con los criterios de calidad
    fn verify_code(&self, code: &str) -> VerificationResult;
    
    /// Obtiene información sobre las capacidades del adaptador
    fn get_capabilities(&self) -> AdapterCapabilities;
}

// ============================================================================
// NUEVO TRAIT: ThinkingFlow para modelos con capacidades de razonamiento
// ============================================================================

#[async_trait]
pub trait ThinkingFlow: CodeGenerationFlow {
    /// Ejecuta con razonamiento visible paso a paso
    async fn execute_with_thinking(&self, problem: &str) -> Result<ThinkingResult, FlowError>;
    
    /// Obtiene los pasos de razonamiento del último análisis
    fn get_reasoning_steps(&self) -> Vec<ReasoningStep>;
    
    /// Configura el modo de pensamiento
    fn set_thinking_mode(&mut self, mode: ThinkingMode);
}

// ============================================================================
// ESTRUCTURAS DE DATOS COMUNES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationResult {
    pub code: String,
    pub language: String,
    pub confidence_score: f64,
    pub attempts_made: u32,
    pub execution_time_ms: u64,
    pub verification_passed: bool,
    pub cost_estimate: Option<CostEstimate>,
    pub model_used: Option<String>,
    pub metrics: CodeGenerationMetrics,
}

// ============================================================================
// NUEVAS ESTRUCTURAS PARA THINKING Y PERFORMANCE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeGenerationMetrics {
    pub cyclomatic_complexity: u32,
    pub lines_of_code: u32,
    pub token_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingResult {
    pub reasoning_trace: Vec<String>,
    pub intermediate_conclusions: Vec<String>,
    pub final_result: CodeGenerationResult,
    pub confidence_evolution: Vec<f64>,
    pub thinking_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_number: usize,
    pub description: String,
    pub confidence: f64,
    pub intermediate_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinkingMode {
    Standard,
    Extended { max_thinking_time_ms: u64 },
    StepByStep { show_intermediate: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub estimated_cost_usd: f64,
    pub model_used: String,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub compilation_success: bool,
    pub tests_passed: bool,
    pub quality_score: f64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    pub name: String,
    pub version: String,
    pub supported_languages: Vec<String>,
    pub max_context_tokens: u32,
    pub supports_function_calling: bool,
    pub supports_code_execution: bool,
    pub supports_thinking: bool,
    pub cost_per_million_input: f64,
    pub cost_per_million_output: f64,
}

// ============================================================================
// MANEJO DE ERRORES
// ============================================================================

#[derive(Debug)]
pub enum FlowError {
    ApiError(String),
    CompilationError(String),
    TimeoutError,
    InvalidPrompt(String),
    NetworkError(String),
    MaxAttemptsReached(u32),
    CostLimitExceeded(f64),
    ThinkingModeNotSupported,
    AdapterNotFound(String),
    InvalidResponse(String),
}

impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowError::ApiError(msg) => write!(f, "Error de API: {}", msg),
            FlowError::CompilationError(msg) => write!(f, "Error de compilación: {}", msg),
            FlowError::TimeoutError => write!(f, "Timeout alcanzado"),
            FlowError::InvalidPrompt(msg) => write!(f, "Prompt inválido: {}", msg),
            FlowError::NetworkError(msg) => write!(f, "Error de red: {}", msg),
            FlowError::MaxAttemptsReached(attempts) => {
                write!(f, "Máximo de intentos alcanzado: {}", attempts)
            }
            FlowError::CostLimitExceeded(limit) => {
                write!(f, "Límite de costo excedido: ${:.4}", limit)
            }
            FlowError::ThinkingModeNotSupported => {
                write!(f, "Modo thinking no soportado por este modelo")
            }
            FlowError::AdapterNotFound(adapter_name) => {
                write!(f, "Adaptador no encontrado: {}", adapter_name)
            }
            FlowError::InvalidResponse(msg) => {
                write!(f, "Respuesta inválida de la IA: {}", msg)
            }
        }
    }
}

impl Error for FlowError {}

// ============================================================================
// MÓDULOS PÚBLICOS
// ============================================================================

pub mod adapters;
pub mod neuro_divergent;
pub mod swarm;
pub mod tools;  // ✨ NUEVO: Sistema de herramientas nativas
pub mod mcp_client; // <-- AÑADIDO
pub mod cost_optimizer;
pub mod performance;

// CLI module is only available when not compiling to WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

// Re-exports para facilitar el uso
pub use adapters::*;
pub use neuro_divergent::*;
pub use swarm::*; 