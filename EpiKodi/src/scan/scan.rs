/*

*/

use crate::library::sources::LibraryConfig;
use crate::library::media_library::ScannedMedia;
use crate::media::data::MediaType;

use crate::constants::{SOURCE_FILE, AUDIO_EXTS, VIDEO_EXTS, IMAGE_EXTS};

use crate::logger::logger::Logger;
use crate::constants::{LOG_FILE, LOG_FILE_MEDIA_ITEMS};

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

        let logger = Logger::new(LOG_FILE);

        //=========== SCAN SOURCES ===========

        let music_source_paths: Vec<_> = self.libraries.music_sources.iter().map(|source| source.path.clone()).collect();
        let video_source_paths: Vec<_> = self.libraries.video_sources.iter().map(|source| source.path.clone()).collect();
        let image_source_paths: Vec<_> = self.libraries.image_sources.iter().map(|source| source.path.clone()).collect();

        logger.info("Scanning sources ...");
        for path in music_source_paths {
            logger.info(&format!("Scanning: {}", path.display()));
            self.scan_audio_libraries(&path);
        }
        for path in video_source_paths {
            logger.info(&format!("Scanning: {}", path.display()));
            self.scan_video_libraries(&path);
        }
        for path in image_source_paths {
            logger.info(&format!("Scanning: {}", path.display()));
            self.scan_image_libraries(&path);
        }
        logger.info("Scanning sources end");

        self.debug_print_items();
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

        let logger = Logger::new(LOG_FILE_MEDIA_ITEMS);
        logger.debug("=== Library Content start ===");

        for item in &self.scan {
            logger.debug(&format!("{} - {} ({})", item.media_type.to_string(), item.name, item.path));
        }

        logger.debug("=== Library Content end ===");
    }

}















#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Creates a unique temp directory for each test.
    fn temp_dir(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("epikodi_scan_{label}_{stamp}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scan_audio_libraries_collects_audio_files() {
        let dir = temp_dir("audio");
        fs::write(dir.join("song.mp3"), b"").unwrap();
        fs::write(dir.join("note.txt"), b"").unwrap();

        let mut scan = Scan::new();
        scan.scan_audio_libraries(&dir);

        assert_eq!(scan.scan.len(), 1);
        assert_eq!(scan.scan[0].media_type, MediaType::Audio);
        assert!(scan.scan[0].path.ends_with("song.mp3"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scan_video_libraries_collects_video_files() {
        let dir = temp_dir("video");
        fs::write(dir.join("movie.mp4"), b"").unwrap();
        fs::write(dir.join("readme.md"), b"").unwrap();

        let mut scan = Scan::new();
        scan.scan_video_libraries(&dir);

        assert_eq!(scan.scan.len(), 1);
        assert_eq!(scan.scan[0].media_type, MediaType::Video);
        assert!(scan.scan[0].path.ends_with("movie.mp4"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scan_image_libraries_collects_image_files() {
        let dir = temp_dir("image");
        fs::write(dir.join("cover.jpg"), b"").unwrap();
        fs::write(dir.join("doc.pdf"), b"").unwrap();

        let mut scan = Scan::new();
        scan.scan_image_libraries(&dir);

        assert_eq!(scan.scan.len(), 1);
        assert_eq!(scan.scan[0].media_type, MediaType::Image);
        assert!(scan.scan[0].path.ends_with("cover.jpg"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scan_audio_libraries_recurse_into_subdirs() {
        let dir = temp_dir("audio_nested");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("nested.mp3"), b"").unwrap();

        let mut scan = Scan::new();
        scan.scan_audio_libraries(&dir);

        assert_eq!(scan.scan.len(), 1);
        assert!(scan.scan[0].path.ends_with("nested.mp3"));

        let _ = fs::remove_dir_all(dir);
    }
}