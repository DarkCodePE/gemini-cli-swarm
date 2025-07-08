// ============================================================================
// SWARM ORCHESTRATOR - Master Control Program (MCP) ruv-swarm + SAFLA
// ============================================================================
// Este módulo implementa el orquestador principal que coordina todos los
// componentes del sistema: adapters, neuro-divergent models, y ruv-FANN core.
// Sigue la metodología SAFLA para análisis, diseño y ejecución optimizada.
// ============================================================================

use crate::{
    CodeGenerationFlow, CodeGenerationResult, FlowError,
    adapters::{AdapterConfig, create_adapter},
    neuro_divergent::{ModelCatalog, ModelSpec},
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub preferred_language: Option<String>,
    pub max_execution_time_ms: Option<u64>,
    pub quality_threshold: Option<f64>,
    pub enable_verification: bool,
    pub use_neural_optimization: bool,
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
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            default_adapter: "gemini".to_string(),
            enable_neural_selection: true,
            enable_adaptive_learning: true,
            performance_monitoring: true,
        }
    }
}

// ============================================================================
// RESULTADO DE EJECUCIÓN
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub result: Option<CodeGenerationResult>,
    pub error: Option<String>,
    pub selected_adapter: String,
    pub selected_model: Option<String>,
    pub execution_time_ms: u64,
    pub performance_score: f64,
}

// ============================================================================
// EL ORQUESTADOR PRINCIPAL (MCP)
// ============================================================================

pub struct SwarmOrchestrator {
    config: SwarmConfig,
    adapters: HashMap<String, Arc<dyn CodeGenerationFlow>>,
    active_tasks: HashMap<String, Task>,
    performance_history: Vec<SwarmExecutionResult>,
    session_id: String,
}

impl SwarmOrchestrator {
    /// Constructor del orquestador
    pub fn new(config: SwarmConfig) -> Self {
        Self {
            config,
            adapters: HashMap::new(),
            active_tasks: HashMap::new(),
            performance_history: Vec::new(),
            session_id: Uuid::new_v4().to_string(),
        }
    }

    /// Inicializa el swarm con adaptadores configurados
    pub async fn initialize(&mut self, adapter_configs: HashMap<String, AdapterConfig>) -> Result<(), FlowError> {
        info!("🚀 Inicializando ruv-swarm Orchestrator - Sesión: {}", self.session_id);
        
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
        Ok(())
    }

    /// Ejecuta una tarea usando la metodología SAFLA
    pub async fn execute_task(&mut self, task: Task) -> SwarmExecutionResult {
        let start_time = std::time::Instant::now();
        let task_id = task.id.clone();
        
        info!("📋 Ejecutando tarea: {} - {}", task_id, task.description);
        
        // FASE 1: ANÁLISIS SAFLA - Seleccionar el mejor adaptador y modelo
        let (selected_adapter_name, selected_model) = self.analyze_and_select(&task).await;
        
        // FASE 2: EJECUCIÓN - Usar el adaptador seleccionado
        let execution_result = match self.adapters.get(&selected_adapter_name) {
            Some(adapter) => {
                // Si se requiere optimización neural, aplicar modelo especializado
                if task.requirements.use_neural_optimization {
                    if let Some(model_spec) = &selected_model {
                        info!("🧠 Aplicando optimización neural con modelo: {}", model_spec.description);
                        // TODO: Integrar el modelo con el adaptador
                    }
                }
                
                // Ejecutar la tarea
                adapter.execute(&task.description).await
            }
            None => Err(FlowError::InvalidPrompt(format!("Adaptador no encontrado: {}", selected_adapter_name)))
        };

        // FASE 3: ANÁLISIS DE RESULTADOS
        let execution_time = start_time.elapsed().as_millis() as u64;
        let result = match execution_result {
            Ok(code_result) => {
                let performance_score = self.calculate_performance_score(&code_result, &task);
                
                info!("🎉 Tarea completada exitosamente en {}ms", execution_time);
                info!("📊 Score de performance: {:.2}", performance_score);
                
                SwarmExecutionResult {
                    task_id: task_id.clone(),
                    success: true,
                    result: Some(code_result),
                    error: None,
                    selected_adapter: selected_adapter_name,
                    selected_model: selected_model.map(|m| m.description),
                    execution_time_ms: execution_time,
                    performance_score,
                }
            }
            Err(e) => {
                error!("❌ Error ejecutando tarea {}: {}", task_id, e);
                
                SwarmExecutionResult {
                    task_id: task_id.clone(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    selected_adapter: selected_adapter_name,
                    selected_model: selected_model.map(|m| m.description),
                    execution_time_ms: execution_time,
                    performance_score: 0.0,
                }
            }
        };

        // FASE 4: APRENDIZAJE ADAPTIVO (SAFLA)
        if self.config.enable_adaptive_learning {
            self.performance_history.push(result.clone());
            self.update_adaptive_strategies(&result);
        }

        // Limpiar tarea activa
        self.active_tasks.remove(&task_id);
        
        result
    }

    /// SAFLA Phase 1: Análisis y selección de adaptador/modelo óptimo
    async fn analyze_and_select(&self, task: &Task) -> (String, Option<ModelSpec>) {
        info!("🔍 SAFLA Fase 1: Análisis y selección óptima");
        
        // Selección de adaptador
        let selected_adapter = if self.adapters.contains_key(&self.config.default_adapter) {
            self.config.default_adapter.clone()
        } else {
            self.adapters.keys().next().unwrap().clone()
        };

        // Selección de modelo neural si está habilitado
        let selected_model = if self.config.enable_neural_selection && task.requirements.use_neural_optimization {
            ModelCatalog::select_best_model_for_task(&task.description)
        } else {
            None
        };

        if let Some(ref model) = selected_model {
            info!("🧠 Modelo neural seleccionado: {}", model.description);
            info!("📈 Score esperado: {:.2}", model.performance_score);
        }

        (selected_adapter, selected_model)
    }

    /// Calcula el score de performance basado en múltiples métricas
    fn calculate_performance_score(&self, result: &CodeGenerationResult, _task: &Task) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Factor 1: Verificación exitosa
        if result.verification_passed {
            score += 0.4;
        }
        factors += 1;

        // Factor 2: Confidence score del LLM
        score += result.confidence_score * 0.3;
        factors += 1;

        // Factor 3: Eficiencia (menos intentos = mejor)
        let efficiency = 1.0 - (result.attempts_made as f64 - 1.0) / 10.0;
        score += efficiency.max(0.0) * 0.2;
        factors += 1;

        // Factor 4: Velocidad de ejecución
        let speed_score = if result.execution_time_ms < 5000 { 0.1 } else { 0.05 };
        score += speed_score;
        factors += 1;

        score / factors as f64
    }

    /// Actualiza estrategias adaptivas basadas en performance histórica
    fn update_adaptive_strategies(&mut self, _result: &SwarmExecutionResult) {
        if self.performance_history.len() < 10 {
            return; // Necesitamos suficiente historia
        }

        let recent_avg = self.performance_history
            .iter()
            .rev()
            .take(5)
            .map(|r| r.performance_score)
            .sum::<f64>() / 5.0;

        if recent_avg < 0.6 {
            warn!("⚠️ Performance detectada baja ({:.2}), considerando ajustes adaptativos", recent_avg);
            // TODO: Implementar ajustes adaptativos automáticos
        }
    }

    /// Obtiene estadísticas del swarm
    pub fn get_stats(&self) -> SwarmStats {
        let total_tasks = self.performance_history.len();
        let successful_tasks = self.performance_history.iter().filter(|r| r.success).count();
        let avg_performance = if total_tasks > 0 {
            self.performance_history.iter().map(|r| r.performance_score).sum::<f64>() / total_tasks as f64
        } else {
            0.0
        };

        SwarmStats {
            session_id: self.session_id.clone(),
            total_tasks,
            successful_tasks,
            success_rate: if total_tasks > 0 { successful_tasks as f64 / total_tasks as f64 } else { 0.0 },
            average_performance_score: avg_performance,
            active_adapters: self.adapters.len(),
            active_tasks: self.active_tasks.len(),
        }
    }
}

// ============================================================================
// ESTADÍSTICAS DEL SWARM
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmStats {
    pub session_id: String,
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub success_rate: f64,
    pub average_performance_score: f64,
    pub active_adapters: usize,
    pub active_tasks: usize,
}

// ============================================================================
// BUILDER DE TAREAS
// ============================================================================

pub struct TaskBuilder;

impl TaskBuilder {
    pub fn new() -> Self {
        Self
    }
    
    pub fn code_generation(description: &str) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::CodeGeneration,
            description: description.to_string(),
            priority: TaskPriority::Medium,
            requirements: TaskRequirements {
                preferred_language: Some("rust".to_string()),
                max_execution_time_ms: Some(30000),
                quality_threshold: Some(0.8),
                enable_verification: true,
                use_neural_optimization: true,
            },
            created_at: std::time::SystemTime::now(),
        }
    }

    pub fn forecasting(description: &str) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::Forecasting,
            description: description.to_string(),
            priority: TaskPriority::High,
            requirements: TaskRequirements {
                preferred_language: None,
                max_execution_time_ms: Some(60000),
                quality_threshold: Some(0.9),
                enable_verification: true,
                use_neural_optimization: true,
            },
            created_at: std::time::SystemTime::now(),
        }
    }
} 