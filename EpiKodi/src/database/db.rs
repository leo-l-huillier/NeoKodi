/*

*/

use rusqlite::{Connection, Result};

use crate::library::media_library::ScannedMedia;
use crate::media::data::MediaType;

#[derive(Debug)]
pub struct MediaRow {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub duration: Option<f32>,
    pub media_type: MediaType,
}

pub struct DB {
    pub conn: Connection,
    pub media_rows: Vec<MediaRow>,
}


impl DB {
    pub fn new() -> Self {
        DB {
            conn: Connection::open("db/library.db").unwrap(),
            media_rows: Vec::new(),
        }
    }

    //TODO: check if this is a corrct/clean way to do this
    pub fn init_db(&mut self) -> Result<()> {
        self.conn.execute(
            "
                CREATE TABLE IF NOT EXISTS media (
                    id INTEGER PRIMARY KEY,
                    path TEXT UNIQUE NOT NULL,
                    title TEXT,
                    duration REAL,
                    media_type TEXT
                )
            ",
            [],
        )?;
         self.conn.execute(
            "
                CREATE TABLE IF NOT EXISTS tags (
                    id INTEGER PRIMARY KEY,
                    name TEXT UNIQUE NOT NULL
                );
            ",
            [],
        )?;
         self.conn.execute(
            "
                CREATE TABLE IF NOT EXISTS media_tags (
                    media_id INTEGER NOT NULL,
                    tag_id INTEGER NOT NULL,
                    PRIMARY KEY (media_id, tag_id),
                    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE,
                    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
                );
            ",
            [],
        )?;
        self.conn.execute(
            "
                CREATE TABLE IF NOT EXISTS playlists (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE COLLATE NOCASE
                );
            ",
            [],
        )?;
        self.conn.execute(
            "
                CREATE TABLE IF NOT EXISTS playlist_items (
                    playlist_id INTEGER NOT NULL,
                    media_id INTEGER NOT NULL,
                    PRIMARY KEY (playlist_id, media_id),
                    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE
                );
            ",
            [],
        )?;
        Ok(())
    }

    //========MEDIA TABLE METHODS========

    pub fn insert_media(&mut self, path: &str, title: &str, duration: f32, media_type: &str) -> Result<()> {
        self.conn.execute(
            "
                INSERT OR IGNORE INTO media (path, title, duration, media_type)
                VALUES (?1, ?2, ?3, ?4)
            ",
            (path, title, duration, media_type),
        )?;
        Ok(())
    }

    pub fn get_all_media(&mut self) -> Result<&Vec<MediaRow>> {

        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, duration, media_type FROM media"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(MediaRow {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                duration: row.get(3)?,
                media_type: MediaType::from_db(&row.get::<_, String>(4)?).unwrap(),
            })
        })?;

        for r in rows {
           self.media_rows.push(r?);
        }
        Ok(&self.media_rows)
    }


    pub fn print_media_rows(&mut self) {
        println!("{:#?}", self.media_rows);
    }

    pub fn upsert_media(&mut self, media: &ScannedMedia) -> rusqlite::Result<()> {
        self.conn.execute(
            "
            INSERT INTO media (path, title, duration, media_type)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(path) DO UPDATE SET
                title = excluded.title,
                duration = excluded.duration,
                media_type = excluded.media_type
            ",
            (
                &media.path,
                &media.name,
                media.duration,
                &media.media_type.to_string(),
            ),
        )?;
        Ok(())
    }

    pub fn upsert_media_from_scan(&mut self, scanned_media: Vec<ScannedMedia>) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "
                INSERT INTO media (path, title, duration, media_type)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(path) DO UPDATE SET
                    title = excluded.title,
                    duration = excluded.duration,
                    media_type = excluded.media_type
                ",
            )?;

            for media in scanned_media {
                stmt.execute((
                    &media.path,
                    &media.name,
                    media.duration,
                    &media.media_type.to_string(),
                ))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn cleanup_missing_media(&mut self, scanned_media: Vec<ScannedMedia>) -> rusqlite::Result<()> {

        let scanned_paths: Vec<String> = scanned_media.iter().map(|m| m.path.clone()).collect();
        let placeholders = scanned_paths.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM media WHERE path NOT IN ({})", placeholders);

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = scanned_paths.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        stmt.execute(rusqlite::params_from_iter(params))?;
        Ok(())
    }

    //========= TAGS TABLE METHODS========

    pub fn get_or_create_tag(&mut self, name: &str) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            [name],
        )?;

        self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
    }

    //TODO: y a pas de gestion d'erreur ici c'est important à corriger
    pub fn get_tag_id(&mut self, name: &str) -> rusqlite::Result<i64> {
        self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
    }


    pub fn add_tag_to_media(&mut self, media_id: i64, tag_id: i64) -> rusqlite::Result<()> {
        self.conn.execute(
            "
                INSERT OR IGNORE INTO media_tags (media_id, tag_id)
                VALUES (?1, ?2)
            ",
            (media_id, tag_id),
        )?;
        Ok(())
    }


    pub fn get_media_by_tag(&mut self, tag_name: &str) -> rusqlite::Result<Vec<i64>> {
        let mut stmt = self.conn.prepare(
            "
                SELECT media.id
                FROM media
                JOIN media_tags ON media.id = media_tags.media_id
                JOIN tags ON tags.id = media_tags.tag_id
                WHERE tags.name = ?1
            ",
        )?;

        let rows = stmt.query_map([tag_name], |row| row.get(0))?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    //========= PLAYLIST TABLE METHODS========


    pub fn create_playlist(&mut self, name: &str) -> rusqlite::Result<i64> {


        self.conn.execute(
            "INSERT OR IGNORE INTO playlists (name) VALUES (?1)",
            [name],
        )?;

        self.conn.query_row(
            "SELECT id FROM playlists WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
    }

    pub fn add_media_to_playlist(&mut self, media_id: i64, playlist_id: i64) -> rusqlite::Result<()> {
        self.conn.execute(
            "
            INSERT OR REPLACE INTO playlist_items
            (playlist_id, media_id)
            VALUES (?1, ?2)
            ",
            (playlist_id, media_id),
        )?;
        Ok(())
    }

    pub fn get_media_from_playlist(&mut self, playlist_id: i64) -> rusqlite::Result<Vec<i64>> {
        let mut stmt = self.conn.prepare(
            "
                SELECT media_id
                FROM playlist_items
                WHERE playlist_id = ?1
                ORDER BY position ASC
            ",
        )?;

        let rows = stmt.query_map([playlist_id], |row| row.get(0))?;

        Ok(rows.filter_map(Result::ok).collect())
    }
      //ORDER BY position ASC
    pub fn get_playlist_id(&mut self, name: &str) -> rusqlite::Result<i64> {
        self.conn.query_row(
            "SELECT id FROM playlists WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
    }









    //========TESTING PURPOSES ONLY========

    pub fn clear_media_table(&mut self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM media",
            [],
        )?;
        Ok(())
    }

    pub fn add_sample_data(&mut self) -> Result<()> {
        self.insert_media("path/to/media1.mp4", "Sample Media 1", 300.0, "video")?;
        self.insert_media("path/to/media2.mp3", "Sample Media 2", 200.0, "audio")?;
        self.insert_media("path/to/media3.jpg", "Sample Media 3", 0.0, "image")?;
        Ok(())
    }

    

} 












#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_db() -> DB {
        let conn = Connection::open_in_memory().unwrap();
        let mut db = DB {
            conn,
            media_rows: Vec::new(),
        };
        db.init_db().unwrap();
        db
    }

    #[test]
    fn test_init_db() {
        let mut db = create_test_db();

        assert!(db.init_db().is_ok());
    }

    #[test]
    fn test_insert_media() {
        let mut db = create_test_db();

        assert!(db.insert_media("path/to/video.mp4", "Test Video", 120.5, "video").is_ok());
    }

    #[test]
    fn test_get_all_media() {
        let mut db = create_test_db();
        db.insert_media("path/to/video.mp4", "Test Video", 120.5, "video").unwrap();
        
        let media = db.get_all_media().unwrap();

        assert_eq!(media.len(), 1);
        assert_eq!(media[0].path, "path/to/video.mp4");
    }

    #[test]
    fn test_get_or_create_tag() {
        let mut db = create_test_db();
        let tag_id1 = db.get_or_create_tag("action").unwrap();
        let tag_id2 = db.get_or_create_tag("action").unwrap();

        assert_eq!(tag_id1, tag_id2);
    }

    #[test]
    fn test_get_tag_id() {
        let mut db = create_test_db();
        db.get_or_create_tag("comedy").unwrap();

        let tag_id = db.get_tag_id("comedy").unwrap();

        assert!(tag_id > 0);
    }

    #[test]
    fn test_add_tag_to_media() {
        let mut db = create_test_db();
        db.insert_media("path/to/video.mp4", "Test Video", 120.5, "video").unwrap();
        
        let tag_id = db.get_or_create_tag("action").unwrap();
        
        assert!(db.add_tag_to_media(1, tag_id).is_ok());
    }

    #[test]
    fn test_get_media_by_tag() {
        let mut db = create_test_db();
        db.insert_media("path/to/video1.mp4", "Video 1", 120.0, "video").unwrap();
        db.insert_media("path/to/video2.mp4", "Video 2", 150.0, "video").unwrap();
        
        let tag_id = db.get_or_create_tag("action").unwrap();
        db.add_tag_to_media(1, tag_id).unwrap();
        
        let media = db.get_media_by_tag("action").unwrap();
        
        assert_eq!(media.len(), 1);
        assert_eq!(media[0], 1);
    }

    #[test]
    fn test_create_playlist() {
        let mut db = create_test_db();
        let playlist_id = db.create_playlist("My Playlist").unwrap();
        
        assert!(playlist_id > 0);
    }

    #[test]
    fn test_add_media_to_playlist() {
        let mut db = create_test_db();
        db.insert_media("path/to/video.mp4", "Test Video", 120.5, "video").unwrap();
        let playlist_id = db.create_playlist("My Playlist").unwrap();
        
        assert!(db.add_media_to_playlist(1, playlist_id).is_ok());
    }

    #[test]
    fn test_get_media_from_playlist() {
        let mut db = create_test_db();
        db.insert_media("path/to/video1.mp4", "Video 1", 120.0, "video").unwrap();
        db.insert_media("path/to/video2.mp4", "Video 2", 150.0, "video").unwrap();
        
        let playlist_id = db.create_playlist("My Playlist").unwrap();
        db.add_media_to_playlist(1, playlist_id).unwrap();
        db.add_media_to_playlist(2, playlist_id).unwrap();
        
        let media = db.get_media_from_playlist(playlist_id).unwrap();
        
        assert_eq!(media.len(), 2);
    }

    #[test]
    fn test_get_playlist_id() {
        let mut db = create_test_db();
        db.create_playlist("Test Playlist").unwrap();
        let id = db.get_playlist_id("Test Playlist").unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_clear_media_table() {
        let mut db = create_test_db();
        db.insert_media("path/to/video.mp4", "Test Video", 120.5, "video").unwrap();
        
        assert!(db.clear_media_table().is_ok());
        let media = db.get_all_media().unwrap();
        
        assert_eq!(media.len(), 0);
    }
}


