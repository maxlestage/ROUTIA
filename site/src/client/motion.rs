//! Le mouvement que le CSS ne fait pas seul : apparitions au défilement, barre de
//! progression, halo qui suit le doigt ou la souris sur les cartes, compteurs et
//! parallaxe. Sans WebAssembly, la page reste complète et immobile.

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
};

use super::{all, document, matches, on_passive, reduced_motion, scroll_y, viewport, window};

pub fn init() {
    let root: HtmlElement = document().document_element().unwrap().unchecked_into();
    reveals();
    let _ = root.class_list().add_1("hydrated");
    scroll_state(&root);
    counters();
    if !reduced_motion() {
        spotlight();
    }
    menu();
}

fn observe(
    selector: &str,
    options: &IntersectionObserverInit,
    f: impl Fn(&IntersectionObserverEntry, &IntersectionObserver) + 'static,
) {
    let callback = Closure::<dyn Fn(js_sys::Array, IntersectionObserver)>::new(
        move |entries: js_sys::Array, io: IntersectionObserver| {
            for entry in entries.iter() {
                if let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() {
                    f(&entry, &io);
                }
            }
        },
    );
    if let Ok(io) =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), options)
    {
        for el in all(selector) {
            io.observe(&el);
        }
    }
    callback.forget();
}

/// `[data-reveal]` reçoit `is-in` quand il entre à l'écran (le CSS l'anime).
fn reveals() {
    let (_, vh) = viewport();
    // Ce qui est déjà à l'écran apparaît tout de suite.
    for el in all("[data-reveal]") {
        if el.get_bounding_client_rect().top() < vh * 0.92 {
            let _ = el.class_list().add_1("is-in");
        }
    }
    let options = IntersectionObserverInit::new();
    options.set_root_margin("0px 0px -8% 0px");
    options.set_threshold(&JsValue::from_f64(0.12));
    observe("[data-reveal]", &options, |entry, io| {
        if entry.is_intersecting() {
            let target = entry.target();
            let _ = target.class_list().add_1("is-in");
            io.unobserve(&target);
        }
    });
}

/// `--scroll` (0 à 1) sur la racine, et `is-scrolled` une fois la page quittée du haut.
fn scroll_state(root: &HtmlElement) {
    let root = root.clone();
    let ticking = Rc::new(Cell::new(false));
    let update = {
        let root = root.clone();
        move || {
            let y = scroll_y();
            let max = (root.scroll_height() as f64 - viewport().1).max(1.0);
            let style = root.style();
            let _ = style.set_property("--scroll", &format!("{:.4}", (y / max).clamp(0.0, 1.0)));
            let _ = style.set_property("--scroll-px", &format!("{y:.0}"));
            let _ = root.class_list().toggle_with_force("is-scrolled", y > 24.0);
        }
    };
    update();
    let update = Rc::new(update);
    on_passive::<web_sys::Event>(&window(), "scroll", move |_| {
        if ticking.replace(true) {
            return;
        }
        let ticking = ticking.clone();
        let update = update.clone();
        let closure = Closure::once_into_js(move || {
            update();
            ticking.set(false);
        });
        let _ = window().request_animation_frame(closure.unchecked_ref());
    });
}

/// `[data-count="1200"]` compte jusqu'à sa valeur quand il apparaît.
fn counters() {
    let options = IntersectionObserverInit::new();
    options.set_threshold(&JsValue::from_f64(0.6));
    observe("[data-count]", &options, |entry, io| {
        if !entry.is_intersecting() {
            return;
        }
        let el = entry.target();
        io.unobserve(&el);
        let target: f64 = el
            .get_attribute("data-count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let suffix = el.get_attribute("data-suffix").unwrap_or_default();
        if reduced_motion() {
            el.set_text_content(Some(&format!("{target}{suffix}")));
            return;
        }
        let start = js_sys::Date::now();
        animate(move |now| {
            let p = ((now - start) / 1400.0).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - p).powi(4);
            el.set_text_content(Some(&format!("{}{suffix}", (target * eased).round())));
            p < 1.0
        });
    });
}

/// Appelle `f` à chaque image tant qu'il rend `true`.
fn animate(f: impl FnMut(f64) -> bool + 'static) {
    type Frame = Rc<std::cell::RefCell<Option<Closure<dyn FnMut()>>>>;
    let f = Rc::new(std::cell::RefCell::new(f));
    let slot: Frame = Rc::new(std::cell::RefCell::new(None));
    let again = slot.clone();
    *slot.borrow_mut() = Some(Closure::new(move || {
        let keep = (f.borrow_mut())(js_sys::Date::now());
        if keep {
            if let Some(c) = again.borrow().as_ref() {
                let _ = window().request_animation_frame(c.as_ref().unchecked_ref());
            }
        } else {
            again.borrow_mut().take();
        }
    }));
    if let Some(c) = slot.borrow().as_ref() {
        let _ = window().request_animation_frame(c.as_ref().unchecked_ref());
    }
}

/// `[data-spotlight]` garde un halo qui suit le pointeur (souris ou doigt) : `--mx`,
/// `--my` en pourcents, lus par le CSS.
fn spotlight() {
    let fine = matches("(hover: hover)");
    for el in all("[data-spotlight]") {
        let Ok(el) = el.dyn_into::<HtmlElement>() else {
            continue;
        };
        let target: Element = el.clone().unchecked_into();
        let moving = el.clone();
        on_passive(&target, "pointermove", move |e: web_sys::PointerEvent| {
            let r = moving.get_bounding_client_rect();
            let x = ((f64::from(e.client_x()) - r.left()) / r.width()).clamp(0.0, 1.0);
            let y = ((f64::from(e.client_y()) - r.top()) / r.height()).clamp(0.0, 1.0);
            let style = moving.style();
            let _ = style.set_property("--mx", &format!("{:.1}%", x * 100.0));
            let _ = style.set_property("--my", &format!("{:.1}%", y * 100.0));
        });
        if !fine {
            let _ = el.class_list().add_1("tilt-touch");
        }
    }
}

/// Le menu des téléphones est un popover natif : un lien choisi le referme.
fn menu() {
    for link in all(".menu a") {
        let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
            if let Some(menu) = document().get_element_by_id("menu")
                && let Ok(hide) = js_sys::Reflect::get(&menu, &JsValue::from_str("hidePopover"))
                && let Some(hide) = hide.dyn_ref::<js_sys::Function>()
            {
                let _ = hide.call0(&menu);
            }
        });
        let _ = link.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}
