use anyhow::Result;
use sdd_core::discovery::Question;
use sdd_storage::filesystem::FilesystemAdapter;
use std::path::PathBuf;

pub async fn execute(path: Option<PathBuf>, list: bool, add: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    tracing::info!("Managing questions at {:?}", project_path);

    let adapter = FilesystemAdapter::new(&project_path);
    let structure = adapter.structure();

    if !structure.exists() {
        println!("✗ No SDD project found at {:?}", project_path);
        println!("  Run 'sdd init' to initialize a new project");
        return Ok(());
    }

    if list {
        println!("❓ Project Questions");
        println!("===================");
        println!("No questions found.");
        println!("\nUse 'sdd questions --add \"<question>\"' to add questions.");
    } else if let Some(question_text) = add {
        let question = Question::new("Q-001", &question_text, &question_text);

        println!("✓ Added question: {}", question_text);
        println!("  ID: {}", question.id);
        println!("  Status: {:?}", question.status);
    } else {
        println!("Usage:");
        println!("  sdd questions --list          List all questions");
        println!("  sdd questions --add \"<text>\"  Add a new question");
    }

    Ok(())
}
