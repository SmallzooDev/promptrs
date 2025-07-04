use jkms::presentation::tui::tui::TUIApp;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_create_dialog_integration() {
    // Setup test environment
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create jkms directory
    let jkms_path = temp_path.join("jkms");
    fs::create_dir_all(&jkms_path).unwrap();
    
    // Create TUIApp
    let mut app = TUIApp::new(temp_path.clone()).unwrap();
    
    // Open create dialog
    app.create_new_prompt().unwrap();
    
    // Verify dialog is open
    assert!(app.is_create_dialog_active());
    assert!(app.get_create_dialog().is_some());
}

#[test]
fn test_create_prompt_with_dialog() {
    // Setup test environment
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create jkms directory
    let jkms_path = temp_path.join("jkms");
    fs::create_dir_all(&jkms_path).unwrap();
    
    // Create TUIApp
    let mut app = TUIApp::new(temp_path.clone()).unwrap();
    
    // Open create dialog
    app.create_new_prompt().unwrap();
    
    // Simulate user input
    if let Some(dialog) = app.get_create_dialog_mut() {
        dialog.add_char('t');
        dialog.add_char('e');
        dialog.add_char('s');
        dialog.add_char('t');
        dialog.add_char('-');
        dialog.add_char('p');
        dialog.add_char('r');
        dialog.add_char('o');
        dialog.add_char('m');
        dialog.add_char('p');
        dialog.add_char('t');
    }
    
    // Confirm creation
    app.confirm_create().unwrap();
    
    // Verify dialog is closed
    assert!(!app.is_create_dialog_active());
    
    // Verify prompt was created
    let created_file = jkms_path.join("test-prompt.md");
    assert!(created_file.exists());
    
    // Verify prompt content
    let content = fs::read_to_string(&created_file).unwrap();
    assert!(content.contains("name: \"test-prompt\""));
}

#[test]
fn test_create_prompt_with_template() {
    // Setup test environment
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create jkms directory
    let jkms_path = temp_path.join("jkms");
    fs::create_dir_all(&jkms_path).unwrap();
    
    // Create TUIApp
    let mut app = TUIApp::new(temp_path.clone()).unwrap();
    
    // Open create dialog
    app.create_new_prompt().unwrap();
    
    // Simulate user input and template selection
    if let Some(dialog) = app.get_create_dialog_mut() {
        dialog.add_char('w');
        dialog.add_char('i');
        dialog.add_char('t');
        dialog.add_char('h');
        dialog.add_char('-');
        dialog.add_char('t');
        dialog.add_char('e');
        dialog.add_char('m');
        dialog.add_char('p');
        dialog.add_char('l');
        dialog.add_char('a');
        dialog.add_char('t');
        dialog.add_char('e');
        
        // Switch to template field and select basic template
        dialog.next_field();
        dialog.next_template(); // Default
        dialog.next_template(); // Basic template
    }
    
    // Confirm creation
    app.confirm_create().unwrap();
    
    // Verify prompt was created with template
    let created_file = jkms_path.join("with-template.md");
    assert!(created_file.exists());
    
    let content = fs::read_to_string(&created_file).unwrap();
    assert!(content.contains("name: \"with-template\""));
    // Should have template content
    assert!(content.contains("# Instruction"));
}

#[test]
fn test_cancel_create_dialog() {
    // Setup test environment
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create jkms directory
    let jkms_path = temp_path.join("jkms");
    fs::create_dir_all(&jkms_path).unwrap();
    
    // Create TUIApp
    let mut app = TUIApp::new(temp_path.clone()).unwrap();
    
    // Open create dialog
    app.create_new_prompt().unwrap();
    
    // Add some text
    if let Some(dialog) = app.get_create_dialog_mut() {
        dialog.add_char('t');
        dialog.add_char('e');
        dialog.add_char('s');
        dialog.add_char('t');
    }
    
    // Cancel dialog
    app.close_create_dialog();
    
    // Verify dialog is closed
    assert!(!app.is_create_dialog_active());
    
    // Verify no prompt was created
    let test_file = jkms_path.join("test.md");
    assert!(!test_file.exists());
}