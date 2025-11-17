
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
}