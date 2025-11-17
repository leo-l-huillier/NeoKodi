



fn scan_media_sources() {
    let library = LibraryConfig::load(SOURCE_FILE);

    for source in &library.music_sources {
        println!("Scanning music source: {:?}", source.path);
        // Implémentez la logique de scan ici
    }

    for source in &library.video_sources {
        println!("Scanning video source: {:?}", source.path);
        // Implémentez la logique de scan ici
    }

    for source in &library.image_sources {
        println!("Scanning image source: {:?}", source.path);
        // Implémentez la logique de scan ici
    }
}