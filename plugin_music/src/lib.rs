use plugin_api::Plugin;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde::Deserialize;
use std::panic;
use std::time::Duration;

// --- STRUCTURES JSON (Pour lire la réponse de MusicBrainz) ---
#[derive(Deserialize, Debug)]
struct ArtistSearchResult {
    artists: Vec<Artist>,
}

#[derive(Deserialize, Debug)]
struct Artist {
    name: String,
    #[serde(rename = "type")]
    artist_type: Option<String>,
    country: Option<String>,
    #[serde(rename = "life-span")]
    life_span: Option<LifeSpan>,
}

#[derive(Deserialize, Debug)]
struct LifeSpan {
    begin: Option<String>,
    end: Option<String>,
}

// --- COEUR DU PLUGIN ---
struct MusicBrainzMetadata;

impl Plugin for MusicBrainzMetadata {
    fn name(&self) -> String { "MusicBrainz".to_string() }
    fn version(&self) -> String { "1.0.0".to_string() }
    fn plugin_type(&self) -> String { "metadata".to_string() }

    fn metadata(&self, artist_name: &str) -> String {
        // Mouchard interne pour voir si ça marche
        println!("DLL [INTERNAL]: Appel metadata() reçu pour '{}'", artist_name);
        
        match search_artist(artist_name) {
            Ok(info) => {
                println!("DLL [INTERNAL]: Succès ! Données récupérées.");
                info
            },
            Err(e) => {
                println!("DLL [INTERNAL]: Erreur -> {}", e);
                format!("Erreur plugin : {}", e)
            },
        }
    }
}

// --- FONCTION DE RECHERCHE HTTP (Version stable avec UREQ) ---
fn search_artist(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("DLL [INTERNAL]: Préparation requête...");

    let url = format!(
        "https://musicbrainz.org/ws/2/artist/?query=artist:{}&fmt=json&limit=1",
        urlencoding::encode(name)
    );

    // Timeout de 5s pour éviter que l'application ne gèle si internet plante
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    let response = agent.get(&url)
        .set("User-Agent", "NeoKodiPlugin/1.0 (educational-purpose)")
        .call();

    match response {
        Ok(resp) => {
            println!("DLL [INTERNAL]: Réponse reçue ! Traitement JSON...");
            let result: ArtistSearchResult = resp.into_json()?;

            if let Some(artist) = result.artists.first() {
                // Construction du texte affiché dans l'appli
                let mut info = format!("🎵 Artiste : {}", artist.name);
                if let Some(t) = &artist.artist_type { info.push_str(&format!("\nType : {}", t)); }
                if let Some(c) = &artist.country { info.push_str(&format!("\nPays : {}", c)); }
                
                if let Some(ls) = &artist.life_span {
                    if let Some(b) = &ls.begin { info.push_str(&format!("\nDébut : {}", b)); }
                    if let Some(e) = &ls.end { info.push_str(&format!(" - Fin : {}", e)); }
                }
                
                Ok(info)
            } else {
                Ok("Aucun artiste trouvé sur MusicBrainz.".to_string())
            }
        },
        Err(e) => {
            println!("DLL [INTERNAL]: Echec HTTP -> {:?}", e);
            // On renvoie l'erreur sous forme de texte pour qu'elle s'affiche dans l'appli
            Ok(format!("Erreur de connexion : {:?}", e)) 
        }
    }
}

// --- FONCTIONS EXPORTÉES (Le pont vers l'application) ---
// C'est ce que l'EXE charge. J'ai ajouté name/version pour être complet.

fn to_c_string(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn name() -> *mut c_char { to_c_string("MusicBrainz".to_string()) }

#[unsafe(no_mangle)]
pub extern "C" fn version() -> *mut c_char { to_c_string("1.0.0".to_string()) }

#[unsafe(no_mangle)]
pub extern "C" fn plugin_type() -> *mut c_char {
    // Sécurité anti-crash
    let result = panic::catch_unwind(|| {
        let p = MusicBrainzMetadata;
        p.plugin_type()
    });
    match result {
        Ok(s) => to_c_string(s),
        Err(_) => to_c_string("error".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn metadata(name_ptr: *const c_char) -> *mut c_char {
    // 🛡️ FILET DE SÉCURITÉ CRITIQUE
    // Empêche le plugin de faire crasher toute l'application en cas de panique
    let result = panic::catch_unwind(|| {
        let artist_name = unsafe {
            if name_ptr.is_null() { "Inconnu" } 
            else { CStr::from_ptr(name_ptr).to_str().unwrap_or("Inconnu") }
        };

        let plugin = MusicBrainzMetadata;
        plugin.metadata(artist_name)
    });

    match result {
        Ok(s) => to_c_string(s),
        Err(_) => to_c_string("💥 ERREUR INTERNE DU PLUGIN".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}