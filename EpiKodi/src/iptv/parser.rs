// src/iptv/parser.rs

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Live,
    Movie,
    Series,
}

#[derive(Debug, Clone)]
pub struct TVChannel {
    pub title: String,
    pub url: String,
    pub group: Option<String>,
    pub content_type: ContentType,
}

pub fn parse_m3u(content: &str) -> Vec<TVChannel> {
    let mut channels = Vec::new();
    let mut current_title = String::new();
    let mut current_group = None;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("#EXTINF:") {
            // 1. Extraction du Titre
            if let Some(comma_index) = line.rfind(',') {
                current_title = line[comma_index + 1..].trim().to_string();
            }

            // 2. Extraction du Groupe
            if let Some(start) = line.find("group-title=\"") {
                let rest = &line[start + 13..];
                if let Some(end) = rest.find('"') {
                    let group_name = rest[..end].to_string();
                    current_group = Some(group_name);
                }
            }
        } else if !line.starts_with("#") && !line.is_empty() {
            // 3. Détection
            let content_type = detect_type(&current_group);

            // LOG DE DEBUG : Décommenter si tu veux voir ce qui se passe
            // if content_type != ContentType::Live {
            //     println!("🔎 Trouvé {:?} dans le groupe '{:?}' -> {}", content_type, current_group, current_title);
            // }

            channels.push(TVChannel {
                title: current_title.clone(),
                url: line.to_string(),
                group: current_group.clone(),
                content_type,
            });

            current_title.clear();
            current_group = None;
        }
    }

    channels
}

fn detect_type(group: &Option<String>) -> ContentType {
    match group {
        Some(g) => {
            let g_upper = g.to_uppercase();

            // Mots-clés pour SÉRIES (très large)
            if g_upper.contains("SERIE")
                || g_upper.contains("SÉRIE")
                || g_upper.contains("SEASON")
                || g_upper.contains("SAISON")
                || g_upper.contains("EPISODE")
            {
                return ContentType::Series;
            }

            // Mots-clés pour FILMS (très large)
            if g_upper.contains("MOVIE") || 
               g_upper.contains("FILM") || 
               g_upper.contains("VOD") || 
               g_upper.contains("CINEMA") || 
               g_upper.contains("4K") || // Souvent les films sont tagués 4K
               g_upper.contains("FHD") || 
               g_upper.contains("HEVC")
            {
                // Attention : Parfois les chaînes live ont aussi FHD,
                // mais dans les M3U IPTV, VOD est souvent explicite.
                // Si ton M3U mélange tout, c'est plus dur.

                // Raffinement : Si ça contient "VOD", c'est sûr que c'est un film/série
                if g_upper.contains("VOD") {
                    return ContentType::Movie;
                }

                // Sinon on tente le coup sur "FILM"
                if g_upper.contains("FILM") || g_upper.contains("MOVIE") {
                    return ContentType::Movie;
                }
            }

            ContentType::Live
        }
        None => ContentType::Live,
    }
}
