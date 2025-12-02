mod media;
use media::data::Media;
use media::video::Video;
use media::audio::Audio;

mod database;
use database::media_library::MediaLibrary;
use database::sources::LibraryConfig;
use std::path::PathBuf;

use std::time::Duration;
use std::thread::sleep;

use std::collections::HashMap;


const SOURCE_FILE: &str = "db/sources.json";



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
    library.scan_libraries();
    library.debug_print_items();


    let paths = fs::read_dir("./media").unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    library.play_by_id(1);

    println!("Playing video for 4 seconds...");
    sleep(Duration::from_secs(4));

    
}