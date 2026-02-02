pub mod plugin_manager;
pub mod manager;
// In your main project: src/plugin/mod.rs

use std::collections::HashMap;

/// Represents metadata about a plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

/// The main trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize the plugin (called once when loaded)
    fn initialize(&mut self) -> Result<(), String>;
    
    /// Handle a media source request
    /// For example: "youtube://watch?v=..." or "netflix://show/123"
    fn can_handle(&self, url: &str) -> bool;
    
    /// Fetch media information from a URL
    fn get_media_info(&self, url: &str) -> Result<MediaInfo, String>;
    
    /// Search for media
    fn search(&self, query: &str) -> Result<Vec<MediaInfo>, String>;
    
    /// Get available categories/sections
    fn get_categories(&self) -> Vec<Category>;
}

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub media_type: MediaType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Playlist,
}

#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
    pub id: String,
}