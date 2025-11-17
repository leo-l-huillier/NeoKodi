mod media;
use media::data::Media;
use media::video::Video;
use media::audio::Audio;

mod database;
use database::sources::LibraryConfig;
use std::path::PathBuf;

use std::time::Duration;
use std::thread::sleep;

use std::collections::HashMap;


const SOURCE_FILE: &str = "db/sources.json";

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

    pub fn scan_audio_libraries(& mut self) {
        for source in &self.libraries.music_sources {
            if let Ok(paths) = fs::read_dir(&source.path) {
                for entry in paths.flatten() {
                    println!("Name: {}", entry.path().display());
                    self.items.insert(
                        self.items.len() as u32,
                        Box::new(Audio::new(
                            entry.path().to_str().unwrap_or_default(),
                            entry.file_name().to_str().unwrap_or_default(),
                        )),
                    );
                }
            } else {
                println!("Warning: folder not found → {}", source.path.display());
            }
        } 
    }

    pub fn scan_video_libraries(& mut self) {
        for source in &self.libraries.video_sources {
            if let Ok(paths) = fs::read_dir(&source.path) {
                for entry in paths.flatten() {
                    println!("Name: {}", entry.path().display());
                    self.items.insert(
                        self.items.len() as u32,
                        Box::new(Video::new(
                            entry.path().to_str().unwrap_or_default(),
                            entry.file_name().to_str().unwrap_or_default(),
                        )),
                    );
                }
            } else {
                println!("Warning: folder not found → {}", source.path.display());
            }
        } 
    }

    pub fn scan_image_libraries(& mut self) {
        for source in &self.libraries.image_sources {
            if let Ok(paths) = fs::read_dir(&source.path) {
                for entry in paths.flatten() {
                    println!("Name: {}", entry.path().display());
                    // Here you would create an Image item and insert it into self.items
                }
            } else {
                println!("Warning: folder not found → {}", source.path.display());
            }
        } 
    }

    pub fn debug_print_items(&self) {
        println!("=== Library Items ===");
        for (id, item) in &self.items {
            println!("{id} → {}", item.info());
        }
    }

    pub fn play_by_id(&mut self, id: u32) {
        if let Some(item) = self.items.get_mut(&id) {
            println!("Playing media ID {id}: {}", item.info());
            item.init();
            item.play();
        } else {
            println!("Error: media with ID {id} not found.");
        }
    }

}



fn test_audio() {
    let mut audio = Audio::new("./media/music/Intro.mp3", "Lo-fi Beats");

    audio.init();

    audio.play();
    println!("Playing for 3s...");
    
    println!("Playing video for 4 seconds...");
    sleep(Duration::from_secs(4));
}

fn test_sources_management() {
    let mut library = LibraryConfig::load(SOURCE_FILE);

    // library.add_audio_source(PathBuf::from("./media/music/"));
    // library.add_video_source(PathBuf::from("./media/videos/"));
    //  library.add_image_source(PathBuf::from("./media/images/"));

    for source in &library.sources {
        println!("{:?}", source.path);
    }

    library.save(SOURCE_FILE);
}

/*
fn main() {
    test_audio();
    test_sources_management();
}*/
use std::fs;

fn main() {
    test_sources_management();

    //test_audio();

    let mut library = MediaLibrary::new();
    library.scan_audio_libraries();
    library.debug_print_items();


    let paths = fs::read_dir("./media").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    library.play_by_id(1);

    println!("Playing video for 4 seconds...");
    sleep(Duration::from_secs(4));

    
}