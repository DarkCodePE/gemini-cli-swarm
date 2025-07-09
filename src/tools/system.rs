// ============================================================================
// SYSTEM TOOLS - Herramientas de Sistema
// ============================================================================

use super::{Tool, ToolParams, ToolResult, ToolError, ToolCategory, RiskLevel, create_parameters_schema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, ProcessExt, DiskExt, NetworkExt, ComponentExt};

// ============================================================================
// SYSTEM INFO TOOL
// ============================================================================

pub struct SystemInfoTool;

impl SystemInfoTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SystemInfoTool {
    fn name(&self) -> &str {
        "system_info"
    }
    
    fn description(&self) -> &str {
        "Obtiene información detallada del sistema: CPU, memoria, discos, red y procesos."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        create_parameters_schema(
            serde_json::json!({
                "include_processes": {
                    "type": "boolean",
                    "description": "Incluir lista de procesos en ejecución"
                },
                "include_disks": {
                    "type": "boolean", 
                    "description": "Incluir información de discos"
                },
                "include_network": {
                    "type": "boolean",
                    "description": "Incluir estadísticas de red"
                },
                "include_components": {
                    "type": "boolean",
                    "description": "Incluir temperaturas de componentes"
                }
            }),
            vec![]
        )
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::System
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, ToolError> {
        let include_processes: bool = params.get_optional("include_processes")?.unwrap_or(false);
        let include_disks: bool = params.get_optional("include_disks")?.unwrap_or(true);
        let include_network: bool = params.get_optional("include_network")?.unwrap_or(false);
        let include_components: bool = params.get_optional("include_components")?.unwrap_or(false);
        
        let mut system = System::new_all();
        system.refresh_all();
        
        // Información básica del sistema
        let os_info = OsInfo {
            name: system.name().unwrap_or_default(),
            version: system.os_version().unwrap_or_default(),
            kernel_version: system.kernel_version().unwrap_or_default(),
            hostname: system.host_name().unwrap_or_default(),
            uptime: system.uptime(),
            boot_time: system.boot_time(),
        };
        
        // Información de CPU
        let cpu_info = CpuInfo {
            brand: system.global_cpu_info().brand().to_string(),
            cpu_count: system.cpus().len(),
            frequency: system.global_cpu_info().frequency(),
            usage: system.global_cpu_info().cpu_usage(),
        };
        
        // Información de memoria
        let memory_info = MemoryInfo {
            total: system.total_memory(),
            used: system.used_memory(),
            free: system.free_memory(),
            available: system.available_memory(),
            total_swap: system.total_swap(),
            used_swap: system.used_swap(),
            free_swap: system.free_swap(),
        };
        
        let mut system_info = SystemInfo {
            os: os_info,
            cpu: cpu_info,
            memory: memory_info,
            disks: Vec::new(),
            network: Vec::new(),
            processes: Vec::new(),
            components: Vec::new(),
        };
        
        // Información de discos
        if include_disks {
            for disk in system.disks() {
                system_info.disks.push(DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    used_space: disk.total_space() - disk.available_space(),
                    file_system: String::from_utf8_lossy(disk.file_system()).to_string(),
                    is_removable: disk.is_removable(),
                });
            }
        }
        
        // Información de red
        if include_network {
            for (interface_name, data) in system.networks() {
                system_info.network.push(NetworkInfo {
                    interface: interface_name.clone(),
                    received: data.received(),
                    transmitted: data.transmitted(),
                    packets_received: data.packets_received(),
                    packets_transmitted: data.packets_transmitted(),
                });
            }
        }
        
        // Lista de procesos
        if include_processes {
            for (pid, process) in system.processes() {
                system_info.processes.push(ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory: process.memory(),
                    virtual_memory: process.virtual_memory(),
                    status: format!("{:?}", process.status()),
                });
            }
            
            // Ordenar por uso de CPU
            system_info.processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            // Limitar a top 20
            system_info.processes.truncate(20);
        }
        
        // Temperaturas de componentes
        if include_components {
            for component in system.components() {
                system_info.components.push(ComponentInfo {
                    label: component.label().to_string(),
                    temperature: component.temperature(),
                    max_temperature: component.max(),
                    critical_temperature: component.critical(),
                });
            }
        }
        
        let message = format!("Información del sistema obtenida: {} ({})", 
            system_info.os.name, system_info.os.version);
        Ok(ToolResult::success(system_info, message))
    }
}

// ============================================================================
// ESTRUCTURAS DE DATOS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    os: OsInfo,
    cpu: CpuInfo,
    memory: MemoryInfo,
    disks: Vec<DiskInfo>,
    network: Vec<NetworkInfo>,
    processes: Vec<ProcessInfo>,
    components: Vec<ComponentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsInfo {
    name: String,
    version: String,
    kernel_version: String,
    hostname: String,
    uptime: u64,
    boot_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CpuInfo {
    brand: String,
    cpu_count: usize,
    frequency: u64,
    usage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryInfo {
    total: u64,
    used: u64,
    free: u64,
    available: u64,
    total_swap: u64,
    used_swap: u64,
    free_swap: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiskInfo {
    name: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    used_space: u64,
    file_system: String,
    is_removable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkInfo {
    interface: String,
    received: u64,
    transmitted: u64,
    packets_received: u64,
    packets_transmitted: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64,
    virtual_memory: u64,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentInfo {
    label: String,
    temperature: f32,
    max_temperature: f32,
    critical_temperature: Option<f32>,
} 