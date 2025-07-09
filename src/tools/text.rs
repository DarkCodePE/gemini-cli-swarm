// ============================================================================
// TEXT TOOLS - Herramientas de Procesamiento de Texto
// ============================================================================

use super::{Tool, ToolParams, ToolResult, ToolError, ToolCategory, RiskLevel, create_parameters_schema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use regex::Regex;

// ============================================================================
// TEXT PROCESS TOOL
// ============================================================================

pub struct TextProcessTool;

impl TextProcessTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TextProcessTool {
    fn name(&self) -> &str {
        "text_process"
    }
    
    fn description(&self) -> &str {
        "Procesa texto con múltiples operaciones: buscar, reemplazar, extraer, contar, formatear."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "text": {
                    "type": "string",
                    "description": "Texto a procesar"
                },
                "operation": {
                    "type": "string",
                    "description": "Operación a realizar",
                    "enum": ["search", "replace", "extract", "count", "format", "analyze", "split"]
                },
                "pattern": {
                    "type": "string",
                    "description": "Patrón regex para búsqueda/reemplazo"
                },
                "replacement": {
                    "type": "string",
                    "description": "Texto de reemplazo (para operación replace)"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Si la búsqueda es sensible a mayúsculas/minúsculas"
                },
                "delimiter": {
                    "type": "string",
                    "description": "Delimitador para split (por defecto: línea nueva)"
                },
                "format_type": {
                    "type": "string",
                    "description": "Tipo de formato",
                    "enum": ["uppercase", "lowercase", "title", "trim", "normalize"]
                }
            }),
            vec!["text", "operation"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Text
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let text: String = params.get("text")?;
        let operation: String = params.get("operation")?;
        let pattern: Option<String> = params.get_optional("pattern")?;
        let replacement: Option<String> = params.get_optional("replacement")?;
        let case_sensitive: bool = params.get_optional("case_sensitive")?.unwrap_or(true);
        let delimiter: String = params.get_optional("delimiter")?.unwrap_or_else(|| "\n".to_string());
        let format_type: Option<String> = params.get_optional("format_type")?;
        
        let result = match operation.as_str() {
            "search" => {
                let pattern = pattern.ok_or_else(|| ToolError::MissingParameter("pattern".to_string()))?;
                search_text(&text, &pattern, case_sensitive)?
            }
            "replace" => {
                let pattern = pattern.ok_or_else(|| ToolError::MissingParameter("pattern".to_string()))?;
                let replacement = replacement.ok_or_else(|| ToolError::MissingParameter("replacement".to_string()))?;
                replace_text(&text, &pattern, &replacement, case_sensitive)?
            }
            "extract" => {
                let pattern = pattern.ok_or_else(|| ToolError::MissingParameter("pattern".to_string()))?;
                extract_text(&text, &pattern, case_sensitive)?
            }
            "count" => {
                count_text(&text, pattern.as_deref())?
            }
            "format" => {
                let format_type = format_type.ok_or_else(|| ToolError::MissingParameter("format_type".to_string()))?;
                format_text(&text, &format_type)?
            }
            "analyze" => {
                analyze_text(&text)?
            }
            "split" => {
                split_text(&text, &delimiter)?
            }
            _ => {
                return Ok(ToolResult::error(format!("Operación no soportada: {}", operation)));
            }
        };
        
        let message = format!("Operación '{}' completada exitosamente", operation);
        Ok(ToolResult::success(result, message))
    }
}

// ============================================================================
// FUNCIONES DE PROCESAMIENTO
// ============================================================================

fn search_text(text: &str, pattern: &str, case_sensitive: bool) -> Result<serde_json::Value, ToolError> {
    let regex_pattern = if case_sensitive {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    
    let regex = Regex::new(&regex_pattern)
        .map_err(|e| ToolError::InvalidParameter("pattern".to_string(), e.to_string()))?;
    
    let mut matches = Vec::new();
    for (line_num, line) in text.lines().enumerate() {
        for mat in regex.find_iter(line) {
            matches.push(serde_json::json!({
                "line": line_num + 1,
                "start": mat.start(),
                "end": mat.end(),
                "match": mat.as_str(),
                "context": line
            }));
        }
    }
    
    Ok(serde_json::json!({
        "matches": matches,
        "count": matches.len(),
        "pattern": pattern
    }))
}

fn replace_text(text: &str, pattern: &str, replacement: &str, case_sensitive: bool) -> Result<serde_json::Value, ToolError> {
    let regex_pattern = if case_sensitive {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    
    let regex = Regex::new(&regex_pattern)
        .map_err(|e| ToolError::InvalidParameter("pattern".to_string(), e.to_string()))?;
    
    let original_count = regex.find_iter(text).count();
    let result_text = regex.replace_all(text, replacement).to_string();
    
    Ok(serde_json::json!({
        "original_text": text,
        "result_text": result_text,
        "replacements_made": original_count,
        "pattern": pattern,
        "replacement": replacement
    }))
}

fn extract_text(text: &str, pattern: &str, case_sensitive: bool) -> Result<serde_json::Value, ToolError> {
    let regex_pattern = if case_sensitive {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    
    let regex = Regex::new(&regex_pattern)
        .map_err(|e| ToolError::InvalidParameter("pattern".to_string(), e.to_string()))?;
    
    let mut extractions = Vec::new();
    for caps in regex.captures_iter(text) {
        let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let mut groups = Vec::new();
        
        for i in 1..caps.len() {
            if let Some(group) = caps.get(i) {
                groups.push(group.as_str());
            }
        }
        
        extractions.push(serde_json::json!({
            "full_match": full_match,
            "groups": groups
        }));
    }
    
    Ok(serde_json::json!({
        "extractions": extractions,
        "count": extractions.len(),
        "pattern": pattern
    }))
}

fn count_text(text: &str, pattern: Option<&str>) -> Result<serde_json::Value, ToolError> {
    let char_count = text.chars().count();
    let byte_count = text.len();
    let line_count = text.lines().count();
    let word_count = text.split_whitespace().count();
    let paragraph_count = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    
    let mut result = serde_json::json!({
        "characters": char_count,
        "bytes": byte_count,
        "lines": line_count,
        "words": word_count,
        "paragraphs": paragraph_count
    });
    
    if let Some(pattern) = pattern {
        let regex = Regex::new(pattern)
            .map_err(|e| ToolError::InvalidParameter("pattern".to_string(), e.to_string()))?;
        let pattern_count = regex.find_iter(text).count();
        result["pattern_matches"] = serde_json::Value::Number(pattern_count.into());
    }
    
    Ok(result)
}

fn format_text(text: &str, format_type: &str) -> Result<serde_json::Value, ToolError> {
    let formatted = match format_type {
        "uppercase" => text.to_uppercase(),
        "lowercase" => text.to_lowercase(),
        "title" => {
            text.split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        "trim" => text.trim().to_string(),
        "normalize" => {
            // Normalizar espacios en blanco
            let normalized = text.lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            normalized
        }
        _ => {
            return Ok(serde_json::json!({
                "error": format!("Tipo de formato no soportado: {}", format_type)
            }));
        }
    };
    
    Ok(serde_json::json!({
        "original": text,
        "formatted": formatted,
        "format_type": format_type
    }))
}

fn analyze_text(text: &str) -> Result<serde_json::Value, ToolError> {
    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();
    let sentence_count = text.split('.').filter(|s| !s.trim().is_empty()).count();
    let paragraph_count = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    
    // Análisis de frecuencia de palabras (top 10)
    let mut word_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for word in text.split_whitespace() {
        let clean_word = word.to_lowercase()
            .trim_matches(|c: char| !c.is_alphabetic())
            .to_string();
        if !clean_word.is_empty() && clean_word.len() > 2 {
            *word_freq.entry(clean_word).or_insert(0) += 1;
        }
    }
    
    let mut word_freq_vec: Vec<_> = word_freq.into_iter().collect();
    word_freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
    word_freq_vec.truncate(10);
    
    // Estadísticas de longitud
    let words: Vec<&str> = text.split_whitespace().collect();
    let avg_word_length = if !words.is_empty() {
        words.iter().map(|w| w.len()).sum::<usize>() as f32 / words.len() as f32
    } else {
        0.0
    };
    
    let avg_sentence_length = if sentence_count > 0 {
        word_count as f32 / sentence_count as f32
    } else {
        0.0
    };
    
    Ok(serde_json::json!({
        "basic_stats": {
            "characters": char_count,
            "words": word_count,
            "sentences": sentence_count,
            "paragraphs": paragraph_count,
            "avg_word_length": avg_word_length,
            "avg_sentence_length": avg_sentence_length
        },
        "top_words": word_freq_vec,
        "readability": {
            "estimated_reading_time_minutes": word_count as f32 / 200.0, // Asumiendo 200 palabras por minuto
            "complexity_score": avg_sentence_length / 10.0 // Puntuación simplificada
        }
    }))
}

fn split_text(text: &str, delimiter: &str) -> Result<serde_json::Value, ToolError> {
    let parts: Vec<&str> = if delimiter == "\\n" {
        text.lines().collect()
    } else {
        text.split(delimiter).collect()
    };
    
    Ok(serde_json::json!({
        "parts": parts,
        "count": parts.len(),
        "delimiter": delimiter
    }))
} 