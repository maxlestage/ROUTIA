//! Le bundle du navigateur : hydrate les îlots et lance les animations de la page.

pub mod export;
pub mod motion;
pub mod scenes;
pub mod store;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    paquets::hydrate(&crate::islands::ISLANDS);
    motion::init();
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

pub fn document() -> web_sys::Document {
    window().document().expect("document")
}

pub fn matches(query: &str) -> bool {
    window()
        .match_media(query)
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

pub fn reduced_motion() -> bool {
    matches("(prefers-reduced-motion: reduce)")
}

/// Tous les éléments qui répondent à `selector`.
pub fn all(selector: &str) -> Vec<web_sys::Element> {
    let Ok(list) = document().query_selector_all(selector) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i))
        .filter_map(|n| n.dyn_into().ok())
        .collect()
}

/// Un écouteur pour toute la vie de la page, `{ passive: true }`.
pub fn on_passive<E: JsCast + 'static>(
    target: &web_sys::EventTarget,
    event: &str,
    f: impl FnMut(E) + 'static,
) {
    let mut f = f;
    let closure =
        Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| f(e.unchecked_into()));
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(true);
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        event,
        closure.as_ref().unchecked_ref(),
        &options,
    );
    closure.forget();
}

/// Appelle `f` dans `ms` millisecondes.
pub fn later(ms: i32, f: impl FnOnce() + 'static) {
    let closure = Closure::once_into_js(f);
    let _ =
        window().set_timeout_with_callback_and_timeout_and_arguments_0(closure.unchecked_ref(), ms);
}

/// La position du défilement, en pixels.
pub fn scroll_y() -> f64 {
    window().scroll_y().unwrap_or(0.0)
}

pub fn viewport() -> (f64, f64) {
    let w = window();
    let get = |v: Result<JsValue, JsValue>| v.ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
    (get(w.inner_width()), get(w.inner_height()))
}
