use dioxus::prelude::*;
use super::route::Route; // Nécessaire pour le Link du PageNotFound

#[component]
pub fn Home() -> Element {
    rsx! {
        h1 { "Bienvenue sur NeoKodi" }
        p { "Sélectionne une catégorie pour commencer." }
    }
}

#[component]
pub fn TV() -> Element {
    rsx! { h1 { "Ma TV" }, p { "Liste des chaînes TV ici..." } }
}

#[component]
pub fn Films() -> Element {
    rsx! {
        h1 { "Mes Films" }
        div { class: "media-grid",
            div { class: "media-card", "Matrix" }
            div { class: "media-card", "Inception" }
            div { class: "media-card", "Interstellar" }
        }
    }
}

#[component]
pub fn Series() -> Element {
    rsx! { 
        h1 { "Mes Séries" }
        div { class: "media-grid",
            div { class: "media-card", "Breaking Bad" }
            div { class: "media-card", "Game of Thrones" }
            div { class: "media-card", "Stranger Things" }
        }
    }
}

// Dans src/front/pages.rs

#[component]
pub fn Music() -> Element {
    rsx! {
        h1 { "Ma Musique" }
        
        // Conteneur Liste Audio
        div { class: "audio-list",
            
            // Exemple de Piste 1
            div { class: "audio-row",
                div { class: "audio-icon", "🎵" } // Icône ou fausse pochette
                div { class: "audio-info",
                    span { class: "audio-title", "Billie Jean" }
                    span { class: "audio-artist", "Michael Jackson" }
                }
                div { class: "audio-duration", "4:54" }
            }

            // Exemple de Piste 2
            div { class: "audio-row",
                div { class: "audio-icon", "🎸" }
                div { class: "audio-info",
                    span { class: "audio-title", "Bohemian Rhapsody" }
                    span { class: "audio-artist", "Queen" }
                }
                div { class: "audio-duration", "5:55" }
            }

            // Exemple de Piste 3
            div { class: "audio-row",
                div { class: "audio-icon", "🎹" }
                div { class: "audio-info",
                    span { class: "audio-title", "Imagine" }
                    span { class: "audio-artist", "John Lennon" }
                }
                div { class: "audio-duration", "3:01" }
            }
        }
    }
}

#[component]
pub fn Images() -> Element {
    rsx! { h1 { "Mes Images" }, p { "Galerie d'images..." } }
}

#[component]
pub fn Addons() -> Element {
    rsx! { h1 { "Add-ons" }, p { "Gérer les add-ons ici..." } }
}

#[component]
pub fn Settings() -> Element {
    rsx! { h1 { "Paramètres" }, p { "Configurer l'application..." } }
}

#[component]
pub fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div { 
            style: "padding: 50px; color: white; text-align: center;",
            h1 { "Oups ! Page introuvable" }
            p { "La route que tu cherches n'existe pas : {route:?}" }
            Link { to: Route::Home {}, "Retour à l'accueil" }
        }
    }
}