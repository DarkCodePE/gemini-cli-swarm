use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::oneshot;
use regex::Regex;

/// Gestor de procesos para ejecutar Gemini CLI de manera interactiva
/// Inspirado en el GeminiProcessManager de Claude Code Flow
pub struct GeminiProcessManager {
    process: Arc<Mutex<Option<Child>>>,
    is_ready: Arc<Mutex<bool>>,
    output_buffer: Arc<Mutex<String>>,
}

impl GeminiProcessManager {
    /// Crea una nueva instancia del gestor de procesos Gemini CLI
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("🚀 Iniciando Gemini CLI en modo interactivo...");
        
        // Crear proceso Gemini CLI - usaremos modo prompt no interactivo
        // Nota: No iniciamos el proceso aquí, lo haremos por comando individual
        let manager = Self {
            process: Arc::new(Mutex::new(None)),
            is_ready: Arc::new(Mutex::new(true)), // Siempre listo en modo no interactivo
            output_buffer: Arc::new(Mutex::new(String::new())),
        };

        log::info!("✅ Gemini CLI configurado en modo no interactivo");
        Ok(manager)
    }

    /// Inicia el monitor de salida del proceso
    fn start_output_monitor(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let process_clone = Arc::clone(&self.process);
        let is_ready_clone = Arc::clone(&self.is_ready);
        let output_buffer_clone = Arc::clone(&self.output_buffer);

        thread::spawn(move || {
            if let Ok(mut process_guard) = process_clone.lock() {
                if let Some(ref mut process) = *process_guard {
                    if let Some(stdout) = process.stdout.take() {
                        let reader = BufReader::new(stdout);
                        
                        for line in reader.lines() {
                            match line {
                                Ok(output) => {
                                    log::debug!("[GEMINI_OUTPUT]: {}", output);
                                    
                                    // Actualizar buffer de salida
                                    if let Ok(mut buffer) = output_buffer_clone.lock() {
                                        buffer.push_str(&output);
                                        buffer.push('\n');
                                    }
                                    
                                    // Detectar cuando Gemini está listo
                                    if Self::is_prompt_ready(&output) {
                                        log::debug!("🟢 Detectado prompt listo");
                                        if let Ok(mut ready) = is_ready_clone.lock() {
                                            *ready = true;
                                        }
                                    }
                                    
                                    // Auto-aceptar confirmaciones
                                    if Self::is_confirmation_prompt(&output) {
                                        log::info!("🤖 Detectada confirmación, respondiendo automáticamente...");
                                        // Nota: En una implementación real, enviaríamos 'y' al stdin
                                    }
                                }
                                Err(e) => {
                                    log::error!("Error leyendo salida de Gemini CLI: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Detecta si el output indica que Gemini está listo para un comando
    fn is_prompt_ready(output: &str) -> bool {
        // Buscar patrones que indiquen que Gemini está esperando input
        let ready_patterns = [
            "> ",           // Prompt típico
            "? ",           // Pregunta
            "Enter ",       // Esperando entrada
            "Continue",     // Continuar
            "gemini>",      // Prompt específico de Gemini
        ];
        
        ready_patterns.iter().any(|pattern| output.contains(pattern))
    }

    /// Detecta prompts de confirmación
    fn is_confirmation_prompt(output: &str) -> bool {
        let confirmation_regex = Regex::new(r"\[y/N\]|Do you want to continue\?|Continue\?").unwrap();
        confirmation_regex.is_match(output)
    }

    /// Espera a que el CLI esté listo
    fn wait_for_ready(&self, timeout: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if let Ok(ready) = self.is_ready.lock() {
                if *ready {
                    return Ok(());
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        Err("Timeout esperando a que Gemini CLI esté listo".into())
    }

    /// Ejecuta un comando usando Gemini CLI en modo no interactivo
    pub async fn execute_command(&self, command: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("💬 Ejecutando comando en Gemini CLI: {}", command.chars().take(50).collect::<String>() + "...");
        
        // Crear canales para comunicación asíncrona
        let (tx, rx) = oneshot::channel();
        let command_owned = command.to_string();
        
        // Ejecutar Gemini CLI en modo no interactivo con --prompt
        thread::spawn(move || {
            let result = Self::execute_gemini_command(&command_owned);
            let _ = tx.send(result);
        });
        
        // Esperar respuesta con timeout aumentado
        match tokio::time::timeout(Duration::from_secs(120), rx).await {
            Ok(Ok(response)) => {
                log::info!("✅ Comando ejecutado exitosamente");
                Ok(response?)
            }
            Ok(Err(e)) => {
                log::error!("❌ Error ejecutando comando: {}", e);
                Err(Box::new(e))
            }
            Err(_) => {
                log::error!("⏰ Timeout ejecutando comando");
                Err("Timeout ejecutando comando en Gemini CLI".into())
            }
        }
    }

    /// Ejecuta un comando usando Gemini CLI en modo no interactivo
    fn execute_gemini_command(command: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("🔧 Ejecutando: npx @google/gemini-cli via stdin...");
        
        // Configurar comando específico para Windows vs Unix/Linux/macOS
        let mut cmd = if cfg!(target_os = "windows") {
            // En Windows, ejecutar a través de cmd.exe para asegurar compatibilidad
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", "npx", "@google/gemini-cli", "--yolo"]);
            cmd
        } else {
            // En sistemas Unix/Linux/macOS, usar npx directamente
            let mut cmd = Command::new("npx");
            cmd.arg("@google/gemini-cli")
                .arg("--yolo"); // Auto-aceptar acciones para evitar confirmaciones
            cmd
        };

        // Iniciar el proceso con pipes para stdin/stdout/stderr
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Error ejecutando Gemini CLI: {}. Asegúrate de tener Node.js instalado.", e))?;

        // Escribir el prompt al stdin del proceso hijo
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(command.as_bytes())
                .map_err(|e| format!("Fallo al escribir a stdin de Gemini CLI: {}", e))?;
        }

        // Esperar que el proceso termine y capturar la salida
        let output = child.wait_with_output()
            .map_err(|e| format!("Error esperando por el proceso de Gemini CLI: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = stdout.trim().to_string();
            
            log::debug!("📤 Respuesta de Gemini CLI ({} chars): {}...", 
                result.len(), 
                result.chars().take(100).collect::<String>()
            );
            
            Ok(result)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_msg = format!("Gemini CLI falló: {}", stderr);
            log::error!("❌ {}", error_msg);
            Err(error_msg.into())
        }
    }

    /// Termina el proceso Gemini CLI
    pub fn kill(&self) {
        log::info!("🛑 Terminando proceso Gemini CLI...");
        
        if let Ok(mut process_guard) = self.process.lock() {
            if let Some(mut process) = process_guard.take() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }
        
        log::info!("✅ Proceso Gemini CLI terminado");
    }
}

impl Drop for GeminiProcessManager {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_detection() {
        assert!(GeminiProcessManager::is_prompt_ready("> Enter your prompt"));
        assert!(GeminiProcessManager::is_prompt_ready("? What would you like to do"));
        assert!(!GeminiProcessManager::is_prompt_ready("Processing your request..."));
    }

    #[test]
    fn test_confirmation_detection() {
        assert!(GeminiProcessManager::is_confirmation_prompt("Do you want to continue? [y/N]"));
        assert!(GeminiProcessManager::is_confirmation_prompt("Continue? [y/N]"));
        assert!(!GeminiProcessManager::is_confirmation_prompt("Normal output"));
    }
} 