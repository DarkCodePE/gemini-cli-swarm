// ============================================================================
// ENJAMBRE CLI - Sistema de Comandos Universal
// ============================================================================
// Inspirado en Claude-Flow pero optimizado para Gemini CLI y el ecosistema ruv
// Comandos principales: init, swarm, hive-mind, memory, neural, tools
// ============================================================================

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

pub mod commands;
pub mod config;
pub mod wizard;

pub use commands::*;
pub use config::*;
pub use wizard::*;

#[derive(Parser)]
#[command(
    name = "enjambre",
    version = "2.0.0-alpha",
    about = "🌊 Enjambre v2.0.0 Alpha: Revolutionary Gemini CLI Orchestration Platform",
    long_about = r#"
🌟 Enjambre v2.0.0 Alpha - Gemini CLI Orchestration Platform

🐝 Hive-Mind Intelligence: Queen-led AI coordination with specialized worker agents
🧠 Neural Networks: 27+ cognitive models with WASM SIMD acceleration  
🔧 87+ Tools: Comprehensive toolkit for swarm orchestration, memory, and automation
🔄 Dynamic Agent Architecture (DAA): Self-organizing agents with fault tolerance
💾 Distributed Memory: Cross-session persistence with namespace management
⚡ Performance: 84.8% task success rate, 2.8-4.4x speed improvement

🚀 Quick Start:
  enjambre init --force              # Initialize with enhanced setup
  enjambre --help                    # Explore all capabilities  
  enjambre hive-mind wizard          # Launch interactive wizard
  enjambre swarm "build me something amazing" --gemini

🛡️ Powered by SAFLA + SPARC + Neuro-Divergent models
"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Skip safety checks (dangerous)
    #[arg(long, global = true)]
    pub dangerously_skip_permissions: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Enjambre with enhanced setup
    #[command(about = "🚀 Initialize Enjambre workspace with full alpha features")]
    Init {
        /// Force initialization even if config exists
        #[arg(long)]
        force: bool,
        
        /// Enable hive-mind coordination
        #[arg(long)]
        hive_mind: bool,
        
        /// Enable neural enhancement
        #[arg(long)]
        neural_enhanced: bool,
        
        /// Setup directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// 🐝 Hive-Mind Commands - AI Coordination System
    #[command(subcommand, about = "🐝 Queen-led AI coordination with specialized worker agents")]
    HiveMind(HiveMindCommands),

    /// 🧠 Neural Commands - Cognitive Computing Engine  
    #[command(subcommand, about = "🧠 27+ neural models with WASM SIMD acceleration")]
    Neural(NeuralCommands),

    /// 💾 Memory Commands - Distributed Memory System
    #[command(subcommand, about = "💾 Cross-session persistence with namespace management")]
    Memory(MemoryCommands),

    /// 🔧 Tools Commands - 87+ MCP Tools Integration
    #[command(subcommand, about = "🔧 Comprehensive toolkit for automation and orchestration")]
    Tools(ToolsCommands),

    /// 📊 Performance Commands - Monitoring & Analytics
    #[command(subcommand, about = "📊 Performance monitoring and analytics")]
    Performance(PerformanceCommands),

    /// 🔄 Workflow Commands - Automation Pipeline
    #[command(subcommand, about = "🔄 Workflow automation and pipeline management")]
    Workflow(WorkflowCommands),

    /// 🎯 Swarm Commands - Optimized Task Execution with Cost Management
    #[command(about = "🎯 Execute tasks with cost optimization and performance monitoring")]
    Swarm(crate::cli::commands::swarm::SwarmArgs),

    /// 🧪 Test Commands - System Testing
    #[command(subcommand, about = "🧪 Test system components and capabilities")]
    Test(TestCommands),

    /// ⚙️ Config Commands - Configuration Management
    #[command(subcommand, about = "⚙️ Manage system configuration")]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
pub enum HiveMindCommands {
    /// Launch interactive hive setup wizard
    #[command(about = "🧙 Interactive hive-mind setup and configuration")]
    Wizard,
    
    /// Deploy intelligent swarm coordination  
    #[command(about = "🚀 Deploy specialized worker agents for complex tasks")]
    Spawn {
        /// Task for the swarm
        task: Vec<String>,
        
        /// Number of agents
        #[arg(short, long, default_value = "6")]
        agents: usize,
        
        /// Use Gemini CLI
        #[arg(long)]
        gemini: bool,
        
        /// Coordination strategy
        #[arg(short, long, default_value = "hierarchical")]
        strategy: String,
        
        /// Memory namespace
        #[arg(long)]
        memory_namespace: Option<String>,
    },
    
    /// Monitor swarm coordination status
    #[command(about = "📊 Monitor coordination and agent status")]
    Status {
        /// Real-time monitoring
        #[arg(long)]
        real_time: bool,
        
        /// Show dashboard
        #[arg(long)]
        dashboard: bool,
    },
    
    /// Test hive-mind coordination
    #[command(about = "🧪 Test coordination capabilities")]
    Test {
        /// Number of test agents
        #[arg(short, long, default_value = "3")]
        agents: usize,
        
        /// Run coordination test
        #[arg(long)]
        coordination_test: bool,
    },
}

#[derive(Subcommand)]
pub enum NeuralCommands {
    /// Train coordination patterns
    #[command(about = "🎓 Train neural patterns from successful operations")]
    Train {
        /// Pattern type to train
        #[arg(short, long)]
        pattern: String,
        
        /// Training epochs
        #[arg(short, long, default_value = "50")]
        epochs: u32,
        
        /// Training data file
        #[arg(short, long)]
        data: Option<PathBuf>,
    },
    
    /// AI-powered predictions
    #[command(about = "🔮 Generate predictions using trained models")]
    Predict {
        /// Model to use for prediction
        #[arg(short, long)]
        model: String,
        
        /// Input data file
        #[arg(short, long)]
        input: Option<PathBuf>,
    },
    
    /// Analyze cognitive behavior
    #[command(about = "🧠 Analyze cognitive patterns and behavior")]
    Analyze {
        /// Behavior type to analyze
        #[arg(short, long)]
        behavior: String,
        
        /// Target to analyze
        #[arg(short, long)]
        target: Option<String>,
    },
    
    /// List available models
    #[command(about = "📋 List all available neural models")]
    List,
}

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Store key-value pair in memory
    #[command(about = "💾 Store data in distributed memory system")]
    Store {
        /// Key name
        key: String,
        
        /// Value to store
        value: String,
        
        /// Memory namespace
        #[arg(short, long, default_value = "default")]
        namespace: String,
    },
    
    /// Query memory entries
    #[command(about = "🔍 Search and retrieve memory entries")]
    Query {
        /// Search query
        query: String,
        
        /// Memory namespace
        #[arg(short, long, default_value = "default")]
        namespace: String,
    },
    
    /// Show memory statistics
    #[command(about = "📊 Display memory usage and statistics")]
    Stats,
    
    /// Export memory to file
    #[command(about = "📤 Export memory data to file")]
    Export {
        /// Output file path
        file: PathBuf,
        
        /// Namespace to export
        #[arg(short, long, default_value = "default")]
        namespace: String,
    },
    
    /// Import memory from file
    #[command(about = "📥 Import memory data from file")]
    Import {
        /// Input file path
        file: PathBuf,
        
        /// Target namespace
        #[arg(short, long, default_value = "default")]
        namespace: String,
    },
    
    /// List all namespaces
    #[command(about = "📋 List all memory namespaces")]
    List,
    
    /// Backup memory system
    #[command(about = "🔄 Create backup of memory system")]
    Backup {
        /// Backup file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Restore from backup
    #[command(about = "🔄 Restore memory from backup")]
    Restore {
        /// Backup file path
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ToolsCommands {
    /// List all available tools
    #[command(about = "📋 List all 87+ available tools")]
    List {
        /// Filter by category
        #[arg(short = 't', long)]
        category: Option<String>,
    },
    
    /// Get tool information
    #[command(about = "ℹ️ Get detailed information about a tool")]
    Info {
        /// Tool name
        tool: String,
    },
    
    /// Execute a tool
    #[command(about = "⚡ Execute a specific tool")]
    Execute {
        /// Tool name
        tool: String,
        
        /// Tool arguments (JSON format)
        #[arg(short, long)]
        args: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PerformanceCommands {
    /// Generate performance report
    #[command(about = "📊 Generate comprehensive performance report")]
    Report {
        /// Report format (json, text, html)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Analyze system bottlenecks
    #[command(about = "🔍 Analyze and identify system bottlenecks")]
    Bottleneck {
        /// Auto-optimize found bottlenecks
        #[arg(long)]
        auto_optimize: bool,
    },
    
    /// Show token usage statistics
    #[command(about = "🪙 Display token usage and cost analysis")]
    Tokens,
    
    /// Run system benchmark
    #[command(about = "⚡ Run comprehensive system benchmark")]
    Benchmark {
        /// Benchmark type
        #[arg(short, long, default_value = "full")]
        bench_type: String,
    },
}

#[derive(Subcommand)]
pub enum WorkflowCommands {
    /// Create new workflow
    #[command(about = "🏗️ Create a new workflow pipeline")]
    Create {
        /// Workflow name
        #[arg(short, long)]
        name: String,
        
        /// Enable parallel execution
        #[arg(long)]
        parallel: bool,
        
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    
    /// Execute workflow
    #[command(about = "⚡ Execute an existing workflow")]
    Execute {
        /// Workflow name
        name: String,
        
        /// Input parameters (JSON)
        #[arg(short, long)]
        params: Option<String>,
    },
    
    /// List workflows
    #[command(about = "📋 List all available workflows")]
    List,
    
    /// Export workflow definition
    #[command(about = "📤 Export workflow to file")]
    Export {
        /// Workflow name
        name: String,
        
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum TestCommands {
    /// Test all system components
    #[command(about = "🧪 Run comprehensive system tests")]
    All,
    
    /// Test memory system
    #[command(about = "💾 Test memory system functionality")]
    Memory,
    
    /// Test neural models
    #[command(about = "🧠 Test neural model capabilities")]
    Neural,
    
    /// Test Gemini CLI integration
    #[command(about = "🔗 Test Gemini CLI integration")]
    Gemini,
    
    /// Test tools system
    #[command(about = "🔧 Test tools execution")]
    Tools,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    #[command(about = "📋 Display current system configuration")]
    Show,
    
    /// Set configuration value
    #[command(about = "⚙️ Set a configuration value")]
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Get configuration value
    #[command(about = "📄 Get a configuration value")]
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Reset to default configuration
    #[command(about = "🔄 Reset to default configuration")]
    Reset {
        /// Confirm reset
        #[arg(long)]
        confirm: bool,
    },
    
    /// Validate configuration
    #[command(about = "✅ Validate current configuration")]
    Validate,
}

/// Print welcome banner
pub fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║           🌊 ENJAMBRE v2.0.0 Alpha - Gemini CLI Orchestration    ║".cyan());
    println!("{}", "║        🐝 Hive-Mind • 🧠 Neural • 🔧 87+ Tools • ⚡ SAFLA       ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════════╝".cyan());
    println!();
}

/// Print quick help
pub fn print_quick_help() {
    println!("{}", "🚀 Quick Start Commands:".bright_green().bold());
    println!("  {} {}", "enjambre init --force".bright_blue(), "Initialize with enhanced setup");
    println!("  {} {}", "enjambre hive-mind wizard".bright_blue(), "Launch interactive wizard");
    println!("  {} {}", "enjambre swarm \"task\" --gemini".bright_blue(), "Execute task with Gemini");
    println!("  {} {}", "enjambre memory stats".bright_blue(), "Check memory usage");
    println!("  {} {}", "enjambre neural list".bright_blue(), "List neural models");
    println!("  {} {}", "enjambre tools list".bright_blue(), "Show available tools");
    println!();
    println!("{}", "For detailed help: enjambre --help".bright_yellow());
} 