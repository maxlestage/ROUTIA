//! Les habitudes dans `localStorage` : rien ne quitte l'appareil.

use crate::habits::{Habit, STORAGE_KEY, load};

fn storage() -> Option<web_sys::Storage> {
    super::window().local_storage().ok().flatten()
}

pub fn read() -> Vec<Habit> {
    storage()
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .map(|json| load(&json))
        .unwrap_or_default()
}

pub fn write(json: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(STORAGE_KEY, json);
    }
}
