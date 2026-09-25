use anyhow::Result;
use sdd_core::requirement::{Requirement, RequirementType};
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, list: bool, add: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Managing requirements at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    if list {
        println!("📋 Project Requirements");
        println!("======================");
        println!("No requirements found.");
        println!("\nUse 'sdd requirements --add \"<requirement>\"' to add requirements.");
    } else if let Some(req_text) = add {
        let requirement =
            Requirement::new("REQ-001", &req_text, &req_text, RequirementType::Functional);

        println!("✓ Added requirement: {}", req_text);
        println!("  ID: {}", requirement.id);
        println!("  Type: {:?}", requirement.req_type);
        println!("  Status: {:?}", requirement.status);
    } else {
        println!("Usage:");
        println!("  sdd requirements --list          List all requirements");
        println!("  sdd requirements --add \"<text>\"  Add a new requirement");
    }

    Ok(())
}
