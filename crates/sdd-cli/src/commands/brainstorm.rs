use anyhow::Result;
use sdd_core::lifecycle_phases::BrainstormingSession;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, title: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Starting brainstorming session at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    let session_title = title.unwrap_or_else(|| "Brainstorming Session".to_string());
    let session = BrainstormingSession::new(&session_title, "Exploration session");

    println!("🧠 Brainstorming Session: {}", session_title);
    println!("=====================================");
    println!("Start exploring your ideas and capturing thoughts.");
    println!("Use 'sdd questions' to add questions and assumptions.");
    println!("Use 'sdd requirements' to capture requirements.");

    println!("\n✓ Brainstorming session started");
    println!("  Session ID: {}", session.id);

    Ok(())
}
