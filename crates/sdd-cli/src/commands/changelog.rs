use anyhow::Result;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Showing changelog at {:?}", project_path);

    let adapter = sdd_storage::filesystem::FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    println!("📜 Project Changelog");
    println!("===================");
    println!("No changes recorded yet.");
    println!("\nChanges are automatically tracked when artifacts are modified.");
    println!("Use 'sdd change <id>' to see details of a specific change.");

    Ok(())
}
