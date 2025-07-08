// ============================================================================
// CLI CONFIGURATION - Configuration Management
// ============================================================================

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub gemini_api_key: Option<String>,
    pub default_adapter: String,
    pub max_concurrent_tasks: usize,
    pub enable_neural_selection: bool,
    pub enable_adaptive_learning: bool,
    pub log_level: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            gemini_api_key: None,
            default_adapter: "gemini".to_string(),
            max_concurrent_tasks: 4,
            enable_neural_selection: true,
            enable_adaptive_learning: true,
            log_level: "info".to_string(),
        }
    }
}

impl CliConfig {
    pub fn load_from_env() -> Self {
        Self {
            gemini_api_key: std::env::var("GEMINI_API_KEY").ok(),
            default_adapter: std::env::var("DEFAULT_ADAPTER").unwrap_or_else(|_| "gemini".to_string()),
            max_concurrent_tasks: std::env::var("MAX_CONCURRENT_TASKS")
                .unwrap_or_default()
                .parse()
                .unwrap_or(4),
            enable_neural_selection: std::env::var("ENABLE_NEURAL_SELECTION")
                .unwrap_or_default()
                .parse()
                .unwrap_or(true),
            enable_adaptive_learning: std::env::var("ENABLE_ADAPTIVE_LEARNING")
                .unwrap_or_default()
                .parse()
                .unwrap_or(true),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".enjambre"))
    }
} 