# 🚀 Gemini CLI Adapter - Sistema ruv-FANN + SAFLA + SPARC

Un adaptador universal para Google's Gemini CLI que implementa la arquitectura modular de **ruvnet** con soporte completo para **SAFLA**, **SPARC**, modelos **Neuro-Divergentes** y runtime **WASM**.

## 🏗️ Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────┐
│                    Tu Aplicación / Claude Code              │
├─────────────────────────────────────────────────────────────┤
│           🎯 Swarm Orchestrator (MCP + SAFLA)              │
│    ┌─────────────────┬─────────────────┬─────────────────┐  │
│    │  Gemini CLI     │   Claude Flow   │   Otros LLMs   │  │
│    │   Adapter       │    Adapter      │   Adapters     │  │
│    └─────────────────┴─────────────────┴─────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│           🧠 Catálogo Neuro-Divergente (Especialistas)      │
│    ┌─────────────────┬─────────────────┬─────────────────┐  │
│    │      LSTM       │     N-BEATS     │  Transformers   │  │
│    │   (Secuencias)  │  (Forecasting)  │   (Lenguaje)    │  │
│    └─────────────────┴─────────────────┴─────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│              ⚡ ruv-FANN Core Engine (Rust)                │
├─────────────────────────────────────────────────────────────┤
│                  🌐 WASM Runtime Universal                  │
└─────────────────────────────────────────────────────────────┘
```

## ✨ Características Principales

- **🔌 Adaptador Universal**: Interfaz común para cualquier LLM usando el trait `CodeGenerationFlow`
- **🧠 Selección Inteligente**: Modelos neuro-divergentes especializados para cada tipo de tarea
- **⚡ SAFLA Methodology**: Análisis → Diseño → Ejecución → Aprendizaje adaptivo
- **🔄 Bucle de Refinamiento**: Generar → Verificar → Refinar automáticamente
- **📊 Monitoreo de Performance**: Métricas en tiempo real y optimización continua
- **🌐 Runtime WASM**: Portabilidad completa (navegador, servidor, edge, IoT)

## 🚀 Instalación y Configuración

### 1. Prerrequisitos

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar dependencias del sistema
cargo --version
```

### 2. Configurar Variables de Entorno

**Opción A: Usando archivo .env (RECOMENDADO)**
```bash
# 1. Crear archivo de configuración
cp variables.example.txt .env

# 2. Editar .env con tu API key real
# Cambiar: GEMINI_API_KEY=tu_gemini_api_key_aqui
# Por: GEMINI_API_KEY=AIzaSy... (tu key real)
```

**Opción B: Variables de entorno tradicionales**
```bash
# API Key de Gemini (requerida)
export GEMINI_API_KEY="tu_api_key_aqui"

# Para Vertex AI (opcional)
export GOOGLE_CLOUD_PROJECT="tu_proyecto_gcp"
export GOOGLE_CLOUD_LOCATION="us-central1"

# Para logging detallado
export RUST_LOG="info"
```

### 3. Compilar y Ejecutar

```bash
# Clonar y entrar al directorio
git clone <tu_repositorio>
cd mi_enjambre_ia

# Compilar el proyecto
cargo build --release

# Ejecutar la demostración
cargo run
```

## 🎯 Uso del Sistema

### Ejemplo Básico: Generación de Código

```rust
use mi_enjambre_ia::{
    adapters::AdapterConfig,
    swarm::{SwarmOrchestrator, SwarmConfig, TaskBuilder},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurar adaptador
    let mut adapter_configs = HashMap::new();
    adapter_configs.insert("gemini".to_string(), AdapterConfig {
        api_key: "tu_api_key".to_string(),
        timeout_seconds: 30,
        max_attempts: 3,
        enable_verification: true,
        // ... otros campos
    });

    // 2. Inicializar swarm
    let mut orchestrator = SwarmOrchestrator::new(SwarmConfig::default());
    orchestrator.initialize(adapter_configs).await?;

    // 3. Crear tarea
    let task = TaskBuilder::code_generation(
        "Crear una función que ordene un vector usando quicksort"
    );

    // 4. Ejecutar con SAFLA
    let result = orchestrator.execute_task(task).await;
    
    if result.success {
        println!("Código generado: {}", result.result.unwrap().code);
    }

    Ok(())
}
```

### Ejemplo Avanzado: Forecasting con N-BEATS

```rust
// El sistema automáticamente selecciona el modelo N-BEATS para tareas de predicción
let forecasting_task = TaskBuilder::forecasting(
    "Predecir ventas de productos para los próximos 3 meses usando datos históricos"
);

let result = orchestrator.execute_task(forecasting_task).await;
// El sistema usará automáticamente el modelo N-BEATS optimizado
```

## 🧠 Modelos Neuro-Divergentes Disponibles

| Modelo | Especialización | Score | Casos de Uso |
|--------|----------------|-------|--------------|
| **LSTM** | Secuencias temporales | 0.85 | Predicción de ventas, análisis de sensores IoT |
| **N-BEATS** | Forecasting avanzado | 0.92 | Demanda energética, forecasting financiero |
| **Transformer** | Procesamiento de lenguaje | 0.88 | Generación de código, análisis de documentos |
| **Custom FANN** | Tareas generales | 0.75 | Clasificación, regresión, prototipado |

### Selección Automática de Modelos

El sistema usa inteligencia artificial para seleccionar automáticamente el mejor modelo:

```rust
// Para tareas de predicción → N-BEATS o LSTM
let task1 = "Predecir ventas del próximo trimestre"; // → N-BEATS

// Para tareas de código → Transformer  
let task2 = "Generar una API REST en Rust"; // → Transformer

// Para tareas generales → Custom FANN
let task3 = "Clasificar datos de clientes"; // → Custom FANN
```

## 📊 Metodología SAFLA

El sistema implementa un ciclo completo de **Análisis → Diseño → Ejecución → Aprendizaje**:

### Fase 1: Análisis 🔍
- Analiza la tarea para seleccionar el adaptador óptimo
- Selecciona el modelo neuro-divergente más adecuado
- Evalúa requisitos de performance y calidad

### Fase 2: Diseño 🎨  
- Crea prompts optimizados para el LLM seleccionado
- Configura parámetros de generación específicos
- Prepara el pipeline de verificación

### Fase 3: Ejecución ⚡
- Ejecuta el bucle "Generar → Verificar → Refinar"
- Monitorea performance en tiempo real
- Aplica optimizaciones dinámicas

### Fase 4: Aprendizaje 📈
- Analiza resultados para futuras optimizaciones
- Actualiza estrategias adaptivas
- Mejora la selección de modelos

## 🔧 Configuración Avanzada

### Personalizar Adaptador

```rust
let custom_config = AdapterConfig {
    api_key: "tu_key".to_string(),
    base_url: Some("https://custom-endpoint.com".to_string()),
    timeout_seconds: 60,
    max_attempts: 5,
    enable_verification: true,
    project_id: Some("mi-proyecto".to_string()),
    location: Some("europe-west1".to_string()),
};
```

### Configurar Swarm

```rust
let swarm_config = SwarmConfig {
    max_concurrent_tasks: 4,
    default_adapter: "gemini".to_string(),
    enable_neural_selection: true,
    enable_adaptive_learning: true,
    performance_monitoring: true,
};
```

### Crear Modelo Personalizado

```rust
use mi_enjambre_ia::neuro_divergent::{ModelType, ModelSpec};

let custom_model = ModelSpec {
    model_type: ModelType::CustomFANN {
        layers: vec![10, 20, 15, 1],
        activation: ActivationType::SigmoidSymmetric,
        learning_rate: 0.001,
    },
    // ... otros campos
};
```

## 🌐 Compilación a WASM

El sistema está preparado para compilarse a WebAssembly:

```bash
# Instalar wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Compilar a WASM
wasm-pack build --target web --out-dir pkg

# Usar en el navegador
import init, { run_swarm } from './pkg/mi_enjambre_ia.js';
await init();
```

## 📈 Monitoreo y Estadísticas

```rust
// Obtener estadísticas del swarm
let stats = orchestrator.get_stats();
println!("Tasa de éxito: {:.1}%", stats.success_rate * 100.0);
println!("Score promedio: {:.2}", stats.average_performance_score);
println!("Tareas completadas: {}", stats.successful_tasks);
```

## 🔍 Troubleshooting

### Error: "GEMINI_API_KEY no encontrada"
```bash
# Configurar la API key
export GEMINI_API_KEY="tu_api_key_real"

# Verificar
echo $GEMINI_API_KEY
```

### Error de compilación con ruv-FANN
```bash
# Limpiar y recompilar
cargo clean
cargo build --release
```

### Modo de demostración
Si no tienes una API key, el sistema ejecutará en modo demostración mostrando todas las capacidades locales.

## 🤝 Contribuir

1. Fork el repositorio
2. Crea una rama para tu feature: `git checkout -b feature/nueva-funcionalidad`
3. Commit tus cambios: `git commit -am 'Agregar nueva funcionalidad'`
4. Push a la rama: `git push origin feature/nueva-funcionalidad`
5. Crea un Pull Request

## 📝 Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo `LICENSE` para detalles.

## 🙏 Agradecimientos

- **ruvnet**: Por la arquitectura modular y la filosofía de diseño
- **SAFLA**: Por la metodología de desarrollo sistemático  
- **SPARC**: Por las mejores prácticas de ingeniería
- **ruv-FANN**: Por el motor de redes neuronales optimizado
- **Google**: Por la API de Gemini y las capacidades de Vertex AI

---

**¡La fábrica de IA inteligente está lista para funcionar! 🚀** 