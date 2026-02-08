use crate::database::db::DB;


use crate::media;
use crate::media::data::Media;
use crate::media::data::MediaType;
use crate::media::audio::Audio;
use crate::media::image::Image;
use crate::media::video::Video;
use crate::media::data::MediaInfo;


use std::fs;
use std::collections::HashMap;
use std::path::Path;


use crate::constants::LOG_FILE;
use crate::scan::scan::Scan;
use std::path::PathBuf;
use std::fs::File;
use lazy_static::lazy_static;
use std::sync::RwLock;

use crate::logger::logger::Logger;

lazy_static! {
    // Une liste de MediaInfo, protégée par un verrou (RwLock) pour que les threads ne se marchent pas dessus.
    pub static ref MEDIA_LIBRARY: RwLock<Vec<MediaInfo>> = RwLock::new(Vec::new());
}

#[derive(Debug, Clone)]
pub struct ScannedMedia {
    pub path: String,
    pub name: String,
    pub duration: f32,
    pub media_type: MediaType,
}
pub struct MediaLibrary {
    pub items: HashMap<i64, Box<dyn Media>>,
    pub scan_lib: Scan,
    pub database: DB,
}

impl MediaLibrary {
    pub fn new() -> Self {

        Self {
            items: HashMap::new(),
            scan_lib: Scan::new(),
            database: DB::new(),
        }
    }

    pub fn init(&mut self) {



        // scan the libraries
        self.database.init_db().unwrap();
        self.scan_lib.scan_libraries();

        // update the database
        self.database.upsert_media_from_scan(self.scan_lib.scan.clone()).unwrap(); //TODO: ce clone me fait chier, il faudrait qu'on utilise juste scan (ca serait meme mieux si on donne la valeur direct comme ca il se fait drop (on en a plus besoin ) et mm en terme de performance c'est pas terrible parce que c'est un gros object )
        self.database.cleanup_missing_media(self.scan_lib.scan.clone()).unwrap(); // TODO to implement, shuld be called every scans
        self.database.get_all_media().unwrap();
        //self.database.print_media_rows();


    
        
        for row in self.database.media_rows.iter() {

            let media: Box<dyn Media> = match row.media_type {
                MediaType::Audio => Box::new(Audio::new(&row.path,&row.title.as_deref().unwrap_or(""))),
                MediaType::Video => Box::new(Video::new(row.id, &row.path, &row.title.as_deref().unwrap_or(""))),
                MediaType::Image => Box::new(Image::new(row.id, &row.path, &row.title.as_deref().unwrap_or(""))), };

            self.items.insert(row.id, media);
        }

    }    

    pub fn reload(&mut self) {
        let logger = Logger::new(LOG_FILE);
        logger.info("🔄 Reloading media library...");
        self.init();
        logger.info(&format!("✅ Media library reloaded with {} items.", self.items.len()));
    }

    pub fn update_media_status_and_time(&mut self, media_id: i64, status: i32, time_stop: f64) {
        let logger = Logger::new(LOG_FILE);
        
        match self.database.update_media_status_and_time(media_id, status, time_stop) {
            Ok(_) => logger.debug(&format!("Updated media ID {} with status {} and time_stop {}", media_id, status, time_stop)),
            Err(e) => logger.error(&format!("Error updating media ID {}: {}", media_id, e)),
        }
    }

    pub fn create_playlist(&mut self, name: &str) {

        let logger = Logger::new(LOG_FILE);

        match self.database.create_playlist(name) {
            Ok(playlist_id) => logger.debug(&format!("Created playlist '{}' with ID {}", name, playlist_id)),
            Err(e) => logger.error(&format!("Playlist '{}' already exists: {}", name, e)),
        }
    }

    pub fn delete_playlist(&mut self, playlist_id: i64) {

        let logger = Logger::new(LOG_FILE);
        match self.database.delete_playlist(playlist_id) {
            Ok(_) => logger.debug(&format!("Deleted playlist with ID {}", playlist_id)),
            Err(e) => logger.error(&format!("Error deleting playlist ID {}: {}", playlist_id, e)),
        }
    }

    pub fn add_media_to_playlist(&mut self, media_id: i64, playlist_id: i64) {

        let logger = Logger::new(LOG_FILE);

        match self.database.add_media_to_playlist(media_id, playlist_id) {
            Ok(_) => logger.debug(&format!("Media ID {} added to Playlist ID {}", media_id, playlist_id)),
            Err(e) => logger.error(&format!("Error adding Media ID {} to Playlist ID {}: {}", media_id, playlist_id, e)),
        }
    }
    
    pub fn remove_media_from_playlist(&mut self, media_id: i64, playlist_id: i64) {

        let logger = Logger::new(LOG_FILE);

        match self.database.remove_media_from_playlist(media_id, playlist_id) {
            Ok(_) => logger.debug(&format!("Media ID {} removed from Playlist ID {}", media_id, playlist_id)),
            Err(e) => logger.error(&format!("Error removing Media ID {} from Playlist ID {}: {}", media_id, playlist_id, e)),
        }
    }

    pub fn get_media_from_playlist(&mut self, playlist_id: i64) -> Vec<i64> {

        let logger = Logger::new(LOG_FILE);

        match self.database.get_media_from_playlist(playlist_id) {
            Ok(media_list) => media_list,
            Err(e) => {
                logger.error(&format!("Error retrieving media from Playlist ID {}: {}", playlist_id, e));
                Vec::new()
            }
        }
    }

    pub fn get_playlist_id(&mut self, name: &str) -> i64 {
        
        let logger = Logger::new(LOG_FILE);

        match self.database.get_playlist_id(name) {
            Ok(playlist_id) => playlist_id,
            Err(e) => {
                logger.error(&format!("Error retrieving playlist ID for '{}': {}", name, e));
                -1
            }
        }
    }

    pub fn get_all_playlists(&mut self) -> Vec<(i64, String)> {

        let logger = Logger::new(LOG_FILE);
        
        match self.database.get_all_playlists() {
            Ok(playlists) => playlists,
            Err(e) => {
                logger.error(&format!("Error retrieving playlists: {}", e));
                Vec::new()
            }
        }
    }

    pub fn add_tag(&mut self, tag_name: &str) {

        let logger = Logger::new(LOG_FILE);

        match self.database.get_or_create_tag(tag_name) {
            Ok(tag_id) => logger.debug(&format!("Tag '{}' has ID {}", tag_name, tag_id)),
            Err(e) => logger.error(&format!("Error adding tag '{}': {}", tag_name, e)),
        }
    }

    pub fn get_all_tags(&mut self) -> Vec<(i64, String)> {

        let logger = Logger::new(LOG_FILE);

        match self.database.get_all_tags() {
            Ok(tags) => tags,
            Err(e) => {
                logger.error(&format!("Error retrieving tags: {}", e));
                Vec::new()
            }
        }
    }

    pub fn remove_tag(&mut self, tag_id: i64) {

        let logger = Logger::new(LOG_FILE);

        match self.database.remove_tag(tag_id) {
            Ok(_) => logger.debug(&format!("Removed tag with ID {}", tag_id)),
            Err(e) => logger.error(&format!("Error removing tag ID {}: {}", tag_id, e)),
        }
    }

    pub fn remove_tag_from_media(&mut self, media_id: i64, tag_id: i64) {
        let logger = Logger::new(LOG_FILE);

        match self.database.remove_tag_from_media(media_id, tag_id) {
            Ok(_) => logger.debug(&format!("Tag ID {} removed from Media ID {}", tag_id, media_id)),
            Err(e) => logger.error(&format!("Error removing Tag ID {} from Media ID {}: {}", tag_id, media_id, e)),
        }
    }

    pub fn add_tag_to_media(&mut self, media_id: i64, tag_id: i64) {

        let logger = Logger::new(LOG_FILE);

        match self.database.add_tag_to_media(media_id, tag_id) {
            Ok(_) => logger.debug(&format!("Tag ID {} added to Media ID {}", tag_id, media_id)),
            Err(e) => logger.error(&format!("Error adding Tag ID {} to Media ID {}: {}", tag_id, media_id, e)),
        }
    }

    pub fn get_tag_id(&mut self, tag_name: &str) -> i64 {
        
        let logger = Logger::new(LOG_FILE);

        match self.database.get_tag_id(tag_name) {
            Ok(tag_id) => tag_id,
            Err(e) => {
                logger.error(&format!("Error retrieving tag ID for '{}': {}", tag_name, e));
                -1
            }
        }
    }

    pub fn add_source(&mut self, path: PathBuf, media_type: MediaType) {
        match media_type {
            MediaType::Audio => self.scan_lib.libraries.add_audio_source(path),
            MediaType::Video => self.scan_lib.libraries.add_video_source(path),
            MediaType::Image => self.scan_lib.libraries.add_image_source(path),
        }

        self.reload();
    }

    pub fn remove_source(&mut self, path: PathBuf, media_type: MediaType) {
        match media_type {
            MediaType::Audio => self.scan_lib.libraries.remove_audio_source(path),
            MediaType::Video => self.scan_lib.libraries.remove_video_source(path),
            MediaType::Image => self.scan_lib.libraries.remove_image_source(path),
        }

        self.reload();
    }

    pub fn get_media_from_path(&mut self, path: PathBuf) -> Vec<MediaInfo> {
        let mut result = Vec::new();

        for (_id, item) in self.items.iter() {
            if item.get_path().starts_with(path.to_str().unwrap()) {
                result.push(item.info());
            }
        }

        result
    }

    pub fn get_all_media(&self) -> Vec<MediaInfo> {
        let mut result = Vec::new();

        for (_id, item) in self.items.iter() {
            result.push(item.info());
        }

        result
    }

    pub fn get_media_by_type(&self, media_type: MediaType) -> Vec<MediaInfo> {
        let mut result = Vec::new();

        for (_id, item) in self.items.iter() {
            if item.media_type() == media_type {
                result.push(item.info());
            }
        }

        result
    }

    pub fn get_media_from_tag(&mut self, tag_id: &str) -> Vec<i64> {
        let media_list = self.database.get_media_by_tag(tag_id).unwrap();
        media_list
    }

   
    pub fn play_id(&mut self, id: i64) {

        let logger = Logger::new(LOG_FILE);

        if let Some(item) = self.items.get_mut(&id) {
            logger.debug(&format!("Playing media ID {id}: {}", item.get_name()));
            item.init();
            item.play();
        } else {
            logger.error(&format!("Error: media with ID {id} not found."));
        }
    }

    pub fn pause_id(&mut self, id: i64) {

        let logger = Logger::new(LOG_FILE);

        logger.debug(&format!("Pausing media ID {id}"));
        if let Some(item) = self.items.get_mut(&id) {
            item.pause();
        } else {
            logger.error(&format!("Error: media with ID {id} not found."));
        }
    }

    pub fn resume_id(&mut self, id: i64) {
        let logger = Logger::new(LOG_FILE);

        if let Some(item) = self.items.get_mut(&id) {
            item.resume();
        } else {
            logger.error(&format!("Error: media with ID {id} not found."));
        }
    }

    pub fn stop_id(&mut self, id: i64) {
        let logger = Logger::new(LOG_FILE);

        if let Some(item) = self.items.get_mut(&id) {
            item.stop();
        } else {
            logger.error(&format!("Error: media with ID {id} not found."));
        }
    }


    pub fn info_id(&self, id: i64) -> Option<MediaInfo> {
        if let Some(item) = self.items.get(&id) {
            Some(item.info())
        } else {
            None
        }
    }

    pub fn media_type_id(&self, id: i64) -> Option<crate::media::data::MediaType> {
        if let Some(item) = self.items.get(&id) {
            Some(item.media_type())
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        println!("🔥 GRAND NETTOYAGE EN COURS...");

        // 1. On vide la mémoire RAM
        self.items.clear();
        
        // 2. On vide la Base de Données
        if let Err(e) = self.database.clear_all_media() {
            println!("❌ Erreur lors du nettoyage de la DB : {}", e);
        } else {
            println!("✅ Base de données vidée.");
        }

        // 3. ON RÉINITIALISE LE SCANNER PROPREMENT
        // Au lieu de supprimer le fichier (ce qui casse tout), on l'écrase avec un JSON vide valide.
        // Cela permet au scanner de repartir sur une base saine.
        let sources_path = Path::new("db/sources.json");
        
        // On écrit un objet JSON vide "{}" pour ne pas faire planter le parser JSON
        if let Err(e) = fs::write(sources_path, "{}") {
             println!("⚠️ Impossible de réinitialiser sources.json : {}", e);
        } else {
             println!("✅ Fichier sources.json réinitialisé (chemins oubliés).");
        }

        // On recharge un scanner tout neuf qui lira ce fichier vide
        self.scan_lib = Scan::new();
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestMedia {
        path: String,
        name: String,
        mtype: MediaType,
        played: bool,
        paused: bool,
        resumed: bool,
        stopped: bool,
    }

    impl TestMedia {
        fn new(id: i64, path: &str, name: &str, mtype: MediaType) -> (i64, Box<dyn Media>) {
            (
                id,
                Box::new(Self {
                    path: path.into(),
                    name: name.into(),
                    mtype,
                    played: false,
                    paused: false,
                    resumed: false,
                    stopped: false,
                }),
            )
        }
    }

    impl Media for TestMedia {
        fn init(&mut self) {}
        fn play(&mut self) { self.played = true; }
        fn pause(&self) { let _ = &self.paused; }
        fn resume(&self) { let _ = &self.resumed; }
        fn stop(&self) { let _ = &self.stopped; }
        fn info(&self) -> MediaInfo {
            MediaInfo {
                id: 0,
                path: self.path.clone(),
                title: Some(self.name.clone()),
                duration: None,
                media_type: MediaType::Audio,
            }
        }
        fn media_type(&self) -> MediaType { MediaType::Audio }
        fn get_name(&self) -> String { self.name.clone() }
        fn get_path(&self) -> String { self.path.clone() }
    }

    fn test_library(items: Vec<(i64, Box<dyn Media>)>) -> MediaLibrary {
        MediaLibrary {
            items: HashMap::from_iter(items),
            scan_lib: Scan { libraries: crate::library::sources::LibraryConfig::load("db/sources.json"), scan: Vec::new() },
            database: DB { conn: rusqlite::Connection::open_in_memory().unwrap(), media_rows: Vec::new() },
        }
    }

    #[test]
    fn info_and_media_type_by_id() {
        let mut lib = test_library(vec![TestMedia::new(1, "/media/a.mp3", "Song", MediaType::Audio)]);
        assert_eq!(lib.info_id(1).unwrap().path, "/media/a.mp3");
        assert!(lib.info_id(99).is_none());
        assert_eq!(lib.media_type_id(1), Some(MediaType::Audio));
        assert_eq!(lib.media_type_id(99), None);
    }

    #[test]
    fn get_all_and_by_type() {
        let lib = test_library(vec![
            TestMedia::new(1, "/media/a.mp3", "Song", MediaType::Audio),
            TestMedia::new(2, "/media/v.mp4", "Clip", MediaType::Video),
        ]);
        let all = lib.get_all_media();
        assert_eq!(all.len(), 2);
        let audios = lib.get_media_by_type(MediaType::Audio);
        //println!("audios: {:?}", audios);
        assert_eq!(audios.len(), 2);
    }

    #[test]
    fn get_media_from_path_filters_prefix() {
        let mut lib = test_library(vec![
            TestMedia::new(1, "/media/music/a.mp3", "A", MediaType::Audio),
            TestMedia::new(2, "/videos/b.mp4", "B", MediaType::Video),
        ]);
        let list = lib.get_media_from_path(PathBuf::from("/media"));
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].path, "/media/music/a.mp3");
    }

    #[test]
    fn play_pause_resume_stop_do_not_panic() {
        let mut lib = test_library(vec![TestMedia::new(1, "/media/a.mp3", "Song", MediaType::Audio)]);
        lib.play_id(1);
        lib.pause_id(1);
        lib.resume_id(1);
        lib.stop_id(1);
    }

    #[test]
    fn add_media_to_playlist_handles_missing() {
        let mut lib = test_library(vec![]);
        // should not panic when ID missing
        lib.play_id(42);
        lib.pause_id(42);
        lib.resume_id(42);
        lib.stop_id(42);
    }
}