// ============================================================================
// SWARM ORCHESTRATOR - Master Control Program (MCP) ruv-swarm + SAFLA
// ============================================================================
// Este módulo implementa el orquestador principal que coordina todos los
// componentes del sistema: adapters, neuro-divergent models, y ruv-FANN core.
// Sigue la metodología SAFLA para análisis, diseño y ejecución optimizada.
// Ahora con cost optimization y performance monitoring integrados.
// ============================================================================

use crate::{
    CodeGenerationFlow, CodeGenerationResult, FlowError, ThinkingResult, ThinkingMode,
    adapters::{AdapterConfig, create_adapter},
    cost_optimizer::{CostOptimizer, TaskComplexity, analyze_task_complexity, ModelChoice, CostConstraints, PriorityLevel, UsageRecord},
    performance::{PerformanceMonitor, AlertThresholds, PerformanceMetrics, PerformanceReport},
    tools::{get_registry, ToolParams, ToolResult, ToolError}, // ✨ NUEVO: Integración con herramientas
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use log::{info, warn, error};

// ============================================================================
// TIPOS DE TAREAS SOPORTADAS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    DataAnalysis,
    Forecasting,
    TextProcessing,
    Classification,
    Regression,
    CustomTask(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub description: String,
    pub priority: TaskPriority,
    pub requirements: TaskRequirements,
    pub created_at: std::time::SystemTime,
    pub thinking_mode: Option<ThinkingMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl From<TaskPriority> for PriorityLevel {
    fn from(priority: TaskPriority) -> Self {
        match priority {
            TaskPriority::Low => PriorityLevel::Low,
            TaskPriority::Medium => PriorityLevel::Balanced,
            TaskPriority::High => PriorityLevel::High,
            TaskPriority::Critical => PriorityLevel::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub preferred_language: Option<String>,
    pub max_execution_time_ms: Option<u64>,
    pub quality_threshold: Option<f64>,
    pub enable_verification: bool,
    pub use_neural_optimization: bool,
    pub max_cost_usd: Option<f64>,
    pub enable_thinking: bool,
}

// ============================================================================
// CONFIGURACIÓN DEL SWARM
// ============================================================================

#[derive(Debug, Clone)]
pub struct SwarmConfig {
    pub max_concurrent_tasks: usize,
    pub default_adapter: String,
    pub enable_neural_selection: bool,
    pub enable_adaptive_learning: bool,
    pub performance_monitoring: bool,
    pub cost_optimization: bool,
    pub cost_constraints: CostConstraints,
    pub alert_thresholds: AlertThresholds,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            default_adapter: "gemini".to_string(),
            enable_neural_selection: true,
            enable_adaptive_learning: true,
            performance_monitoring: true,
            cost_optimization: true,
            cost_constraints: CostConstraints {
                max_cost_per_task: Some(0.50),
                monthly_budget: Some(100.0),
                current_month_spent: 0.0,
                priority_level: PriorityLevel::Balanced,
            },
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

// ============================================================================
// RESULTADO DE EJECUCIÓN MEJORADO
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub result: Option<CodeGenerationResult>,
    pub thinking_result: Option<ThinkingResult>,
    pub error: Option<String>,
    pub selected_adapter: String,
    pub selected_model: ModelChoice,
    pub execution_time_ms: u64,
    pub performance_score: f64,
    pub cost_actual: f64,
    pub cost_saved: f64,        // Cuánto se ahorró vs el modelo más caro
    pub optimization_applied: bool,
}

// ============================================================================
// EL ORQUESTADOR PRINCIPAL (MCP) CON OPTIMIZACIONES
// ============================================================================

pub struct SwarmOrchestrator {
    config: SwarmConfig,
    adapters: HashMap<String, Arc<dyn CodeGenerationFlow>>,
    active_tasks: HashMap<String, Task>,
    performance_history: Vec<SwarmExecutionResult>,
    session_id: String,
    cost_optimizer: CostOptimizer,
    performance_monitor: PerformanceMonitor,
    total_cost_saved: f64,
    tool_usage_stats: HashMap<String, ToolUsageStats>,
}

impl SwarmOrchestrator {
    /// Constructor del orquestador con optimizaciones integradas
    pub fn new(config: SwarmConfig) -> Self {
        let cost_optimizer = CostOptimizer::new(config.cost_constraints.clone());
        let performance_monitor = PerformanceMonitor::new(config.alert_thresholds.clone());

        Self {
            cost_optimizer,
            performance_monitor,
            config,
            adapters: HashMap::new(),
            active_tasks: HashMap::new(),
            performance_history: Vec::new(),
            session_id: Uuid::new_v4().to_string(),
            total_cost_saved: 0.0,
            tool_usage_stats: HashMap::new(),
        }
    }

    /// Inicializa el swarm con adaptadores configurados
    pub async fn initialize(&mut self, adapter_configs: HashMap<String, AdapterConfig>) -> Result<(), FlowError> {
        info!("🚀 Inicializando ruv-swarm Orchestrator v2.0 - Sesión: {}", self.session_id);
        info!("💡 Optimizaciones habilitadas: Cost Optimization, Performance Monitoring");
        
        for (adapter_name, config) in adapter_configs {
            match create_adapter(&adapter_name, config).await {
                Ok(adapter) => {
                    self.adapters.insert(adapter_name.clone(), adapter);
                    info!("✅ Adaptador '{}' inicializado correctamente", adapter_name);
                }
                Err(e) => {
                    error!("❌ Error inicializando adaptador '{}': {}", adapter_name, e);
                    return Err(e);
                }
            }
        }

        if self.adapters.is_empty() {
            return Err(FlowError::InvalidPrompt("No se pudo inicializar ningún adaptador".to_string()));
        }

        info!("🎯 Swarm inicializado con {} adaptadores", self.adapters.len());
        info!("📊 Performance target: 84.8% success rate, 2.8-4.4x speed improvement");
        Ok(())
    }

    /// Ejecuta una tarea usando la metodología SAFLA + optimizaciones
    pub async fn execute_task(&mut self, task: Task) -> SwarmExecutionResult {
        let start_time = std::time::Instant::now();
        let task_id = task.id.clone();
        
        info!("📋 Ejecutando tarea optimizada: {} - {}", task_id, task.description);
        
        // FASE 1: ANÁLISIS SAFLA + COST OPTIMIZATION
        let (selected_model, task_complexity) = self.analyze_and_optimize_selection(&task).await;
        
        // FASE 2: VERIFICACIÓN DE CONSTRAINS DE COSTO
        let cost_check_result = self.verify_cost_constraints(&task, &selected_model, &task_complexity);
        if let Err(cost_error) = cost_check_result {
            warn!("💰 Tarea rechazada por límites de costo: {}", cost_error);
            return SwarmExecutionResult {
                task_id,
                success: false,
                result: None,
                thinking_result: None,
                error: Some(cost_error.to_string()),
                selected_adapter: "none".to_string(),
                selected_model,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                performance_score: 0.0,
                cost_actual: 0.0,
                cost_saved: 0.0,
                optimization_applied: true,
            };
        }

        // FASE 3: INICIAR TRACKING DE PERFORMANCE
        let task_tracker = self.performance_monitor.start_task(
            task_id.clone(), 
            selected_model.clone(), 
            (task_complexity.reasoning_required + task_complexity.code_complexity + task_complexity.context_length) / 3.0
        );

        // FASE 4: SELECCIONAR ADAPTADOR Y EJECUTAR
        let selected_adapter_name = self.select_adapter_for_model(&selected_model);
        let execution_result = match self.adapters.get(&selected_adapter_name) {
            Some(adapter) => {
                info!("🎯 Ejecutando con modelo optimizado: {:?}", selected_model);
                
                // Para thinking mode, necesitaríamos implementar un mecanismo diferente
                // ya que el downcast de trait objects no es directo en Rust.
                // Por ahora, usamos ejecución estándar con prompt mejorado para thinking.
                if task.thinking_mode.is_some() || task_complexity.thinking_needed {
                    info!("🧠 Preparando prompt mejorado para modo thinking");
                    let thinking_prompt = format!(
                        "Piensa paso a paso sobre este problema. Muestra tu razonamiento antes de dar la respuesta final.\n\nProblema: {}\n\nPor favor:\n1. Analiza el problema\n2. Considera diferentes enfoques\n3. Explica tu razonamiento\n4. Proporciona la solución final",
                        task.description
                    );
                    adapter.execute(&thinking_prompt).await
                } else {
                    adapter.execute(&task.description).await
                }
            }
            None => Err(FlowError::InvalidPrompt(format!("Adaptador no encontrado: {}", selected_adapter_name)))
        };

        // FASE 5: PROCESAR RESULTADOS Y MÉTRICAS
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match execution_result {
            Ok(result) => {
                // Completar tracking con resultado exitoso
                task_tracker.complete(Ok(&result), None);
                
                let cost_saved = self.calculate_cost_savings(&selected_model, &result);
                self.total_cost_saved += cost_saved;
                
                // Registrar uso para aprendizaje futuro
                self.record_usage_for_learning(&task, &selected_model, &result, true, execution_time);
                
                info!("✅ Tarea completada exitosamente");
                info!("💰 Costo: ${:.4}, Ahorro: ${:.4}", 
                    result.cost_estimate.as_ref().map(|c| c.estimated_cost_usd).unwrap_or(0.0),
                    cost_saved
                );

                SwarmExecutionResult {
                    task_id,
                    success: true,
                    result: Some(result.clone()),
                    thinking_result: None,
                    error: None,
                    selected_adapter: selected_adapter_name,
                    selected_model,
                    execution_time_ms: execution_time,
                    performance_score: if result.verification_passed { 0.9 } else { 0.6 },
                    cost_actual: result.cost_estimate.as_ref().map(|c| c.estimated_cost_usd).unwrap_or(0.0),
                    cost_saved,
                    optimization_applied: true,
                }
            },
            Err(error) => {
                // Completar tracking con error
                task_tracker.complete(Err(&error), None);
                
                // Registrar fallo para aprendizaje
                self.record_usage_for_learning(&task, &selected_model, &CodeGenerationResult {
                    code: String::new(),
                    language: String::new(),
                    confidence_score: 0.0,
                    attempts_made: 1,
                    execution_time_ms: execution_time,
                    verification_passed: false,
                    cost_estimate: None,
                    model_used: Some(format!("{:?}", selected_model)),
                }, false, execution_time);

                error!("❌ Error ejecutando tarea: {}", error);

                SwarmExecutionResult {
                    task_id,
                    success: false,
                    result: None,
                    thinking_result: None,
                    error: Some(error.to_string()),
                    selected_adapter: selected_adapter_name,
                    selected_model,
                    execution_time_ms: execution_time,
                    performance_score: 0.0,
                    cost_actual: 0.0,
                    cost_saved: 0.0,
                    optimization_applied: true,
                }
            }
        }
    }

    /// Análisis SAFLA mejorado con optimización de costos
    async fn analyze_and_optimize_selection(&mut self, task: &Task) -> (ModelChoice, TaskComplexity) {
        // Analizar complejidad de la tarea
        let task_complexity = analyze_task_complexity(&task.description);
        
        info!("🔍 Análisis de complejidad:");
        info!("  - Razonamiento: {:.2}", task_complexity.reasoning_required);
        info!("  - Código: {:.2}", task_complexity.code_complexity);
        info!("  - Contexto: {:.2}", task_complexity.context_length);
        info!("  - Thinking needed: {}", task_complexity.thinking_needed);

        // Seleccionar modelo óptimo usando cost optimizer
        let selected_model = self.cost_optimizer.select_optimal_model(&task_complexity, &task.description);
        
        info!("🎯 Modelo seleccionado: {:?}", selected_model);
        
        // Obtener recomendaciones de optimización
        let recommendations = self.cost_optimizer.get_optimization_recommendations();
        if !recommendations.is_empty() {
            info!("💡 Recomendaciones de optimización:");
            for rec in recommendations {
                info!("  - {}: {}", rec.category, rec.description);
            }
        }

        (selected_model, task_complexity)
    }

    /// Verifica las constraints de costo antes de ejecutar
    fn verify_cost_constraints(&self, task: &Task, model: &ModelChoice, complexity: &TaskComplexity) -> Result<(), FlowError> {
        // Estimar tokens basado en la descripción de la tarea
        let estimated_input_tokens = task.description.split_whitespace().count() as u32;
        let estimated_output_tokens = match complexity.code_complexity {
            x if x > 0.7 => 2000, // Código complejo
            x if x > 0.4 => 1000, // Código medio
            _ => 500,             // Código simple
        };

        let cost_estimate = self.cost_optimizer.estimate_cost(model, estimated_input_tokens, estimated_output_tokens);
        
        // Verificar contra task requirements
        if let Some(max_cost) = task.requirements.max_cost_usd {
            if cost_estimate.estimated_cost_usd > max_cost {
                return Err(FlowError::CostLimitExceeded(max_cost));
            }
        }

        // Verificar contra constraints globales
        self.cost_optimizer.check_cost_constraints(cost_estimate.estimated_cost_usd)
    }

    /// Calcula cuánto dinero se ahorró vs el modelo más caro
    fn calculate_cost_savings(&self, _selected_model: &ModelChoice, result: &CodeGenerationResult) -> f64 {
        if let Some(cost_estimate) = &result.cost_estimate {
            // Comparar con Claude 3.7 Sonnet (el más caro)
            let expensive_model_cost = (cost_estimate.input_tokens as f64 / 1_000_000.0) * 3.00 +
                                     (cost_estimate.output_tokens as f64 / 1_000_000.0) * 15.00;
            
            (expensive_model_cost - cost_estimate.estimated_cost_usd).max(0.0)
        } else {
            0.0
        }
    }

    /// Registra el uso para aprendizaje futuro del sistema
    fn record_usage_for_learning(&mut self, task: &Task, model: &ModelChoice, result: &CodeGenerationResult, success: bool, _execution_time: u64) {
        let task_complexity = analyze_task_complexity(&task.description);
        
        let usage_record = UsageRecord {
            timestamp: std::time::SystemTime::now(),
            model_used: model.clone(),
            task_complexity,
            actual_cost: result.cost_estimate.as_ref().map(|c| c.estimated_cost_usd).unwrap_or(0.0),
            success,
            user_satisfaction: None, // Puede ser agregado por el usuario posteriormente
        };

        self.cost_optimizer.record_usage(usage_record);
    }

    /// Selecciona el adaptador apropiado para un modelo específico
    fn select_adapter_for_model(&self, model: &ModelChoice) -> String {
        match model {
            ModelChoice::Gemini2Pro | ModelChoice::Gemini25Pro | ModelChoice::Gemini25Flash => "gemini".to_string(),
            ModelChoice::Claude35Sonnet | ModelChoice::Claude37Sonnet => "claude".to_string(),
            ModelChoice::AutoSelect => self.config.default_adapter.clone(),
        }
    }

    /// Obtiene métricas de performance actuales
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_monitor.get_current_metrics()
    }

    /// Obtiene reporte completo de performance
    pub fn get_performance_report(&self) -> PerformanceReport {
        self.performance_monitor.get_performance_report()
    }

    /// Obtiene estadísticas de uso y optimización
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        let cost_stats = self.cost_optimizer.get_usage_stats();
        let performance_metrics = self.get_performance_metrics();
        
        OptimizationStats {
            total_cost_saved: self.total_cost_saved,
            total_tasks_executed: cost_stats.total_tasks,
            average_cost_per_task: cost_stats.cost_per_successful_task,
            success_rate: cost_stats.success_rate,
            claude_flow_comparison: ClaudeFlowComparison {
                target_success_rate: 0.848,
                current_success_rate: performance_metrics.success_rate,
                target_speed_improvement: 3.6,
                current_speed_improvement: performance_metrics.speed_improvement_factor,
                performance_gap: (0.848 - performance_metrics.success_rate).max(0.0),
            },
            recommendations: self.cost_optimizer.get_optimization_recommendations(),
        }
    }

    /// Actualiza las constraints de costo
    pub fn update_cost_constraints(&mut self, constraints: CostConstraints) {
        self.cost_optimizer.update_constraints(constraints.clone());
        self.config.cost_constraints = constraints;
    }

    /// Exporta métricas detalladas en JSON
    pub fn export_detailed_metrics(&self) -> Result<String, serde_json::Error> {
        let metrics = DetailedMetrics {
            performance_report: self.get_performance_report(),
            optimization_stats: self.get_optimization_stats(),
            session_id: self.session_id.clone(),
            export_timestamp: std::time::SystemTime::now(),
        };

        serde_json::to_string_pretty(&metrics)
    }
    
    /// Obtener esquemas de herramientas para Gemini Function Calling
    pub fn get_function_schemas(&self) -> Vec<serde_json::Value> {
        get_registry().get_function_schemas()
    }
    
    /// Ejecutar herramienta por nombre
    pub async fn execute_tool(&mut self, tool_name: &str, params: ToolParams) -> Result<ToolResult, ToolError> {
        log::info!("🔧 Ejecutando herramienta: {}", tool_name);
        
        let start_time = std::time::Instant::now();
        let result = get_registry().execute(tool_name, params).await;
        let execution_time = start_time.elapsed();
        
        match &result {
            Ok(tool_result) => {
                log::info!("✅ Herramienta '{}' completada en {:?}: {}", 
                    tool_name, execution_time, tool_result.message);
                
                // Registrar métricas de uso de herramientas
                self.tool_usage_stats
                    .entry(tool_name.to_string())
                    .and_modify(|stats| {
                        stats.total_calls += 1;
                        stats.total_time += execution_time;
                        if tool_result.success {
                            stats.successful_calls += 1;
                        }
                    })
                    .or_insert(ToolUsageStats {
                        total_calls: 1,
                        successful_calls: if tool_result.success { 1 } else { 0 },
                        total_time: execution_time,
                        last_used: std::time::SystemTime::now(),
                    });
            }
            Err(error) => {
                log::warn!("❌ Error ejecutando herramienta '{}': {}", tool_name, error);
            }
        }
        
        result
    }
    
    /// Ejecutar múltiples herramientas en paralelo
    pub async fn execute_tools_parallel(&mut self, tool_calls: Vec<(String, ToolParams)>) -> Vec<Result<ToolResult, ToolError>> {
        log::info!("🔧⚡ Ejecutando {} herramientas en paralelo", tool_calls.len());
        
        let futures: Vec<_> = tool_calls.into_iter()
            .map(|(tool_name, params)| {
                let tool_name_clone = tool_name.clone();
                async move {
                    get_registry().execute(&tool_name_clone, params).await
                }
            })
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        // Actualizar estadísticas para todas las herramientas
        for (result, _) in results.iter().zip(0..) {
            // Las estadísticas individuales se actualizan en execute_tool
        }
        
        results
    }
    
    /// Obtener estadísticas de uso de herramientas
    pub fn get_tool_usage_stats(&self) -> &std::collections::HashMap<String, ToolUsageStats> {
        &self.tool_usage_stats
    }
    
    /// Listar herramientas disponibles
    pub fn list_available_tools(&self) -> Vec<&str> {
        get_registry().list_all()
    }
    
    /// Listar herramientas por categoría
    pub fn list_tools_by_category(&self, category: &crate::tools::ToolCategory) -> Vec<&str> {
        get_registry().list_by_category(category)
    }
    
    /// Crear parámetros de herramienta desde JSON
    pub fn create_tool_params(&self, json_params: serde_json::Value) -> Result<ToolParams, ToolError> {
        match json_params {
            serde_json::Value::Object(map) => {
                let mut params = ToolParams::new();
                for (key, value) in map {
                    params.data.insert(key, value);
                }
                Ok(params)
            }
            _ => Err(ToolError::InvalidParameter(
                "params".to_string(), 
                "Los parámetros deben ser un objeto JSON".to_string()
            ))
        }
    }
}

// ============================================================================
// ESTRUCTURAS AUXILIARES PARA ESTADÍSTICAS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_cost_saved: f64,
    pub total_tasks_executed: usize,
    pub average_cost_per_task: f64,
    pub success_rate: f64,
    pub claude_flow_comparison: ClaudeFlowComparison,
    pub recommendations: Vec<crate::cost_optimizer::OptimizationRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeFlowComparison {
    pub target_success_rate: f64,
    pub current_success_rate: f64,
    pub target_speed_improvement: f64,
    pub current_speed_improvement: f64,
    pub performance_gap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedMetrics {
    pub performance_report: PerformanceReport,
    pub optimization_stats: OptimizationStats,
    pub session_id: String,
    pub export_timestamp: std::time::SystemTime,
}

// ============================================================================
// BUILDER PATTERN PARA CREAR TAREAS
// ============================================================================

pub struct TaskBuilder {
    task_type: TaskType,
    description: String,
    priority: TaskPriority,
    requirements: TaskRequirements,
    thinking_mode: Option<ThinkingMode>,
}

impl TaskBuilder {
    pub fn new(task_type: TaskType, description: String) -> Self {
        Self {
            task_type,
            description,
            priority: TaskPriority::Medium,
            requirements: TaskRequirements {
                preferred_language: None,
                max_execution_time_ms: None,
                quality_threshold: Some(0.8),
                enable_verification: true,
                use_neural_optimization: true,
                max_cost_usd: None,
                enable_thinking: false,
            },
            thinking_mode: None,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_thinking_mode(mut self, mode: ThinkingMode) -> Self {
        self.thinking_mode = Some(mode);
        self.requirements.enable_thinking = true;
        self
    }

    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.requirements.max_cost_usd = Some(max_cost);
        self
    }

    pub fn build(self) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            task_type: self.task_type,
            description: self.description,
            priority: self.priority,
            requirements: self.requirements,
            created_at: std::time::SystemTime::now(),
            thinking_mode: self.thinking_mode,
        }
    }

    // Métodos de conveniencia
    pub fn code_generation(description: &str) -> Task {
        TaskBuilder::new(TaskType::CodeGeneration, description.to_string()).build()
    }

    pub fn complex_reasoning(description: &str) -> Task {
        TaskBuilder::new(TaskType::CodeGeneration, description.to_string())
            .with_thinking_mode(ThinkingMode::StepByStep { show_intermediate: true })
            .build()
    }

    pub fn budget_task(description: &str, max_cost: f64) -> Task {
        TaskBuilder::new(TaskType::CodeGeneration, description.to_string())
            .with_max_cost(max_cost)
            .with_priority(TaskPriority::Low)
            .build()
    }
} 

/// Estadísticas de uso de herramientas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub total_time: std::time::Duration,
    pub last_used: std::time::SystemTime,
}

impl ToolUsageStats {
    /// Tasa de éxito de la herramienta
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.successful_calls as f64 / self.total_calls as f64
        }
    }
    
    /// Tiempo promedio de ejecución
    pub fn average_execution_time(&self) -> std::time::Duration {
        if self.total_calls == 0 {
            std::time::Duration::from_secs(0)
        } else {
            self.total_time / self.total_calls as u32
        }
    }
} 