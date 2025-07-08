// ============================================================================
// SWARM COMMAND - Direct Task Execution
// ============================================================================

use super::{create_orchestrator, print_success, print_error, print_info};
use crate::swarm::TaskBuilder;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::time::Instant;

pub async fn handle_swarm_command(
    task: String,
    agents: usize,
    strategy: String,
    use_gemini: bool,
    memory_namespace: Option<String>,
    parallel: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    
    // Print banner for swarm execution
    println!("{}", "🐝 ENJAMBRE SWARM EXECUTION".bright_cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    
    print_info(&format!("Task: {}", task));
    print_info(&format!("Agents: {}", agents));
    print_info(&format!("Strategy: {}", strategy));
    print_info(&format!("Using: {}", if use_gemini { "Gemini CLI" } else { "Gemini API" }));
    
    if let Some(namespace) = &memory_namespace {
        print_info(&format!("Memory Namespace: {}", namespace));
    }
    
    if parallel {
        print_info("Parallel execution: ENABLED");
    }
    
    println!();

    // Set Gemini mode based on flag
    if use_gemini {
        std::env::set_var("GEMINI_USE_INTERACTIVE", "true");
    }

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Initializing swarm orchestrator...");

    // Initialize orchestrator
    let mut orchestrator = match create_orchestrator().await {
        Ok(orch) => {
            pb.set_message("✅ Orchestrator initialized");
            pb.finish_and_clear();
            orch
        }
        Err(e) => {
            pb.finish_and_clear();
            print_error(&format!("Failed to initialize orchestrator: {}", e));
            return Err(e);
        }
    };

    // Create task
    let swarm_task = TaskBuilder::code_generation(&task);
    
    // Execute with progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("🐝🧠⚡🔧 ")
            .template("{spinner:.bright_yellow} {msg}")
            .unwrap(),
    );
    pb.set_message("Executing swarm coordination...");

    let start_time = Instant::now();
    let result = orchestrator.execute_task(swarm_task).await;
    let execution_time = start_time.elapsed();

    pb.finish_and_clear();

    // Display results
    println!("{}", "📊 EXECUTION RESULTS".bright_green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
    
    if result.success {
        print_success("Task completed successfully!");
        println!("⏱️  Execution Time: {:.2}s", execution_time.as_secs_f64());
        println!("📊 Performance Score: {:.2}", result.performance_score);
        
        if let Some(model) = result.selected_model {
            println!("🧠 Model Used: {}", model.bright_blue());
        }
        
        if let Some(code_result) = result.result {
            println!("🔧 Attempts Made: {}", code_result.attempts_made);
            println!("✓ Verification: {}", 
                if code_result.verification_passed { 
                    "PASSED".green() 
                } else { 
                    "FAILED".red() 
                }
            );
            println!("📝 Language: {}", code_result.language.bright_yellow());
            println!("🎯 Confidence: {:.1}%", code_result.confidence_score * 100.0);
            
            // Display code if not too long
            if code_result.code.len() < 2000 {
                println!("\n{}", "💻 GENERATED CODE:".bright_magenta().bold());
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".magenta());
                println!("{}", code_result.code.bright_white());
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".magenta());
            } else {
                println!("\n📄 Code generated ({} characters) - too large to display", code_result.code.len());
                println!("🎯 Preview: {}...", code_result.code.chars().take(200).collect::<String>());
            }
        }
    } else {
        print_error("Task execution failed!");
        println!("⏱️  Execution Time: {:.2}s", execution_time.as_secs_f64());
        if let Some(error) = result.error {
            println!("❌ Error: {}", error.red());
        }
    }

    println!();
    print_info("Swarm execution completed. Use 'enjambre memory stats' to check stored context.");
    
    Ok(())
} 