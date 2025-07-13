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
pub mod config;
pub mod performance;
pub mod workflow;
pub mod test;

// Re-exports de funciones principales
pub use init::handle_init;
pub use swarm::execute_swarm_command;
pub use hive_mind::handle_hive_mind_command;
pub use neural::handle_neural_command;
pub use memory::handle_memory_command;
pub use tools::handle_tools_command;
pub use test::handle_test_command;

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
    let percentage = (current as f64 / total as f64) * 100.0;
    println!("{} [{}/{}] {:.1}% - {}", 
        "⏳".bright_yellow(), 
        current, 
        total, 
        percentage,
        message.bright_white()
    );
} 