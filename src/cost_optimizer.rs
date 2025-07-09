// ============================================================================
// COST OPTIMIZER - Sistema Inteligente de Selección de Modelos
// ============================================================================
// Este módulo implementa la optimización automática de costo/rendimiento
// seleccionando el modelo más eficiente para cada tipo de tarea.
// ============================================================================

use crate::{CodeGenerationFlow, FlowError, ThinkingMode, CostEstimate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ENUMS Y ESTRUCTURAS PRINCIPALES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ModelChoice {
    Gemini2Pro,         // $0.10/$0.40 - Ultra económico para tareas simples
    Gemini25Pro,        // $1.25/$10.00 - Balanceado, thinking mode
    Gemini25Flash,      // Más rápido, menor costo
    Claude35Sonnet,     // $3.00/$15.00 - Excelente para código
    Claude37Sonnet,     // Premium, mejor razonamiento
    AutoSelect,         // Selección automática
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplexity {
    pub reasoning_required: f64,    // 0.0 - 1.0
    pub code_complexity: f64,       // 0.0 - 1.0
    pub context_length: f64,        // 0.0 - 1.0
    pub thinking_needed: bool,
    pub multimodal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConstraints {
    pub max_cost_per_task: Option<f64>,
    pub monthly_budget: Option<f64>,
    pub current_month_spent: f64,
    pub priority_level: PriorityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityLevel {
    Low,        // Priorizar costo sobre calidad
    Balanced,   // Balance costo/calidad
    High,       // Priorizar calidad sobre costo
    Critical,   // Mejor modelo disponible sin límites
}

// ============================================================================
// COST OPTIMIZER PRINCIPAL
// ============================================================================

pub struct CostOptimizer {
    model_capabilities: HashMap<ModelChoice, ModelCapabilities>,
    usage_history: Vec<UsageRecord>,
    current_constraints: CostConstraints,
}

#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    pub cost_per_million_input: f64,
    pub cost_per_million_output: f64,
    pub performance_score: f64,      // 0.0 - 1.0
    pub thinking_support: bool,
    pub max_context_tokens: u32,
    pub specializations: Vec<TaskType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    SimpleQuery,
    ComplexReasoning,
    DataAnalysis,
    CreativeWriting,
    Mathematical,
    Multimodal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: std::time::SystemTime,
    pub model_used: ModelChoice,
    pub task_complexity: TaskComplexity,
    pub actual_cost: f64,
    pub success: bool,
    pub user_satisfaction: Option<f64>,
}

impl Default for CostOptimizer {
    fn default() -> Self {
        let mut model_capabilities = HashMap::new();
        
        // Gemini 2.0 Pro - Ultra económico
        model_capabilities.insert(ModelChoice::Gemini2Pro, ModelCapabilities {
            cost_per_million_input: 0.10,
            cost_per_million_output: 0.40,
            performance_score: 0.75,
            thinking_support: false,
            max_context_tokens: 2_000_000,
            specializations: vec![TaskType::SimpleQuery, TaskType::CodeGeneration],
        });
        
        // Gemini 2.5 Pro - Thinking model balanceado
        model_capabilities.insert(ModelChoice::Gemini25Pro, ModelCapabilities {
            cost_per_million_input: 1.25,
            cost_per_million_output: 10.00,
            performance_score: 0.90,
            thinking_support: true,
            max_context_tokens: 1_000_000,
            specializations: vec![TaskType::ComplexReasoning, TaskType::Mathematical, TaskType::CodeGeneration],
        });
        
        // Claude 3.5 Sonnet - Excelente para código
        model_capabilities.insert(ModelChoice::Claude35Sonnet, ModelCapabilities {
            cost_per_million_input: 3.00,
            cost_per_million_output: 15.00,
            performance_score: 0.88,
            thinking_support: false,
            max_context_tokens: 200_000,
            specializations: vec![TaskType::CodeGeneration, TaskType::DataAnalysis],
        });
        
        // Claude 3.7 Sonnet - Premium thinking
        model_capabilities.insert(ModelChoice::Claude37Sonnet, ModelCapabilities {
            cost_per_million_input: 3.00,
            cost_per_million_output: 15.00,
            performance_score: 0.95,
            thinking_support: true,
            max_context_tokens: 200_000,
            specializations: vec![TaskType::ComplexReasoning, TaskType::CodeGeneration, TaskType::Mathematical],
        });

        Self {
            model_capabilities,
            usage_history: Vec::new(),
            current_constraints: CostConstraints {
                max_cost_per_task: None,
                monthly_budget: None,
                current_month_spent: 0.0,
                priority_level: PriorityLevel::Balanced,
            },
        }
    }
}

impl CostOptimizer {
    pub fn new(constraints: CostConstraints) -> Self {
        let mut optimizer = Self::default();
        optimizer.current_constraints = constraints;
        optimizer
    }

    /// Selecciona el modelo óptimo basado en la complejidad de la tarea y restricciones
    pub fn select_optimal_model(&self, task_complexity: &TaskComplexity, task_description: &str) -> ModelChoice {
        // Detectar tipo de tarea automáticamente
        let task_type = self.detect_task_type(task_description);
        
        // Calcular score de complejidad general
        let complexity_score = (task_complexity.reasoning_required + 
                              task_complexity.code_complexity + 
                              task_complexity.context_length) / 3.0;

        // Aplicar lógica de selección
        match (&self.current_constraints.priority_level, complexity_score) {
            // Prioridad crítica -> Mejor modelo disponible
            (PriorityLevel::Critical, _) => {
                if task_complexity.thinking_needed {
                    ModelChoice::Claude37Sonnet
                } else {
                    ModelChoice::Claude35Sonnet
                }
            },
            
            // Tareas simples con prioridad de costo
            (PriorityLevel::Low, score) if score < 0.3 => {
                ModelChoice::Gemini2Pro  // 36x más barato que Claude
            },
            
            // Tareas que requieren thinking
            (_, _) if task_complexity.thinking_needed => {
                if complexity_score > 0.7 {
                    ModelChoice::Claude37Sonnet  // Premium thinking
                } else {
                    ModelChoice::Gemini25Pro     // Thinking balanceado
                }
            },
            
            // Tareas de código complejas
            (_, score) if matches!(task_type, TaskType::CodeGeneration) && score > 0.6 => {
                ModelChoice::Claude35Sonnet
            },
            
            // Caso balanceado por defecto
            (PriorityLevel::Balanced, score) => {
                if score > 0.7 {
                    ModelChoice::Gemini25Pro
                } else if score > 0.4 {
                    ModelChoice::Gemini25Flash
                } else {
                    ModelChoice::Gemini2Pro
                }
            },
            
            // Otros casos
            _ => ModelChoice::Gemini25Pro,
        }
    }

    /// Estima el costo de una tarea para un modelo específico
    pub fn estimate_cost(&self, model: &ModelChoice, input_tokens: u32, estimated_output_tokens: u32) -> CostEstimate {
        if let Some(capabilities) = self.model_capabilities.get(model) {
            let input_cost = (input_tokens as f64 / 1_000_000.0) * capabilities.cost_per_million_input;
            let output_cost = (estimated_output_tokens as f64 / 1_000_000.0) * capabilities.cost_per_million_output;
            
            CostEstimate {
                input_tokens,
                output_tokens: estimated_output_tokens,
                estimated_cost_usd: input_cost + output_cost,
                model_used: format!("{:?}", model),
            }
        } else {
            CostEstimate {
                input_tokens,
                output_tokens: estimated_output_tokens,
                estimated_cost_usd: 0.0,
                model_used: "Unknown".to_string(),
            }
        }
    }

    /// Verifica si una tarea excede las restricciones de costo
    pub fn check_cost_constraints(&self, estimated_cost: f64) -> Result<(), FlowError> {
        // Verificar límite por tarea
        if let Some(max_cost) = self.current_constraints.max_cost_per_task {
            if estimated_cost > max_cost {
                return Err(FlowError::CostLimitExceeded(max_cost));
            }
        }

        // Verificar presupuesto mensual
        if let Some(monthly_budget) = self.current_constraints.monthly_budget {
            if self.current_constraints.current_month_spent + estimated_cost > monthly_budget {
                return Err(FlowError::CostLimitExceeded(monthly_budget));
            }
        }

        Ok(())
    }

    /// Registra el uso de un modelo para aprendizaje futuro
    pub fn record_usage(&mut self, record: UsageRecord) {
        self.usage_history.push(record);
        
        // Mantener solo los últimos 1000 registros
        if self.usage_history.len() > 1000 {
            self.usage_history.remove(0);
        }
    }

    /// Obtiene estadísticas de uso y eficiencia
    pub fn get_usage_stats(&self) -> UsageStats {
        let total_records = self.usage_history.len();
        if total_records == 0 {
            return UsageStats::default();
        }

        let total_cost: f64 = self.usage_history.iter().map(|r| r.actual_cost).sum();
        let success_rate = self.usage_history.iter().filter(|r| r.success).count() as f64 / total_records as f64;
        let avg_satisfaction = self.usage_history.iter()
            .filter_map(|r| r.user_satisfaction)
            .sum::<f64>() / total_records as f64;

        UsageStats {
            total_tasks: total_records,
            total_cost_usd: total_cost,
            success_rate,
            avg_satisfaction,
            cost_per_successful_task: if success_rate > 0.0 { total_cost / (total_records as f64 * success_rate) } else { 0.0 },
        }
    }

    /// Detecta automáticamente el tipo de tarea basado en la descripción
    fn detect_task_type(&self, description: &str) -> TaskType {
        let description_lower = description.to_lowercase();
        
        if description_lower.contains("code") || description_lower.contains("function") || 
           description_lower.contains("class") || description_lower.contains("programming") {
            TaskType::CodeGeneration
        } else if description_lower.contains("analyze") || description_lower.contains("data") ||
                  description_lower.contains("statistics") {
            TaskType::DataAnalysis
        } else if description_lower.contains("math") || description_lower.contains("calculate") ||
                  description_lower.contains("equation") {
            TaskType::Mathematical
        } else if description_lower.contains("think") || description_lower.contains("reason") ||
                  description_lower.contains("complex") {
            TaskType::ComplexReasoning
        } else if description_lower.contains("image") || description_lower.contains("video") ||
                  description_lower.contains("audio") {
            TaskType::Multimodal
        } else if description_lower.contains("write") || description_lower.contains("story") ||
                  description_lower.contains("creative") {
            TaskType::CreativeWriting
        } else {
            TaskType::SimpleQuery
        }
    }

    /// Actualiza las restricciones de costo
    pub fn update_constraints(&mut self, constraints: CostConstraints) {
        self.current_constraints = constraints;
    }

    /// Obtiene recomendaciones de optimización
    pub fn get_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        let stats = self.get_usage_stats();

        // Recomendar modelos más económicos si la tasa de éxito es alta
        if stats.success_rate > 0.9 && stats.cost_per_successful_task > 0.01 {
            recommendations.push(OptimizationRecommendation {
                category: "Costo".to_string(),
                description: "Considera usar modelos más económicos como Gemini 2.0 Pro para tareas simples".to_string(),
                potential_savings: stats.cost_per_successful_task * 0.8, // 80% de ahorro estimado
            });
        }

        // Recomendar thinking mode si las tareas complejas fallan frecuentemente
        let complex_tasks_failed = self.usage_history.iter()
            .filter(|r| r.task_complexity.reasoning_required > 0.7 && !r.success)
            .count();
        
        if complex_tasks_failed > 2 {
            recommendations.push(OptimizationRecommendation {
                category: "Calidad".to_string(),
                description: "Activa el modo thinking para tareas complejas de razonamiento".to_string(),
                potential_savings: 0.0,
            });
        }

        recommendations
    }
}

// ============================================================================
// ESTRUCTURAS AUXILIARES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub total_tasks: usize,
    pub total_cost_usd: f64,
    pub success_rate: f64,
    pub avg_satisfaction: f64,
    pub cost_per_successful_task: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub description: String,
    pub potential_savings: f64,
}

// ============================================================================
// UTILIDADES PARA ANÁLISIS DE COMPLEJIDAD
// ============================================================================

pub fn analyze_task_complexity(description: &str) -> TaskComplexity {
    let description_lower = description.to_lowercase();
    let word_count = description.split_whitespace().count();
    
    // Analizar complejidad de razonamiento
    let reasoning_indicators = ["analyze", "explain", "reason", "think", "complex", "difficult", "solve"];
    let reasoning_score = reasoning_indicators.iter()
        .filter(|&word| description_lower.contains(word))
        .count() as f64 / reasoning_indicators.len() as f64;
    
    // Analizar complejidad de código
    let code_indicators = ["function", "class", "algorithm", "implement", "code", "programming", "api"];
    let code_score = code_indicators.iter()
        .filter(|&word| description_lower.contains(word))
        .count() as f64 / code_indicators.len() as f64;
    
    // Analizar longitud del contexto
    let context_score = (word_count as f64 / 100.0).min(1.0); // Normalizar a 0-1
    
    // Detectar necesidad de thinking
    let thinking_indicators = ["step by step", "explain reasoning", "think through", "analyze carefully"];
    let thinking_needed = thinking_indicators.iter()
        .any(|&phrase| description_lower.contains(phrase)) || reasoning_score > 0.6;
    
    // Detectar multimodal
    let multimodal_indicators = ["image", "video", "audio", "visual", "picture"];
    let multimodal = multimodal_indicators.iter()
        .any(|&word| description_lower.contains(word));

    TaskComplexity {
        reasoning_required: reasoning_score,
        code_complexity: code_score,
        context_length: context_score,
        thinking_needed,
        multimodal,
    }
} 