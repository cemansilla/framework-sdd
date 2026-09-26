use anyhow::Result;
use sdd_storage::{parse_changelog, ChangelogEntry};
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

    let changelog_path = structure.artifact_path("changes", "CHANGELOG.md");

    if !changelog_path.exists() {
        println!("✗ No changelog found at {:?}", changelog_path);
        println!("  Run 'sdd init' to create the changelog file");
        return Ok(());
    }

    let content = tokio::fs::read_to_string(&changelog_path).await?;
    let entries = parse_changelog(&content);

    println!("{}", render(&entries));

    Ok(())
}

/// Format parsed entries for terminal output.
fn render(entries: &[ChangelogEntry]) -> String {
    if entries.is_empty() {
        return "📜 Project Changelog\n===================\nNo changes recorded yet.".to_string();
    }

    let mut out = format!(
        "📜 Project Changelog\n===================\n{} entries recorded\n",
        entries.len()
    );

    for entry in entries {
        let date = entry.date.as_deref().unwrap_or("undated");
        out.push_str(&format!("\n[{}] {} — {}\n", entry.id, date, entry.title));
        for (key, value) in &entry.fields {
            out.push_str(&format!("  {}: {}\n", key, value));
        }
    }

    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty() {
        let output = render(&[]);
        assert!(output.contains("No changes recorded yet."));
    }

    #[test]
    fn test_render_entries() {
        let entries =
            parse_changelog("## [CHG-001] 2026-09-26 — Traceable entries\n\n- **Impacto**: CLI\n");
        let output = render(&entries);
        assert!(output.contains("1 entries recorded"));
        assert!(output.contains("[CHG-001] 2026-09-26 — Traceable entries"));
        assert!(output.contains("Impacto: CLI"));
    }

    #[test]
    fn test_render_undated_entry() {
        let entries = parse_changelog("## [TASK-FW-200] — Reads disk\n");
        let output = render(&entries);
        assert!(output.contains("[TASK-FW-200] undated — Reads disk"));
    }
}
