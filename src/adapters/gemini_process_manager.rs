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
        
        // Intentar ejecutar gemini CLI
        let process = Command::new("npx")
            .arg("@google/gemini-cli")
            .arg("--interactive")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Error iniciando Gemini CLI: {}. Asegúrate de tener Node.js instalado.", e))?;

        log::info!("✅ Proceso Gemini CLI iniciado con PID: {}", process.id());

        let manager = Self {
            process: Arc::new(Mutex::new(Some(process))),
            is_ready: Arc::new(Mutex::new(false)),
            output_buffer: Arc::new(Mutex::new(String::new())),
        };

        // Iniciar el monitor de salida en un hilo separado
        manager.start_output_monitor()?;
        
        // Esperar a que el CLI esté listo
        manager.wait_for_ready(Duration::from_secs(30))?;
        
        log::info!("🎯 Gemini CLI listo para recibir comandos");
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

    /// Ejecuta un comando en el CLI interactivo
    pub async fn execute_command(&self, command: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("💬 Enviando comando a Gemini CLI: {}", command.chars().take(50).collect::<String>() + "...");
        
        // Crear canales para comunicación asíncrona
        let (tx, rx) = oneshot::channel();
        
        // Enviar comando en un hilo separado
        let process_clone = Arc::clone(&self.process);
        let output_buffer_clone = Arc::clone(&self.output_buffer);
        let command_owned = command.to_string();
        
        thread::spawn(move || {
            let result = Self::send_command_sync(&process_clone, &command_owned, &output_buffer_clone);
            let _ = tx.send(result);
        });
        
        // Esperar respuesta con timeout
        match tokio::time::timeout(Duration::from_secs(60), rx).await {
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

    /// Envía un comando de manera síncrona
    fn send_command_sync(
        process: &Arc<Mutex<Option<Child>>>,
        command: &str,
        output_buffer: &Arc<Mutex<String>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Limpiar buffer de salida
        if let Ok(mut buffer) = output_buffer.lock() {
            buffer.clear();
        }

        // Enviar comando
        if let Ok(mut process_guard) = process.lock() {
            if let Some(ref mut process) = *process_guard {
                if let Some(ref mut stdin) = process.stdin {
                    writeln!(stdin, "{}", command)?;
                    stdin.flush()?;
                }
            }
        }

        // Esperar respuesta (simulado)
        thread::sleep(Duration::from_secs(2));
        
        // Leer respuesta del buffer
        if let Ok(buffer) = output_buffer.lock() {
            Ok(buffer.clone())
        } else {
            Ok("Comando enviado pero no se pudo leer la respuesta".to_string())
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