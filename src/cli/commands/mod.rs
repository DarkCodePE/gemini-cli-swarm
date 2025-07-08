// ============================================================================
// COMMANDS MODULE - Implementation of CLI Commands
// ============================================================================

pub mod init;
pub mod hive_mind;
pub mod neural;
pub mod memory;
pub mod tools;
pub mod performance;
pub mod workflow;
pub mod test;
pub mod config;
pub mod swarm;

pub use init::*;
pub use hive_mind::*;
pub use neural::*;
pub use memory::*;
pub use tools::*;
pub use performance::*;
pub use workflow::*;
pub use test::*;
pub use config::*;
pub use swarm::*;

use crate::{
    adapters::AdapterConfig,
    swarm::{SwarmOrchestrator, SwarmConfig, TaskBuilder},
    neuro_divergent::ModelCatalog,
};
use colored::*;
use std::collections::HashMap;
use std::error::Error;

/// Run the main command dispatcher
pub async fn run_command(cli: crate::cli::Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::cli::Commands;
    
    // Initialize logging if verbose
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.command {
        Commands::Init { force, hive_mind, neural_enhanced, path } => {
            handle_init(force, hive_mind, neural_enhanced, path).await
        }
        Commands::HiveMind(cmd) => {
            handle_hive_mind_command(cmd).await
        }
        Commands::Neural(cmd) => {
            handle_neural_command(cmd).await
        }
        Commands::Memory(cmd) => {
            handle_memory_command(cmd).await
        }
        Commands::Tools(cmd) => {
            handle_tools_command(cmd).await
        }
        Commands::Performance(cmd) => {
            handle_performance_command(cmd).await
        }
        Commands::Workflow(cmd) => {
            handle_workflow_command(cmd).await
        }
        Commands::Swarm { task, agents, strategy, gemini, memory_namespace, parallel } => {
            handle_swarm_command(task, agents, strategy, gemini, memory_namespace, parallel).await
        }
        Commands::Test(cmd) => {
            handle_test_command(cmd).await
        }
        Commands::Config(cmd) => {
            handle_config_command(cmd).await
        }
    }
}

/// Create a SwarmOrchestrator instance with default configuration
pub async fn create_orchestrator() -> Result<SwarmOrchestrator, Box<dyn Error + Send + Sync>> {
    let gemini_api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            eprintln!("{}", "⚠️  GEMINI_API_KEY no encontrada. Configure usando:".yellow());
            eprintln!("   export GEMINI_API_KEY=\"tu_api_key_aqui\"");
            eprintln!("   o cree un archivo .env con GEMINI_API_KEY=tu_api_key_aqui");
            return Err("API Key requerida".into());
        }
    };

    let gemini_config = AdapterConfig {
        api_key: gemini_api_key,
        base_url: None,
        timeout_seconds: 60,
        max_attempts: 3,
        enable_verification: true,
        project_id: std::env::var("GOOGLE_CLOUD_PROJECT").ok(),
        location: std::env::var("GOOGLE_CLOUD_LOCATION").ok(),
    };
    
    let mut adapter_configs = HashMap::new();
    adapter_configs.insert("gemini".to_string(), gemini_config);

    let swarm_config = SwarmConfig {
        max_concurrent_tasks: 4,
        default_adapter: "gemini".to_string(),
        enable_neural_selection: true,
        enable_adaptive_learning: true,
        performance_monitoring: true,
    };
    
    let mut orchestrator = SwarmOrchestrator::new(swarm_config);
    orchestrator.initialize(adapter_configs).await?;
    
    Ok(orchestrator)
}

/// Print success message with emoji
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message.green());
}

/// Print error message with emoji  
pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message.red());
}

/// Print warning message with emoji
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message.yellow());
}

/// Print info message with emoji
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message.bright_blue());
} 