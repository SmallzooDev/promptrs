use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn should_list_prompts_with_list_command() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();

    let prompt1 = r#"---
name: "Code Review"
tags: ["code", "review"]
---
# Code Review Template"#;

    let prompt2 = r#"---
name: "Bug Report"
tags: ["bug", "issue"]
---
# Bug Report Template"#;

    std::fs::write(prompts_dir.join("code-review.md"), prompt1).unwrap();
    std::fs::write(prompts_dir.join("bug-report.md"), prompt2).unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("list")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Code Review"))
        .stdout(predicate::str::contains("Bug Report"))
        .stdout(predicate::str::contains("code, review"))
        .stdout(predicate::str::contains("bug, issue"));
}

#[test]
fn should_handle_empty_directory_with_list_command() {
    // Arrange
    let temp_dir = tempdir().unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("list")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No prompts found"));
}

#[test]
fn should_get_prompt_content_by_name() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();

    let prompt = r#"---
name: "Code Review"
tags: ["code", "review"]
---
# Code Review Template

This is the code review template content."#;

    std::fs::write(prompts_dir.join("code-review.md"), prompt).unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("get")
        .arg("code-review")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("# Code Review Template"))
        .stdout(predicate::str::contains(
            "This is the code review template content.",
        ));
}

#[test]
fn should_handle_nonexistent_prompt() {
    // Arrange
    let temp_dir = tempdir().unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("get")
        .arg("nonexistent")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Prompt not found"));
}

#[test]
fn should_create_new_prompt_with_create_command() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("create")
        .arg("test-prompt")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify the file was created
    let created_file = prompts_dir.join("test-prompt.md");
    assert!(created_file.exists());

    // Verify the content
    let content = std::fs::read_to_string(&created_file).unwrap();
    assert!(content.contains("name: \"test-prompt\""));
}

#[test]
fn should_create_prompt_with_template() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("create")
        .arg("new-prompt")
        .arg("--template")
        .arg("basic")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify the file was created
    let created_file = prompts_dir.join("new-prompt.md");
    assert!(created_file.exists());

    // Verify the content has the template guide
    let content = std::fs::read_to_string(&created_file).unwrap();
    assert!(content.contains("name: \"new-prompt\""));
    assert!(content.contains("# Instruction"));
    assert!(content.contains("# Context"));
    assert!(content.contains("# Input Data"));
    assert!(content.contains("# Output Indicator"));
}

#[test]
fn should_fail_when_creating_duplicate_prompt() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    // Create an existing prompt
    let existing_content = r#"---
name: "existing"
tags: []
---
# Existing"#;
    std::fs::write(prompts_dir.join("existing.md"), existing_content).unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("create")
        .arg("existing")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn should_fail_when_template_not_found() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();

    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("create")
        .arg("new-prompt")
        .arg("--template")
        .arg("invalid-template")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown template"));
}

#[test]
fn should_edit_prompt_with_external_editor() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let original_content = r#"---
name: "Test Prompt"
tags: ["test"]
---
# Original Content"#;
    
    std::fs::write(prompts_dir.join("test-prompt.md"), original_content).unwrap();
    
    // Create a mock editor script that modifies the file
    let mock_editor_path = temp_dir.path().join("mock_editor.sh");
    let mock_editor_content = r#"#!/bin/bash
echo '---
name: "Test Prompt"
tags: ["test", "edited"]
---
# Edited Content' > "$1""#;
    
    std::fs::write(&mock_editor_path, mock_editor_content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&mock_editor_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("edit")
        .arg("test-prompt")
        .arg("--path")
        .arg(temp_dir.path())
        .env("EDITOR", mock_editor_path.to_str().unwrap())
        .assert()
        .success();
    
    // Verify the file was edited
    let edited_content = std::fs::read_to_string(prompts_dir.join("test-prompt.md")).unwrap();
    assert!(edited_content.contains("Edited Content"));
    assert!(edited_content.contains("edited"));
}

#[test]
fn should_fail_when_editing_nonexistent_prompt() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("edit")
        .arg("nonexistent")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Prompt not found"));
}

#[test]
fn should_delete_prompt_with_force_flag() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let prompt_content = r#"---
name: "Test Prompt"
tags: ["test"]
---
# Test Content"#;
    
    let prompt_file = prompts_dir.join("test-prompt.md");
    std::fs::write(&prompt_file, prompt_content).unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("delete")
        .arg("test-prompt")
        .arg("--force")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
    
    // Verify the file was deleted
    assert!(!prompt_file.exists());
}

#[test]
fn should_require_force_flag_for_deletion() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let prompt_content = r#"---
name: "Test Prompt"
tags: ["test"]
---
# Test Content"#;
    
    let prompt_file = prompts_dir.join("test-prompt.md");
    std::fs::write(&prompt_file, prompt_content).unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("delete")
        .arg("test-prompt")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Use --force to skip confirmation"));
    
    // Verify the file was NOT deleted
    assert!(prompt_file.exists());
}

#[test]
fn should_fail_when_deleting_nonexistent_prompt() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("delete")
        .arg("nonexistent")
        .arg("--force")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Prompt not found"));
}

#[test]
fn should_copy_prompt_to_clipboard() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let prompt_content = r#"---
name: "Test Prompt"
tags: ["test"]
---
# Test Content
This is the prompt content to copy."#;
    
    std::fs::write(prompts_dir.join("test-prompt.md"), prompt_content).unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("copy")
        .arg("test-prompt")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Copied to clipboard"));
}

#[test]
fn should_fail_when_copying_nonexistent_prompt() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("copy")
        .arg("nonexistent")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Prompt not found"));
}

#[test]
fn should_search_prompts_by_name() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let prompt1 = r#"---
name: "Code Review Assistant"
tags: ["code", "review"]
---
# Code Review"#;
    
    let prompt2 = r#"---
name: "Bug Analysis"
tags: ["debug", "analysis"]
---
# Bug Analysis"#;
    
    let prompt3 = r#"---
name: "Documentation Helper"
tags: ["docs", "writing"]
---
# Documentation"#;
    
    std::fs::write(prompts_dir.join("code-review.md"), prompt1).unwrap();
    std::fs::write(prompts_dir.join("bug-analysis.md"), prompt2).unwrap();
    std::fs::write(prompts_dir.join("documentation.md"), prompt3).unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("search")
        .arg("review")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Code Review Assistant"))
        .stdout(predicate::str::contains("[code, review]"))
        .stdout(predicate::str::contains("Bug Analysis").not())
        .stdout(predicate::str::contains("Documentation Helper").not());
}

#[test]
fn should_search_prompts_by_tag() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let prompt1 = r#"---
name: "Code Review"
tags: ["code", "review"]
---
# Code Review"#;
    
    let prompt2 = r#"---
name: "Bug Report"
tags: ["bug", "code"]
---
# Bug Report"#;
    
    std::fs::write(prompts_dir.join("code-review.md"), prompt1).unwrap();
    std::fs::write(prompts_dir.join("bug-report.md"), prompt2).unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("search")
        .arg("code")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Code Review"))
        .stdout(predicate::str::contains("Bug Report"));
}

#[test]
fn should_show_message_when_no_search_results() {
    // Arrange
    let temp_dir = tempdir().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir(&prompts_dir).unwrap();
    
    let prompt = r#"---
name: "Test Prompt"
tags: ["test"]
---
# Test"#;
    
    std::fs::write(prompts_dir.join("test.md"), prompt).unwrap();
    
    // Act & Assert
    let mut cmd = Command::cargo_bin("fink").unwrap();
    cmd.arg("search")
        .arg("nonexistent")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No prompts found matching"));
}
