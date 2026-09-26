//! Integration tests that run the real CLI binary.
//!
//! These tests were written first (TASK-FW-202) and failed against the
//! original implementation: `architecture` panicked on a duplicate clap
//! short flag and `init` produced a wrong `project_id` for "." paths.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sdd"))
}

#[test]
fn architecture_show_parses_without_panic() {
    let output = bin().args(["architecture", "--show"]).output().unwrap();
    assert!(
        output.status.success(),
        "expected success, got {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn architecture_style_parses_without_panic() {
    let output = bin()
        .args(["architecture", "--style", "hexagonal"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "expected success, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn init_derives_project_id_from_current_directory() {
    let temp = tempfile::TempDir::new().unwrap();
    let dir = temp.path().join("my-proj");
    std::fs::create_dir(&dir).unwrap();

    let output = bin().current_dir(&dir).arg("init").output().unwrap();
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest = std::fs::read_to_string(dir.join(".sdd/manifest.json")).unwrap();
    assert!(
        manifest.contains("\"project_id\": \"my-proj\""),
        "expected project_id my-proj, manifest was:\n{}",
        manifest
    );
}

#[test]
fn changelog_runs_in_initialized_project() {
    let temp = tempfile::TempDir::new().unwrap();
    let init = bin().current_dir(temp.path()).arg("init").output().unwrap();
    assert!(init.status.success());

    let output = bin()
        .current_dir(temp.path())
        .arg("changelog")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No changes recorded yet."), "{}", stdout);
}

#[test]
fn validate_passes_in_initialized_project() {
    let temp = tempfile::TempDir::new().unwrap();
    let init = bin().current_dir(temp.path()).arg("init").output().unwrap();
    assert!(init.status.success());

    let output = bin()
        .current_dir(temp.path())
        .arg("validate")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "validate should pass, got {:?}: {}",
        output.status.code(),
        stdout
    );
    assert!(stdout.contains("0 error(s)"), "{}", stdout);
}

#[test]
fn validate_fails_when_required_artifact_missing() {
    let temp = tempfile::TempDir::new().unwrap();
    let init = bin().current_dir(temp.path()).arg("init").output().unwrap();
    assert!(init.status.success());
    std::fs::remove_file(temp.path().join(".sdd/brief/brief.md")).unwrap();

    let output = bin()
        .current_dir(temp.path())
        .arg("validate")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "validate must fail when brief.md is missing"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✗ brief"), "{}", stdout);
}

#[test]
fn status_distinguishes_empty_directories_from_content() {
    let temp = tempfile::TempDir::new().unwrap();
    let init = bin().current_dir(temp.path()).arg("init").output().unwrap();
    assert!(init.status.success());

    let output = bin().current_dir(temp.path()).arg("status").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Brief: ✓"), "{}", stdout);
    assert!(stdout.contains("Requirements: ✗"), "{}", stdout);
    assert!(stdout.contains("Tasks: ✗"), "{}", stdout);
}

#[test]
fn binary_is_named_sdd() {
    let version = bin().arg("--version").output().unwrap();
    assert!(version.status.success());
    let stdout = String::from_utf8_lossy(&version.stdout);
    assert!(stdout.starts_with("sdd "), "unexpected version: {}", stdout);

    let help = bin().arg("--help").output().unwrap();
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("changelog"), "{}", stdout);
    assert!(stdout.contains("validate"), "{}", stdout);
}
