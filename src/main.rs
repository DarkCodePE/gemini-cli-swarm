// ============================================================================
// ENJAMBRE v2.0.0 Alpha - Gemini CLI Orchestration Platform
// ============================================================================
// Revolutionary AI swarm coordination system powered by SAFLA + SPARC methodology
// with 27+ neural models, 87+ tools, and hive-mind intelligence.
// ============================================================================

use clap::Parser;
use colored::*;
use dotenv::dotenv;
use std::process;

mod adapters;
mod cli;
mod neuro_divergent;
mod swarm;

use crate::cli::{print_banner, print_quick_help, Cli};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if present
    let _ = dotenv();

    // Parse command line arguments
    let cli = Cli::parse();

    // Print banner for main commands
    if !matches!(cli.command, cli::Commands::Config(_)) {
        print_banner();
    }

    // Run the command
    if let Err(e) = cli::commands::run_command(cli).await {
        eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
        
        // Show help if it's a basic error
        if e.to_string().contains("API key") {
            println!();
            println!("{}", "💡 Quick Fix:".bright_yellow().bold());
            println!("  1. Get your API key: {}", "https://makersuite.google.com/app/apikey".bright_cyan());
            println!("  2. Set it: {} export GEMINI_API_KEY=\"your_key_here\"", "$".bright_blue());
            println!("  3. Or create .env file with: GEMINI_API_KEY=your_key_here");
            println!("  4. Run: {} enjambre init --force", "$".bright_blue());
            println!();
        }
        
        print_quick_help();
        process::exit(1);
    }
}