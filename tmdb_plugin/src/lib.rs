use reqwest;
use plugin_api::Plugin;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;



// Structs to deserialize the JSON response from TMDb
#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    id: u32,
    title: String,
    #[serde(rename = "release_date")]
    release_date: String,
    overview: String,
    #[serde(rename = "vote_average")]
    vote_average: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<Movie>,
    page: u32,
    #[serde(rename = "total_results")]
    total_results: u32,
}




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


struct TMDBMetadata;

impl Plugin for TMDBMetadata {
    fn name(&self) -> String {
        "TMDB".to_string()
    }

    fn version(&self) -> String {
        "1.0.0".to_string()
    }

    fn plugin_type(&self) -> String {
        "film_metadata".to_string()
    }

    fn metadata(&self, film_name: &str) -> String {
        match search_film(film_name) {
            Ok(info) => info,
            Err(e) => format!("Error searching for '{}': {}", film_name, e),
        }
    }
}


fn search_film(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let movie_query = name;
    // Vérifie bien que cette clé est valide !
    let api_key = "94de72cdcd31f017c82bc13fd54979b0"; 
    
    println!("🔍 [TMDB DLL] Recherche lancée pour : '{}'", movie_query);

    let url = format!(
        "https://api.themoviedb.org/3/search/movie?api_key={}&query={}&language=fr-FR", // J'ai ajouté language=fr-FR pour toi ;)
        api_key,
        urlencoding::encode(&movie_query)
    );

    println!("🌍 [TMDB DLL] URL appelée : {}", url); // Attention, ça affiche ta clé API dans la console

    let response = reqwest::blocking::get(&url)?;
    
    // 👇 DEBUG DU STATUS HTTP
    let status = response.status();
    println!("📡 [TMDB DLL] Statut HTTP : {}", status);

    if !status.is_success() {
        println!("❌ [TMDB DLL] Erreur API !");
        return Ok(format!("Erreur API TMDB: {}", status));
    }
    
    // 👇 ON LIT LE JSON BRUT AVANT DE LE PARSER (Crucial pour debug)
    let raw_body = response.text()?; 
    println!("📦 [TMDB DLL] Réponse brute : {}", raw_body);

    // On re-parse le JSON depuis la string
    let search_results: SearchResponse = serde_json::from_str(&raw_body)?;

    if search_results.results.is_empty() {
        println!("⚠️ [TMDB DLL] Aucun résultat trouvé dans le JSON.");
        return Ok(format!("Aucun film trouvé pour '{}'", movie_query));
    } else {
        println!("✅ [TMDB DLL] {} films trouvés.", search_results.results.len());
        
        let mut result = String::new();
        for (index, movie) in search_results.results.iter().take(1).enumerate() { 
            result.push_str(&format!("🎬 {} ({})\n", movie.title, movie.release_date.split('-').next().unwrap_or("????")));
            result.push_str(&format!("⭐ Note: {}/10\n", movie.vote_average));
            
            // Tu peux aussi afficher un synopsis un peu plus long si tu n'as qu'un seul film
            let overview = if movie.overview.len() > 500 {
                format!("{}...", &movie.overview[0..500])
            } else {
                movie.overview.clone()
            };
            result.push_str(&format!("📝 {}\n", overview));
        }
        return Ok(result);
    }
}

/*fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print search results for "Inception"
    let film_info = search_film("Inception")?;
    println!("{}", film_info);
    
    Ok(())
}*/


// Export a C-compatible function that can be called via libloading
#[unsafe(no_mangle)]
pub extern "C" fn name() -> *mut c_char {

    let metadata = TMDBMetadata;
    let name = metadata.name();
    
    CString::new(name).unwrap().into_raw()
}

// Export a C-compatible function that can be called via libloading
#[unsafe(no_mangle)]
pub extern "C" fn version() -> *mut c_char {

    let metadata = TMDBMetadata;
    let version = metadata.version();
    
    CString::new(version).unwrap().into_raw()
}

// Export a C-compatible function that can be called via libloading
#[unsafe(no_mangle)]
pub extern "C" fn plugin_type() -> *mut c_char {

    let metadata = TMDBMetadata;
    let plugin_type = metadata.plugin_type();
    
   CString::new(plugin_type).unwrap().into_raw()
}

// Export a C-compatible function that can be called via libloading
#[unsafe(no_mangle)]
pub extern "C" fn metadata(name_ptr: *const c_char) -> *mut c_char {
    // Safety: We assume the caller passes a valid C string
    let film_name = unsafe {
        if name_ptr.is_null() {
            "invalid name"
        } else {
            CStr::from_ptr(name_ptr).to_str().unwrap_or("invalid name")
        }
    };
    
    let metadata = TMDBMetadata;
    let result = metadata.metadata(film_name);
    
    // Convert result to C string
    CString::new(result).unwrap().into_raw()
}

// Helper function to free strings allocated by the plugin
#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}

/*#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print search results for "Inception"
    let film_info = search_film("Inception").await?;
    println!("{}", film_info);
    
    Ok(())
}*/