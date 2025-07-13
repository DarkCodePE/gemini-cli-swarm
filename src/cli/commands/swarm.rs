// ============================================================================
// SWARM COMMAND - Integrado con Cost Optimization y Performance Monitoring
// ============================================================================

use crate::{
    swarm::{SwarmOrchestrator, SwarmConfig, TaskBuilder, TaskType, TaskPriority},
    adapters::AdapterConfig,
    cost_optimizer::{CostConstraints, PriorityLevel, ModelChoice},
    performance::AlertThresholds,
    ThinkingMode,
};
use chrono;
use clap::Args;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Args)]
pub struct SwarmArgs {
    /// La tarea a ejecutar
    pub task: String,

    /// Activar modo Gemini CLI
    #[arg(long)]
    pub gemini: bool,

    /// Modo thinking habilitado  
    #[arg(long)]
    pub thinking: bool,

    /// Modo thinking avanzado con pasos intermedios
    #[arg(long)]
    pub thinking_verbose: bool,

    /// Límite de costo por tarea en USD
    #[arg(long, value_name = "USD")]
    pub max_cost: Option<f64>,

    /// Presupuesto diario en USD
    #[arg(long, value_name = "USD")]
    pub daily_budget: Option<f64>,

    /// Prioridad de la tarea (low, medium, high, critical)
    #[arg(long, value_enum, default_value = "medium")]
    pub priority: CliPriority,

    /// Selección específica de modelo
    #[arg(long, value_enum)]
    pub model: Option<CliModelChoice>,

    /// Mostrar métricas de performance en tiempo real
    #[arg(long)]
    pub metrics: bool,

    /// Exportar reporte detallado al finalizar
    #[arg(long)]
    pub export_report: bool,

    /// Mostrar recomendaciones de optimización
    #[arg(long)]
    pub recommendations: bool,

    /// Modo verboso para debugging
    #[arg(long, short)]
    pub verbose: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum CliPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl From<CliPriority> for TaskPriority {
    fn from(cli_priority: CliPriority) -> Self {
        match cli_priority {
            CliPriority::Low => TaskPriority::Low,
            CliPriority::Medium => TaskPriority::Medium,
            CliPriority::High => TaskPriority::High,
            CliPriority::Critical => TaskPriority::Critical,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CliModelChoice {
    Gemini15Flash,
    Gemini15Pro,
    Gemini15ProExp,
    Auto,
}

impl From<CliModelChoice> for ModelChoice {
    fn from(cli_model: CliModelChoice) -> Self {
        match cli_model {
            CliModelChoice::Gemini15Flash => ModelChoice::Gemini15Flash,
            CliModelChoice::Gemini15Pro => ModelChoice::Gemini15Pro,
            CliModelChoice::Gemini15ProExp => ModelChoice::Gemini15ProExp,
            CliModelChoice::Auto => ModelChoice::Auto,
        }
    }
}

pub async fn execute_swarm_command(args: SwarmArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.verbose {
        println!("{}", "🔍 Modo verboso activado".bright_blue());
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    println!("{}", "🚀 Inicializando Enjambre Swarm v2.0".bright_green().bold());
    println!("{}", "💡 Con Cost Optimization y Performance Monitoring".bright_cyan());
    println!();

    let cost_constraints = CostConstraints {
        max_cost_per_request: args.max_cost,
        daily_budget: args.daily_budget,
        priority: match args.priority {
            CliPriority::Low => PriorityLevel::Low,
            CliPriority::Medium => PriorityLevel::Medium,
            CliPriority::High => PriorityLevel::High,
            CliPriority::Critical => PriorityLevel::Critical,
        },
    };

    let alert_thresholds = AlertThresholds::default();

    let swarm_config = SwarmConfig {
        max_concurrent_tasks: 4,
        default_adapter: if args.gemini { "gemini".to_string() } else { "gemini".to_string() },
        enable_neural_selection: true,
        enable_adaptive_learning: true,
        performance_monitoring: true,
        cost_optimization: true,
        cost_constraints,
        alert_thresholds,
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap());
    spinner.set_message("Inicializando adaptadores optimizados...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut orchestrator = SwarmOrchestrator::new(swarm_config);

    let mut adapter_configs = HashMap::new();
    
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .unwrap_or_else(|_| {
            spinner.finish_with_message("⚠️ No se encontró API key");
            eprintln!("{}", "⚠️  ADVERTENCIA: No se encontró GEMINI_API_KEY en variables de entorno".yellow());
            eprintln!("{}", "   Configura tu API key con: export GEMINI_API_KEY=your_api_key".cyan());
            String::new()
        });

    if !api_key.is_empty() {
        let adapter_config = AdapterConfig {
            api_key,
            base_url: None,
            timeout_seconds: 120,
            max_attempts: 3,
            enable_verification: true,
            project_id: std::env::var("GOOGLE_PROJECT_ID").ok(),
            location: std::env::var("GOOGLE_LOCATION").ok(),
        };

        adapter_configs.insert("gemini".to_string(), adapter_config);
    }

    match orchestrator.initialize(adapter_configs).await {
        Ok(_) => {
            spinner.finish_with_message("✅ Adaptadores inicializados correctamente");
        }
        Err(e) => {
            spinner.finish_with_message("❌ Error en inicialización");
            eprintln!("{} {}", "Error:".red().bold(), e);
            return Err(e.into());
        }
    }

    println!();
    println!("{}", "📋 Configurando tarea...".bright_blue());

    let mut task_builder = TaskBuilder::new(TaskType::CodeGeneration, args.task.clone())
        .with_priority(args.priority.into());

    if args.thinking_verbose {
        task_builder = task_builder.with_thinking_mode(ThinkingMode::StepByStep { show_intermediate: true });
        println!("  🧠 Modo thinking: Paso a paso con detalles");
    } else if args.thinking {
        task_builder = task_builder.with_thinking_mode(ThinkingMode::Extended { max_thinking_time_ms: 30000 });
        println!("  🧠 Modo thinking: Extendido");
    }

    if let Some(max_cost) = args.max_cost {
        task_builder = task_builder.with_max_cost(max_cost);
        println!("  💰 Límite de costo: ${:.3}", max_cost);
    }

    let task = task_builder.build();

    println!("  📝 Descripción: {}", args.task.bright_white());
    println!("  🎯 Prioridad: {:?}", args.priority);
    if let Some(model) = &args.model {
        println!("  🤖 Modelo específico: {:?}", model);
    } else {
        println!("  🤖 Modelo: Selección automática optimizada");
    }

    if args.metrics || args.recommendations {
        println!();
        println!("{}", "📊 Análisis de optimización...".bright_cyan());
        
        let current_metrics = orchestrator.get_performance_metrics();
        let optimization_stats = orchestrator.get_optimization_stats();

        if args.metrics {
            println!("  📈 Success Rate Actual: {:.1}%", current_metrics.success_rate * 100.0);
            println!("  ⏱️ Avg. Response Time: {}ms", current_metrics.average_response_time_ms);
            println!("  💰 Ahorro Total: ${:.3}", optimization_stats.total_cost_saved);
        }

        if args.recommendations && !optimization_stats.recommendations.is_empty() {
            println!("  💡 Recomendaciones:");
            for rec in &optimization_stats.recommendations {
                println!("    - Modelo {:?}: {}", rec.model, rec.reason.bright_yellow());
                println!("      Costo Estimado: ${:.4}, Confianza: {:.1}%", rec.estimated_cost, rec.confidence * 100.0);
            }
        }
    }

    println!();
    println!("{}", "⚡ Ejecutando tarea con optimizaciones...".bright_green().bold());

    let execution_bar = ProgressBar::new_spinner();
    execution_bar.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap());
    execution_bar.set_message("Analizando complejidad y seleccionando modelo óptimo...");
    execution_bar.enable_steady_tick(Duration::from_millis(120));

    let start_time = std::time::Instant::now();
    let result = orchestrator.execute_task(task).await;
    let execution_time = start_time.elapsed();

    execution_bar.finish_and_clear();

    println!();
    if result.success {
        println!("{}", "🎉 ¡Tarea completada exitosamente!".bright_green().bold());
        
        println!();
        println!("{}", "📊 Métricas de Optimización:".bright_cyan().bold());
        println!("  🤖 Modelo usado: {:?}", result.selected_model);
        println!("  🔧 Adaptador: {}", result.selected_adapter);
        println!("  ⏱️  Tiempo total: {:.2}s", execution_time.as_secs_f64());
        println!("  💰 Costo real: ${:.4}", result.cost_actual);
        
        if result.cost_saved > 0.0 {
            println!("  💚 Ahorro vs modelo caro: ${:.4}", result.cost_saved);
            let savings_percent = (result.cost_saved / (result.cost_actual + result.cost_saved)) * 100.0;
            println!("  📈 Ahorro porcentual: {:.1}%", savings_percent);
        }
        
        println!("  🎯 Score de performance: {:.1}%", result.performance_score * 100.0);

        if let Some(code_result) = &result.result {
            println!();
            println!("{}", "📝 Resultado Generado:".bright_white().bold());
            println!("{}", "─".repeat(60).bright_black());
            println!("{}", code_result.code);
            println!("{}", "─".repeat(60).bright_black());
            
            println!();
            println!("{}", "🔍 Detalles Técnicos:".bright_blue());
            println!("  📋 Lenguaje: {}", code_result.language);
            println!("  🎯 Confianza: {:.1}%", code_result.confidence_score * 100.0);
            println!("  🔄 Intentos: {}", code_result.attempts_made);
            println!("  ✅ Verificación: {}", if code_result.verification_passed { "Pasó ✓" } else { "Falló ✗" });
        }

        if let Some(thinking_result) = &result.thinking_result {
            println!();
            println!("{}", "🧠 Proceso de Razonamiento:".bright_magenta().bold());
            for (i, step) in thinking_result.reasoning_trace.iter().enumerate() {
                println!("  {}. {}", i + 1, step);
            }
            
            if !thinking_result.intermediate_conclusions.is_empty() {
                println!();
                println!("{}", "💡 Conclusiones Intermedias:".bright_yellow());
                for conclusion in &thinking_result.intermediate_conclusions {
                    println!("  • {}", conclusion);
                }
            }
            
            println!("  ⏱️ Tiempo de thinking: {:.2}s", thinking_result.thinking_time_ms as f64 / 1000.0);
        }

    } else {
        println!("{}", "❌ Error en la ejecución".bright_red().bold());
        if let Some(error) = &result.error {
            println!("  📝 Detalle: {}", error.red());
        }
    }

    let performance_report = orchestrator.get_performance_report();
    if !performance_report.alerts.is_empty() {
        println!();
        println!("{}", "⚠️ Alertas de Performance:".bright_yellow().bold());
        for alert in &performance_report.alerts {
            let severity_icon = match alert.severity {
                crate::performance::AlertSeverity::Low => "🔵",
                crate::performance::AlertSeverity::Medium => "🟡",
                crate::performance::AlertSeverity::High => "🟠",
                crate::performance::AlertSeverity::Critical => "🔴",
            };
            println!("  {} {}: {}", severity_icon, alert.metric_name, alert.message);
            println!("    Valor: {:.2}, Umbral: {:.2}", alert.current_value, alert.threshold);
        }
    }

    if args.export_report {
        println!();
        println!("{}", "📄 Exportando reporte detallado...".bright_blue());
        
        match orchestrator.export_detailed_metrics() {
            Ok(json_report) => {
                let filename = format!("enjambre_report_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                std::fs::write(&filename, json_report)?;
                println!("  ✅ Reporte guardado: {}", filename.bright_green());
            }
            Err(e) => {
                println!("  ❌ Error exportando reporte: {}", e.to_string().red());
            }
        }
    }

    if args.metrics {
        println!();
        println!("{}", "📊 Comparación de Performance:".bright_cyan().bold());
        let comparison = orchestrator.get_optimization_stats().claude_flow_comparison;
        
        println!("  🎯 Success Rate:");
        println!("    Target: {:.1}%", comparison.target_success_rate * 100.0);
        println!("    Actual: {:.1}%", comparison.current_success_rate * 100.0);
        
        if comparison.current_success_rate >= comparison.target_success_rate {
            println!("    {} ¡Objetivo alcanzado!", "✅".bright_green());
        } else {
            println!("    {} Gap: {:.1}%", "📈".bright_yellow(), comparison.performance_gap * 100.0);
        }
        
        println!("  ⚡ Speed Improvement:");
        println!("    Target: {:.1}x", comparison.target_speed_improvement);
        println!("    Actual: {:.1}x", comparison.current_speed_improvement);
        
        if comparison.current_speed_improvement >= comparison.target_speed_improvement {
            println!("    {} ¡Objetivo alcanzado!", "✅".bright_green());
        } else {
            println!("    {} Necesita mejora", "📈".bright_yellow());
        }
    }

    println!();
    println!("{}", "🎯 Ejecución completada".bright_green().bold());
    
    Ok(())
} 