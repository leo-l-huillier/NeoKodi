use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
// On enlève "directories" pour simplifier au max

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub media_path: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = PathBuf::from("config.json"); // <--- Juste ici, à la racine !
        
        // Si le fichier existe, on le charge
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }

        // Sinon, on prend le dossier courant par défaut
        let default_path = std::env::current_dir()
            .unwrap_or(PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        let new_config = AppConfig { media_path: default_path };
        new_config.save(); // On le crée tout de suite !
        new_config
    }

    pub fn save(&self) {
        let config_path = PathBuf::from("config.json");
        let json = serde_json::to_string_pretty(self).unwrap();
        let _ = fs::write(config_path, json);
    }
}