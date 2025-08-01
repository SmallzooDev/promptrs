use crate::utils::error::{Result, FinkError, PromptError, ExternalError};
use crate::utils::frontmatter::FrontmatterUpdater;
use crate::utils::templates::TemplateGenerator;
use crate::utils::config::Config;
use crate::utils::constants::PROMPTS_DIR;
use std::path::PathBuf;
use std::cell::RefCell;
use crate::application::models::{PromptMetadata, PromptFilter, SearchType, PromptType};
use crate::application::repository::{PromptRepository, FileSystemRepository};
use crate::application::traits::PromptApplication;
use crate::storage::FileSystem;
use crate::external::{ClipboardManager, editor::EditorLauncher};

pub struct DefaultPromptApplication {
    repository: Box<dyn PromptRepository>,
    clipboard: RefCell<ClipboardManager>,
    editor_launcher: RefCell<EditorLauncher>,
}

impl DefaultPromptApplication {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        let storage = FileSystem::new(base_path);
        let repository = Box::new(FileSystemRepository::new(storage));
        let clipboard = RefCell::new(ClipboardManager::new());

        Ok(Self {
            repository,
            clipboard,
            editor_launcher: RefCell::new(EditorLauncher::new()),
        })
    }
    
    pub fn with_config(config: &Config) -> Result<Self> {
        let storage = FileSystem::new(config.storage_path().to_path_buf());
        let repository = Box::new(FileSystemRepository::new(storage));
        let clipboard = RefCell::new(ClipboardManager::new());
        let editor_launcher = EditorLauncher::with_editor(config.editor());

        Ok(Self {
            repository,
            clipboard,
            editor_launcher: RefCell::new(editor_launcher),
        })
    }
    
    pub fn update_editor(&self, editor: &str) {
        *self.editor_launcher.borrow_mut() = EditorLauncher::with_editor(editor);
    }
    
    // Helper methods for cleaner code
    fn find_prompt_metadata(&self, name: &str) -> Result<PromptMetadata> {
        self.repository.find_by_name(name)
            .map_err(FinkError::from)?
            .ok_or_else(|| FinkError::Prompt(PromptError::NotFound(name.to_string())))
    }
    
    fn get_prompt_file_path(&self, metadata: &PromptMetadata) -> PathBuf {
        self.repository
            .get_base_path()
            .join(PROMPTS_DIR)
            .join(&metadata.file_path)
    }
}

impl PromptApplication for DefaultPromptApplication {
    fn list_prompts(&self, filter: Option<PromptFilter>) -> Result<Vec<PromptMetadata>> {
        let mut prompts = self.repository.list_all()
            .map_err(FinkError::from)?;
        
        if let Some(filter) = filter {
            if let Some(tags) = filter.tags {
                prompts.retain(|p| p.tags.iter().any(|t| tags.contains(t)));
            }
        }
        
        Ok(prompts)
    }

    fn get_prompt(&self, identifier: &str) -> Result<(PromptMetadata, String)> {
        let metadata = self.find_prompt_metadata(identifier)?;
        
        let content = self.repository.get_content(&metadata.file_path)
            .map_err(FinkError::from)?;
        
        Ok((metadata, content))
    }

    fn copy_to_clipboard(&self, content: &str) -> Result<()> {
        self.clipboard.borrow_mut().copy(content)
            .map_err(|e| FinkError::External(ExternalError::ClipboardError(e.to_string())))
    }

    fn search_prompts(&self, query: &str, search_type: SearchType) -> Result<Vec<PromptMetadata>> {
        self.repository.search(query, search_type)
            .map_err(FinkError::from)
    }

    fn create_prompt(&self, name: &str, template: Option<&str>) -> Result<()> {
        let normalized_name = name.to_lowercase().replace(' ', "-");
        
        // Check if prompt already exists
        if self.repository.prompt_exists(&normalized_name) {
            return Err(FinkError::Prompt(PromptError::AlreadyExists(name.to_string())));
        }
        
        let content = TemplateGenerator::generate(name, template)?;
        
        // Create the prompt using repository
        self.repository.create_prompt(&normalized_name, &content)
            .map_err(FinkError::from)?;
        Ok(())
    }

    fn create_prompt_with_content(&self, name: &str, template: Option<&str>, content: Option<String>) -> Result<()> {
        let normalized_name = name.to_lowercase().replace(' ', "-");
        
        // Check if prompt already exists
        if self.repository.prompt_exists(&normalized_name) {
            return Err(FinkError::Prompt(PromptError::AlreadyExists(name.to_string())));
        }
        
        let prompt_content = TemplateGenerator::generate_with_content(name, template, content.as_deref())?;
        
        // Create the prompt using repository
        self.repository.create_prompt(&normalized_name, &prompt_content)
            .map_err(FinkError::from)?;
        Ok(())
    }

    fn create_prompt_with_type(&self, name: &str, template: Option<&str>, prompt_type: PromptType) -> Result<()> {
        let normalized_name = name.to_lowercase().replace(' ', "-");
        
        // Check if prompt already exists
        if self.repository.prompt_exists(&normalized_name) {
            return Err(FinkError::Prompt(PromptError::AlreadyExists(name.to_string())));
        }
        
        let content = TemplateGenerator::generate_with_type(name, template, prompt_type)?;
        
        // Create the prompt using repository
        self.repository.create_prompt(&normalized_name, &content)
            .map_err(FinkError::from)?;
        Ok(())
    }

    fn create_prompt_with_content_and_type(&self, name: &str, template: Option<&str>, content: Option<String>, prompt_type: PromptType) -> Result<()> {
        let normalized_name = name.to_lowercase().replace(' ', "-");
        
        // Check if prompt already exists
        if self.repository.prompt_exists(&normalized_name) {
            return Err(FinkError::Prompt(PromptError::AlreadyExists(name.to_string())));
        }
        
        let prompt_content = TemplateGenerator::generate_with_content_and_type(name, template, content.as_deref(), prompt_type)?;
        
        // Create the prompt using repository
        self.repository.create_prompt(&normalized_name, &prompt_content)
            .map_err(FinkError::from)?;
        Ok(())
    }

    fn edit_prompt(&self, name: &str) -> Result<()> {
        let metadata = self.find_prompt_metadata(name)?;
        let file_path = self.get_prompt_file_path(&metadata);
        
        self.editor_launcher.borrow().launch(&file_path)?;
        
        Ok(())
    }

    fn delete_prompt(&self, name: &str, force: bool) -> Result<()> {
        let metadata = self.find_prompt_metadata(name)?;
        
        if !force {
            return Err(FinkError::Validation(crate::utils::error::ValidationError::InvalidInput(
                "confirmation", 
                "Deletion cancelled. Use --force to skip confirmation.".to_string()
            )));
        }
        
        self.repository.delete_prompt(&metadata.file_path)
            .map_err(FinkError::from)
    }

    fn copy_prompt(&self, name: &str) -> Result<()> {
        // Get the prompt content
        let (_, content) = self.get_prompt(name)?;
        
        // Copy to clipboard
        self.copy_to_clipboard(&content)?;
        
        Ok(())
    }

    fn get_base_path(&self) -> &std::path::Path {
        self.repository.get_base_path()
    }

    fn update_prompt_tags(&self, name: &str, tags: Vec<String>) -> Result<()> {
        let metadata = self.find_prompt_metadata(name)?;
        
        let content = self.repository.read_prompt(&metadata)?;
        let updated_content = FrontmatterUpdater::update_tags(&content, name, &tags)?;
        
        self.repository.write_prompt(&metadata, &updated_content)?;
        
        Ok(())
    }
    
    fn get_clipboard_content(&self) -> Result<String> {
        self.clipboard.borrow_mut().get_content()
            .map_err(|e| FinkError::External(ExternalError::ClipboardError(e.to_string())))
    }
}