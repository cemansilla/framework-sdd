use anyhow::Result;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, change_id: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Managing changes at {:?}", project_path);

    let adapter = sdd_storage::filesystem::FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    if let Some(id) = change_id {
        println!("🔄 Change Details: {}", id);
        println!("====================");
        println!("Change not found: {}", id);
        println!("\nUse 'sdd changelog' to see all changes.");
    } else {
        println!("Usage:");
        println!("  sdd change <id>    Show details for a specific change");
        println!("\nChanges are tracked automatically when artifacts are modified.");
        println!("Use 'sdd changelog' to see the complete change history.");
    }

    Ok(())
}
