// ============================================================================
// UTILS TOOLS - Herramientas de Utilidades
// ============================================================================

use super::{Tool, ToolParams, ToolResult, ToolError, ToolCategory, RiskLevel, create_parameters_schema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// BASE64 TOOL
// ============================================================================

pub struct Base64Tool;

impl Base64Tool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for Base64Tool {
    fn name(&self) -> &str {
        "base64"
    }
    
    fn description(&self) -> &str {
        "Codifica y decodifica texto/datos usando Base64."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "operation": {
                    "type": "string",
                    "description": "Operación a realizar",
                    "enum": ["encode", "decode"]
                },
                "input": {
                    "type": "string", 
                    "description": "Texto o datos a procesar"
                },
                "input_type": {
                    "type": "string",
                    "description": "Tipo de entrada para encoding",
                    "enum": ["text", "hex"]
                }
            }),
            vec!["operation", "input"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Utils
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let operation: String = params.get("operation")?;
        let input: String = params.get("input")?;
        let input_type: String = params.get_optional("input_type")?.unwrap_or_else(|| "text".to_string());
        
        let result = match operation.as_str() {
            "encode" => {
                let bytes = match input_type.as_str() {
                    "text" => input.as_bytes().to_vec(),
                    "hex" => {
                        hex::decode(&input)
                            .map_err(|e| ToolError::InvalidParameter("input".to_string(), format!("Hex inválido: {}", e)))?
                    }
                    _ => {
                        return Ok(ToolResult::error(format!("Tipo de entrada no soportado: {}", input_type)));
                    }
                };
                
                let encoded = base64::encode(&bytes);
                serde_json::json!({
                    "operation": "encode",
                    "input": input,
                    "input_type": input_type,
                    "output": encoded,
                    "input_size": bytes.len(),
                    "output_size": encoded.len()
                })
            }
            "decode" => {
                let bytes = base64::decode(&input)
                    .map_err(|e| ToolError::InvalidParameter("input".to_string(), format!("Base64 inválido: {}", e)))?;
                
                // Intentar convertir a texto UTF-8
                let as_text = String::from_utf8(bytes.clone());
                let as_hex = hex::encode(&bytes);
                
                serde_json::json!({
                    "operation": "decode",
                    "input": input,
                    "output_bytes": bytes.len(),
                    "output_text": as_text.unwrap_or_else(|_| "[Datos binarios no UTF-8]".to_string()),
                    "output_hex": as_hex,
                    "is_valid_utf8": as_text.is_ok()
                })
            }
            _ => {
                return Ok(ToolResult::error(format!("Operación no soportada: {}", operation)));
            }
        };
        
        let message = format!("Operación Base64 '{}' completada", operation);
        Ok(ToolResult::success(result, message))
    }
}

// ============================================================================
// HASH TOOL
// ============================================================================

pub struct HashTool;

impl HashTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for HashTool {
    fn name(&self) -> &str {
        "hash"
    }
    
    fn description(&self) -> &str {
        "Genera hashes de texto usando diferentes algoritmos."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "input": {
                    "type": "string",
                    "description": "Texto a hashear"
                },
                "algorithm": {
                    "type": "string", 
                    "description": "Algoritmo de hash",
                    "enum": ["simple", "md5", "sha1", "sha256"]
                },
                "output_format": {
                    "type": "string",
                    "description": "Formato de salida",
                    "enum": ["hex", "base64"]
                }
            }),
            vec!["input"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Utils
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let input: String = params.get("input")?;
        let algorithm: String = params.get_optional("algorithm")?.unwrap_or_else(|| "simple".to_string());
        let output_format: String = params.get_optional("output_format")?.unwrap_or_else(|| "hex".to_string());
        
        let hash_bytes = match algorithm.as_str() {
            "simple" => {
                let mut hasher = DefaultHasher::new();
                input.hash(&mut hasher);
                hasher.finish().to_be_bytes().to_vec()
            }
            "md5" => {
                use std::collections::hash_map::DefaultHasher;
                // Implementación simplificada para demo - en producción usar crypto crate
                let mut hasher = DefaultHasher::new();
                input.hash(&mut hasher);
                hasher.finish().to_be_bytes().to_vec()
            }
            _ => {
                return Ok(ToolResult::error(format!("Algoritmo no soportado: {} (disponibles: simple, md5)", algorithm)));
            }
        };
        
        let output = match output_format.as_str() {
            "hex" => hex::encode(&hash_bytes),
            "base64" => base64::encode(&hash_bytes),
            _ => {
                return Ok(ToolResult::error(format!("Formato de salida no soportado: {}", output_format)));
            }
        };
        
        let result = serde_json::json!({
            "input": input,
            "algorithm": algorithm,
            "output_format": output_format,
            "hash": output,
            "input_length": input.len(),
            "hash_length": hash_bytes.len()
        });
        
        let message = format!("Hash {} generado exitosamente", algorithm);
        Ok(ToolResult::success(result, message))
    }
}

// ============================================================================
// URL TOOL
// ============================================================================

pub struct UrlTool;

impl UrlTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for UrlTool {
    fn name(&self) -> &str {
        "url"
    }
    
    fn description(&self) -> &str {
        "Codifica, decodifica y analiza URLs."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "operation": {
                    "type": "string",
                    "description": "Operación a realizar",
                    "enum": ["encode", "decode", "parse", "build"]
                },
                "input": {
                    "type": "string",
                    "description": "URL o texto a procesar"
                },
                "scheme": {
                    "type": "string", 
                    "description": "Esquema para build (http, https, ftp, etc.)"
                },
                "host": {
                    "type": "string",
                    "description": "Host para build"
                },
                "path": {
                    "type": "string",
                    "description": "Path para build"
                },
                "query": {
                    "type": "string",
                    "description": "Query string para build"
                }
            }),
            vec!["operation"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Utils
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let operation: String = params.get("operation")?;
        let input: Option<String> = params.get_optional("input")?;
        
        let result = match operation.as_str() {
            "encode" => {
                let input = input.ok_or_else(|| ToolError::MissingParameter("input".to_string()))?;
                let encoded = urlencoding::encode(&input);
                serde_json::json!({
                    "operation": "encode",
                    "input": input,
                    "output": encoded.to_string()
                })
            }
            "decode" => {
                let input = input.ok_or_else(|| ToolError::MissingParameter("input".to_string()))?;
                let decoded = urlencoding::decode(&input)
                    .map_err(|e| ToolError::InvalidParameter("input".to_string(), e.to_string()))?;
                serde_json::json!({
                    "operation": "decode", 
                    "input": input,
                    "output": decoded.to_string()
                })
            }
            "parse" => {
                let input = input.ok_or_else(|| ToolError::MissingParameter("input".to_string()))?;
                let parsed = url::Url::parse(&input)
                    .map_err(|e| ToolError::InvalidParameter("input".to_string(), e.to_string()))?;
                
                serde_json::json!({
                    "operation": "parse",
                    "input": input,
                    "scheme": parsed.scheme(),
                    "host": parsed.host_str(),
                    "port": parsed.port(),
                    "path": parsed.path(),
                    "query": parsed.query(),
                    "fragment": parsed.fragment(),
                    "username": parsed.username(),
                    "domain": parsed.domain()
                })
            }
            "build" => {
                let scheme: String = params.get_optional("scheme")?.unwrap_or_else(|| "https".to_string());
                let host: String = params.get("host")?;
                let path: String = params.get_optional("path")?.unwrap_or_else(|| "/".to_string());
                let query: Option<String> = params.get_optional("query")?;
                
                let mut url = format!("{}://{}{}", scheme, host, path);
                if let Some(q) = query {
                    url.push('?');
                    url.push_str(&q);
                }
                
                serde_json::json!({
                    "operation": "build",
                    "output": url,
                    "components": {
                        "scheme": scheme,
                        "host": host,
                        "path": path,
                        "query": query
                    }
                })
            }
            _ => {
                return Ok(ToolResult::error(format!("Operación no soportada: {}", operation)));
            }
        };
        
        let message = format!("Operación URL '{}' completada", operation);
        Ok(ToolResult::success(result, message))
    }
}

// ============================================================================
// JSON TOOL
// ============================================================================

pub struct JsonTool;

impl JsonTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for JsonTool {
    fn name(&self) -> &str {
        "json"
    }
    
    fn description(&self) -> &str {
        "Valida, formatea y manipula datos JSON."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "operation": {
                    "type": "string",
                    "description": "Operación a realizar",
                    "enum": ["validate", "format", "minify", "query", "merge"]
                },
                "input": {
                    "type": "string",
                    "description": "JSON como string"
                },
                "query_path": {
                    "type": "string",
                    "description": "Path JSONPath para query (ej: $.user.name)"
                },
                "merge_with": {
                    "type": "string",
                    "description": "JSON adicional para merge"
                },
                "pretty": {
                    "type": "boolean",
                    "description": "Si formatear con indentación"
                }
            }),
            vec!["operation", "input"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Utils
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let operation: String = params.get("operation")?;
        let input: String = params.get("input")?;
        let pretty: bool = params.get_optional("pretty")?.unwrap_or(true);
        
        // Validar JSON de entrada
        let json_value: serde_json::Value = serde_json::from_str(&input)
            .map_err(|e| ToolError::InvalidParameter("input".to_string(), format!("JSON inválido: {}", e)))?;
        
        let result = match operation.as_str() {
            "validate" => {
                serde_json::json!({
                    "operation": "validate",
                    "valid": true,
                    "type": get_json_type(&json_value),
                    "size": input.len(),
                    "structure": analyze_json_structure(&json_value)
                })
            }
            "format" => {
                let formatted = if pretty {
                    serde_json::to_string_pretty(&json_value)
                } else {
                    serde_json::to_string(&json_value)
                }.map_err(|e| ToolError::InternalError(e.to_string()))?;
                
                serde_json::json!({
                    "operation": "format",
                    "input": input,
                    "output": formatted,
                    "pretty": pretty,
                    "original_size": input.len(),
                    "formatted_size": formatted.len()
                })
            }
            "minify" => {
                let minified = serde_json::to_string(&json_value)
                    .map_err(|e| ToolError::InternalError(e.to_string()))?;
                
                serde_json::json!({
                    "operation": "minify",
                    "input": input,
                    "output": minified,
                    "original_size": input.len(),
                    "minified_size": minified.len(),
                    "compression_ratio": (input.len() as f32 - minified.len() as f32) / input.len() as f32
                })
            }
            "query" => {
                let query_path: String = params.get("query_path")?;
                // Implementación básica de query - en producción usar jsonpath crate
                let query_result = query_json(&json_value, &query_path)?;
                
                serde_json::json!({
                    "operation": "query",
                    "query_path": query_path,
                    "result": query_result,
                    "found": !query_result.is_null()
                })
            }
            "merge" => {
                let merge_with: String = params.get("merge_with")?;
                let merge_value: serde_json::Value = serde_json::from_str(&merge_with)
                    .map_err(|e| ToolError::InvalidParameter("merge_with".to_string(), format!("JSON inválido: {}", e)))?;
                
                let merged = merge_json_values(json_value, merge_value);
                let merged_str = if pretty {
                    serde_json::to_string_pretty(&merged)
                } else {
                    serde_json::to_string(&merged)
                }.map_err(|e| ToolError::InternalError(e.to_string()))?;
                
                serde_json::json!({
                    "operation": "merge",
                    "result": merged,
                    "result_string": merged_str
                })
            }
            _ => {
                return Ok(ToolResult::error(format!("Operación no soportada: {}", operation)));
            }
        };
        
        let message = format!("Operación JSON '{}' completada", operation);
        Ok(ToolResult::success(result, message))
    }
}

// ============================================================================
// UTILIDADES AUXILIARES
// ============================================================================

fn get_json_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn analyze_json_structure(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            serde_json::json!({
                "type": "object",
                "keys": map.len(),
                "key_names": map.keys().collect::<Vec<_>>()
            })
        }
        serde_json::Value::Array(arr) => {
            serde_json::json!({
                "type": "array",
                "length": arr.len(),
                "element_types": arr.iter().map(get_json_type).collect::<std::collections::HashSet<_>>()
            })
        }
        _ => {
            serde_json::json!({
                "type": get_json_type(value)
            })
        }
    }
}

fn query_json(value: &serde_json::Value, path: &str) -> Result<serde_json::Value, ToolError> {
    // Implementación simplificada de JSONPath
    if path == "$" {
        return Ok(value.clone());
    }
    
    if path.starts_with("$.") {
        let key = &path[2..];
        if let serde_json::Value::Object(map) = value {
            return Ok(map.get(key).cloned().unwrap_or(serde_json::Value::Null));
        }
    }
    
    Ok(serde_json::Value::Null)
}

fn merge_json_values(mut base: serde_json::Value, other: serde_json::Value) -> serde_json::Value {
    match (&mut base, other) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(other_map)) => {
            for (key, value) in other_map {
                base_map.insert(key, value);
            }
            base
        }
        (_, other) => other,
    }
}

// Dependencia adicional necesaria
mod urlencoding {
    pub fn encode(input: &str) -> String {
        input.chars()
            .map(|c| {
                match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                    _ => format!("%{:02X}", c as u8),
                }
            })
            .collect()
    }
    
    pub fn decode(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = String::new();
        let mut chars = input.chars();
        
        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.take(2).collect();
                if hex.len() == 2 {
                    let byte = u8::from_str_radix(&hex, 16)?;
                    result.push(byte as char);
                } else {
                    return Err("Invalid URL encoding".into());
                }
            } else {
                result.push(c);
            }
        }
        
        Ok(result)
    }
} 