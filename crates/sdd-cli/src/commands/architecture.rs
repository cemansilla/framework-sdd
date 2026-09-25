use anyhow::Result;
use sdd_core::architecture::{Architecture, ArchitectureStyle};
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, show: bool, style: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Managing architecture at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    if show {
        println!("🏗️  Project Architecture");
        println!("======================");
        println!("No architecture defined yet.");
        println!("\nUse 'sdd architecture --style <style>' to define architecture.");
    } else if let Some(style_name) = style {
        let arch_style = match style_name.to_lowercase().as_str() {
            "hexagonal" => ArchitectureStyle::Hexagonal,
            "layered" => ArchitectureStyle::Layered,
            "microservices" => ArchitectureStyle::Microservices,
            "event-driven" => ArchitectureStyle::EventDriven,
            _ => {
                println!("✗ Unknown architecture style: {}", style_name);
                println!("  Available styles: hexagonal, layered, microservices, event-driven");
                return Ok(());
            }
        };

        let architecture =
            Architecture::new("project-architecture", "Project Architecture", arch_style);

        println!("✓ Defined architecture: {:?}", architecture.style);
        println!("  ID: {}", architecture.id);
        println!("  Name: {}", architecture.name);
    } else {
        println!("Usage:");
        println!("  sdd architecture --show              Show current architecture");
        println!("  sdd architecture --style <style>     Define architecture style");
        println!("\nAvailable styles:");
        println!("  hexagonal      Hexagonal architecture (ports and adapters)");
        println!("  layered        Layered architecture");
        println!("  microservices  Microservices architecture");
        println!("  event-driven   Event-driven architecture");
    }

    Ok(())
}
