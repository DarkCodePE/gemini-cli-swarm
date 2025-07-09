// ============================================================================
// PERFORMANCE MONITOR - Sistema de Monitoreo de Métricas en Tiempo Real
// ============================================================================
// Este módulo implementa el monitoreo de performance para alcanzar los
// estándares de Claude-Flow: 84.8% success rate, 2.8-4.4x speed improvement
// ============================================================================

use crate::{CodeGenerationResult, FlowError, cost_optimizer::ModelChoice};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, Duration, Instant};

// ============================================================================
// MÉTRICAS PRINCIPALES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub success_rate: f64,              // Target: 84.8%
    pub avg_response_time_ms: f64,      // Target: optimized
    pub speed_improvement_factor: f64,  // Target: 2.8-4.4x
    pub cost_efficiency: f64,           // Costo/tarea exitosa
    pub user_satisfaction: f64,         // 0.0 - 5.0
    pub throughput_tasks_per_hour: f64,
    pub error_rate: f64,
    pub avg_tokens_per_second: f64,
}

#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub id: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration_ms: Option<u64>,
    pub model_used: ModelChoice,
    pub success: bool,
    pub error: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost: f64,
    pub user_rating: Option<f64>,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model: ModelChoice,
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub avg_response_time_ms: f64,
    pub avg_cost_per_task: f64,
    pub avg_user_rating: f64,
    pub specialization_performance: HashMap<String, f64>, // task_type -> success_rate
}

// ============================================================================
// PERFORMANCE MONITOR PRINCIPAL
// ============================================================================

pub struct PerformanceMonitor {
    task_history: VecDeque<TaskExecution>,
    model_performance: HashMap<ModelChoice, ModelPerformance>,
    baseline_metrics: Option<PerformanceMetrics>,
    real_time_buffer: VecDeque<RealTimeMetric>,
    alert_thresholds: AlertThresholds,
    session_start: Instant,
}

#[derive(Debug, Clone)]
struct RealTimeMetric {
    timestamp: Instant,
    metric_type: MetricType,
    value: f64,
}

#[derive(Debug, Clone)]
enum MetricType {
    ResponseTime,
    SuccessRate,
    TokensPerSecond,
    CostPerTask,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub min_success_rate: f64,          // 0.80 (80%)
    pub max_response_time_ms: f64,      // 30000 (30s)
    pub max_cost_per_task: f64,         // $0.50
    pub min_tokens_per_second: f64,     // 50
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_success_rate: 0.80,
            max_response_time_ms: 30000.0,
            max_cost_per_task: 0.50,
            min_tokens_per_second: 50.0,
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            task_history: VecDeque::with_capacity(1000),
            model_performance: HashMap::new(),
            baseline_metrics: None,
            real_time_buffer: VecDeque::with_capacity(100),
            alert_thresholds: AlertThresholds::default(),
            session_start: Instant::now(),
        }
    }
}

impl PerformanceMonitor {
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self {
            alert_thresholds: thresholds,
            ..Default::default()
        }
    }

    /// Inicia el tracking de una nueva tarea
    pub fn start_task(&mut self, task_id: String, model: ModelChoice, complexity_score: f64) -> TaskTracker {
        let task = TaskExecution {
            id: task_id.clone(),
            start_time: SystemTime::now(),
            end_time: None,
            duration_ms: None,
            model_used: model.clone(),
            success: false,
            error: None,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
            user_rating: None,
            complexity_score,
        };

        TaskTracker {
            task_id,
            start_instant: Instant::now(),
            monitor: self as *mut PerformanceMonitor,
        }
    }

    /// Completa el tracking de una tarea
    pub fn complete_task(&mut self, 
        task_id: &str, 
        result: Result<&CodeGenerationResult, &FlowError>,
        user_rating: Option<f64>
    ) {
        let mut task_data_for_updates: Option<TaskExecution> = None;
        
        if let Some(task) = self.task_history.iter_mut().find(|t| t.id == task_id) {
            task.end_time = Some(SystemTime::now());
            
            if let Ok(duration_since) = task.end_time.unwrap().duration_since(task.start_time) {
                task.duration_ms = Some(duration_since.as_millis() as u64);
            }

            match result {
                Ok(code_result) => {
                    task.success = code_result.verification_passed;
                    task.output_tokens = code_result.code.split_whitespace().count() as u32; // Estimación
                    if let Some(cost_estimate) = &code_result.cost_estimate {
                        task.cost = cost_estimate.estimated_cost_usd;
                        task.input_tokens = cost_estimate.input_tokens;
                        task.output_tokens = cost_estimate.output_tokens;
                    }
                },
                Err(error) => {
                    task.success = false;
                    task.error = Some(error.to_string());
                }
            }

            task.user_rating = user_rating;

            // Crear copia para las actualizaciones
            task_data_for_updates = Some(task.clone());
        }

        // Actualizar métricas usando la copia (después de liberar el borrow)
        if let Some(task_data) = task_data_for_updates {
            self.update_real_time_metrics(&task_data);
            self.update_model_performance(&task_data);
        }

        // Mantener buffer de tareas limitado
        if self.task_history.len() > 1000 {
            self.task_history.pop_front();
        }
    }

    /// Calcula métricas actuales de performance
    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        let recent_tasks: Vec<_> = self.task_history.iter()
            .filter(|task| {
                if let Some(end_time) = task.end_time {
                    if let Ok(duration) = SystemTime::now().duration_since(end_time) {
                        return duration.as_secs() < 3600; // Últimas 1 hora
                    }
                }
                false
            })
            .collect();

        if recent_tasks.is_empty() {
            return PerformanceMetrics {
                success_rate: 0.0,
                avg_response_time_ms: 0.0,
                speed_improvement_factor: 1.0,
                cost_efficiency: 0.0,
                user_satisfaction: 0.0,
                throughput_tasks_per_hour: 0.0,
                error_rate: 0.0,
                avg_tokens_per_second: 0.0,
            };
        }

        let total_tasks = recent_tasks.len();
        let successful_tasks = recent_tasks.iter().filter(|t| t.success).count();
        let success_rate = successful_tasks as f64 / total_tasks as f64;

        let avg_response_time: f64 = recent_tasks.iter()
            .filter_map(|t| t.duration_ms)
            .map(|d| d as f64)
            .sum::<f64>() / total_tasks as f64;

        let total_cost: f64 = recent_tasks.iter().map(|t| t.cost).sum();
        let cost_efficiency = if successful_tasks > 0 {
            total_cost / successful_tasks as f64
        } else {
            0.0
        };

        let avg_user_satisfaction: f64 = recent_tasks.iter()
            .filter_map(|t| t.user_rating)
            .sum::<f64>() / recent_tasks.iter().filter(|t| t.user_rating.is_some()).count().max(1) as f64;

        let session_duration_hours = self.session_start.elapsed().as_secs_f64() / 3600.0;
        let throughput = total_tasks as f64 / session_duration_hours.max(1.0/3600.0); // Min 1 segundo

        let error_rate = 1.0 - success_rate;

        let avg_tokens_per_second: f64 = recent_tasks.iter()
            .filter_map(|task| {
                task.duration_ms.map(|duration| {
                    let total_tokens = task.input_tokens + task.output_tokens;
                    total_tokens as f64 / (duration as f64 / 1000.0)
                })
            })
            .sum::<f64>() / total_tasks as f64;

        // Calcular mejora de velocidad (comparar con baseline si existe)
        let speed_improvement_factor = if let Some(baseline) = &self.baseline_metrics {
            baseline.avg_response_time_ms / avg_response_time.max(1.0)
        } else {
            1.0 // Sin baseline, no hay mejora medible
        };

        PerformanceMetrics {
            success_rate,
            avg_response_time_ms: avg_response_time,
            speed_improvement_factor,
            cost_efficiency,
            user_satisfaction: avg_user_satisfaction,
            throughput_tasks_per_hour: throughput,
            error_rate,
            avg_tokens_per_second,
        }
    }

    /// Establece métricas baseline para comparación
    pub fn set_baseline(&mut self, metrics: PerformanceMetrics) {
        self.baseline_metrics = Some(metrics);
    }

    /// Obtiene alertas basadas en métricas actuales
    pub fn get_alerts(&self) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        let metrics = self.get_current_metrics();

        if metrics.success_rate < self.alert_thresholds.min_success_rate {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::High,
                category: "Success Rate".to_string(),
                message: format!("Tasa de éxito baja: {:.1}% (mínimo: {:.1}%)", 
                    metrics.success_rate * 100.0, 
                    self.alert_thresholds.min_success_rate * 100.0),
                recommendation: "Considera usar modelos más robustos o activar modo thinking".to_string(),
            });
        }

        if metrics.avg_response_time_ms > self.alert_thresholds.max_response_time_ms {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Medium,
                category: "Response Time".to_string(),
                message: format!("Tiempo de respuesta alto: {:.0}ms (máximo: {:.0}ms)", 
                    metrics.avg_response_time_ms, 
                    self.alert_thresholds.max_response_time_ms),
                recommendation: "Considera usar modelos más rápidos como Gemini Flash".to_string(),
            });
        }

        if metrics.cost_efficiency > self.alert_thresholds.max_cost_per_task {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Low,
                category: "Cost Efficiency".to_string(),
                message: format!("Costo por tarea alto: ${:.3} (máximo: ${:.3})", 
                    metrics.cost_efficiency, 
                    self.alert_thresholds.max_cost_per_task),
                recommendation: "Considera optimizar selección de modelos para tareas simples".to_string(),
            });
        }

        if metrics.avg_tokens_per_second < self.alert_thresholds.min_tokens_per_second {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Medium,
                category: "Throughput".to_string(),
                message: format!("Throughput bajo: {:.1} tokens/s (mínimo: {:.1})", 
                    metrics.avg_tokens_per_second, 
                    self.alert_thresholds.min_tokens_per_second),
                recommendation: "Verifica conectividad de red o considera modelos más eficientes".to_string(),
            });
        }

        alerts
    }

    /// Obtiene performance por modelo
    pub fn get_model_performance(&self) -> Vec<ModelPerformance> {
        self.model_performance.values().cloned().collect()
    }

    /// Obtiene reporte detallado de performance
    pub fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.get_current_metrics();
        let alerts = self.get_alerts();
        let model_performance = self.get_model_performance();

        let claude_flow_comparison = ClaudeFlowComparison {
            target_success_rate: 0.848,
            current_success_rate: metrics.success_rate,
            target_speed_improvement: 3.6, // Promedio de 2.8-4.4x
            current_speed_improvement: metrics.speed_improvement_factor,
            performance_gap: (0.848 - metrics.success_rate).max(0.0),
        };

        PerformanceReport {
            timestamp: SystemTime::now(),
            current_metrics: metrics,
            alerts,
            model_performance,
            claude_flow_comparison,
            recommendations: self.generate_recommendations(),
        }
    }

    /// Actualiza métricas en tiempo real
    fn update_real_time_metrics(&mut self, task: &TaskExecution) {
        let now = Instant::now();

        if let Some(duration) = task.duration_ms {
            self.real_time_buffer.push_back(RealTimeMetric {
                timestamp: now,
                metric_type: MetricType::ResponseTime,
                value: duration as f64,
            });
        }

        self.real_time_buffer.push_back(RealTimeMetric {
            timestamp: now,
            metric_type: MetricType::SuccessRate,
            value: if task.success { 1.0 } else { 0.0 },
        });

        self.real_time_buffer.push_back(RealTimeMetric {
            timestamp: now,
            metric_type: MetricType::CostPerTask,
            value: task.cost,
        });

        if let Some(duration) = task.duration_ms {
            let tokens_per_second = (task.input_tokens + task.output_tokens) as f64 / (duration as f64 / 1000.0);
            self.real_time_buffer.push_back(RealTimeMetric {
                timestamp: now,
                metric_type: MetricType::TokensPerSecond,
                value: tokens_per_second,
            });
        }

        // Mantener buffer limitado (últimos 100 metrics)
        while self.real_time_buffer.len() > 100 {
            self.real_time_buffer.pop_front();
        }
    }

    /// Actualiza performance específica del modelo
    fn update_model_performance(&mut self, task: &TaskExecution) {
        let performance = self.model_performance.entry(task.model_used.clone())
            .or_insert_with(|| ModelPerformance {
                model: task.model_used.clone(),
                total_tasks: 0,
                successful_tasks: 0,
                avg_response_time_ms: 0.0,
                avg_cost_per_task: 0.0,
                avg_user_rating: 0.0,
                specialization_performance: HashMap::new(),
            });

        performance.total_tasks += 1;
        if task.success {
            performance.successful_tasks += 1;
        }

        // Actualizar promedio de tiempo de respuesta
        if let Some(duration) = task.duration_ms {
            performance.avg_response_time_ms = 
                (performance.avg_response_time_ms * (performance.total_tasks - 1) as f64 + duration as f64) / 
                performance.total_tasks as f64;
        }

        // Actualizar promedio de costo
        performance.avg_cost_per_task = 
            (performance.avg_cost_per_task * (performance.total_tasks - 1) as f64 + task.cost) / 
            performance.total_tasks as f64;

        // Actualizar rating promedio
        if let Some(rating) = task.user_rating {
            let current_rating_count = performance.total_tasks - if task.user_rating.is_some() { 0 } else { 1 };
            performance.avg_user_rating = 
                (performance.avg_user_rating * current_rating_count as f64 + rating) / 
                (current_rating_count + 1) as f64;
        }
    }

    /// Genera recomendaciones basadas en métricas actuales
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let metrics = self.get_current_metrics();

        if metrics.success_rate < 0.85 {
            recommendations.push("Considera activar modo thinking para tareas complejas para mejorar la tasa de éxito".to_string());
        }

        if metrics.cost_efficiency > 0.10 {
            recommendations.push("Usa Gemini 2.0 Pro para tareas simples para reducir costos hasta 36x".to_string());
        }

        if metrics.avg_response_time_ms > 15000.0 {
            recommendations.push("Considera usar Gemini Flash para mejorar velocidad de respuesta".to_string());
        }

        if metrics.speed_improvement_factor < 2.8 {
            recommendations.push("Optimiza la selección de modelos para alcanzar el objetivo de 2.8-4.4x de mejora".to_string());
        }

        recommendations
    }
}

// ============================================================================
// ESTRUCTURAS AUXILIARES
// ============================================================================

pub struct TaskTracker {
    task_id: String,
    start_instant: Instant,
    monitor: *mut PerformanceMonitor,
}

impl TaskTracker {
    pub fn complete(self, result: Result<&CodeGenerationResult, &FlowError>, user_rating: Option<f64>) {
        unsafe {
            (*self.monitor).complete_task(&self.task_id, result, user_rating);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub severity: AlertSeverity,
    pub category: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: SystemTime,
    pub current_metrics: PerformanceMetrics,
    pub alerts: Vec<PerformanceAlert>,
    pub model_performance: Vec<ModelPerformance>,
    pub claude_flow_comparison: ClaudeFlowComparison,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeFlowComparison {
    pub target_success_rate: f64,
    pub current_success_rate: f64,
    pub target_speed_improvement: f64,
    pub current_speed_improvement: f64,
    pub performance_gap: f64,
}

// ============================================================================
// UTILIDADES DE EXPORT Y ANÁLISIS
// ============================================================================

impl PerformanceMonitor {
    /// Exporta métricas en formato JSON
    pub fn export_metrics_json(&self) -> Result<String, serde_json::Error> {
        let report = self.get_performance_report();
        serde_json::to_string_pretty(&report)
    }

    /// Obtiene tendencias de performance
    pub fn get_performance_trends(&self, hours: u64) -> PerformanceTrends {
        let cutoff_time = SystemTime::now() - Duration::from_secs(hours * 3600);
        
        let relevant_tasks: Vec<_> = self.task_history.iter()
            .filter(|task| task.start_time > cutoff_time)
            .collect();

        if relevant_tasks.is_empty() {
            return PerformanceTrends::default();
        }

        // Calcular tendencias por hora
        let mut hourly_metrics = Vec::new();
        for hour in 0..hours {
            let hour_start = cutoff_time + Duration::from_secs(hour * 3600);
            let hour_end = hour_start + Duration::from_secs(3600);
            
            let hour_tasks: Vec<_> = relevant_tasks.iter()
                .filter(|task| task.start_time >= hour_start && task.start_time < hour_end)
                .collect();

            if !hour_tasks.is_empty() {
                let success_rate = hour_tasks.iter().filter(|t| t.success).count() as f64 / hour_tasks.len() as f64;
                let avg_response_time = hour_tasks.iter()
                    .filter_map(|t| t.duration_ms)
                    .map(|d| d as f64)
                    .sum::<f64>() / hour_tasks.len() as f64;

                hourly_metrics.push(HourlyMetric {
                    hour: hour as u32,
                    success_rate,
                    avg_response_time_ms: avg_response_time,
                    task_count: hour_tasks.len(),
                });
            }
        }

        let overall_trend = if hourly_metrics.len() >= 2 {
            let first_success = hourly_metrics.first().unwrap().success_rate;
            let last_success = hourly_metrics.last().unwrap().success_rate;
            if last_success > first_success { TrendDirection::Improving } 
            else if last_success < first_success { TrendDirection::Degrading }
            else { TrendDirection::Stable }
        } else {
            TrendDirection::Stable
        };

        PerformanceTrends {
            hourly_metrics,
            overall_trend,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceTrends {
    pub hourly_metrics: Vec<HourlyMetric>,
    pub overall_trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetric {
    pub hour: u32,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TrendDirection {
    #[default]
    Stable,
    Improving,
    Degrading,
} 