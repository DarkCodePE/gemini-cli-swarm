use enjambre::cli::{Cli, Commands};
use clap::Parser;
use std::process;

#[tokio::main]
async fn main() {
    // Configurar logging
    env_logger::init();
    
    // Parsear argumentos de línea de comandos
    let cli = Cli::parse();
    
    // Ejecutar el comando correspondiente
    let result = match cli.command {
        Commands::Init { force, hive_mind, neural_enhanced, path } => {
            enjambre::cli::commands::handle_init(force, hive_mind, neural_enhanced, path).await
        }
        Commands::Swarm(args) => {
            enjambre::cli::commands::execute_swarm_command(args).await
        }
        Commands::HiveMind(cmd) => {
            enjambre::cli::commands::handle_hive_mind_command(cmd).await
        }
        Commands::Neural(cmd) => {
            enjambre::cli::commands::handle_neural_command(cmd).await
        }
        Commands::Memory(cmd) => {
            enjambre::cli::commands::handle_memory_command(cmd).await
        }
        Commands::Tools(cmd) => {
            enjambre::cli::commands::handle_tools_command(cmd).await
        }
        Commands::Test(cmd) => {
            enjambre::cli::commands::handle_test_command(cmd).await
        }
        Commands::Config(cmd) => {
            enjambre::cli::commands::config::handle_config_command(cmd).await
        }
        Commands::Performance(cmd) => {
            enjambre::cli::commands::performance::handle_performance_command(cmd).await
        }
        Commands::Workflow(cmd) => {
            enjambre::cli::commands::workflow::handle_workflow_command(cmd).await
        }
    };
    
    // Manejar errores
    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        process::exit(1);
    }
}