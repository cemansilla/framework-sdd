use anyhow::Result;
use std::path::PathBuf;

/// Run real project checks.
///
/// Hard failures (missing project, broken manifest, missing/empty required
/// artifacts) exit with code 1; incomplete-but-optional artifacts (no
/// requirements/tasks yet) are reported as warnings.
pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Validating project at {:?}", project_path);

    let adapter = sdd_storage::filesystem::FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        anyhow::bail!("validation failed: no SDD project");
    }

    println!("✓ Validating SDD project at {:?}", project_path);

    let mut errors = 0usize;
    let mut warnings = 0usize;

    // Manifest must exist and parse.
    match adapter.load_manifest().await {
        Ok(manifest) => println!("  ✓ manifest.json (project: {})", manifest.project_id),
        Err(err) => {
            println!("  ✗ manifest.json is missing or invalid: {}", err);
            errors += 1;
        }
    }

    // Required artifacts: exist and are not empty.
    let layout = structure.layout();
    let required = [
        ("brief", &layout.brief.join("brief.md")),
        ("architecture", &layout.architecture.join("architecture.md")),
        ("changelog", &layout.changes.join("CHANGELOG.md")),
        ("config", &layout.config.join("project.md")),
    ];
    for (name, file) in required {
        if !file.exists() {
            println!("  ✗ {}: missing {:?}", name, file);
            errors += 1;
        } else if std::fs::metadata(file)
            .map(|m| m.len() == 0)
            .unwrap_or(true)
        {
            println!("  ✗ {}: empty {:?}", name, file);
            errors += 1;
        } else {
            println!("  ✓ {}", name);
        }
    }

    // Optional-but-expected content: requirements and tasks.
    for (name, dir) in [
        ("requirements", &layout.requirements),
        ("tasks", &layout.tasks),
    ] {
        match count_markdown_files(dir) {
            0 => {
                println!("  ! {}: no markdown files yet in {:?}", name, dir);
                warnings += 1;
            }
            n => println!("  ✓ {}: {} file(s)", name, n),
        }
    }

    println!(
        "\n✓ Validation complete: {} error(s), {} warning(s)",
        errors, warnings
    );

    if errors > 0 {
        anyhow::bail!("validation failed with {} error(s)", errors);
    }

    Ok(())
}

fn count_markdown_files(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
                .count()
        })
        .unwrap_or(0)
}
