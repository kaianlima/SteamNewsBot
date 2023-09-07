pub use anyhow::{Error, Result};

use crate::structs::SteamApp;

// Return if String match any of Strings in Vec
pub fn match_exactly(app_searched: SteamApp, text: String) -> Option<SteamApp> {
    if app_searched.name.to_lowercase() == text.to_lowercase() { Some(app_searched) } else { None }
}

// Return Vec of Strings matched only in its initial part
// and add a matching score
pub fn search_initial_text(app_searched: &mut SteamApp, text: String) {
    if let Some(index) = app_searched.name.to_lowercase().find(&text.to_lowercase()) {
        if index == 0 { app_searched.search_score += 10 };
    };
}

//
pub fn search_partial_text(apps_searched: &mut SteamApp, text: String) {
    if let Some(index) = apps_searched.name.to_lowercase().find(&text.to_lowercase()) {
        if index == 0 { apps_searched.search_score += 10 };
    };
}

pub fn search_in(apps_searched: &mut Vec<SteamApp>, text: String) -> (&mut Vec<SteamApp>, Option<SteamApp>) {
    let search: (&mut Vec<SteamApp>, Option<SteamApp>) = 'search_block: {
        for app in apps_searched.iter_mut() {
            let app_matched = match_exactly(app.clone(), text.clone());
            
            if app_matched.is_some() {
                break 'search_block (apps_searched, app_matched);
            }

            search_initial_text(app, text.clone());
            search_partial_text(app, text.clone());
            
        }
        (apps_searched, None)
    };
    search
}