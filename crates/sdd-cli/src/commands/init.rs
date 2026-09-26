use anyhow::Result;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::{Path, PathBuf};

/// Derive the project ID from the target directory name, resolving paths
/// like `.` via `canonicalize`. Falls back to `sdd-project` if neither works.
fn project_id_for(path: &Path) -> String {
    path.file_name()
        .map(std::ffi::OsStr::to_os_string)
        .or_else(|| {
            std::fs::canonicalize(path)
                .ok()
                .and_then(|resolved| resolved.file_name().map(std::ffi::OsStr::to_os_string))
        })
        .and_then(|name| name.to_str().map(str::to_string))
        .unwrap_or_else(|| "sdd-project".to_string())
}

pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Initializing SDD project at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);

    // Generate a project ID based on directory name; for paths like "." fall
    // back to the canonicalized directory so init from any cwd works.
    let project_id = project_id_for(&project_path);

    adapter.initialize(&project_id).await?;

    println!("✓ SDD project initialized at {:?}", project_path);
    println!("✓ Created .sdd/ directory structure");
    println!("✓ Created initial configuration files");
    println!("\nNext steps:");
    println!("  1. Edit .sdd/config/project.md to configure your project");
    println!("  2. Run 'sdd brainstorm' to start exploring your idea");
    println!("  3. Run 'sdd status' to see project status");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_uses_directory_name() {
        assert_eq!(project_id_for(Path::new("/tmp/some-dir")), "some-dir");
    }

    #[test]
    fn test_project_id_falls_back_for_root() {
        assert_eq!(project_id_for(Path::new("/")), "sdd-project");
    }

    #[test]
    fn test_project_id_resolves_current_dir() {
        let resolved = project_id_for(Path::new("."));
        assert_ne!(resolved, "sdd-project");
        assert!(!resolved.is_empty());
    }
}
