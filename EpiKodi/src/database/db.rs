use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct MediaRow {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub duration: Option<f32>,
    pub media_type: Option<String>,
}

pub struct DB {
    pub conn: Connection,
    pub media_rows: Vec<MediaRow>,
}


impl DB {
    pub fn new(conn: Connection) -> Self {
        DB {
            conn,
            media_rows: Vec::new(),
        }
    }

    pub fn open_database(&mut self) -> Result<()> {
        self.conn = Connection::open("library.db")?;
        Ok(())
    }

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
        Ok(())
    }

    pub fn insert_media(&mut self, path: &str,title: &str,duration: f32,media_type: &str) -> Result<()> {
        self.conn.execute(
            "
            INSERT OR IGNORE INTO media (path, title, duration, media_type)
            VALUES (?1, ?2, ?3, ?4)
            ",
            (path, title, duration, media_type),
        )?;
        Ok(())
    }

    pub fn get_all_media(&mut self) -> Result<()> {

        self.open_database()?;
        self.init_db()?;

        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, duration, media_type FROM media"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(MediaRow {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                duration: row.get(3)?,
                media_type: row.get(4)?,
            })
        })?;

        for r in rows {
           self.media_rows.push(r?);
        }
        Ok(())
    }

    pub fn print_media_rows(&mut self) {
        println!("{:#?}", self.media_rows);
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






