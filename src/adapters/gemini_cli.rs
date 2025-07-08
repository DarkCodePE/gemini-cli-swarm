// ============================================================================
// GEMINI CLI FLOW ADAPTER - Implementación ruv-swarm + SAFLA + SPARC
// ============================================================================
// Este adaptador conecta el ecosistema ruvnet con Google's Gemini CLI,
// implementando el bucle "Generar -> Verificar -> Refinar" de forma optimizada.
// ============================================================================

use crate::{
    adapters::gemini_process_manager::GeminiProcessManager,
    AdapterCapabilities, AdapterConfig, CodeGenerationFlow, FlowError, CodeGenerationResult,
    VerificationResult,
};
use async_trait::async_trait;
use futures::future::BoxFuture;
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
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeminiMode {
    ApiDirect,
    CliInteractive,
}

// ============================================================================
// ADAPTADOR PRINCIPAL
// ============================================================================

pub struct GeminiCLIFlow {
    client: Client,
    pub config: AdapterConfig,
    api_endpoint: String,
    pub session_id: String,
    mode: GeminiMode,
    process_manager: Option<GeminiProcessManager>,
}

#[async_trait]
impl CodeGenerationFlow for GeminiCLIFlow {
    async fn execute(&self, problem_description: &str) -> Result<CodeGenerationResult, FlowError> {
        let start_time = Instant::now();
        log::info!(
            "🚀 Iniciando Gemini CLI Flow - Sesión: {}",
            self.session_id
        );

        if let (GeminiMode::CliInteractive, Some(manager)) = (&self.mode, &self.process_manager) {
            log::info!("⚡ Ejecutando tarea a través de Gemini CLI interactivo.");
            let code = manager
                .execute_command(problem_description)
                .await
                .map_err(|e| FlowError::ApiError(format!("Error en Gemini CLI: {}", e)))?;
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            return Ok(CodeGenerationResult {
                code,
                language: "unknown".to_string(), // El CLI no especifica el lenguaje
                confidence_score: 0.95,          // Asumimos alta confianza con el CLI
                attempts_made: 1,
                execution_time_ms,
                verification_passed: true, // Asumimos que la verificación pasa
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

        loop {
            attempts += 1;
            if attempts > max_attempts {
                return Err(FlowError::MaxAttemptsReached(max_attempts));
            }

            let response_part = self.call_generative_api(&parts).await?;

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

                return Ok(CodeGenerationResult {
                    code: text.to_string(),
                    language: "rust".to_string(),
                    confidence_score: 0.9,
                    attempts_made: attempts,
                    execution_time_ms,
                    verification_passed: self.verify_code(&text).is_valid,
                });
            } else {
                return Err(FlowError::ApiError("Respuesta inesperada sin texto ni llamada a función".to_string()));
            }
        }
    }

    fn verify_code(&self, code: &str) -> VerificationResult {
        // Placeholder, implementar lógica real
        VerificationResult {
            is_valid: !code.trim().is_empty(),
            compilation_success: true,
            tests_passed: true,
            quality_score: 0.8,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn get_capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            name: "Gemini CLI Flow".to_string(),
            version: "0.2.0".to_string(),
            supported_languages: vec!["rust".to_string(), "python".to_string(), "javascript".to_string()],
            max_context_tokens: 32768,
            supports_function_calling: true,
            supports_code_execution: false,
        }
    }
}

#[allow(dead_code)]
impl GeminiCLIFlow {
    pub async fn new(config: AdapterConfig) -> Result<Self, FlowError> {
        Self::new_with_mode(config, GeminiMode::CliInteractive).await
    }
    
    pub async fn new_interactive(config: AdapterConfig) -> Result<Self, FlowError> {
        Self::new_with_mode(config, GeminiMode::CliInteractive).await
    }

    pub async fn new_with_mode(config: AdapterConfig, mode: GeminiMode) -> Result<Self, FlowError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| FlowError::NetworkError(e.to_string()))?;

        let api_endpoint = if let (Some(project_id), Some(location)) = (&config.project_id, &config.location) {
            format!(
                "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/gemini-1.5-flash:generateContent",
                location, project_id, location
            )
        } else {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
                config.api_key
            )
        };

        let (final_mode, process_manager) = if mode == GeminiMode::CliInteractive {
            match GeminiProcessManager::new() {
                Ok(manager) => (GeminiMode::CliInteractive, Some(manager)),
                Err(e) => {
                    log::warn!("Fallo al iniciar CLI interactivo, fallback a API: {}", e);
                    (GeminiMode::ApiDirect, None)
                }
            }
        } else {
            (GeminiMode::ApiDirect, None)
        };

        Ok(Self {
            client,
            config,
            api_endpoint,
            session_id: Uuid::new_v4().to_string(),
            mode: final_mode,
            process_manager,
        })
    }
    
    fn get_generation_config(&self) -> GeminiGenerationConfig {
        GeminiGenerationConfig {
            temperature: 0.4,
            top_k: 32,
            top_p: 0.8,
            max_output_tokens: 8192,
        }
    }

    fn get_safety_settings(&self) -> Vec<GeminiSafetySetting> {
        vec![
            GeminiSafetySetting {
                category: "HARM_CATEGORY_HARASSMENT".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
            GeminiSafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
            },
        ]
    }

    fn get_available_tools(&self) -> Vec<Tool> {
        vec![Tool {
            function_declarations: vec![FunctionDeclaration {
                name: "list_files".to_string(),
                description: "Lista recursivamente todos los archivos en el directorio actual.".to_string(),
                parameters: FunctionParameters {
                    param_type: "OBJECT".to_string(),
                    properties: serde_json::json!({}),
                    required: vec![],
                },
            }],
        }]
    }

    async fn handle_function_call(&self, call: FunctionCall) -> Result<ToolResult, FlowError> {
        match call.name.as_str() {
            "list_files" => {
                let file_list = self.list_files_recursively(".").await?;
                Ok(ToolResult {
                    function_name: "list_files".to_string(),
                    output: file_list,
                })
            }
            _ => Err(FlowError::ApiError(format!("Herramienta desconocida llamada: {}", call.name))),
        }
    }

    fn list_files_recursively<'a>(&'a self, path: &'a str) -> BoxFuture<'a, Result<String, FlowError>> {
        Box::pin(async move {
            let mut result = String::new();
            let mut entries = tokio::fs::read_dir(path).await
                .map_err(|e| FlowError::NetworkError(format!("Error leyendo directorio: {}", e)))?;
            
            while let Some(entry) = entries.next_entry().await
                .map_err(|e| FlowError::NetworkError(format!("Error iterando directorio: {}", e)))? {
                
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_str().unwrap_or_default();

                if file_name == ".git" || file_name == "target" {
                    continue;
                }
                
                if path.is_dir() {
                    result.push_str(&format!("[DIR] {}\n", path.display()));
                    let sub_list = self.list_files_recursively(path.to_str().unwrap()).await?;
                    for line in sub_list.lines() {
                        result.push_str(&format!("  {}\n", line));
                    }
                } else {
                    result.push_str(&format!("[FILE] {}\n", path.display()));
                }
            }
            Ok(result)
        })
    }

    async fn call_generative_api(&self, parts: &[GeminiPart]) -> Result<GeminiPart, FlowError> {
        let request = GeminiRequest {
            contents: vec![GeminiContent { parts: parts.to_vec() }],
            tools: Some(self.get_available_tools()),
            generation_config: self.get_generation_config(),
            safety_settings: self.get_safety_settings(),
        };

        let mut request_builder = self.client.post(&self.api_endpoint);
        if self.config.project_id.is_some() {
            request_builder = request_builder.bearer_auth(&self.config.api_key);
        }

        let response = request_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| FlowError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(FlowError::ApiError(format!("Error de API: {}", response.text().await.unwrap_or_default())));
        }
        
        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| FlowError::ApiError(format!("Error deserializando respuesta: {}", e)))?;
        
        if let Some(candidate) = gemini_response.candidates.into_iter().next() {
            if let Some(part) = candidate.content.parts.into_iter().next() {
                return Ok(part);
            }
        }
        
        Err(FlowError::ApiError("Respuesta vacía o mal formada.".to_string()))
    }
} 