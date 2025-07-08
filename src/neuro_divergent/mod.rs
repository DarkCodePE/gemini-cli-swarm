// ============================================================================
// NEURO-DIVERGENT MODELS - Catálogo de Cerebros Especializados
// ============================================================================
// Este módulo contiene los "planos" de diferentes arquitecturas de redes
// neuronales optimizadas para tareas específicas, siguiendo el patrón
// neuro-divergente de ruvnet.
// ============================================================================

use serde::{Deserialize, Serialize};
// Temporalmente comentado debido a problemas con submódulos
// use ruv_fann::Network;

// ============================================================================
// TIPOS DE MODELOS ESPECIALIZADOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    /// LSTM - Ideal para secuencias temporales largas
    LSTM {
        hidden_size: usize,
        num_layers: usize,
        dropout: f64,
    },
    /// TCN - Temporal Convolutional Network para análisis de series
    TCN {
        num_channels: usize,
        kernel_size: usize,
        dropout: f64,
    },
    /// N-BEATS - Neural Basis Expansion Analysis para forecasting
    NBEATS {
        forecast_length: usize,
        backcast_length: usize,
        hidden_layer_units: usize,
    },
    /// Transformer - Para procesamiento de lenguaje y patrones complejos
    Transformer {
        d_model: usize,
        num_heads: usize,
        num_layers: usize,
        max_seq_length: usize,
    },
    /// CNN - Convolutional Neural Network para patrones espaciales
    CNN {
        num_filters: usize,
        filter_size: usize,
        pooling_size: usize,
    },
    /// Custom FANN - Red neuronal personalizada usando ruv-FANN
    CustomFANN {
        layers: Vec<usize>,
        activation: ActivationType,
        learning_rate: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationType {
    Linear,
    Sigmoid,
    SigmoidStepwise,
    SigmoidSymmetric,
    SigmoidSymmetricStepwise,
    Tanh,
    TanhStepwise,
    Threshold,
    ThresholdSymmetric,
    LinearPiece,
    LinearPieceSymmetric,
    SinSymmetric,
    CosSymmetric,
    Sin,
    Cos,
}

// ============================================================================
// CAPACIDADES DE MODELOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub can_handle_sequences: bool,
    pub can_handle_text: bool,
    pub can_handle_images: bool,
    pub can_handle_tabular: bool,
    pub optimal_for_forecasting: bool,
    pub supports_online_learning: bool,
    pub memory_efficient: bool,
    pub gpu_optimized: bool,
}

// ============================================================================
// ESPECIFICACIÓN DE MODELOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub model_type: ModelType,
    pub capabilities: ModelCapabilities,
    pub description: String,
    pub use_cases: Vec<String>,
    pub performance_score: f64, // 0.0 - 1.0
}

// ============================================================================
// CATÁLOGO DE MODELOS PREDEFINIDOS
// ============================================================================

pub struct ModelCatalog;

impl ModelCatalog {
    /// Obtiene todos los modelos disponibles en el catálogo
    pub fn get_available_models() -> Vec<ModelSpec> {
        vec![
            // LSTM para series temporales
            ModelSpec {
                model_type: ModelType::LSTM {
                    hidden_size: 128,
                    num_layers: 2,
                    dropout: 0.2,
                },
                capabilities: ModelCapabilities {
                    can_handle_sequences: true,
                    can_handle_text: true,
                    can_handle_images: false,
                    can_handle_tabular: true,
                    optimal_for_forecasting: true,
                    supports_online_learning: false,
                    memory_efficient: false,
                    gpu_optimized: true,
                },
                description: "LSTM optimizada para análisis de series temporales y secuencias".to_string(),
                use_cases: vec![
                    "Predicción de ventas".to_string(),
                    "Análisis de sensores IoT".to_string(),
                    "Procesamiento de texto secuencial".to_string(),
                ],
                performance_score: 0.85,
            },
            
            // N-BEATS para forecasting avanzado
            ModelSpec {
                model_type: ModelType::NBEATS {
                    forecast_length: 24,
                    backcast_length: 168,
                    hidden_layer_units: 512,
                },
                capabilities: ModelCapabilities {
                    can_handle_sequences: true,
                    can_handle_text: false,
                    can_handle_images: false,
                    can_handle_tabular: true,
                    optimal_for_forecasting: true,
                    supports_online_learning: false,
                    memory_efficient: true,
                    gpu_optimized: true,
                },
                description: "N-BEATS para forecasting de alta precisión sin características externas".to_string(),
                use_cases: vec![
                    "Predicción de demanda energética".to_string(),
                    "Forecasting financiero".to_string(),
                    "Planificación de inventario".to_string(),
                ],
                performance_score: 0.92,
            },
            
            // Transformer para LLM y procesamiento complejo
            ModelSpec {
                model_type: ModelType::Transformer {
                    d_model: 512,
                    num_heads: 8,
                    num_layers: 6,
                    max_seq_length: 2048,
                },
                capabilities: ModelCapabilities {
                    can_handle_sequences: true,
                    can_handle_text: true,
                    can_handle_images: false,
                    can_handle_tabular: false,
                    optimal_for_forecasting: false,
                    supports_online_learning: false,
                    memory_efficient: false,
                    gpu_optimized: true,
                },
                description: "Transformer para comprensión de lenguaje y patrones complejos".to_string(),
                use_cases: vec![
                    "Generación de código".to_string(),
                    "Análisis de documentos".to_string(),
                    "Traducción automática".to_string(),
                ],
                performance_score: 0.88,
            },
            
            // ruv-FANN personalizable
            ModelSpec {
                model_type: ModelType::CustomFANN {
                    layers: vec![10, 15, 10, 1],
                    activation: ActivationType::SigmoidSymmetric,
                    learning_rate: 0.01,
                },
                capabilities: ModelCapabilities {
                    can_handle_sequences: false,
                    can_handle_text: false,
                    can_handle_images: false,
                    can_handle_tabular: true,
                    optimal_for_forecasting: false,
                    supports_online_learning: true,
                    memory_efficient: true,
                    gpu_optimized: false,
                },
                description: "Red neuronal FANN completamente personalizable para tareas generales".to_string(),
                use_cases: vec![
                    "Clasificación general".to_string(),
                    "Regresión simple".to_string(),
                    "Prototipado rápido".to_string(),
                ],
                performance_score: 0.75,
            },
        ]
    }
    
    /// Selecciona el mejor modelo para una tarea específica
    pub fn select_best_model_for_task(task_description: &str) -> Option<ModelSpec> {
        let models = Self::get_available_models();
        let task_lower = task_description.to_lowercase();
        
        // Lógica simple de selección basada en palabras clave
        if task_lower.contains("predicción") || task_lower.contains("forecasting") || task_lower.contains("serie") {
            // Para tareas de predicción, preferir N-BEATS o LSTM
            if task_lower.contains("alta precisión") || task_lower.contains("avanzado") {
                models.into_iter().find(|m| matches!(m.model_type, ModelType::NBEATS { .. }))
            } else {
                models.into_iter().find(|m| matches!(m.model_type, ModelType::LSTM { .. }))
            }
        } else if task_lower.contains("código") || task_lower.contains("texto") || task_lower.contains("lenguaje") {
            // Para tareas de código/texto, usar Transformer
            models.into_iter().find(|m| matches!(m.model_type, ModelType::Transformer { .. }))
        } else {
            // Para tareas generales, usar ruv-FANN personalizable
            models.into_iter().find(|m| matches!(m.model_type, ModelType::CustomFANN { .. }))
        }
    }
}

// ============================================================================
// BUILDER DE MODELOS USANDO ruv-FANN - TEMPORALMENTE DESHABILITADO
// ============================================================================
// Comentado temporalmente debido a problemas con submódulos de ruv-fann

// pub struct ModelBuilder;

// impl ModelBuilder {
//     /// Construye una instancia física del modelo usando ruv-FANN
//     pub fn build_fann_network(spec: &ModelSpec) -> Result<Network<f64>, String> {
//         match &spec.model_type {
//             ModelType::CustomFANN { layers, .. } => {
//                 Ok(Network::new(layers))
//             }
//             ModelType::LSTM { hidden_size, num_layers, .. } => {
//                 // Aproximación LSTM usando FANN multicapa
//                 let mut lstm_layers = vec![*hidden_size]; // Input
//                 for _ in 0..*num_layers {
//                     lstm_layers.push(*hidden_size);
//                 }
//                 lstm_layers.push(*hidden_size / 4); // Output reducido
//                 Ok(Network::new(&lstm_layers))
//             }
//             ModelType::NBEATS { forecast_length, backcast_length, hidden_layer_units } => {
//                 // Aproximación N-BEATS usando FANN
//                 let layers = vec![
//                     *backcast_length,
//                     *hidden_layer_units,
//                     *hidden_layer_units / 2,
//                     *forecast_length,
//                 ];
//                 Ok(Network::new(&layers))
//             }
//             ModelType::Transformer { d_model, num_heads, .. } => {
//                 // Aproximación Transformer usando FANN
//                 let layers = vec![
//                     *d_model,
//                     *d_model * 2,
//                     *num_heads * 64,
//                     *d_model,
//                 ];
//                 Ok(Network::new(&layers))
//             }
//             ModelType::TCN { num_channels, .. } => {
//                 // Aproximación TCN usando FANN
//                 let layers = vec![*num_channels, *num_channels * 2, *num_channels, 1];
//                 Ok(Network::new(&layers))
//             }
//             ModelType::CNN { num_filters, .. } => {
//                 // Aproximación CNN usando FANN
//                 let layers = vec![*num_filters * 8, *num_filters * 4, *num_filters, 1];
//                 Ok(Network::new(&layers))
//             }
//         }
//     }
// } 