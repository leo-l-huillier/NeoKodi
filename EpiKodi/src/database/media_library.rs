
/*
ce fichier s'occupe de gerer la bibliotheque media,
appeler les fonctions de scan des differentes sources,
et stocker les medias trouves dans une structure de donnees
TODO: recuperer seulement les bonnes extensions de fichiers
TODO: checker les dossiers en recurssif
*/
use crate::database::sources::LibraryConfig;


use crate::media::data::Media;
use crate::media::video::Video;
use crate::media::audio::Audio;
use crate::media::image::Image;

use std::fs;
use std::collections::HashMap;
use std::path::Path;

const SOURCE_FILE: &str = "db/sources.json";
const AUDIO_EXTS: [&str; 4] = ["mp3", "wav", "flac", "ogg"];
const VIDEO_EXTS: [&str; 4] = ["mp4", "mkv", "avi", "mov"];
const IMAGE_EXTS: [&str; 4] = ["jpg", "png", "bmp", "gif"];


pub struct MediaLibrary {
    pub libraries: LibraryConfig,
    pub items: HashMap<u32, Box<dyn Media>>,
}

impl MediaLibrary {
    pub fn new() -> Self {
        Self {
            libraries: LibraryConfig::load(SOURCE_FILE),
            items: HashMap::new(),
        }
    }

    //=========== SCANNING FUNCTIONS ===========
    pub fn scan_libraries(&mut self) {
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
        println!("Scanning  sources end ...");
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
                    println!("audio file: {}", path.display());
                    self.items.insert(
                        self.items.len() as u32,
                        Box::new(Audio::new(
                            path.to_str().unwrap_or_default(),
                            path.file_name().unwrap().to_str().unwrap_or_default(),
                        )),
                    );
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

                // Extension check
                let is_video = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| VIDEO_EXTS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if is_video {
                    println!("video file: {}", path.display());
                    self.items.insert(
                        self.items.len() as u32,
                        Box::new(Video::new(
                            path.to_str().unwrap_or_default(),
                            path.file_name().unwrap().to_str().unwrap_or_default(),
                        )),
                    );
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
                    println!("image file: {}", path.display());
                    self.items.insert(
                        self.items.len() as u32,
                        Box::new(Image::new(
                            path.to_str().unwrap_or_default(),
                            path.file_name().unwrap().to_str().unwrap_or_default(),
                        )),
                    );
                }
            }
        }
    }

    // pour afficher la liste des items dans la bibliotheque
    //TODO : a enlever plus tard, c'est juste pour debug
    pub fn debug_print_items(&self) {
        println!("=== Library Content start ===");
        for (id, item) in &self.items {
            println!("{id} → {}", item.info());
        }
        println!("=== Library Content end ===");
    }



    //=========== MEDIA FUNCTIONS ===========

    pub fn play_id(&mut self, id: u32) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Playing media ID {id}: {}", item.info());
            item.init();
            item.play();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }

    pub fn pause_id(&mut self, id: u32) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Pausing media ID {id}: {}", item.info());
            item.pause();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }

    pub fn resume_id(&mut self, id: u32) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Resuming media ID {id}: {}", item.info());
            item.resume();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }

        pub fn stop_id(&mut self, id: u32) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Stopping media ID {id}: {}", item.info());
            item.stop();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }


    pub fn info_id(&self, id: u32) -> Option<String> {
        if let Some(item) = self.items.get(&id) {
            Some(item.info())
        } else {
            None
        }
    }

    pub fn media_type_id(&self, id: u32) -> Option<crate::media::data::MediaType> {
        if let Some(item) = self.items.get(&id) {
            Some(item.media_type())
        } else {
            None
        }
    }

}


