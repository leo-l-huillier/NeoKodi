mod media;
use media::data::Media;
use media::video::Video;
use media::audio::Audio;

mod database;
use database::media_library::MediaLibrary;
use database::sources::LibraryConfig;

use std::time::Duration;
use std::thread::sleep;


use std::fs;

fn main() {

    let mut library = MediaLibrary::new();
    library.scan_libraries();

    library.debug_print_items();

    library.play_by_id(1);

    println!("Playing video for 4 seconds...");
    sleep(Duration::from_secs(4));

}