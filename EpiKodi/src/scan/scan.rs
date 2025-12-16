/*

*/

use crate::library::sources::LibraryConfig;
use crate::library::media_library::ScannedMedia;
use crate::media::data::MediaType;

use crate::constants::constants::{SOURCE_FILE, AUDIO_EXTS, VIDEO_EXTS, IMAGE_EXTS};


use std::path::Path;
use std::fs;


pub struct Scan {
    pub libraries: LibraryConfig,
    pub scan: Vec<ScannedMedia>,
}

impl Scan {
    pub fn new() -> Self {
        Self {
            libraries: LibraryConfig::load(SOURCE_FILE),
            scan: Vec::new(),
        }
    }

    pub fn scan_libraries(&mut self) {

        //=========== SCAN SOURCES ===========

        let music_source_paths: Vec<_> = self.libraries.music_sources.iter().map(|source| source.path.clone()).collect();
        let video_source_paths: Vec<_> = self.libraries.video_sources.iter().map(|source| source.path.clone()).collect();
        let image_source_paths: Vec<_> = self.libraries.image_sources.iter().map(|source| source.path.clone()).collect();

        println!("Scanning sources ...");
        for path in music_source_paths {
            println!("Scanning: {}", path.display());
            self.scan_audio_libraries(&path);
        }
        for path in video_source_paths {
            println!("Scanning: {}", path.display());
            self.scan_video_libraries(&path);
        }
        for path in image_source_paths {
            println!("Scanning: {}", path.display());
            self.scan_image_libraries(&path);
        }
        println!("Scanning  sources end");
    }
    
    fn scan_audio_libraries(&mut self, folder: &Path) {
        if let Ok(entries) = fs::read_dir(folder) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // Recurse
                    self.scan_audio_libraries(&path);
                    continue;
                }

                // Extension check
                let is_audio = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| AUDIO_EXTS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if is_audio {
                    /*println!("audio file: {}", path.display());
                    self.items.insert(
                        self.items.len() as i64,
                        Box::new(Audio::new(
                            path.to_str().unwrap_or_default(),
                            path.file_name().unwrap().to_str().unwrap_or_default(),
                        )),
                    );*/
                    self.scan.push(ScannedMedia {
                        path: path.to_string_lossy().to_string(),
                        name: path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        duration: 0.0, //TODO: get duration
                        media_type: MediaType::Audio,
                    });
                }
            }
        }
    }

    pub fn scan_video_libraries(&mut self, folder: &Path) {
        
        if let Ok(entries) = fs::read_dir(folder) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // Recurse
                    self.scan_video_libraries(&path);
                    continue;
                }

                let is_video = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| VIDEO_EXTS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if is_video {
                    /*  
                    println!("video file: {}", path.display());
                    self.items.insert(
                        self.items.len() as i64,
                        Box::new(Video::new(
                            path.to_str().unwrap_or_default(),
                            path.file_name().unwrap().to_str().unwrap_or_default(),
                        )),
                    */
                    self.scan.push(ScannedMedia {
                        path: path.to_string_lossy().to_string(),
                        name: path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        duration: 0.0, //TODO: get duration
                        media_type: MediaType::Video,
                    });
                }
            }
        }
    }

    pub fn scan_image_libraries(&mut self, folder: &Path) {
        if let Ok(entries) = fs::read_dir(folder) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // Recurse
                    self.scan_image_libraries(&path);
                    continue;
                }

                // Extension check
                let is_image = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| IMAGE_EXTS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if is_image {
                    /*println!("image file: {}", path.display());
                    self.items.insert(
                        self.items.len() as i64,
                        Box::new(Image::new(
                            path.to_str().unwrap_or_default(),
                            path.file_name().unwrap().to_str().unwrap_or_default(),
                        )),
                    );*/
                    self.scan.push(ScannedMedia {
                        path: path.to_string_lossy().to_string(),
                        name: path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        duration: 0.0, //TODO: get duration
                        media_type: MediaType::Image,
                    });
                }
            }
        }
    }

    // pour afficher la liste des items dans la bibliotheque
    //TODO : a enlever plus tard, c'est juste pour debug
    pub fn debug_print_items(&self) {
        println!("=== Library Content start ===");

        for item in &self.scan {
            println!("{} - {} ({})", item.media_type.to_string(), item.name, item.path);
        }

        println!("=== Library Content end ===");
    }

}