use plugin_api::Greeter;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde::Deserialize;

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

// Our implementation that searches MusicBrainz
struct MusicBrainzGreeter;

impl Greeter for MusicBrainzGreeter {
    fn greet(&self, artist_name: &str) -> String {
        // Search for the artist on MusicBrainz
        match search_artist(artist_name) {
            Ok(info) => info,
            Err(e) => format!("Error searching for '{}': {}", artist_name, e),
        }
    }
}

fn search_artist(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // MusicBrainz API endpoint
    let url = format!(
        "https://musicbrainz.org/ws/2/artist/?query=artist:{}&fmt=json&limit=1",
        urlencoding::encode(name)
    );

    // Create a client with proper User-Agent (MusicBrainz requires this)
    let client = reqwest::blocking::Client::builder()
        .user_agent("RustLibloadingExample/0.1.0 (educational-purpose)")
        .build()?;

    // Make the API request
    let response = client.get(&url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()).into());
    }

    let result: ArtistSearchResult = response.json()?;

    if let Some(artist) = result.artists.first() {
        let mut info = format!("🎵 Artist: {}", artist.name);
        
        if let Some(artist_type) = &artist.artist_type {
            info.push_str(&format!("\n   Type: {}", artist_type));
        }
        
        if let Some(country) = &artist.country {
            info.push_str(&format!("\n   Country: {}", country));
        }
        
        if let Some(life_span) = &artist.life_span {
            if let Some(begin) = &life_span.begin {
                info.push_str(&format!("\n   Active from: {}", begin));
            }
            if let Some(end) = &life_span.end {
                info.push_str(&format!(" to {}", end));
            } else if life_span.begin.is_some() {
                info.push_str(" to present");
            }
        }
        
        Ok(info)
    } else {
        Ok(format!("No artist found for '{}'", name))
    }
}

// We need this for URL encoding
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

// Export a C-compatible function that can be called via libloading
#[unsafe(no_mangle)]
pub extern "C" fn greet(name_ptr: *const c_char) -> *mut c_char {
    // Safety: We assume the caller passes a valid C string
    let artist_name = unsafe {
        if name_ptr.is_null() {
            "The Beatles"
        } else {
            CStr::from_ptr(name_ptr).to_str().unwrap_or("The Beatles")
        }
    };
    
    let greeter = MusicBrainzGreeter;
    let result = greeter.greet(artist_name);
    
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