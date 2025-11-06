
mod media;

use media::data::Media;
use media::video::Video;
use media::audio::Audio;

fn main() {
    let video = Video {
        name: "Rust Tutorial".to_string(),
        path: "/videos/rust_tutorial.mp4".to_string(),
        duration: 120.0,
    };

    let audio = Audio {
        name: "Lo-fi Beats".to_string(),
        path: "DJ Rusty".to_string(),
    };

    // We can call methods directly
    video.play();
    audio.play();

    println!("{}", video.info());
    println!("{}", audio.info());

    // You can also store them in a vector of trait objects if needed:
    let library: Vec<Box<dyn Media>> = vec![Box::new(video), Box::new(audio)];

    for item in library.iter() {
        item.play(); // dynamic dispatch
    }
}