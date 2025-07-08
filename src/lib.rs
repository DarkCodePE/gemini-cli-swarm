// ============================================================================
// ruv-FANN + SAFLA + SPARC - Adaptador Universal para LLMs
// ============================================================================
// Este módulo implementa la arquitectura modular de ruvnet para crear
// adaptadores universales que funcionen con cualquier LLM (Gemini, Claude, etc.)
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

// CLI module is only available when not compiling to WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

// Re-exports para facilitar el uso
pub use adapters::*;
pub use neuro_divergent::*;
pub use swarm::*; 