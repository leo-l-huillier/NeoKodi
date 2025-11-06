
mod media;

use media::data::Media;
use media::video::Video;
use media::audio::Audio;

use crate::media::image::Image;

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

    let image: Image = Image {
        name: "Rust Logo".to_string(),
        path: "/images/rust_logo.png".to_string(),
    };

    // We can call methods directly
    video.play();
    video.pause();
    video.stop();
    audio.play();
    image.play();

    println!("{}", video.info());
    println!("{}", audio.info());

    // You can also store them in a vector of trait objects if needed:
    let library: Vec<Box<dyn Media>> = vec![Box::new(video), Box::new(audio)];

    for item in library.iter() {
        item.play(); // dynamic dispatch
    }
}