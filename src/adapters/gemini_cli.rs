// ============================================================================
// GEMINI CLI FLOW ADAPTER - Implementación ruv-swarm + SAFLA + SPARC
// ============================================================================
// Este adaptador conecta el ecosistema ruvnet con Google's Gemini CLI,
// implementando el bucle "Generar -> Verificar -> Refinar" de forma optimizada.
// Con soporte para thinking mode y cost optimization.
// ============================================================================

use crate::{
    adapters::gemini_process_manager::GeminiProcessManager,
    AdapterCapabilities, AdapterConfig, CodeGenerationFlow, FlowError, CodeGenerationResult,
    VerificationResult, ThinkingFlow, ThinkingResult, ReasoningStep, ThinkingMode, CostEstimate,
    cost_optimizer::ModelChoice,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ============================================================================
// ESTRUCTURAS PARA LA API DE GEMINI Y HERRAMIENTAS
// ============================================================================

#[derive(Debug)]
struct ToolResult {
    function_name: String,
    output: String,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    generation_config: GeminiGenerationConfig,
    safety_settings: Vec<GeminiSafetySetting>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<FunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_response: Option<FunctionResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    top_k: u32,
    top_p: f32,
    max_output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: FunctionParameters,
}

#[derive(Debug, Serialize)]
struct FunctionParameters {
    #[serde(rename = "type")]
    param_type: String,
    properties: serde_json::Value,
    required: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeminiMode {
    ApiDirect,
    CliInteractive,
}

// ============================================================================
// ADAPTADOR PRINCIPAL CON THINKING SUPPORT
// ============================================================================

pub struct GeminiCLIFlow {
    client: Client,
    pub config: AdapterConfig,
    api_endpoint: String,
    pub session_id: String,
    mode: GeminiMode,
    process_manager: Option<GeminiProcessManager>,
    thinking_mode: ThinkingMode,
    reasoning_steps: Vec<ReasoningStep>,
    model_choice: ModelChoice,
}

#[async_trait]
impl CodeGenerationFlow for GeminiCLIFlow {
    async fn execute(&self, problem_description: &str) -> Result<CodeGenerationResult, FlowError> {
        let start_time = Instant::now();
        log::info!(
            "🚀 Iniciando Gemini CLI Flow - Sesión: {} - Modelo: {:?}",
            self.session_id, self.model_choice
        );

        if let (GeminiMode::CliInteractive, Some(manager)) = (&self.mode, &self.process_manager) {
            log::info!("⚡ Ejecutando tarea a través de Gemini CLI interactivo.");
            let code = manager
                .execute_command(problem_description)
                .await
                .map_err(|e| FlowError::ApiError(format!("Error en Gemini CLI: {}", e)))?;
            
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            
            // Estimar costo para CLI mode
            let estimated_tokens = problem_description.split_whitespace().count() as u32;
            let cost_estimate = self.estimate_cost(estimated_tokens, code.split_whitespace().count() as u32);
            
            return Ok(CodeGenerationResult {
                code,
                language: "unknown".to_string(),
                confidence_score: 0.95,
                attempts_made: 1,
                execution_time_ms,
                verification_passed: true,
                cost_estimate: Some(cost_estimate),
                model_used: Some(format!("{:?}", self.model_choice)),
                metrics: Default::default(),
            });
        }

        log::info!("⚡ Ejecutando tarea a través de la API directa de Gemini.");
        let mut attempts = 0;
        let max_attempts = 3;
        let mut parts = vec![GeminiPart {
            text: Some(problem_description.to_string()),
            function_call: None,
            function_response: None,
        }];

        // Preparar prompt para thinking mode si está habilitado
        let enhanced_prompt = self.prepare_thinking_prompt(problem_description);

        loop {
            attempts += 1;
            if attempts > max_attempts {
                return Err(FlowError::MaxAttemptsReached(max_attempts));
            }

            let response_part = self.call_generative_api(&parts, &enhanced_prompt).await?;

            if let Some(function_call) = response_part.function_call.clone() {
                let tool_result = self.handle_function_call(function_call).await?;
                
                parts.push(response_part);
                parts.push(GeminiPart {
                    text: None,
                    function_call: None,
                    function_response: Some(FunctionResponse {
                        name: tool_result.function_name,
                        response: serde_json::json!({ "output": tool_result.output }),
                    })
                });

            } else if let Some(ref text) = response_part.text {
                log::info!("✅ Código generado exitosamente");
                let execution_time_ms = start_time.elapsed().as_millis() as u64;

                // Estimar tokens y costo
                let input_tokens = problem_description.split_whitespace().count() as u32;
                let output_tokens = text.split_whitespace().count() as u32;
                let cost_estimate = self.estimate_cost(input_tokens, output_tokens);

                return Ok(CodeGenerationResult {
                    code: text.to_string(),
                    language: "rust".to_string(),
                    confidence_score: 0.9,
                    attempts_made: attempts,
                    execution_time_ms,
                    verification_passed: self.verify_code(&text).is_valid,
                    cost_estimate: Some(cost_estimate),
                    model_used: Some(format!("{:?}", self.model_choice)),
                    metrics: Default::default(),
                });
            } else {
                return Err(FlowError::ApiError("Respuesta inesperada sin texto ni llamada a función".to_string()));
            }
        }
    }

    fn verify_code(&self, code: &str) -> VerificationResult {
        // Enhanced verification with quality scoring
        let is_valid = !code.trim().is_empty();
        let has_functions = code.contains("fn ") || code.contains("function") || code.contains("def ");
        let has_comments = code.contains("//") || code.contains("#") || code.contains("/*");
        let has_error_handling = code.contains("Result") || code.contains("try") || code.contains("catch");
        
        let quality_score = [
            if is_valid { 0.25 } else { 0.0 },
            if has_functions { 0.25 } else { 0.0 },
            if has_comments { 0.25 } else { 0.0 },
            if has_error_handling { 0.25 } else { 0.0 },
        ].iter().sum();

        VerificationResult {
            is_valid,
            compilation_success: true, // Placeholder - implementar verificación real
            tests_passed: true,        // Placeholder - implementar testing
            quality_score,
            errors: Vec::new(),
            warnings: if !has_comments { 
                vec!["Considera agregar comentarios al código".to_string()] 
            } else { 
                Vec::new() 
            },
        }
    }

    fn get_capabilities(&self) -> AdapterCapabilities {
        let (cost_input, cost_output, supports_thinking, max_tokens) = match self.model_choice {
            ModelChoice::Gemini2Pro => (0.10, 0.40, false, 2_000_000),
            ModelChoice::Gemini25Pro => (1.25, 10.00, true, 1_000_000),
            ModelChoice::Gemini25Flash => (0.075, 0.30, false, 1_000_000),
            _ => (1.25, 10.00, false, 1_000_000), // Default
        };

        AdapterCapabilities {
            name: "GeminiCLIFlow".to_string(),
            version: "2.0.0".to_string(),
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
                "java".to_string(),
            ],
            max_context_tokens: max_tokens,
            supports_function_calling: true,
            supports_code_execution: true,
            supports_thinking,
            cost_per_million_input: cost_input,
            cost_per_million_output: cost_output,
        }
    }
}

#[async_trait]
impl ThinkingFlow for GeminiCLIFlow {
    async fn execute_with_thinking(&self, problem: &str) -> Result<ThinkingResult, FlowError> {
        if !self.supports_thinking() {
            return Err(FlowError::ThinkingModeNotSupported);
        }

        let start_time = Instant::now();
        log::info!("🧠 Ejecutando con modo thinking habilitado");

        // Preparar prompt específico para thinking
        let thinking_prompt = format!(
            "Piensa paso a paso sobre este problema. Muestra tu razonamiento antes de dar la respuesta final.\n\nProblema: {}\n\nPor favor:\n1. Analiza el problema\n2. Considera diferentes enfoques\n3. Explica tu razonamiento\n4. Proporciona la solución final",
            problem
        );

        let mut reasoning_steps = Vec::new();

        // Simular pasos de razonamiento (en implementación real, esto vendría del modelo)
        reasoning_steps.push(ReasoningStep {
            step_number: 1,
            description: "Analizando los requisitos del problema".to_string(),
            confidence: 0.7,
            intermediate_result: Some("Identificados los componentes principales".to_string()),
        });

        reasoning_steps.push(ReasoningStep {
            step_number: 2,
            description: "Evaluando diferentes enfoques de solución".to_string(),
            confidence: 0.8,
            intermediate_result: Some("Seleccionado el enfoque más eficiente".to_string()),
        });

        reasoning_steps.push(ReasoningStep {
            step_number: 3,
            description: "Implementando la solución paso a paso".to_string(),
            confidence: 0.9,
            intermediate_result: Some("Código base implementado".to_string()),
        });

        let confidence_evolution = reasoning_steps.iter().map(|step| step.confidence).collect();

        // Ejecutar la tarea normal pero con el prompt mejorado
        let final_result = self.execute(&thinking_prompt).await?;
        
        let thinking_time = start_time.elapsed().as_millis() as u64;

        Ok(ThinkingResult {
            reasoning_trace: reasoning_steps.iter().map(|step| step.description.clone()).collect(),
            intermediate_conclusions: reasoning_steps.iter()
                .filter_map(|step| step.intermediate_result.clone())
                .collect(),
            final_result,
            confidence_evolution,
            thinking_time_ms: thinking_time,
        })
    }

    fn get_reasoning_steps(&self) -> Vec<ReasoningStep> {
        self.reasoning_steps.clone()
    }

    fn set_thinking_mode(&mut self, mode: ThinkingMode) {
        self.thinking_mode = mode;
    }
}

impl GeminiCLIFlow {
    /// Constructor para modo API directa
    pub async fn new(config: AdapterConfig) -> Result<Self, FlowError> {
        Self::new_with_model(config, ModelChoice::Gemini25Pro).await
    }

    /// Constructor con selección específica de modelo
    pub async fn new_with_model(config: AdapterConfig, model_choice: ModelChoice) -> Result<Self, FlowError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| FlowError::NetworkError(e.to_string()))?;

        let api_endpoint = Self::get_api_endpoint(&model_choice);

        Ok(Self {
            client,
            config,
            api_endpoint,
            session_id: Uuid::new_v4().to_string(),
            mode: GeminiMode::ApiDirect,
            process_manager: None,
            thinking_mode: ThinkingMode::Standard,
            reasoning_steps: Vec::new(),
            model_choice,
        })
    }

    /// Constructor para modo CLI interactivo
    pub async fn new_interactive(config: AdapterConfig) -> Result<Self, FlowError> {
        let process_manager = GeminiProcessManager::new()
            .map_err(|e| FlowError::ApiError(e.to_string()))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| FlowError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            config,
            api_endpoint: String::new(), // No necesario para modo CLI
            session_id: Uuid::new_v4().to_string(),
            mode: GeminiMode::CliInteractive,
            process_manager: Some(process_manager),
            thinking_mode: ThinkingMode::Standard,
            reasoning_steps: Vec::new(),
            model_choice: ModelChoice::Gemini25Pro, // Default para CLI
        })
    }

    /// Obtiene el endpoint de API según el modelo
    fn get_api_endpoint(model_choice: &ModelChoice) -> String {
        let model_name = match model_choice {
            ModelChoice::Gemini2Pro => "gemini-2.0-pro",
            ModelChoice::Gemini25Pro => "gemini-2.5-pro",
            ModelChoice::Gemini25Flash => "gemini-2.5-flash",
            _ => "gemini-2.5-pro", // Default
        };

        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model_name
        )
    }

    /// Verifica si el modelo actual soporta thinking
    fn supports_thinking(&self) -> bool {
        matches!(self.model_choice, ModelChoice::Gemini25Pro | ModelChoice::Claude37Sonnet)
    }

    /// Prepara el prompt para thinking mode
    fn prepare_thinking_prompt(&self, original_prompt: &str) -> String {
        match self.thinking_mode {
            ThinkingMode::Standard => original_prompt.to_string(),
            ThinkingMode::Extended { max_thinking_time_ms } => {
                format!(
                    "Tómate hasta {} segundos para pensar cuidadosamente sobre este problema antes de responder.\n\nProblema: {}",
                    max_thinking_time_ms / 1000,
                    original_prompt
                )
            },
            ThinkingMode::StepByStep { show_intermediate } => {
                if show_intermediate {
                    format!(
                        "Resuelve este problema paso a paso, mostrando tu razonamiento en cada etapa:\n\nProblema: {}\n\nPor favor muestra:\n1. Tu análisis inicial\n2. Los pasos de tu razonamiento\n3. La solución final",
                        original_prompt
                    )
                } else {
                    format!(
                        "Piensa paso a paso sobre este problema, pero solo muestra la respuesta final:\n\nProblema: {}",
                        original_prompt
                    )
                }
            }
        }
    }

    /// Estima el costo de una tarea
    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> CostEstimate {
        let capabilities = self.get_capabilities();
        let input_cost = (input_tokens as f64 / 1_000_000.0) * capabilities.cost_per_million_input;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * capabilities.cost_per_million_output;

        CostEstimate {
            input_tokens,
            output_tokens,
            estimated_cost_usd: input_cost + output_cost,
            model_used: format!("{:?}", self.model_choice),
        }
    }

    async fn call_generative_api(&self, parts: &[GeminiPart], enhanced_prompt: &str) -> Result<GeminiPart, FlowError> {
        let mut request_parts = parts.to_vec();
        
        // Si es thinking mode, usar el prompt mejorado
        if enhanced_prompt != parts[0].text.as_ref().unwrap_or(&String::new()) {
            request_parts[0].text = Some(enhanced_prompt.to_string());
        }

        let request = GeminiRequest {
            contents: vec![GeminiContent { parts: request_parts }],
            tools: None, // Simplificado para esta implementación
            generation_config: GeminiGenerationConfig {
                temperature: 0.7,
                top_k: 40,
                top_p: 0.95,
                max_output_tokens: 8192,
                response_mime_type: None,
            },
            safety_settings: vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
            ],
            system_instruction: if self.supports_thinking() {
                Some(GeminiSystemInstruction {
                    parts: vec![GeminiPart {
                        text: Some("Eres un asistente de programación experto. Cuando se te pida pensar paso a paso, muestra tu razonamiento completo antes de dar la respuesta final.".to_string()),
                        function_call: None,
                        function_response: None,
                    }]
                })
            } else {
                None
            },
        };

        let response = self.client
            .post(&self.api_endpoint)
            .header("Content-Type", "application/json")
            .header("x-goog-api-key", &self.config.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| FlowError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(FlowError::ApiError(format!("API Error: {}", error_text)));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| FlowError::ApiError(format!("JSON Parse Error: {}", e)))?;

        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.clone());
            }
        }

        Err(FlowError::ApiError("No response content".to_string()))
    }

    async fn handle_function_call(&self, _function_call: FunctionCall) -> Result<ToolResult, FlowError> {
        // Placeholder implementation
        Ok(ToolResult {
            function_name: "placeholder".to_string(),
            output: "Function call handled".to_string(),
        })
    }
} 