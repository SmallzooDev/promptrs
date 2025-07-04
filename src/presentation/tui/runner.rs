use crate::presentation::tui::tui::{TUIApp, AppMode};
use crate::presentation::tui::screens::QuickSelectScreen;
use crate::utils::config::Config;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::path::PathBuf;

pub struct TUI {
    app: TUIApp,
}

impl TUI {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        let app = TUIApp::new(base_path)?;
        Ok(Self { app })
    }

    pub fn app(&self) -> &TUIApp {
        &self.app
    }
}

pub fn run_app(base_path: PathBuf) -> Result<TUI> {
    TUI::new(base_path)
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn handle_event(&self, app: &mut TUIApp, event: Event) -> Result<()> {
        if let Event::Key(key) = event {
            // Clear any error message on key press
            if app.has_error() {
                app.clear_error();
                return Ok(());
            }
            
            // Handle confirmation dialog first if showing
            if app.is_showing_confirmation() {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.confirm_action()?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_confirmation();
                    }
                    _ => {} // Ignore other keys while confirmation dialog is showing
                }
                return Ok(());
            }
            
            // Handle tag filter dialog if showing
            if app.is_tag_filter_dialog_active() {
                let mut should_close = false;
                let mut selected_tag = None;
                let mut should_clear_filter = false;
                
                if let Some(filter_dialog) = app.get_tag_filter_dialog_mut() {
                    match key.code {
                        KeyCode::Esc => {
                            should_close = true;
                        }
                        KeyCode::Up => {
                            filter_dialog.move_up();
                        }
                        KeyCode::Down => {
                            filter_dialog.move_down();
                        }
                        KeyCode::Enter => {
                            selected_tag = filter_dialog.get_selected_tag().cloned();
                            should_close = true;
                        }
                        KeyCode::Char('c') => {
                            should_clear_filter = true;
                            should_close = true;
                        }
                        _ => {}
                    }
                }
                
                if should_close {
                    app.close_tag_filter();
                }
                
                if let Some(tag) = selected_tag {
                    app.set_tag_filter(&tag);
                }
                
                if should_clear_filter {
                    app.clear_tag_filter();
                }
                
                return Ok(());
            }
            
            // Handle tag management dialog if showing
            if app.is_tag_management_active() {
                use crate::presentation::tui::components::TagInputMode;
                
                let mut should_close = false;
                let mut new_tag_to_add = None;
                let mut tag_to_remove = None;
                let mut should_refresh = false;
                
                // First, handle the dialog input
                if let Some(tag_dialog) = app.get_tag_dialog_mut() {
                    match tag_dialog.input_mode() {
                        TagInputMode::ViewTags => {
                            match key.code {
                                KeyCode::Esc => {
                                    should_close = true;
                                }
                                KeyCode::Char('a') => {
                                    tag_dialog.start_adding_tag();
                                }
                                KeyCode::Char('r') => {
                                    tag_dialog.start_removing_tag();
                                }
                                _ => {}
                            }
                        }
                        TagInputMode::AddingTag => {
                            match key.code {
                                KeyCode::Esc => {
                                    tag_dialog.cancel_input();
                                }
                                KeyCode::Enter => {
                                    new_tag_to_add = tag_dialog.get_new_tag();
                                    tag_dialog.cancel_input();
                                    should_refresh = true;
                                }
                                KeyCode::Char(c) => {
                                    tag_dialog.add_char(c);
                                }
                                KeyCode::Backspace => {
                                    tag_dialog.delete_char();
                                }
                                _ => {}
                            }
                        }
                        TagInputMode::RemovingTag => {
                            match key.code {
                                KeyCode::Esc => {
                                    tag_dialog.cancel_input();
                                }
                                KeyCode::Up => {
                                    tag_dialog.move_selection_up();
                                }
                                KeyCode::Down => {
                                    tag_dialog.move_selection_down();
                                }
                                KeyCode::Enter => {
                                    tag_to_remove = tag_dialog.get_selected_tag_for_removal();
                                    tag_dialog.cancel_input();
                                    should_refresh = true;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                
                // Now handle the actions outside of the mutable borrow
                if should_close {
                    app.close_tag_management();
                }
                
                if let Some(new_tag) = new_tag_to_add {
                    if let Err(e) = app.add_tag_to_selected(&new_tag) {
                        eprintln!("Error adding tag: {}", e);
                    }
                }
                
                if let Some(tag) = tag_to_remove {
                    if let Err(e) = app.remove_tag_from_selected(&tag) {
                        eprintln!("Error removing tag: {}", e);
                    }
                }
                
                if should_refresh {
                    // Refresh the dialog with updated tags
                    let updated_tags = app.get_selected_prompt_tags();
                    app.tag_dialog = Some(crate::presentation::tui::components::TagManagementDialog::new(updated_tags));
                }
                
                return Ok(());
            }
            
            // Handle create dialog if showing
            if app.is_create_dialog_active() {
                use crate::presentation::tui::components::DialogField;
                
                let mut should_close = false;
                let mut should_confirm = false;
                
                if let Some(create_dialog) = app.get_create_dialog_mut() {
                    match key.code {
                        KeyCode::Esc => {
                            should_close = true;
                        }
                        KeyCode::Tab => {
                            create_dialog.next_field();
                        }
                        KeyCode::Enter => {
                            if create_dialog.is_valid() {
                                should_confirm = true;
                            }
                        }
                        KeyCode::Left => {
                            if create_dialog.current_field() == DialogField::Template {
                                create_dialog.previous_template();
                            }
                        }
                        KeyCode::Right => {
                            if create_dialog.current_field() == DialogField::Template {
                                create_dialog.next_template();
                            }
                        }
                        KeyCode::Char('h') => {
                            if create_dialog.current_field() == DialogField::Template {
                                create_dialog.previous_template();
                            } else {
                                create_dialog.add_char('h');
                            }
                        }
                        KeyCode::Char('l') => {
                            if create_dialog.current_field() == DialogField::Template {
                                create_dialog.next_template();
                            } else {
                                create_dialog.add_char('l');
                            }
                        }
                        KeyCode::Char(c) => {
                            if create_dialog.current_field() == DialogField::Filename {
                                create_dialog.add_char(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if create_dialog.current_field() == DialogField::Filename {
                                create_dialog.delete_char();
                            }
                        }
                        _ => {}
                    }
                }
                
                if should_close {
                    app.close_create_dialog();
                }
                
                if should_confirm {
                    if let Err(e) = app.confirm_create() {
                        app.set_error(format!("Failed to create prompt: {}", e));
                    }
                }
                
                return Ok(());
            }

            // Handle search mode
            if app.is_search_active() {
                match key.code {
                    KeyCode::Esc => {
                        app.deactivate_search();
                    }
                    KeyCode::Char(c) => {
                        let current_query = app.get_search_query().to_string();
                        app.set_search_query(&format!("{}{}", current_query, c));
                    }
                    KeyCode::Backspace => {
                        let current_query = app.get_search_query();
                        if !current_query.is_empty() {
                            let new_query = current_query[..current_query.len() - 1].to_string();
                            app.set_search_query(&new_query);
                        }
                    }
                    KeyCode::Enter => {
                        // Keep search active but allow selection
                        if matches!(app.mode(), AppMode::QuickSelect) {
                            app.copy_selected_to_clipboard()?;
                            app.quit();
                        }
                    }
                    _ => {} // Ignore other keys in search mode
                }
                return Ok(());
            }

            // Check for search activation (/)
            if key.code == KeyCode::Char('/') {
                app.activate_search();
                return Ok(());
            }

            // Normal key handling
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.quit();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous();
                }
                KeyCode::Enter => {
                    // In QuickSelect mode, copy and quit
                    // In Management mode, we'll handle this differently later
                    if matches!(app.mode(), AppMode::QuickSelect) {
                        app.copy_selected_to_clipboard()?;
                        app.quit();
                    }
                }
                KeyCode::Char('m') => {
                    app.toggle_mode();
                }
                KeyCode::Char('e') => {
                    if matches!(app.mode(), AppMode::Management) {
                        // For now, just mark that edit was requested
                        // The actual editor launch will be handled in the main loop
                        app.set_pending_action(Some(crate::presentation::tui::tui::PendingAction::Edit));
                    }
                }
                KeyCode::Char('d') => {
                    if matches!(app.mode(), AppMode::Management) {
                        app.show_delete_confirmation();
                    }
                }
                KeyCode::Char('n') => {
                    if matches!(app.mode(), AppMode::Management) {
                        if let Err(e) = app.create_new_prompt() {
                            // TODO: Show error in UI
                            eprintln!("Error creating prompt: {}", e);
                        }
                    }
                }
                KeyCode::Char('t') => {
                    if matches!(app.mode(), AppMode::Management) {
                        app.open_tag_management();
                    }
                }
                KeyCode::Char('f') => {
                    // Open tag filter dialog in both modes
                    app.open_tag_filter();
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub fn run(base_path: PathBuf, config: &Config) -> Result<()> {
    run_with_mode(base_path, config, false)
}

pub fn run_manage_mode(base_path: PathBuf, config: &Config) -> Result<()> {
    run_with_mode(base_path, config, true)
}

fn run_with_mode(_base_path: PathBuf, config: &Config, manage_mode: bool) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mode = if manage_mode { AppMode::Management } else { AppMode::QuickSelect };
    let mut app = TUIApp::new_with_mode_and_config(config, mode)?;
    let event_handler = EventHandler::new();

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            if manage_mode {
                // TODO: Render management screen
                let screen = QuickSelectScreen::new(&app);
                screen.render(f, f.size());
            } else {
                let screen = QuickSelectScreen::new(&app);
                screen.render(f, f.size());
            }
        })?;

        // Handle events
        if let Ok(event) = event::read() {
            event_handler.handle_event(&mut app, event)?;
        }

        // Handle pending actions that require exiting TUI temporarily
        if let Some(action) = app.take_pending_action() {
            match action {
                crate::presentation::tui::tui::PendingAction::Edit => {
                    // Exit TUI temporarily
                    disable_raw_mode()?;
                    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
                    
                    // Edit the prompt
                    let result = app.edit_selected();
                    
                    // Restore TUI
                    enable_raw_mode()?;
                    execute!(io::stdout(), terminal::EnterAlternateScreen)?;
                    
                    // Force a full redraw by clearing the terminal
                    terminal.clear()?;
                    
                    if let Err(e) = result {
                        // TODO: Show error in UI
                        eprintln!("Error editing prompt: {}", e);
                    }
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}