// media-center/src/plugin/manager.rs

use std::path::{Path, PathBuf};
use std::sync::Arc;
use libloading::{Library, Symbol};
use plugin_api::{Plugin, PluginMetadata};

type PluginCreate = unsafe fn() -> *mut dyn Plugin;

pub struct PluginManager {
    plugins: Vec<PluginInstance>,
    plugin_dir: PathBuf,
}

struct PluginInstance {
    plugin: Box<dyn Plugin>,
    _lib: Arc<Library>,
    metadata: PluginMetadata,
}

impl PluginManager {
    pub fn new(plugin_dir: impl AsRef<Path>) -> Self {
        Self {
            plugins: Vec::new(),
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }
    
    pub fn load_all(&mut self) -> Result<usize, String> {
         println!("aaaaaaaaaaaaaaaa");
        let entries = std::fs::read_dir(&self.plugin_dir)
            .map_err(|e| format!("Failed to read plugin dir: {}", e))?;
        
        let mut loaded = 0;
         println!("aaaaaaaaaaaaaaaa");
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            #[cfg(target_os = "linux")]
            let is_lib = path.extension().and_then(|s| s.to_str()) == Some("so");
            #[cfg(target_os = "macos")]
            let is_lib = path.extension().and_then(|s| s.to_str()) == Some("dylib");
            #[cfg(target_os = "windows")]
            let is_lib = path.extension().and_then(|s| s.to_str()) == Some("dll");
             println!("aaaaaaaaaaaa4aaaa");
            
            if is_lib {
                match self.load_plugin(&path) {
                    Ok(_) => {
                        println!("Loaded plugin: {}", path.display());
                        loaded += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to load {}: {}", path.display(), e);
                    }
                }
            }
             println!("aaaaaaaaaaaaaaaa");
        }
        
        Ok(loaded)
    }
    
    pub fn load_plugin(&mut self, path: &Path) -> Result<(), String> {
        unsafe {
             println!("bbbbbbbbbbbbbbbbbbbb");
            let lib = Library::new(path)
                .map_err(|e| format!("Failed to load library: {}", e))?;
              println!("bbbbbbbbbbbbbbbbbbbb");
            let constructor: Symbol<PluginCreate> = lib
                .get(b"create_plugin")
                .map_err(|e| format!("Failed to find create_plugin: {}", e))?;
              println!("bbbbbbbbbbbbbbbbbbbb");
            let plugin_ptr = constructor();
              println!("bbbbbbbbbbb4bbbbbbbbb");
            let mut plugin = Box::from_raw(plugin_ptr);
              println!("bbbbbbbbbb4bbbbbbbbbb");
            plugin.initialize()
                .map_err(|e| format!("Plugin initialization failed: {}", e))?;
            
            let metadata = plugin.metadata();
            
            self.plugins.push(PluginInstance {
                plugin,
                _lib: Arc::new(lib),
                metadata,
            });
            
            Ok(())
        }
    }
    
    pub fn get_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.iter().map(|p| &p.metadata).collect()
    }
    
    pub fn find_handler(&self, url: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.plugin.can_handle(url))
            .map(|p| p.plugin.as_ref())
    }
}