mod media;

use media::data::Media;
use media::video::Video;
use media::audio::Audio;
use media::image::Image;
use std::time::Duration;
use std::thread::sleep;

fn main() {
    // --- Audio Test ---
    let mut audio = Audio::new("media/test.mp3", "Lo-fi Beats");
    audio.init();
    audio.play();
    println!("Playing audio for 3 seconds...");
    sleep(Duration::from_secs(3));
    audio.pause();
    println!("Audio paused for 2 seconds...");
    sleep(Duration::from_secs(2));
    audio.resume();
    println!("Audio resumed for 2 seconds...");
    sleep(Duration::from_secs(2));
    audio.stop();
    println!("Audio stopped.\n");

    // --- Image Test ---
    let mut image = Image::new("media/test_image.svg", "Test Image");
    image.init();
    image.play();
    println!("Showing image for 4 seconds...");
    sleep(Duration::from_secs(4));
    image.stop();
    println!("Image stopped.");
    sleep(Duration::from_millis(500));

    // --- Video Test ---
    let mut video = Video::new("media/DRG_LogoIntro_Lower_Sound_720p30.mp4", "Demo Video");
    video.init();
    video.play();
    println!("Playing video for 4 seconds...");
    sleep(Duration::from_secs(4));
    video.pause();
    println!("Video paused for 2 seconds...");
    sleep(Duration::from_secs(2));
    video.play(); // resume
    println!("Video resumed for 2 seconds...");
    sleep(Duration::from_secs(2));
    video.stop();
    println!("Video stopped.\n");
}