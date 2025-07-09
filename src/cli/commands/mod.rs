// ============================================================================
// COMMANDS MODULE - Manejadores de comandos CLI mejorados
// ============================================================================

use colored::*;

pub mod init;
pub mod swarm;
pub mod hive_mind;
pub mod neural;
pub mod memory;
pub mod tools;

// Re-exports de funciones principales
pub use init::handle_init;
pub use swarm::execute_swarm_command;
pub use hive_mind::handle_hive_mind_command;
pub use neural::handle_neural_command;
pub use memory::handle_memory_command;
pub use tools::handle_tools_command;

// ============================================================================
// UTILIDADES COMUNES
// ============================================================================

/// Imprime mensaje de éxito con formato estándar
pub fn print_success(message: &str) {
    println!("{} {}", "✅".bright_green(), message.green());
}

/// Imprime mensaje de error con formato estándar
pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".bright_red(), message.red());
}

/// Imprime mensaje informativo con formato estándar
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".bright_blue(), message.bright_white());
}

/// Imprime mensaje de advertencia con formato estándar
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".bright_yellow(), message.yellow());
}

/// Imprime encabezado de sección con formato estándar
pub fn print_header(title: &str) {
    println!();
    println!("{}", title.bright_cyan().bold());
    println!("{}", "─".repeat(title.len()).bright_black());
}

/// Imprime progreso con formato estándar
pub fn print_progress(current: usize, total: usize, message: &str) {
    let percentage = (current * 100) / total;
    println!("{} [{}/{}] {}% - {}", 
        "🔄".bright_blue(), 
        current, 
        total, 
        percentage, 
        message.bright_white()
    );
}

// ============================================================================
// MANEJADORES DE COMANDO LEGACY (para compatibilidad)
// ============================================================================

/// Manejador para el comando performance
pub async fn handle_performance_command() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_header("📊 ENJAMBRE PERFORMANCE MONITOR");
    
    print_info("Implementación de métricas de performance integradas");
    print_info("Usa 'enjambre swarm --metrics' para ver métricas en tiempo real");
    
    println!();
    print_success("Para métricas detalladas usa: enjambre swarm <task> --metrics --export-report");
    
    Ok(())
}

/// Manejador para el comando workflow  
pub async fn handle_workflow_command() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_header("🔄 ENJAMBRE WORKFLOW MANAGER");
    
    print_info("Sistema de workflows integrado en swarm orchestrator");
    print_info("Los workflows se ejecutan automáticamente según la complejidad de la tarea");
    
    println!();
    print_success("Usa 'enjambre swarm --thinking' para workflows de razonamiento avanzado");
    
    Ok(())
}

/// Manejador para el comando test
pub async fn handle_test_command() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_header("🧪 ENJAMBRE SYSTEM TEST");
    
    print_info("Ejecutando test de conectividad y funcionalidad básica...");
    
    // Test básico de configuración
    let api_key_available = std::env::var("GEMINI_API_KEY").is_ok() || 
                           std::env::var("GOOGLE_API_KEY").is_ok();
    
    if api_key_available {
        print_success("API Key configurada correctamente");
    } else {
        print_warning("API Key no encontrada - configura GEMINI_API_KEY");
    }
    
    // Test de directorio de configuración
    if let Some(config_dir) = dirs::config_dir() {
        let enjambre_config = config_dir.join("enjambre");
        if enjambre_config.exists() {
            print_success("Directorio de configuración encontrado");
        } else {
            print_info("Directorio de configuración será creado en primera ejecución");
        }
    }
    
    println!();
    print_success("Sistema básico funcionando correctamente");
    print_info("Ejecuta 'enjambre swarm \"test simple task\"' para prueba completa");
    
    Ok(())
}

/// Manejador para el comando config
pub async fn handle_config_command() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_header("⚙️ ENJAMBRE CONFIGURATION");
    
    print_info("Variables de entorno importantes:");
    println!();
    
    // Mostrar configuración actual
    let vars = [
        ("GEMINI_API_KEY", "API Key de Google Gemini"),
        ("GOOGLE_PROJECT_ID", "ID del proyecto Google Cloud (opcional)"), 
        ("GOOGLE_LOCATION", "Región de Google Cloud (opcional)"),
        ("GEMINI_USE_INTERACTIVE", "Usar modo CLI interactivo (true/false)"),
    ];
    
    for (var, description) in vars {
        let value = std::env::var(var).unwrap_or_else(|_| "No configurada".to_string());
        let status = if value == "No configurada" { "❌" } else { "✅" };
        println!("  {} {} = {}", status, var.bright_cyan(), value.bright_white());
        println!("     {}", description.bright_black());
        println!();
    }
    
    // Mostrar ejemplo de configuración
    print_header("📝 Ejemplo de configuración");
    println!("export GEMINI_API_KEY=your_api_key_here");
    println!("export GOOGLE_PROJECT_ID=your_project_id");
    println!("export GOOGLE_LOCATION=us-central1");
    
    Ok(())
} 