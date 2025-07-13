// ============================================================================
// PERFORMANCE MONITOR - Monitor de Rendimiento del Sistema
// ============================================================================

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub response_time_ms: u64,
    pub error_rate: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            response_time_ms: 5000,
            error_rate: 0.05,
            memory_usage_mb: 1024,
            cpu_usage_percent: 80.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub success_rate: f64,
    pub average_response_time_ms: u64,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub uptime_seconds: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            success_rate: 1.0,
            average_response_time_ms: 100,
            total_requests: 0,
            failed_requests: 0,
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
            uptime_seconds: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: String,
    pub metrics: PerformanceMetrics,
    pub alerts: Vec<PerformanceAlert>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
}

pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: PerformanceMetrics,
    thresholds: AlertThresholds,
    request_times: Vec<Duration>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: PerformanceMetrics::default(),
            thresholds: AlertThresholds::default(),
            request_times: Vec::new(),
        }
    }
    
    pub fn with_thresholds(thresholds: AlertThresholds) -> Self {
        Self {
            start_time: Instant::now(),
            metrics: PerformanceMetrics::default(),
            thresholds,
            request_times: Vec::new(),
        }
    }
    
    pub fn record_request(&mut self, duration: Duration, success: bool) {
        self.metrics.total_requests += 1;
        if !success {
            self.metrics.failed_requests += 1;
        }
        
        self.request_times.push(duration);
        
        // Mantener solo los últimos 100 tiempos de respuesta
        if self.request_times.len() > 100 {
            self.request_times.remove(0);
        }
        
        // Actualizar métricas
        self.update_metrics();
    }
    
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    pub fn get_report(&self) -> PerformanceReport {
        let alerts = self.check_alerts();
        let recommendations = self.generate_recommendations(&alerts);
        
        PerformanceReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics: self.metrics.clone(),
            alerts,
            recommendations,
        }
    }
    
    fn update_metrics(&mut self) {
        // Calcular tasa de éxito
        if self.metrics.total_requests > 0 {
            self.metrics.success_rate = 1.0 - (self.metrics.failed_requests as f64 / self.metrics.total_requests as f64);
        }
        
        // Calcular tiempo promedio de respuesta
        if !self.request_times.is_empty() {
            let total_ms: u64 = self.request_times.iter()
                .map(|d| d.as_millis() as u64)
                .sum();
            self.metrics.average_response_time_ms = total_ms / self.request_times.len() as u64;
        }
        
        // Actualizar uptime
        self.metrics.uptime_seconds = self.start_time.elapsed().as_secs();
    }
    
    fn check_alerts(&self) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        
        // Verificar tiempo de respuesta
        if self.metrics.average_response_time_ms > self.thresholds.response_time_ms {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::High,
                message: "Tiempo de respuesta elevado".to_string(),
                metric_name: "response_time".to_string(),
                current_value: self.metrics.average_response_time_ms as f64,
                threshold: self.thresholds.response_time_ms as f64,
            });
        }
        
        // Verificar tasa de error
        let error_rate = 1.0 - self.metrics.success_rate;
        if error_rate > self.thresholds.error_rate {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Critical,
                message: "Tasa de error elevada".to_string(),
                metric_name: "error_rate".to_string(),
                current_value: error_rate,
                threshold: self.thresholds.error_rate,
            });
        }
        
        alerts
    }
    
    fn generate_recommendations(&self, alerts: &[PerformanceAlert]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for alert in alerts {
            match alert.metric_name.as_str() {
                "response_time" => {
                    recommendations.push("Considera optimizar las consultas a la base de datos".to_string());
                    recommendations.push("Implementa caché para respuestas frecuentes".to_string());
                }
                "error_rate" => {
                    recommendations.push("Revisa los logs para identificar errores comunes".to_string());
                    recommendations.push("Implementa reintentos automáticos para fallos transitorios".to_string());
                }
                _ => {}
            }
        }
        
        recommendations
    }
} 