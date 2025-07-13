// ============================================================================
// COST OPTIMIZER - Optimizador de Costos para Modelos de IA
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelChoice {
    Gemini15Flash,
    Gemini15Pro,
    Gemini15ProExp,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,
    Medium,
    Complex,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConstraints {
    pub max_cost_per_request: Option<f64>,
    pub daily_budget: Option<f64>,
    pub priority: PriorityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub model: ModelChoice,
    pub reason: String,
    pub estimated_cost: f64,
    pub confidence: f64,
}

pub struct CostOptimizer;

impl CostOptimizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn optimize_model_selection(
        &self,
        complexity: TaskComplexity,
        constraints: &CostConstraints,
    ) -> ModelChoice {
        match (complexity, &constraints.priority) {
            (TaskComplexity::Simple, _) => ModelChoice::Gemini15Flash,
            (TaskComplexity::Medium, PriorityLevel::Low) => ModelChoice::Gemini15Flash,
            (TaskComplexity::Medium, _) => ModelChoice::Gemini15Pro,
            (TaskComplexity::Complex, PriorityLevel::Critical) => ModelChoice::Gemini15ProExp,
            (TaskComplexity::Complex, _) => ModelChoice::Gemini15Pro,
            (TaskComplexity::Critical, _) => ModelChoice::Gemini15ProExp,
        }
    }
    
    pub fn get_recommendations(&self, task: &str) -> Vec<OptimizationRecommendation> {
        let _complexity = analyze_task_complexity(task);
        vec![
            OptimizationRecommendation {
                model: ModelChoice::Gemini15Flash,
                reason: "Modelo rápido y económico para tareas simples".to_string(),
                estimated_cost: 0.01,
                confidence: 0.8,
            }
        ]
    }
}

pub fn analyze_task_complexity(task: &str) -> TaskComplexity {
    let task_lower = task.to_lowercase();
    
    if task_lower.contains("simple") || task_lower.contains("básico") {
        TaskComplexity::Simple
    } else if task_lower.contains("complejo") || task_lower.contains("avanzado") {
        TaskComplexity::Complex
    } else if task_lower.contains("crítico") || task_lower.contains("urgente") {
        TaskComplexity::Critical
    } else {
        TaskComplexity::Medium
    }
} 