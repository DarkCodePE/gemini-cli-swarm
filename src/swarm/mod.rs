// ============================================================================
// SWARM ORCHESTRATOR v2.0 - Orquestador de Agentes con Herramientas Nativas
// ============================================================================

use crate::{
    CodeGenerationFlow, CodeGenerationResult, FlowError, ThinkingResult, ThinkingMode,
    adapters::{AdapterConfig, create_adapter},
    cost_optimizer::{CostOptimizer, TaskComplexity, analyze_task_complexity, ModelChoice, CostConstraints, PriorityLevel},
    performance::{PerformanceMonitor, AlertThresholds, PerformanceMetrics, PerformanceReport},
    tools::{get_registry, ToolParams, ToolResult, ToolError},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use log::{info, error};

// ============================================================================
// ESTRUCTURAS DE DATOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: u32,
    pub task: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<u32>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub original_objective: String,
    pub steps: Vec<TaskStep>,
}

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
            TaskPriority::Medium => PriorityLevel::Medium,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            max_concurrent_tasks: 5,
            default_adapter: "gemini".to_string(),
            enable_neural_selection: true,
            enable_adaptive_learning: true,
            performance_monitoring: true,
            cost_optimization: true,
            cost_constraints: CostConstraints {
                max_cost_per_request: Some(0.50),
                daily_budget: Some(100.0),
                priority: PriorityLevel::Medium,
            },
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

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
    pub cost_saved: f64,
    pub optimization_applied: bool,
}

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
    /// Constructor principal
    pub fn new(config: SwarmConfig) -> Self {
        let cost_optimizer = CostOptimizer::new();
        let performance_monitor = PerformanceMonitor::with_thresholds(config.alert_thresholds.clone());
        
        Self {
            config,
            adapters: HashMap::new(),
            active_tasks: HashMap::new(),
            performance_history: Vec::new(),
            session_id: Uuid::new_v4().to_string(),
            cost_optimizer,
            performance_monitor,
            total_cost_saved: 0.0,
            tool_usage_stats: HashMap::new(),
        }
    }

    pub async fn create_execution_plan(&self, objective: &str) -> Result<ExecutionPlan, FlowError> {
        let _available_tools = self.list_available_tools().join(", ");
        
        // Crear un plan básico
        let steps = vec![
            TaskStep {
                id: 1,
                task: format!("Analizar objetivo: {}", objective),
                tools: vec!["safla_memory".to_string()],
                depends_on: vec![],
                details: Some("Análisis inicial del objetivo usando SAFLA".to_string()),
            },
            TaskStep {
                id: 2,
                task: "Ejecutar plan con ruv-swarm".to_string(),
                tools: vec!["ruv_swarm".to_string()],
                depends_on: vec![1],
                details: Some("Delegar ejecución a ruv-swarm".to_string()),
            },
        ];

        Ok(ExecutionPlan {
            original_objective: objective.to_string(),
            steps,
        })
    }

    pub async fn initialize(&mut self, adapter_configs: HashMap<String, AdapterConfig>) -> Result<(), FlowError> {
        for (name, config) in adapter_configs {
            match create_adapter(&name, config).await {
                Ok(adapter) => {
                    self.adapters.insert(name, adapter);
                }
                Err(e) => {
                    error!("Error inicializando adaptador {}: {}", name, e);
                    return Err(e);
                }
            }
        }
        
        if self.adapters.is_empty() {
            return Err(FlowError::AdapterNotFound("No se pudo inicializar ningún adaptador".to_string()));
        }
        
        Ok(())
    }

    pub async fn execute_task(&mut self, task: Task) -> SwarmExecutionResult {
        let start_time = std::time::Instant::now();
        let task_id = task.id.clone();
        
        // Análisis y optimización simplificados
        let task_complexity = analyze_task_complexity(&task.description);
        let selected_model = self.cost_optimizer.optimize_model_selection(
            task_complexity,
            &self.config.cost_constraints,
        );
        
        let selected_adapter = self.select_adapter_for_model(&selected_model);
        
        // Ejecutar tarea
        let result = if let Some(adapter) = self.adapters.get(&selected_adapter) {
            adapter.execute(&task.description).await
        } else {
            Err(FlowError::AdapterNotFound(selected_adapter.clone()))
        };
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Crear resultado
        match result {
            Ok(code_result) => {
                SwarmExecutionResult {
                    task_id,
                    success: true,
                    result: Some(code_result),
                    thinking_result: None,
                    error: None,
                    selected_adapter,
                    selected_model,
                    execution_time_ms: execution_time,
                    performance_score: 0.85,
                    cost_actual: 0.01,
                    cost_saved: 0.0,
                    optimization_applied: true,
                }
            }
            Err(e) => {
                SwarmExecutionResult {
                    task_id,
                    success: false,
                    result: None,
                    thinking_result: None,
                    error: Some(e.to_string()),
                    selected_adapter,
                    selected_model,
                    execution_time_ms: execution_time,
                    performance_score: 0.0,
                    cost_actual: 0.0,
                    cost_saved: 0.0,
                    optimization_applied: false,
                }
            }
        }
    }

    fn select_adapter_for_model(&self, model: &ModelChoice) -> String {
        match model {
            ModelChoice::Gemini15Pro | ModelChoice::Gemini15Flash | ModelChoice::Gemini15ProExp => "gemini".to_string(),
            ModelChoice::Auto => self.config.default_adapter.clone(),
        }
    }

    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        self.performance_monitor.get_metrics()
    }

    pub fn get_performance_report(&self) -> PerformanceReport {
        self.performance_monitor.get_report()
    }

    pub fn get_optimization_stats(&self) -> OptimizationStats {
        let performance_metrics = self.get_performance_metrics();
        
        OptimizationStats {
            total_cost_saved: self.total_cost_saved,
            total_tasks_executed: self.performance_history.len(),
            average_cost_per_task: if !self.performance_history.is_empty() {
                self.performance_history.iter().map(|r| r.cost_actual).sum::<f64>() / self.performance_history.len() as f64
            } else { 0.0 },
            success_rate: performance_metrics.success_rate,
            claude_flow_comparison: ClaudeFlowComparison {
                target_success_rate: 0.848,
                current_success_rate: performance_metrics.success_rate,
                target_speed_improvement: 3.6,
                current_speed_improvement: 1.0,
                performance_gap: (0.848 - performance_metrics.success_rate).max(0.0f64),
            },
            recommendations: self.cost_optimizer.get_recommendations("tarea general"),
        }
    }

    pub fn update_cost_constraints(&mut self, constraints: CostConstraints) {
        self.config.cost_constraints = constraints;
    }

    pub fn export_detailed_metrics(&self) -> Result<String, serde_json::Error> {
        let metrics = DetailedMetrics {
            performance_report: self.get_performance_report(),
            optimization_stats: self.get_optimization_stats(),
            session_id: self.session_id.clone(),
            export_timestamp: std::time::SystemTime::now(),
        };
        
        serde_json::to_string_pretty(&metrics)
    }

    // Métodos de herramientas
    pub fn get_function_schemas(&self) -> Vec<serde_json::Value> {
        let registry = get_registry();
        registry.get_function_schemas()
    }

    pub async fn execute_tool(&mut self, tool_name: &str, params: ToolParams) -> Result<ToolResult, ToolError> {
        let start_time = std::time::Instant::now();
        
        let registry = get_registry();
        let result = registry.execute(tool_name, params).await;
        
        let execution_time = start_time.elapsed();
        
        // Actualizar estadísticas
        let stats = self.tool_usage_stats.entry(tool_name.to_string())
            .or_insert_with(|| ToolUsageStats {
                total_calls: 0,
                successful_calls: 0,
                total_time: std::time::Duration::from_secs(0),
                last_used: std::time::SystemTime::now(),
            });
        
        stats.total_calls += 1;
        stats.total_time += execution_time;
        stats.last_used = std::time::SystemTime::now();
        
        if result.is_ok() {
            stats.successful_calls += 1;
        }
        
        result
    }

    pub async fn execute_tools_parallel(&mut self, tool_calls: Vec<(String, ToolParams)>) -> Vec<Result<ToolResult, ToolError>> {
        let mut results = Vec::new();
        
        for (tool_name, params) in tool_calls {
            let result = self.execute_tool(&tool_name, params).await;
            results.push(result);
        }
        
        results
    }

    pub fn get_tool_usage_stats(&self) -> &HashMap<String, ToolUsageStats> {
        &self.tool_usage_stats
    }

    pub fn list_available_tools(&self) -> Vec<String> {
        let registry = get_registry();
        registry.list_all().into_iter().map(|s| s.to_string()).collect()
    }

    pub fn list_tools_by_category(&self, category: &crate::tools::ToolCategory) -> Vec<String> {
        let registry = get_registry();
        registry.list_by_category(category).into_iter().map(|s| s.to_string()).collect()
    }

    pub fn create_tool_params(&self, json_params: serde_json::Value) -> Result<ToolParams, ToolError> {
        if let serde_json::Value::Object(map) = json_params {
            let mut params = ToolParams::new();
            for (key, value) in map {
                params.data.insert(key, value);
            }
            Ok(params)
        } else {
            Err(ToolError::InvalidParameter("params".to_string(), "Expected JSON object".to_string()))
        }
    }
}

// ============================================================================
// FUNCIONES AUXILIARES
// ============================================================================

fn extract_json_from_response(response: &str) -> String {
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start..].find("```") {
            let json_start = start + 7;
            let json_end = start + end;
            return response[json_start..json_end].trim().to_string();
        }
    }
    
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            return response[start..=end].to_string();
        }
    }
    
    response.to_string()
}

// ============================================================================
// ESTRUCTURAS DE ESTADÍSTICAS
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
// TASK BUILDER
// ============================================================================

#[derive(Debug, Clone)]
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
                quality_threshold: None,
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
        Self::new(TaskType::CodeGeneration, description.to_string()).build()
    }

    pub fn complex_reasoning(description: &str) -> Task {
        Self::new(TaskType::CodeGeneration, description.to_string())
            .with_thinking_mode(ThinkingMode::Extended { max_thinking_time_ms: 30000 })
            .build()
    }

    pub fn budget_task(description: &str, max_cost: f64) -> Task {
        Self::new(TaskType::CodeGeneration, description.to_string())
            .with_max_cost(max_cost)
            .with_priority(TaskPriority::Low)
            .build()
    }
}

// ============================================================================
// ESTADÍSTICAS DE HERRAMIENTAS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub total_time: std::time::Duration,
    pub last_used: std::time::SystemTime,
}

impl ToolUsageStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_calls > 0 {
            self.successful_calls as f64 / self.total_calls as f64
        } else {
            0.0
        }
    }

    pub fn average_execution_time(&self) -> std::time::Duration {
        if self.total_calls > 0 {
            self.total_time / self.total_calls as u32
        } else {
            std::time::Duration::from_secs(0)
        }
    }
} 