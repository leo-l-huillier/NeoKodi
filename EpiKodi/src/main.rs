/* 
mod media;

use media::data::Media;
use media::video::Video;
use media::audio::Audio;
use std::time::Duration;
use std::thread::sleep;

fn main() {
    let mut audio = Audio::new("media/test.mp3", "Lo-fi Beats");

    audio.init();

    audio.play();
    println!("Playing for 3s...");

    sleep(Duration::from_secs(4));
}*/


mod media;
use media::data::Media;
use media::video::Video;
use media::audio::Audio;

mod database;
use database::library::LibraryConfig;
use std::path::PathBuf;


use std::time::Duration;
use std::thread::sleep;


const SOURCE_FILE: &str = "db/sources.json";

fn test_audio() {
    let mut audio = Audio::new("media/test.mp3", "Lo-fi Beats");

    audio.init();

    audio.play();
    println!("Playing for 3s...");

    sleep(Duration::from_secs(4));
}

fn test_sources_management() {
    let mut library = LibraryConfig::load(SOURCE_FILE);
    for source in &library.sources {
        println!("Source existante : {:?}", source.path);
    }

    library.add_source(PathBuf::from("/home/leo/Music2"));
    library.add_video_source(PathBuf::from("./video/videooo2"));
    library.add_image_source(PathBuf::from("./images/cacdlimage2"));
    library.add_music_source(PathBuf::from("./music/cacdlmusic2"));

    library.save(SOURCE_FILE);

    println!("Sources actuelles : {:?}", library.sources);
    println!("Sources musicales actuelles : {:?}", library.music_sources);
    println!("Sources vidéo actuelles : {:?}", library.video_sources);
    println!("Sources d'images actuelles : {:?}", library.image_sources);
}

fn main() {
    test_audio();
    test_sources_management();
}