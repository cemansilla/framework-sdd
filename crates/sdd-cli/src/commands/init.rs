use anyhow::Result;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Initializing SDD project at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);

    // Generate a project ID based on directory name
    let project_id = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sdd-project")
        .to_string();

    adapter.initialize(&project_id).await?;

    println!("✓ SDD project initialized at {:?}", project_path);
    println!("✓ Created .sdd/ directory structure");
    println!("✓ Created initial configuration files");
    println!("\nNext steps:");
    println!("  1. Edit .sdd/config.toml to configure your project");
    println!("  2. Run 'sdd brainstorm' to start exploring your idea");
    println!("  3. Run 'sdd status' to see project status");

    Ok(())
}
