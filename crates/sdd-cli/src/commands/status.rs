use anyhow::Result;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::{Path, PathBuf};

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
    let layout = structure.layout();

    println!("SDD Project Status");
    println!("==================");
    println!("Path: {:?}", project_path);
    println!("Project ID: {}", manifest.project_id);
    println!("Version: {}", manifest.version);
    println!("\nArtifacts:");
    println!(
        "  Brief: {}",
        check_mark(&layout.brief.join("brief.md"), false)
    );
    println!(
        "  Requirements: {}",
        check_mark(&layout.requirements, true)
    );
    println!(
        "  Architecture: {}",
        check_mark(&layout.architecture.join("architecture.md"), false)
    );
    println!("  Tasks: {}", check_mark(&layout.tasks, true));

    Ok(())
}

/// `✓` when a file exists and is non-empty, or when a directory contains at
/// least one markdown file (`with_content`), otherwise `✗`.
fn check_mark(path: &Path, with_content: bool) -> &'static str {
    let ok = if with_content {
        path.is_dir() && count_markdown_files(path) > 0
    } else {
        path.is_file() && std::fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
    };
    if ok { "✓" } else { "✗" }
}

fn count_markdown_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
                .count()
        })
        .unwrap_or(0)
}
