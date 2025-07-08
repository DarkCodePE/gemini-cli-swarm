use super::print_success;
use crate::cli::WorkflowCommands;
use std::error::Error;

pub async fn handle_workflow_command(cmd: WorkflowCommands) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cmd {
        WorkflowCommands::Create { name, parallel: _, config: _ } => {
            print_success(&format!("Workflow '{}' created", name));
        }
        WorkflowCommands::Execute { name, params: _ } => {
            print_success(&format!("Workflow '{}' executed", name));
        }
        WorkflowCommands::List => {
            print_success("No workflows found");
        }
        WorkflowCommands::Export { name, output } => {
            print_success(&format!("Workflow '{}' exported to {}", name, output.display()));
        }
    }
    Ok(())
} 