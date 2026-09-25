use anyhow::Result;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, change_id: String) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!(
        "Analyzing impact for change {} at {:?}",
        change_id,
        project_path
    );

    let adapter = sdd_storage::filesystem::FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    println!("📊 Impact Analysis for Change: {}", change_id);
    println!("=====================================");
    println!("Change not found: {}", change_id);
    println!("\nUse 'sdd changelog' to see all changes.");

    Ok(())
}
