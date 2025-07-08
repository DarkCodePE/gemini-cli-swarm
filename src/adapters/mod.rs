// ============================================================================
// MÓDULO DE ADAPTADORES - ruv-swarm Universal LLM Adapters
// ============================================================================
// Este módulo contiene todos los adaptadores disponibles para diferentes LLMs
// siguiendo el patrón CodeGenerationFlow del sistema ruvnet.
// ============================================================================

pub mod gemini_cli;
pub mod gemini_process_manager;
// pub mod claude_flow; // Para futuras implementaciones (pendiente)

// Re-exports públicos
pub use gemini_cli::GeminiCLIFlow;

// Función factory para crear adaptadores dinámicamente
use crate::{CodeGenerationFlow, FlowError};
use std::sync::Arc;

pub async fn create_adapter(adapter_type: &str, config: AdapterConfig) -> Result<Arc<dyn CodeGenerationFlow>, FlowError> {
    // Verificar si se debe usar modo interactivo
    let use_interactive = std::env::var("GEMINI_USE_INTERACTIVE")
        .unwrap_or_default()
        .parse::<bool>()
        .unwrap_or(false);

    match adapter_type.to_lowercase().as_str() {
        "gemini" | "gemini-cli" => {
            let adapter = if use_interactive {
                log::info!("🔄 Creando adaptador Gemini en modo CLI interactivo");
                GeminiCLIFlow::new_interactive(config).await?
            } else {
                log::info!("📡 Creando adaptador Gemini en modo API directa");
                GeminiCLIFlow::new(config).await?
            };
            Ok(Arc::new(adapter))
        }
        _ => Err(FlowError::InvalidPrompt(format!("Adaptador no soportado: {}", adapter_type)))
    }
}

// Configuración común para todos los adaptadores
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout_seconds: u64,
    pub max_attempts: u32,
    pub enable_verification: bool,
    pub project_id: Option<String>, // Para Gemini/Vertex AI
    pub location: Option<String>,   // Para Gemini/Vertex AI
} 