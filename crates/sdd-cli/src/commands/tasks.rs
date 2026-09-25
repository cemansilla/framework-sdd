use anyhow::Result;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, list: bool, task_id: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Managing tasks at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    if let Some(id) = task_id {
        println!("📝 Task Details: {}", id);
        println!("================");
        println!("Task not found: {}", id);
        println!("\nUse 'sdd tasks --list' to see all available tasks.");
    } else if list {
        println!("📋 Project Tasks");
        println!("===============");
        println!("No tasks found.");
        println!("\nTasks are generated from requirements and architecture.");
        println!("Run 'sdd plan' to generate tasks from your project definition.");
    } else {
        println!("Usage:");
        println!("  sdd tasks --list          List all tasks");
        println!("  sdd tasks <id>            Show details for a specific task");
    }

    Ok(())
}
