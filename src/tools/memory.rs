// ============================================================================
// MEMORY TOOLS - Herramientas de Memoria Persistente
// ============================================================================

use super::{Tool, ToolParams, ToolResult, ToolError, ToolCategory, RiskLevel, create_parameters_schema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs as async_fs;

// ============================================================================
// MEMORY STORE TOOL
// ============================================================================

pub struct MemoryStoreTool;

impl MemoryStoreTool {
    pub fn new() -> Self {
        Self
    }
    
    fn get_memory_dir() -> Result<PathBuf, ToolError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ToolError::InternalError("No se pudo obtener directorio home".to_string()))?;
        Ok(home_dir.join(".enjambre").join("memory"))
    }
}

#[async_trait]
impl Tool for MemoryStoreTool {
    fn name(&self) -> &str {
        "memory_store"
    }
    
    fn description(&self) -> &str {
        "Almacena datos en memoria persistente con namespace. Los datos persisten entre sesiones."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "key": {
                    "type": "string",
                    "description": "Clave única para identificar los datos"
                },
                "value": {
                    "type": "string",
                    "description": "Valor a almacenar (puede ser JSON serializado)"
                },
                "namespace": {
                    "type": "string",
                    "description": "Namespace para organizar los datos (por defecto: 'default')"
                },
                "ttl_hours": {
                    "type": "integer",
                    "description": "Tiempo de vida en horas (opcional)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags para categorizar los datos"
                },
                "metadata": {
                    "type": "object",
                    "description": "Metadata adicional como objeto JSON"
                }
            }),
            vec!["key", "value"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let key: String = params.get("key")?;
        let value: String = params.get("value")?;
        let namespace: String = params.get_optional("namespace")?.unwrap_or_else(|| "default".to_string());
        let ttl_hours: Option<u64> = params.get_optional("ttl_hours")?;
        let tags: Option<Vec<String>> = params.get_optional("tags")?;
        let metadata: Option<serde_json::Value> = params.get_optional("metadata")?;
        
        let memory_dir = Self::get_memory_dir()?;
        async_fs::create_dir_all(&memory_dir).await?;
        
        let namespace_dir = memory_dir.join(&namespace);
        async_fs::create_dir_all(&namespace_dir).await?;
        
        // Crear entrada de memoria
        let entry = MemoryEntry {
            key: key.clone(),
            value,
            namespace: namespace.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: ttl_hours.map(|hours| chrono::Utc::now() + chrono::Duration::hours(hours as i64)),
            tags: tags.unwrap_or_default(),
            metadata: metadata.unwrap_or(serde_json::Value::Null),
            access_count: 0,
            last_accessed: None,
        };
        
        // Guardar en archivo
        let file_path = namespace_dir.join(format!("{}.json", sanitize_filename(&key)));
        let json_content = serde_json::to_string_pretty(&entry)
            .map_err(|e| ToolError::InternalError(format!("Error serializando: {}", e)))?;
        
        async_fs::write(&file_path, json_content).await?;
        
        let result_data = serde_json::json!({
            "key": key,
            "namespace": namespace,
            "stored_at": entry.created_at,
            "expires_at": entry.expires_at,
            "path": file_path.to_string_lossy()
        });
        
        let message = format!("Datos almacenados exitosamente: {}:{}", namespace, key);
        Ok(ToolResult::success(result_data, message))
    }
}

// ============================================================================
// MEMORY RETRIEVE TOOL
// ============================================================================

pub struct MemoryRetrieveTool;

impl MemoryRetrieveTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MemoryRetrieveTool {
    fn name(&self) -> &str {
        "memory_retrieve"
    }
    
    fn description(&self) -> &str {
        "Recupera datos de memoria persistente por clave y namespace. Incluye búsqueda por tags."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "key": {
                    "type": "string",
                    "description": "Clave de los datos a recuperar (opcional si se usa search)"
                },
                "namespace": {
                    "type": "string",
                    "description": "Namespace de los datos (por defecto: 'default')"
                },
                "search": {
                    "type": "string",
                    "description": "Buscar por contenido o tags en lugar de clave exacta"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Filtrar por tags específicos"
                },
                "include_expired": {
                    "type": "boolean",
                    "description": "Si incluir entradas expiradas"
                },
                "limit": {
                    "type": "integer",
                    "description": "Máximo número de resultados (por defecto: 10)"
                }
            }),
            vec![]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let key: Option<String> = params.get_optional("key")?;
        let namespace: String = params.get_optional("namespace")?.unwrap_or_else(|| "default".to_string());
        let search: Option<String> = params.get_optional("search")?;
        let filter_tags: Option<Vec<String>> = params.get_optional("tags")?;
        let include_expired: bool = params.get_optional("include_expired")?.unwrap_or(false);
        let limit: usize = params.get_optional("limit")?.unwrap_or(10);
        
        let memory_dir = MemoryStoreTool::get_memory_dir()?;
        let namespace_dir = memory_dir.join(&namespace);
        
        if !namespace_dir.exists() {
            return Ok(ToolResult::success(
                serde_json::json!([]),
                format!("Namespace '{}' no existe", namespace)
            ));
        }
        
        let mut results = Vec::new();
        let now = chrono::Utc::now();
        
        // Si se especifica una clave exacta
        if let Some(key) = key {
            let file_path = namespace_dir.join(format!("{}.json", sanitize_filename(&key)));
            if file_path.exists() {
                if let Ok(content) = async_fs::read_to_string(&file_path).await {
                    if let Ok(mut entry) = serde_json::from_str::<MemoryEntry>(&content) {
                        // Verificar expiración
                        if !include_expired && entry.is_expired(now) {
                            return Ok(ToolResult::error(format!("Entrada expirada: {}", key)));
                        }
                        
                        // Actualizar estadísticas de acceso
                        entry.access_count += 1;
                        entry.last_accessed = Some(now);
                        
                        // Guardar estadísticas actualizadas
                        let updated_content = serde_json::to_string_pretty(&entry)
                            .map_err(|e| ToolError::InternalError(format!("Error serializando: {}", e)))?;
                        let _ = async_fs::write(&file_path, updated_content).await;
                        
                        results.push(entry);
                    }
                }
            }
        } else {
            // Búsqueda en todo el namespace
            let mut entries = async_fs::read_dir(&namespace_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Ok(content) = async_fs::read_to_string(entry.path()).await {
                            if let Ok(memory_entry) = serde_json::from_str::<MemoryEntry>(&content) {
                                // Filtrar por expiración
                                if !include_expired && memory_entry.is_expired(now) {
                                    continue;
                                }
                                
                                // Filtrar por búsqueda de texto
                                if let Some(search_term) = &search {
                                    if !memory_entry.matches_search(search_term) {
                                        continue;
                                    }
                                }
                                
                                // Filtrar por tags
                                if let Some(tags) = &filter_tags {
                                    if !memory_entry.has_tags(tags) {
                                        continue;
                                    }
                                }
                                
                                results.push(memory_entry);
                                
                                if results.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Ordenar por fecha de actualización (más reciente primero)
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        let result_data = serde_json::json!({
            "entries": results,
            "count": results.len(),
            "namespace": namespace,
            "search_term": search,
            "filter_tags": filter_tags
        });
        
        let message = format!("Encontradas {} entradas en namespace '{}'", results.len(), namespace);
        Ok(ToolResult::success(result_data, message))
    }
}

// ============================================================================
// MEMORY LIST TOOL
// ============================================================================

pub struct MemoryListTool;

impl MemoryListTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MemoryListTool {
    fn name(&self) -> &str {
        "memory_list"
    }
    
    fn description(&self) -> &str {
        "Lista todos los namespaces y claves en memoria persistente."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "namespace": {
                    "type": "string",
                    "description": "Namespace específico a listar (opcional)"
                },
                "show_stats": {
                    "type": "boolean",
                    "description": "Incluir estadísticas de uso"
                }
            }),
            vec![]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let namespace_filter: Option<String> = params.get_optional("namespace")?;
        let show_stats: bool = params.get_optional("show_stats")?.unwrap_or(false);
        
        let memory_dir = MemoryStoreTool::get_memory_dir()?;
        
        if !memory_dir.exists() {
            return Ok(ToolResult::success(
                serde_json::json!([]),
                "No hay datos en memoria".to_string()
            ));
        }
        
        let mut namespaces = HashMap::new();
        let now = chrono::Utc::now();
        
        // Listar directorios (namespaces)
        let mut entries = async_fs::read_dir(&memory_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() {
                let namespace_name = entry.file_name().to_string_lossy().to_string();
                
                // Filtrar por namespace si se especifica
                if let Some(filter) = &namespace_filter {
                    if namespace_name != *filter {
                        continue;
                    }
                }
                
                let mut namespace_info = NamespaceInfo {
                    name: namespace_name.clone(),
                    entries: Vec::new(),
                    total_entries: 0,
                    expired_entries: 0,
                    total_size: 0,
                };
                
                // Listar archivos en el namespace
                let namespace_path = entry.path();
                let mut namespace_entries = async_fs::read_dir(&namespace_path).await?;
                
                while let Some(file_entry) = namespace_entries.next_entry().await? {
                    if let Some(ext) = file_entry.path().extension() {
                        if ext == "json" {
                            let file_path = file_entry.path();
                            let file_size = file_entry.metadata().await?.len();
                            namespace_info.total_size += file_size;
                            namespace_info.total_entries += 1;
                            
                            if show_stats {
                                if let Ok(content) = async_fs::read_to_string(&file_path).await {
                                    if let Ok(memory_entry) = serde_json::from_str::<MemoryEntry>(&content) {
                                        if memory_entry.is_expired(now) {
                                            namespace_info.expired_entries += 1;
                                        }
                                        
                                        let expired = memory_entry.is_expired(now);
                                        namespace_info.entries.push(EntryInfo {
                                            key: memory_entry.key,
                                            created_at: memory_entry.created_at,
                                            updated_at: memory_entry.updated_at,
                                            expires_at: memory_entry.expires_at,
                                            access_count: memory_entry.access_count,
                                            tags: memory_entry.tags,
                                            size: file_size,
                                            expired,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                
                namespaces.insert(namespace_name, namespace_info);
            }
        }
        
        let result_data = serde_json::json!({
            "namespaces": namespaces,
            "total_namespaces": namespaces.len(),
            "show_stats": show_stats
        });
        
        let message = format!("Encontrados {} namespaces en memoria", namespaces.len());
        Ok(ToolResult::success(result_data, message))
    }
}

// ============================================================================
// ESTRUCTURAS DE DATOS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MemoryEntry {
    key: String,
    value: String,
    namespace: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    tags: Vec<String>,
    metadata: serde_json::Value,
    access_count: u64,
    last_accessed: Option<chrono::DateTime<chrono::Utc>>,
}

impl MemoryEntry {
    fn is_expired(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        self.expires_at.map(|exp| now > exp).unwrap_or(false)
    }
    
    fn matches_search(&self, search_term: &str) -> bool {
        let search_lower = search_term.to_lowercase();
        self.key.to_lowercase().contains(&search_lower) ||
        self.value.to_lowercase().contains(&search_lower) ||
        self.tags.iter().any(|tag| tag.to_lowercase().contains(&search_lower))
    }
    
    fn has_tags(&self, required_tags: &[String]) -> bool {
        required_tags.iter().all(|tag| self.tags.contains(tag))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NamespaceInfo {
    name: String,
    entries: Vec<EntryInfo>,
    total_entries: u32,
    expired_entries: u32,
    total_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EntryInfo {
    key: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    access_count: u64,
    tags: Vec<String>,
    size: u64,
    expired: bool,
}

// ============================================================================
// UTILIDADES
// ============================================================================

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
} 