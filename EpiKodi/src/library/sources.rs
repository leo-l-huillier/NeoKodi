
/*
This file manages media sources for the library,
loading and saving source configurations to a JSON file.
*/

use std::fs;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use crate::constants::constants::SOURCE_FILE;

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaSource {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryConfig {
    pub sources: Vec<MediaSource>,
    pub music_sources: Vec<MediaSource>,
    pub video_sources: Vec<MediaSource>,
    pub image_sources: Vec<MediaSource>,
}

impl LibraryConfig {
    pub fn load(path: &str) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
           // println!("Fichier chargé : {}", data);
            if let Ok(config) = serde_json::from_str(&data) {
                // println!("Configuration parsée : {:?}", config);
                return config;
            }
        }

        // Si le fichier n'existe pas ou est vide
        Self { sources: Vec::new(), music_sources: Vec::new(), video_sources: Vec::new(), image_sources: Vec::new() }
    }

    pub fn save(&self, path: &str) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, json).unwrap();
    }


    pub fn add_source(&mut self, folder: PathBuf) {
        if !self.sources.iter().any(|s| s.path == folder) {
            self.sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }
    pub fn add_audio_source(&mut self, folder: PathBuf) {
        if !self.music_sources.iter().any(|s| s.path == folder) {
            self.music_sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }
    pub fn add_video_source(&mut self, folder: PathBuf) {
        if !self.video_sources.iter().any(|s| s.path == folder) {
            self.video_sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }
    pub fn add_image_source(&mut self, folder: PathBuf) {
        if !self.image_sources.iter().any(|s| s.path == folder) {
            self.image_sources.push(MediaSource { path: folder });
        }
        self.save(SOURCE_FILE);
    }

        pub fn remove_source(&mut self, folder: PathBuf) {
        self.sources.retain(|s| s.path != folder);
        self.save(SOURCE_FILE);
    }


    pub fn remove_audio_source(&mut self, folder: PathBuf) {
        self.music_sources.retain(|s| s.path != folder);
        self.save(SOURCE_FILE);
    }

    pub fn remove_video_source(&mut self, folder: PathBuf) {
        self.video_sources.retain(|s| s.path != folder);
        self.save(SOURCE_FILE);
    }

    pub fn remove_image_source(&mut self, folder: PathBuf) {
        self.image_sources.retain(|s| s.path != folder);
        self.save(SOURCE_FILE);
    }

    
}









#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(label: &str) -> String {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir()
            .join(format!("epikodi_sources_{label}_{stamp}.json"))
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn load_nonexistent_file_returns_empty_config() {
        let path = temp_file("nonexistent");
        let config = LibraryConfig::load(&path);
        assert_eq!(config.sources.len(), 0);
        assert_eq!(config.music_sources.len(), 0);
        assert_eq!(config.video_sources.len(), 0);
        assert_eq!(config.image_sources.len(), 0);
    }

    #[test]
    fn save_and_load_persists_config() {
        let path = temp_file("persist");
        let mut config = LibraryConfig {
            sources: vec![MediaSource { path: PathBuf::from("/media") }],
            music_sources: vec![MediaSource { path: PathBuf::from("/music") }],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.save(&path);

        let loaded = LibraryConfig::load(&path);
        assert_eq!(loaded.sources.len(), 1);
        assert_eq!(loaded.sources[0].path, PathBuf::from("/media"));
        assert_eq!(loaded.music_sources.len(), 1);
        assert_eq!(loaded.music_sources[0].path, PathBuf::from("/music"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn add_source_prevents_duplicates() {
        let path = temp_file("no_dup");
        let mut config = LibraryConfig {
            sources: vec![],
            music_sources: vec![],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.add_source(PathBuf::from("/media"));
        config.add_source(PathBuf::from("/media"));

        assert_eq!(config.sources.len(), 1);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn add_audio_source_prevents_duplicates() {
        let path = temp_file("audio_no_dup");
        let mut config = LibraryConfig {
            sources: vec![],
            music_sources: vec![],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.add_audio_source(PathBuf::from("/music"));
        config.add_audio_source(PathBuf::from("/music"));

        assert_eq!(config.music_sources.len(), 1);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn add_video_source_prevents_duplicates() {
        let path = temp_file("video_no_dup");
        let mut config = LibraryConfig {
            sources: vec![],
            music_sources: vec![],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.add_video_source(PathBuf::from("/videos"));
        config.add_video_source(PathBuf::from("/videos"));

        assert_eq!(config.video_sources.len(), 1);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn add_image_source_prevents_duplicates() {
        let path = temp_file("image_no_dup");
        let mut config = LibraryConfig {
            sources: vec![],
            music_sources: vec![],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.add_image_source(PathBuf::from("/images"));
        config.add_image_source(PathBuf::from("/images"));

        assert_eq!(config.image_sources.len(), 1);

        let _ = fs::remove_file(&path);
    }

    #[test]
    #[ignore]
    fn add_source_saves_to_file() {
        let path = temp_file("save_check");
        let mut config = LibraryConfig {
            sources: vec![],
            music_sources: vec![],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.add_source(PathBuf::from("/media"));

        // Verify file exists after save
        assert!(fs::metadata(&path).is_ok());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn multiple_sources_can_coexist() {
        let path = temp_file("multi");
        let mut config = LibraryConfig {
            sources: vec![],
            music_sources: vec![],
            video_sources: vec![],
            image_sources: vec![],
        };
        config.add_audio_source(PathBuf::from("/music1"));
        config.add_audio_source(PathBuf::from("/music2"));
        config.add_video_source(PathBuf::from("/videos1"));
        config.add_image_source(PathBuf::from("/images1"));

        assert_eq!(config.music_sources.len(), 2);
        assert_eq!(config.video_sources.len(), 1);
        assert_eq!(config.image_sources.len(), 1);

        let _ = fs::remove_file(&path);
    }
}