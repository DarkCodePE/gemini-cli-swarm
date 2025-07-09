// ============================================================================
// ENJAMBRE CLI v2.0 - Sistema de Agentes Autónomos con Optimizaciones Avanzadas
// ============================================================================
// Nuevo sistema CLI con:
// - Cost Optimization: Selección inteligente de modelos para optimizar costo/rendimiento
// - Performance Monitoring: Métricas en tiempo real y comparación con Claude-Flow
// - Thinking Mode: Soporte para modelos con capacidades de razonamiento
// - Enhanced UX: Interfaz mejorada con progress bars, colores y reportes detallados
// ============================================================================

use clap::Parser;
use colored::*;
use std::process;

// Importar módulos principales
use enjambre::cli::{Cli, Commands, print_banner};
use enjambre::cli::commands::{
    execute_swarm_command,
    handle_init,
    handle_hive_mind_command,
    handle_neural_command,
    handle_memory_command,
    handle_tools_command,
    handle_performance_command,
    handle_workflow_command,
    handle_test_command,
    handle_config_command,
};

#[tokio::main]
async fn main() {
    // Configurar logging básico
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    // Parsear argumentos CLI
    let cli = Cli::parse();

    // Configurar verbose logging si está habilitado
    if cli.verbose {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("debug")
        ).init();
    }

    // Mostrar banner principal
    print_banner();

    // Ejecutar comando correspondiente
    let result = match cli.command {
        Commands::Init { force, hive_mind, neural_enhanced, path } => {
            handle_init(force, hive_mind, neural_enhanced, path).await
        }
        
        Commands::Swarm(swarm_args) => {
            execute_swarm_command(swarm_args).await
        }
        
        Commands::HiveMind(hive_cmd) => {
            handle_hive_mind_command(hive_cmd).await
        }
        
        Commands::Neural(neural_cmd) => {
            handle_neural_command(neural_cmd).await
        }
        
        Commands::Memory(memory_cmd) => {
            handle_memory_command(memory_cmd).await
        }
        
        Commands::Tools(tools_cmd) => {
            handle_tools_command(tools_cmd).await
        }
        
        Commands::Performance(_perf_cmd) => {
            handle_performance_command().await  // Simplificado para v2.0
        }
        
        Commands::Workflow(_workflow_cmd) => {
            handle_workflow_command().await  // Simplificado para v2.0
        }
        
        Commands::Test(_test_cmd) => {
            handle_test_command().await  // Simplificado para v2.0
        }
        
        Commands::Config(_config_cmd) => {
            handle_config_command().await  // Simplificado para v2.0
        }
    };

    // Manejar resultado y mostrar ayuda si es necesario
    match result {
        Ok(_) => {
            // Éxito - no mostrar nada adicional
        }
        Err(e) => {
            eprintln!();
            eprintln!("{} {}", "❌ Error:".bright_red().bold(), e.to_string().red());
            eprintln!();
            
            // Mostrar ayuda contextual para errores comunes
            if e.to_string().contains("API") || e.to_string().contains("key") {
                eprintln!("{}", "💡 Ayuda:".bright_yellow().bold());
                eprintln!("   Configure su API key de Gemini:");
                eprintln!("   export GEMINI_API_KEY=\"su_api_key_aqui\"");
                eprintln!();
                eprintln!("   O use el comando de configuración:");
                eprintln!("   enjambre config show");
            } else if e.to_string().contains("connection") || e.to_string().contains("network") {
                eprintln!("{}", "💡 Ayuda:".bright_yellow().bold());
                eprintln!("   Verifique su conexión a internet");
                eprintln!("   Pruebe: enjambre test gemini");
            } else {
                eprintln!("{}", "💡 Ayuda:".bright_yellow().bold());
                eprintln!("   Use 'enjambre --help' para ver todos los comandos disponibles");
                eprintln!("   Use 'enjambre <comando> --help' para ayuda específica");
            }
            
            eprintln!();
            process::exit(1);
        }
    }
}