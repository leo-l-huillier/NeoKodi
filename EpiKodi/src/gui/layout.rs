use super::route::Route; // On a besoin de connaître les routes pour les liens
use super::style::GLOBAL_STYLE;
use dioxus::prelude::*; // On importe le style

fn nav_items() -> Vec<(Route, &'static str)> {
    vec![
        (Route::Home {}, "Accueil"),
        (Route::Iptv {}, "Iptv"),
        (Route::Videos {}, "Videos"),
        (Route::Series {}, "Series"),
        (Route::Music {}, "Musique"),
        (Route::Images {}, "Images"),
        (Route::Plugins {}, "Add-ons"),
        (Route::Settings {}, "Parametres"),
    ]
}

pub fn AppLayout() -> Element {
    rsx! {
        style { "{GLOBAL_STYLE}" }
        div { class: "container",
            // -------- MENU LATERAL --------
            nav { class: "sidebar",
                for (route, label) in nav_items() {
                    Link { to: route, class: "nav-item", "{label}" }
                }
            }

            // -------- ZONE CONTENU DYNAMIQUE --------
            main { class: "content",
                Outlet::<Route> {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::nav_items;
    use crate::gui::route::Route;

    #[test]
    fn nav_items_contains_expected_routes_in_order() {
        let items = nav_items();

        assert_eq!(items.len(), 8);

        let expected_labels = [
            "Accueil",
            "Iptv",
            "Videos",
            "Series",
            "Musique",
            "Images",
            "Add-ons",
            "Parametres",
        ];

        for (index, (route, label)) in items.iter().enumerate() {
            assert_eq!(*label, expected_labels[index]);

            match index {
                0 => assert!(matches!(route, Route::Home {})),
                1 => assert!(matches!(route, Route::Iptv {})),
                2 => assert!(matches!(route, Route::Videos {})),
                3 => assert!(matches!(route, Route::Series {})),
                4 => assert!(matches!(route, Route::Music {})),
                5 => assert!(matches!(route, Route::Images {})),
                6 => assert!(matches!(route, Route::Plugins {})),
                7 => assert!(matches!(route, Route::Settings {})),
                _ => unreachable!(),
            }
        }
    }
}
