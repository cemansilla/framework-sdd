use anyhow::Result;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Checking project status at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    let manifest = adapter.load_manifest().await?;

    println!("SDD Project Status");
    println!("==================");
    println!("Path: {:?}", project_path);
    println!("Project ID: {}", manifest.project_id);
    println!("Version: {}", manifest.version);
    println!("\nArtifacts:");
    println!(
        "  Brief: {}",
        if structure.layout().brief.exists() {
            "✓"
        } else {
            "✗"
        }
    );
    println!(
        "  Requirements: {}",
        if structure.layout().requirements.exists() {
            "✓"
        } else {
            "✗"
        }
    );
    println!(
        "  Architecture: {}",
        if structure.layout().architecture.exists() {
            "✓"
        } else {
            "✗"
        }
    );
    println!(
        "  Tasks: {}",
        if structure.layout().tasks.exists() {
            "✓"
        } else {
            "✗"
        }
    );

    Ok(())
}
