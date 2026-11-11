//! Les parties interactives. Chaque îlot est rendu par le serveur comme une vue
//! ordinaire puis, compilé en WebAssembly, s'hydrate dans le navigateur à partir du même
//! code.

use paquets::prelude::*;

use crate::habits::{COLORS, Cell, Day, Habit, WEEKDAYS, frequency_label, top_streak};

/// Chaque îlot, sous le nom donné à [`island`].
pub const ISLANDS: [(&str, paquets::active::IslandFn); 3] =
    [("hero", hero), ("tower", tower), ("tracker", tracker)];

// ----- Animation 2D de l'accueil ----------------------------------------------------

/// Les anneaux d'habitudes, dessinés en 2D dans un canvas : trois anneaux qui se
/// remplissent, sept jours en orbite (ceux qui sont « faits » s'allument), des formes
/// qui flottent. Le canvas suit le pointeur et le défilement.
pub fn hero(_props: &str) -> Node {
    canvas()
        .class("art art--hero")
        .attr("aria-hidden", "true")
        .use_action(start_hero)
        .into()
}

#[cfg(target_arch = "wasm32")]
fn start_hero(canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    crate::client::scenes::hero(canvas)
}

#[cfg(not(target_arch = "wasm32"))]
fn start_hero(_canvas: &paquets::active::Dom) -> impl FnOnce() + use<> {
    || {}
}

// ----- La tour des jours (2D) -------------------------------------------------------

/// Chaque jour fait est une brique qui tombe, rebondit et se pose sur la pile. Le
/// bouton en ajoute une ; la pile se remplit seule quand elle apparaît.
pub fn tower(_props: &str) -> Node {
    let count = use_state(0u32);
    let drops = use_state(0u32);
    div()
        .class("tower")
        .child(
            canvas()
                .class("art art--tower")
                .attr("aria-hidden", "true")
                .use_action(move |el| start_tower(el, drops, count)),
        )
        .child(
            div()
                .class("tower__ui")
                .child(
                    p().class("tower__count")
                        .child(strong().text_dyn(move || count.get().to_string()))
                        .child(span().text_dyn(move || {
                            if count.get() > 1 {
                                " jours d'affilée"
                            } else {
                                " jour d'affilée"
                            }
                            .to_string()
                        })),
                )
                .child(
                    button()
                        .class("btn btn--glass")
                        .attr("type", "button")
                        .on_click(move |_| drops.update(|d| *d += 1))
                        .child(span().class("btn__icon").text("＋"))
                        .child(span().text("Ajouter un jour")),
                ),
        )
        .into()
}

#[cfg(target_arch = "wasm32")]
fn start_tower(
    canvas: &paquets::active::Dom,
    drops: State<u32>,
    count: State<u32>,
) -> impl FnOnce() + use<> {
    crate::client::scenes::tower(canvas, drops, count)
}

#[cfg(not(target_arch = "wasm32"))]
fn start_tower(
    _canvas: &paquets::active::Dom,
    _drops: State<u32>,
    _count: State<u32>,
) -> impl FnOnce() + use<> {
    || {}
}

// ----- Le tracker -------------------------------------------------------------------

/// Aujourd'hui, dans le fuseau du visiteur (navigateur) ou du serveur.
pub fn today() -> Day {
    #[cfg(target_arch = "wasm32")]
    {
        let now = js_sys::Date::new_0();
        Day::from_millis(now.get_time(), -(now.get_timezone_offset() as i32))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0.0, |d| d.as_millis() as f64);
        Day::from_millis(ms, 0)
    }
}

/// Enregistre la liste dans le navigateur (`localStorage`).
fn persist(habits: State<Vec<Habit>>) {
    let json = habits.with(|list| crate::habits::save(list));
    #[cfg(target_arch = "wasm32")]
    crate::client::store::write(&json);
    let _ = json;
}

fn notify(toast: State<String>, text: &str) {
    toast.set(text.to_owned());
    #[cfg(target_arch = "wasm32")]
    crate::client::later(2400, move || toast.set(String::new()));
}

fn new_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let n = (js_sys::Math::random() * 1e12) as u64;
        format!("{:x}-{n:x}", js_sys::Date::now() as u64)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        format!("h{}", NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// L'application : créer une habitude, cocher la journée, suivre la série, voir
/// l'historique (jours manqués compris), exporter en texte ou en image. Tout reste dans
/// le navigateur.
pub fn tracker(_props: &str) -> Node {
    let habits = use_state(Vec::<Habit>::new());
    let day = use_state(today());
    let loaded = use_state(false);
    let name = use_state(String::new());
    let frequency = use_state(7u8);
    let color = use_state(0u8);
    let toast = use_state(String::new());
    let confirm = use_state(String::new());

    let add = move || {
        let label = crate::habits::clean_name(&name.get());
        if label.is_empty() {
            notify(toast, "Donnez un nom à l'habitude");
            return;
        }
        let habit = Habit::new(new_id(), &label, frequency.get(), color.get(), day.get());
        habits.update(|list| list.insert(0, habit));
        persist(habits);
        name.set(String::new());
        color.update(|c| *c = (*c + 1) % COLORS.len() as u8);
        notify(toast, "Habitude créée ✨");
    };

    div()
        .class("tracker")
        .use_action(move |_el| {
            #[cfg(target_arch = "wasm32")]
            {
                habits.set(crate::client::store::read());
                day.set(today());
                loaded.set(true);
            }
            || {}
        })
        .child(summary(habits, day))
        .child(
            form()
                .class("composer glass")
                .attr("autocomplete", "off")
                .on_submit(move |e| {
                    e.prevent_default();
                    add();
                })
                .child(
                    label()
                        .class("field")
                        .child(span().class("field__label").text("Nouvelle habitude"))
                        .child(
                            input()
                                .attr("type", "text")
                                .attr("name", "name")
                                .attr("maxlength", "40")
                                .attr("placeholder", "Méditer, lire, courir…")
                                .attr("enterkeyhint", "done")
                                .bind_value(name),
                        ),
                )
                .child(
                    fieldset()
                        .class("freq")
                        .child(legend().class("field__label").text("Fréquence"))
                        .child(div().class("freq__options").children([7u8, 5, 3, 1].map(|n| {
                            button()
                                .class("chip")
                                .attr("type", "button")
                                .attr_dyn("aria-pressed", move || {
                                    (frequency.get() == n).to_string()
                                })
                                .on_click(move |_| frequency.set(n))
                                .text(if n == 7 { "Chaque jour".to_string() } else { format!("{n}×/sem.") })
                        }))),
                )
                .child(
                    fieldset()
                        .class("swatches")
                        .child(legend().class("field__label").text("Couleur"))
                        .child(div().class("swatches__row").children(
                            COLORS.iter().enumerate().map(|(i, c)| {
                                let i = i as u8;
                                button()
                                    .class("swatch")
                                    .attr("type", "button")
                                    .attr("aria-label", format!("Couleur {}", i + 1))
                                    .attr_dyn("aria-pressed", move || (color.get() == i).to_string())
                                    .style(format!("--c:{c}"))
                                    .on_click(move |_| color.set(i))
                            }),
                        )),
                )
                .child(
                    button()
                        .class("btn btn--primary composer__submit")
                        .attr("type", "submit")
                        .child(span().class("btn__icon").text("＋"))
                        .child(span().text("Créer")),
                ),
        )
        .child(
            ul().class("habits")
                .attr("aria-live", "polite")
                .children_dyn(move || {
                    let today = day.get();
                    let list = habits.get();
                    if list.is_empty() {
                        let text = if loaded.get() || cfg!(not(target_arch = "wasm32")) {
                            "Aucune habitude pour l'instant. Créez la première juste au-dessus : elle restera sur cet appareil."
                        } else {
                            "Chargement…"
                        };
                        return vec![li().class("habits__empty glass").child(
                            div().class("empty-orb").attr("aria-hidden", "true"),
                        ).child(p().text(text)).into()];
                    }
                    list.into_iter()
                        .enumerate()
                        .map(|(i, h)| card(h, i, today, habits, confirm, toast).into())
                        .collect()
                }),
        )
        .child(exports(habits, day, toast))
        .child(
            p().class("toast")
                .attr("role", "status")
                .class_if("is-visible", move || !toast.get().is_empty())
                .text_dyn(move || toast.get()),
        )
        .into()
}

/// Le bandeau du haut : la meilleure série, les habitudes du jour et un anneau de
/// progression.
fn summary(habits: State<Vec<Habit>>, day: State<Day>) -> Element {
    let stats = move || {
        let today = day.get();
        habits.with(|list| {
            let total = list.len();
            let done = list.iter().filter(|h| h.is_done(today)).count();
            let best = top_streak(list, today).map(|(h, s)| (h.name.clone(), s, h.unit()));
            (total, done, best)
        })
    };
    div()
        .class("streak-banner glass")
        .child(
            div()
                .class("ring")
                .attr("aria-hidden", "true")
                .style_dyn("--p", move || {
                    let (total, done, _) = stats();
                    let p = (done * 100).checked_div(total).unwrap_or(0);
                    format!("{p}")
                })
                .child(span().class("ring__value").text_dyn(move || {
                    let (total, done, _) = stats();
                    format!("{done}/{total}")
                })),
        )
        .child(
            div()
                .class("streak-banner__text")
                .child(p().class("eyebrow").text("Aujourd'hui"))
                .child(h2().class("streak-banner__title").text_dyn(move || {
                    let (total, done, _) = stats();
                    match (total, done) {
                        (0, _) => "Votre première habitude vous attend".to_string(),
                        (t, d) if d == t => "Tout est fait. Bravo ! 🎉".to_string(),
                        (t, d) => format!("Encore {} à faire", t - d),
                    }
                }))
                .child(
                    p().class("streak-banner__best")
                        .text_dyn(move || match stats().2 {
                            Some((name, s, unit)) if s > 0 => {
                                format!("🔥 Meilleure série : {s} {} · {name}", unit.label(s))
                            }
                            _ => "Cochez un jour pour lancer une série.".to_string(),
                        }),
                ),
        )
}

fn card(
    h: Habit,
    index: usize,
    today: Day,
    habits: State<Vec<Habit>>,
    confirm: State<String>,
    toast: State<String>,
) -> Element {
    let streak = h.streak(today);
    let best = h.best_streak(today);
    let done = h.is_done(today);
    let id = h.id.clone();
    let toggle_id = id.clone();
    let delete_id = id.clone();
    // Cinq semaines, du lundi au dimanche ; la dernière ligne est la semaine en cours.
    let first = today.week_start().shift(-28);
    let cells = (0..35).map(|i| {
        let d = first.shift(i);
        let cell = h.cell(d, today);
        let cell_id = id.clone();
        let (y, m, dd) = d.ymd();
        let state = match cell {
            Cell::Done => "fait",
            Cell::Missed => "manqué",
            Cell::Pending => "à faire",
            Cell::Rest => "repos",
            Cell::Empty => "",
        };
        let label = format!(
            "{dd} {} {y} {state}",
            crate::habits::MONTHS[(m - 1) as usize]
        );
        button()
            .class("day")
            .class(cell.class())
            .class_if("is-today", move || d == today)
            .attr("type", "button")
            .attr("aria-label", label.trim().to_owned())
            .bool_attr("disabled", move || d > today)
            .style(format!("--i:{i}"))
            .on_click(move |_| {
                let target = cell_id.clone();
                habits.update(|list| {
                    if let Some(h) = list.iter_mut().find(|h| h.id == target) {
                        h.toggle(d);
                    }
                });
                persist(habits);
            })
    });
    let asking = {
        let id = id.clone();
        move || confirm.get() == id
    };
    let asking2 = asking.clone();
    li().class("habit glass")
        .class_if("is-done", move || done)
        .style(format!("--c:{};--n:{index}", h.color()))
        .child(
            div()
                .class("habit__head")
                .child(
                    div()
                        .class("habit__title")
                        .child(h3().text(h.name.clone()))
                        .child(p().class("habit__meta").text(format!(
                            "{} · record {best} · {} % sur 30 j",
                            h.frequency_label(),
                            h.rate(today, 30)
                        ))),
                )
                .child(
                    div()
                        .class("flame")
                        .class_if("is-lit", move || streak > 0)
                        .attr(
                            "aria-label",
                            format!("Série : {streak} {}", h.unit().label(streak)),
                        )
                        .child(
                            span()
                                .class("flame__icon")
                                .attr("aria-hidden", "true")
                                .text("🔥"),
                        )
                        .child(strong().text(streak.to_string()))
                        .child(small().text(h.unit().label(streak))),
                ),
        )
        .child(
            button()
                .class("check")
                .class_if("is-done", move || done)
                .attr("type", "button")
                .attr("aria-pressed", done.to_string())
                .on_click(move |_| {
                    let target = toggle_id.clone();
                    let mut now_done = false;
                    habits.update(|list| {
                        if let Some(h) = list.iter_mut().find(|h| h.id == target) {
                            h.toggle(today);
                            now_done = h.is_done(today);
                        }
                    });
                    persist(habits);
                    if now_done {
                        notify(toast, "Journée cochée ✅");
                    }
                })
                .child(span().class("check__box").attr("aria-hidden", "true"))
                .child(span().text(if done {
                    "Fait aujourd'hui"
                } else {
                    "Marquer comme fait"
                })),
        )
        .child(
            div()
                .class("history")
                .child(
                    div()
                        .class("history__days")
                        .attr("aria-hidden", "true")
                        .children(WEEKDAYS.iter().map(|d| span().text(*d))),
                )
                .child(div().class("history__grid").children(cells))
                .child(p().class("history__legend").text(format!(
                    "{} jour(s) manqué(s) sur 30 jours · touchez un jour pour le corriger",
                    h.missed(today, 30)
                ))),
        )
        .child(
            div().class("habit__foot").child(
                button()
                    .class("link-btn")
                    .class_if("is-danger", asking.clone())
                    .attr("type", "button")
                    .on_click(move |_| {
                        if confirm.get() == delete_id {
                            let target = delete_id.clone();
                            habits.update(|list| list.retain(|h| h.id != target));
                            persist(habits);
                            confirm.set(String::new());
                            notify(toast, "Habitude supprimée");
                        } else {
                            confirm.set(delete_id.clone());
                        }
                    })
                    .text_dyn(move || {
                        if asking2() {
                            "Confirmer la suppression"
                        } else {
                            "Supprimer"
                        }
                        .to_string()
                    }),
            ),
        )
}

/// Le bilan à copier, à télécharger en texte ou en image.
fn exports(habits: State<Vec<Habit>>, day: State<Day>, toast: State<String>) -> Element {
    let text = move || habits.with(|list| crate::habits::export_text(list, day.get()));
    div()
        .class("exports")
        .child(p().class("eyebrow").text("Exporter le bilan"))
        .child(
            div()
                .class("exports__row")
                .child(
                    button()
                        .class("btn btn--glass")
                        .attr("type", "button")
                        .on_click(move |_| {
                            let _t = text();
                            #[cfg(target_arch = "wasm32")]
                            crate::client::export::copy(&_t);
                            notify(toast, "Bilan copié dans le presse-papiers");
                        })
                        .text("Copier le texte"),
                )
                .child(
                    button()
                        .class("btn btn--glass")
                        .attr("type", "button")
                        .on_click(move |_| {
                            let _t = text();
                            #[cfg(target_arch = "wasm32")]
                            crate::client::export::download_text(&_t, "routia-bilan.txt");
                            notify(toast, "Fichier texte téléchargé");
                        })
                        .text("Télécharger .txt"),
                )
                .child(
                    button()
                        .class("btn btn--glass")
                        .attr("type", "button")
                        .on_click(move |_| {
                            #[cfg(target_arch = "wasm32")]
                            habits.with(|list| {
                                crate::client::export::download_image(list, day.get())
                            });
                            notify(toast, "Image téléchargée");
                        })
                        .text("Image .png"),
                ),
        )
}

/// Le texte d'une fréquence, pour les pages.
pub fn frequency(n: u8) -> String {
    frequency_label(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn islands_render_on_the_server() {
        for (name, f) in ISLANDS {
            let html = paquets::active::island(name, "", f).render();
            assert!(
                html.starts_with(&format!(r#"<active-island data-island="{name}""#)),
                "{html}"
            );
        }
        let html = paquets::active::island("tracker", "", tracker).render();
        assert!(html.contains("Nouvelle habitude"));
        assert!(html.contains("Aucune habitude"));
        assert!(html.contains("Image .png"));
    }
}
