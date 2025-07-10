// ============================================================================
// FILESYSTEM TOOLS - Herramientas de Sistema de Archivos
// ============================================================================

use super::{Tool, ToolParams, ToolResult, ToolError, ToolCategory, RiskLevel, create_parameters_schema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use glob::glob;

use tokio::fs as async_fs;

// ============================================================================
// LIST FILES TOOL
// ============================================================================

pub struct ListFilesTool;

impl ListFilesTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    
    fn description(&self) -> &str {
        "Lista archivos y directorios en una ruta específica. Soporta patrones glob y búsqueda recursiva."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Ruta del directorio a listar (por defecto: directorio actual)"
                },
                "pattern": {
                    "type": "string", 
                    "description": "Patrón glob para filtrar archivos (ej: '*.rs', '**/*.py')"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Si debe buscar recursivamente en subdirectorios"
                },
                "show_hidden": {
                    "type": "boolean",
                    "description": "Si debe mostrar archivos ocultos (que empiezan con .)"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Profundidad máxima para búsqueda recursiva"
                }
            }),
            vec![]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let path: String = params.get_optional("path")?.unwrap_or_else(|| ".".to_string());
        let pattern: Option<String> = params.get_optional("pattern")?;
        let recursive: bool = params.get_optional("recursive")?.unwrap_or(false);
        let show_hidden: bool = params.get_optional("show_hidden")?.unwrap_or(false);
        let max_depth: Option<usize> = params.get_optional("max_depth")?;
        
        let path_buf = PathBuf::from(&path);
        
        if !path_buf.exists() {
            return Ok(ToolResult::error(format!("La ruta no existe: {}", path)));
        }
        
        let mut files = Vec::new();
        
        if let Some(pattern) = pattern {
            // Usar glob para patrones
            let glob_pattern = if pattern.starts_with('/') || pattern.contains(':') {
                pattern
            } else {
                format!("{}/{}", path, pattern)
            };
            
            match glob(&glob_pattern) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(path) => {
                                if let Some(file_info) = get_file_info(&path, show_hidden).await? {
                                    files.push(file_info);
                                }
                            }
                            Err(e) => {
                                log::warn!("Error procesando glob entry: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    return Ok(ToolResult::error(format!("Error en patrón glob: {}", e)));
                }
            }
        } else if recursive {
            // Búsqueda recursiva con walkdir
            let walker = WalkDir::new(&path_buf);
            let walker = if let Some(depth) = max_depth {
                walker.max_depth(depth)
            } else {
                walker
            };
            
            for entry in walker {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if !show_hidden && is_hidden(path) {
                            continue;
                        }
                        if let Some(file_info) = get_file_info(path, show_hidden).await? {
                            files.push(file_info);
                        }
                    }
                    Err(e) => {
                        log::warn!("Error accediendo a entrada: {}", e);
                    }
                }
            }
        } else {
            // Listar solo directorio actual
            let mut entries = async_fs::read_dir(&path_buf).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if !show_hidden && is_hidden(&path) {
                    continue;
                }
                if let Some(file_info) = get_file_info(&path, show_hidden).await? {
                    files.push(file_info);
                }
            }
        }
        
        // Ordenar por nombre
        files.sort_by(|a, b| a.name.cmp(&b.name));
        
        let message = format!("Encontrados {} elementos en '{}'", files.len(), path);
        Ok(ToolResult::success(files, message))
    }
}

// ============================================================================
// READ FILE TOOL
// ============================================================================

pub struct ReadFileTool;

impl ReadFileTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    
    fn description(&self) -> &str {
        "Lee el contenido de un archivo de texto o binario. Soporta diferentes encodings y puede leer parcialmente archivos grandes."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Ruta del archivo a leer"
                },
                "encoding": {
                    "type": "string",
                    "description": "Encoding del archivo (utf-8, ascii, latin1, binary)",
                    "enum": ["utf-8", "ascii", "latin1", "binary"]
                },
                "max_size": {
                    "type": "integer",
                    "description": "Tamaño máximo en bytes a leer (por defecto: 1MB)"
                },
                "start_byte": {
                    "type": "integer",
                    "description": "Posición de inicio para lectura parcial"
                },
                "end_byte": {
                    "type": "integer",
                    "description": "Posición de fin para lectura parcial"
                }
            }),
            vec!["path"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let path: String = params.get("path")?;
        let encoding: String = params.get_optional("encoding")?.unwrap_or_else(|| "utf-8".to_string());
        let max_size: usize = params.get_optional("max_size")?.unwrap_or(1024 * 1024); // 1MB default
        let start_byte: Option<usize> = params.get_optional("start_byte")?;
        let end_byte: Option<usize> = params.get_optional("end_byte")?;
        
        let path_buf = PathBuf::from(&path);
        
        if !path_buf.exists() {
            return Ok(ToolResult::error(format!("El archivo no existe: {}", path)));
        }
        
        if !path_buf.is_file() {
            return Ok(ToolResult::error(format!("La ruta no es un archivo: {}", path)));
        }
        
        // Obtener metadata del archivo
        let metadata = async_fs::metadata(&path_buf).await?;
        let file_size = metadata.len() as usize;
        
        if file_size > max_size && start_byte.is_none() {
            return Ok(ToolResult::error(format!(
                "Archivo demasiado grande ({} bytes). Máximo permitido: {} bytes. Usa start_byte/end_byte para lectura parcial.",
                file_size, max_size
            )));
        }
        
        // Leer archivo
        let content = if encoding == "binary" {
            let bytes = if let (Some(start), Some(end)) = (start_byte, end_byte) {
                read_file_range(&path_buf, start, end).await?
            } else {
                async_fs::read(&path_buf).await?
            };
            base64::encode(&bytes)
        } else {
            let text = if let (Some(start), Some(end)) = (start_byte, end_byte) {
                let bytes = read_file_range(&path_buf, start, end).await?;
                String::from_utf8_lossy(&bytes).to_string()
            } else {
                async_fs::read_to_string(&path_buf).await?
            };
            text
        };
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("file_size".to_string(), serde_json::Value::Number(file_size.into()));
        metadata.insert("encoding".to_string(), serde_json::Value::String(encoding.clone()));
        metadata.insert("path".to_string(), serde_json::Value::String(path.clone()));
        
        let result_data = serde_json::json!({
            "content": content,
            "size": file_size,
            "encoding": encoding,
            "is_partial": start_byte.is_some() || end_byte.is_some()
        });
        
        let message = format!("Archivo leído exitosamente: {} ({} bytes)", path, file_size);
        Ok(ToolResult::success(result_data, message).with_metadata(metadata))
    }
}

// ============================================================================
// WRITE FILE TOOL
// ============================================================================

pub struct WriteFileTool;

impl WriteFileTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    
    fn description(&self) -> &str {
        "Escribe contenido a un archivo. Puede crear nuevos archivos o sobrescribir existentes. Soporta contenido de texto y binario."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Ruta del archivo a escribir"
                },
                "content": {
                    "type": "string",
                    "description": "Contenido a escribir al archivo"
                },
                "encoding": {
                    "type": "string",
                    "description": "Encoding del contenido (utf-8, binary-base64)",
                    "enum": ["utf-8", "binary-base64"]
                },
                "append": {
                    "type": "boolean",
                    "description": "Si debe agregar al final en lugar de sobrescribir"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Si debe crear directorios padre si no existen"
                },
                "backup": {
                    "type": "boolean",
                    "description": "Si debe crear backup del archivo existente"
                }
            }),
            vec!["path", "content"]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let path: String = params.get("path")?;
        let content: String = params.get("content")?;
        let encoding: String = params.get_optional("encoding")?.unwrap_or_else(|| "utf-8".to_string());
        let append: bool = params.get_optional("append")?.unwrap_or(false);
        let create_dirs: bool = params.get_optional("create_dirs")?.unwrap_or(false);
        let backup: bool = params.get_optional("backup")?.unwrap_or(false);
        
        let path_buf = PathBuf::from(&path);
        
        // Crear directorios padre si es necesario
        if create_dirs {
            if let Some(parent) = path_buf.parent() {
                async_fs::create_dir_all(parent).await?;
            }
        }
        
        // Crear backup si es necesario
        if backup && path_buf.exists() {
            let backup_path = format!("{}.backup", path);
            async_fs::copy(&path_buf, &backup_path).await?;
        }
        
        // Escribir contenido
        match encoding.as_str() {
            "utf-8" => {
                if append {
                    let mut existing_content = if path_buf.exists() {
                        async_fs::read_to_string(&path_buf).await?
                    } else {
                        String::new()
                    };
                    existing_content.push_str(&content);
                    async_fs::write(&path_buf, existing_content).await?;
                } else {
                    async_fs::write(&path_buf, &content).await?;
                }
            }
            "binary-base64" => {
                let bytes = base64::decode(&content)
                    .map_err(|e| ToolError::InvalidParameter("content".to_string(), format!("Base64 inválido: {}", e)))?;
                if append {
                    let mut existing_bytes = if path_buf.exists() {
                        async_fs::read(&path_buf).await?
                    } else {
                        Vec::new()
                    };
                    existing_bytes.extend_from_slice(&bytes);
                    async_fs::write(&path_buf, existing_bytes).await?;
                } else {
                    async_fs::write(&path_buf, bytes).await?;
                }
            }
            _ => {
                return Ok(ToolResult::error(format!("Encoding no soportado: {}", encoding)));
            }
        }
        
        let metadata = async_fs::metadata(&path_buf).await?;
        let file_size = metadata.len();
        
        let mut result_metadata = std::collections::HashMap::new();
        result_metadata.insert("file_size".to_string(), serde_json::Value::Number(file_size.into()));
        result_metadata.insert("encoding".to_string(), serde_json::Value::String(encoding));
        result_metadata.insert("append".to_string(), serde_json::Value::Bool(append));
        
        let result_data = serde_json::json!({
            "path": path,
            "size": file_size,
            "operation": if append { "append" } else { "write" }
        });
        
        let message = format!("Archivo {} exitosamente: {} ({} bytes)", 
            if append { "actualizado" } else { "escrito" }, path, file_size);
        Ok(ToolResult::success(result_data, message).with_metadata(result_metadata))
    }
}

// ============================================================================
// UTILIDADES AUXILIARES
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    path: String,
    size: u64,
    is_dir: bool,
    is_file: bool,
    is_symlink: bool,
    modified: Option<String>,
    created: Option<String>,
    permissions: String,
    extension: Option<String>,
}

async fn get_file_info(path: &Path, _show_hidden: bool) -> Result<Option<FileInfo>, ToolError> {
    let metadata = match async_fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    
    let path_str = path.to_string_lossy().to_string();
    
    let modified = metadata.modified()
        .ok()
        .and_then(|time| {
            use std::time::UNIX_EPOCH;
            time.duration_since(UNIX_EPOCH)
                .ok()
                .map(|duration| {
                    let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0);
                    datetime.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                })
        });
    
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string());
    
    // Obtener permisos (simplificado)
    let permissions = if metadata.is_dir() {
        "directory".to_string()
    } else if metadata.permissions().readonly() {
        "readonly".to_string()
    } else {
        "readwrite".to_string()
    };
    
    Ok(Some(FileInfo {
        name,
        path: path_str,
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        is_symlink: metadata.file_type().is_symlink(),
        modified,
        created: None, // Simplificado por compatibilidad
        permissions,
        extension,
    }))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

async fn read_file_range(path: &Path, start: usize, end: usize) -> Result<Vec<u8>, ToolError> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    
    let mut file = async_fs::File::open(path).await?;
    file.seek(std::io::SeekFrom::Start(start as u64)).await?;
    
    let length = end.saturating_sub(start);
    let mut buffer = vec![0u8; length];
    let bytes_read = file.read(&mut buffer).await?;
    buffer.truncate(bytes_read);
    
    Ok(buffer)
} 