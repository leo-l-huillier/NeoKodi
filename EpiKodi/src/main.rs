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

    pub fn scan_audio_libraries(&self) {
        for source in &self.libraries.music_sources {
            if let Ok(paths) = fs::read_dir(&source.path) {
                for entry in paths.flatten() {
                    println!("Name: {}", entry.path().display());
                }
            } else {
                println!("Warning: folder not found → {}", source.path.display());
            }
        } 

        for source in &self.libraries.music_sources {
            //println!("{:?}", source.path);
            let paths = fs::read_dir("./media").unwrap();
            for path in paths {
                println!("Name: {}", path.unwrap().path().display())
            }
        }
    }
}



fn test_audio() {
    let mut audio = Audio::new("media/test.mp3", "Lo-fi Beats");

    audio.init();

    audio.play();
    println!("Playing for 3s...");

    // --- Video Test ---
    let mut video = Video::new("media/DRG_LogoIntro_Lower_Sound_720p30.mp4", "Demo Video");
    video.init();
    video.play();
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

    let library = MediaLibrary::new();
    library.scan_audio_libraries();


    let paths = fs::read_dir("./media").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}