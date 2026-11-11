//! Les pages du site, écrites avec les vues d'Active (via le prélude de Paquets).

use paquets::prelude::*;

use crate::habits::{Cell, Day, Habit};
use crate::islands;

const FONTS: &str = "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Syne:wght@600;700;800&display=swap";

/// Ce qu'il faut pour rendre une page.
pub struct Ctx {
    /// L'adresse publique, pour les URL canoniques et Open Graph.
    pub origin: String,
    /// La version de `site.css` (`?v=`).
    pub css: &'static str,
}

impl Ctx {
    pub fn new(origin: String, css: &'static str) -> Ctx {
        Ctx { origin, css }
    }
}

// ----- Petits morceaux --------------------------------------------------------------

/// Chaque lettre monte à son tour (CSS) ; les lecteurs d'écran lisent le texte entier.
fn split(text: &str, offset: usize) -> Element {
    let mut root = span()
        .class("split")
        .attr("aria-label", text.to_owned())
        .attr("role", "text");
    let mut i = offset;
    let words: Vec<&str> = text.split(' ').collect();
    for (w, word) in words.iter().enumerate() {
        let letters: Vec<Node> = word
            .chars()
            .map(|c| {
                i += 1;
                span()
                    .class("ch")
                    .attr("aria-hidden", "true")
                    .style(format!("--i:{i}"))
                    .text(c.to_string())
                    .into()
            })
            .collect();
        root = root.child(
            span()
                .class("word")
                .attr("aria-hidden", "true")
                .children(letters),
        );
        if w + 1 < words.len() {
            root = root.text(" ");
            i += 1;
        }
    }
    root
}

/// Un titre de section : un numéro, une étiquette, un titre et une phrase.
fn heading(num: &str, label: &str, title: &str, lead: &str) -> Element {
    header()
        .class("heading")
        .attr("data-reveal", "")
        .child(
            p().class("eyebrow")
                .child(span().class("eyebrow__num").text(num.to_owned()))
                .text(label.to_owned()),
        )
        .child(h2().class("heading__title").text(title.to_owned()))
        .child(p().class("heading__lead").text(lead.to_owned()))
}

/// Une forme 2D animée (elle tourne et change de contour).
fn shape(class: &str) -> Element {
    span()
        .class("shape")
        .class(class.to_owned())
        .attr("aria-hidden", "true")
}

fn logo() -> Element {
    span()
        .class("logo")
        .attr("aria-hidden", "true")
        .child(span().class("logo__ring logo__ring--a"))
        .child(span().class("logo__ring logo__ring--b"))
        .child(span().class("logo__dot"))
}

// ----- Mise en page -----------------------------------------------------------------

fn layout(ctx: &Ctx, title: &str, description: &str, path: &str, content: Element) -> Page {
    let mut seo = Seo::new()
        .site_name("Routia")
        .base_url(ctx.origin.clone())
        .canonical(path.to_owned())
        .image("/assets/og.svg")
        .og_type("website")
        .locale("fr_FR")
        .meta_name("author", "Maxime Nathan Lestage")
        .meta_name("apple-mobile-web-app-title", "Routia")
        .json_ld(
            r#"{"@context":"https://schema.org","@type":"SoftwareApplication","name":"Routia","applicationCategory":"HealthApplication","operatingSystem":"iOS, Web","offers":{"@type":"Offer","price":"0","priceCurrency":"EUR"}}"#,
        );
    if path == "/404" {
        seo = seo.noindex(true);
    }
    let head = r##"<meta name="color-scheme" content="dark"/><meta name="theme-color" content="#0b0820"/><link rel="icon" href="/assets/logo.svg" type="image/svg+xml"/><link rel="apple-touch-icon" href="/assets/apple-touch-icon.png"/><link rel="preconnect" href="https://fonts.googleapis.com"/><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>"##;
    let root =
        div()
            .class("site")
            .child(a().class("skip").href("#main").text("Aller au contenu"))
            .child(div().class("progress").attr("aria-hidden", "true"))
            .child(div().class("aurora").attr("aria-hidden", "true").children(
                [1, 2, 3].map(|i| span().class(format!("aurora__blob aurora__blob--{i}"))),
            ))
            .child(navbar(path))
            .child(content.id("main"))
            .child(site_footer())
            .child(div().class("grain").attr("aria-hidden", "true"));
    Page::new(title)
        .lang("fr")
        .description(description)
        .seo(seo)
        .plain()
        .view_transitions()
        .stylesheet(FONTS)
        .stylesheet(format!("/site.css?v={}", ctx.css))
        .head(head)
        .body(root)
}

fn navbar(path: &str) -> Element {
    let home = path == "/";
    let link = |anchor: &str| {
        if home {
            format!("#{anchor}")
        } else {
            format!("/#{anchor}")
        }
    };
    let links = [
        ("fonctionnalites", "Fonctionnalités"),
        ("series", "Séries"),
        ("ios", "iPhone"),
        ("faq", "FAQ"),
    ];
    header()
        .class("nav")
        .child(
            div()
                .class("nav__bar glass")
                .child(
                    a().class("nav__brand")
                        .href("/")
                        .attr("aria-label", "Routia, accueil")
                        .child(logo())
                        .child(span().text("Routia")),
                )
                .child(
                    nav()
                        .class("nav__links")
                        .attr("aria-label", "Sections")
                        .children(links.map(|(id, label)| a().href(link(id)).text(label))),
                )
                .child(
                    a().class("btn btn--primary btn--sm nav__cta")
                        .href("/app/")
                        .text("Ouvrir l'app"),
                )
                .child(
                    button()
                        .class("nav__toggle")
                        .attr("type", "button")
                        .attr("aria-label", "Menu")
                        .attr("popovertarget", "menu")
                        .child(span())
                        .child(span()),
                ),
        )
        .child(
            nav()
                .class("menu glass")
                .id("menu")
                .attr("popover", "")
                .attr("aria-label", "Menu")
                .children(links.map(|(id, label)| a().href(link(id)).text(label)))
                .child(
                    a().class("btn btn--primary")
                        .href("/app/")
                        .text("Ouvrir l'app"),
                ),
        )
}

fn site_footer() -> Element {
    footer()
        .class("footer")
        .child(
            div()
                .class("footer__inner")
                .child(
                    div()
                        .class("footer__brand")
                        .child(logo())
                        .child(strong().text("Routia"))
                        .child(p().text("Une habitude à la fois. Écrit en Rust avec Paquets, et en Swift pour l'iPhone.")),
                )
                .child(
                    nav().class("footer__links").attr("aria-label", "Pied de page")
                        .child(a().href("/app/").text("L'app web"))
                        .child(a().href("/#ios").text("L'app iPhone"))
                        .child(a().href("/#faq").text("Questions"))
                        .child(a().href("https://github.com/maxlestage/ROUTIA").attr("rel", "noopener").text("Code source")),
                ),
        )
        .child(p().class("footer__legal").text("© 2026 Routia · Vos données restent sur votre appareil."))
}

// ----- Accueil ----------------------------------------------------------------------

pub fn home(ctx: &Ctx) -> Page {
    let content = main()
        .class("home")
        .child(hero())
        .child(marquee())
        .child(features())
        .child(streaks())
        .child(steps())
        .child(history())
        .child(ios())
        .child(privacy())
        .child(faq())
        .child(cta());
    layout(
        ctx,
        "Routia — le tracker d'habitudes minimaliste",
        "Routia : créez une habitude, cochez la journée, suivez votre série et votre historique. Une PWA ultra-légère et une app iPhone, sans compte ni serveur.",
        "/",
        content,
    )
}

fn hero() -> Element {
    section()
        .class("hero")
        .attr("aria-labelledby", "hero-title")
        .child(div().class("hero__art").child(island("hero", "", islands::hero)))
        .child(
            div()
                .class("hero__inner")
                .child(
                    p().class("pill")
                        .child(span().class("pill__dot"))
                        .text("PWA + iPhone · sans compte · hors ligne"),
                )
                .child(
                    h1().class("hero__title")
                        .id("hero-title")
                        .child(split("Une habitude", 0))
                        .child(br())
                        .child(span().class("hero__grad").child(split("à la fois.", 12))),
                )
                .child(p().class("hero__lead").text(
                    "Routia est un tracker d'habitudes minimaliste : vous créez une habitude, vous cochez la journée, et la série s'allonge. Rien d'autre à apprendre.",
                ))
                .child(
                    div()
                        .class("hero__actions")
                        .child(
                            a().class("btn btn--primary btn--lg")
                                .href("/app/")
                                .child(span().text("Commencer maintenant"))
                                .child(span().class("btn__arrow").attr("aria-hidden", "true").text("→")),
                        )
                        .child(a().class("btn btn--glass btn--lg").href("#ios").text("L'app iPhone")),
                ),
        )
        .child(hero_cards())
        .child(
            a().class("scroll-hint")
                .href("#fonctionnalites")
                .attr("aria-label", "Découvrir")
                .child(span()),
        )
}

/// Des cartes de verre qui flottent autour des anneaux.
fn hero_cards() -> Element {
    div()
        .class("hero__cards")
        .attr("aria-hidden", "true")
        .child(
            div()
                .class("float-card float-card--a glass")
                .child(span().class("float-card__emoji").text("🔥"))
                .child(strong().text("12 jours"))
                .child(small().text("Méditer")),
        )
        .child(
            div()
                .class("float-card float-card--b glass")
                .child(span().class("float-card__check").text("✓"))
                .child(strong().text("Fait aujourd'hui"))
                .child(small().text("Lire 20 pages")),
        )
        .child(
            div()
                .class("float-card float-card--c glass")
                .child(div().class("mini-grid").children((0..14).map(|i| {
                    span()
                        .class(if [3, 9].contains(&i) {
                            "is-missed"
                        } else {
                            "is-done"
                        })
                        .style(format!("--i:{i}"))
                })))
                .child(small().text("2 semaines")),
        )
}

fn marquee() -> Element {
    let items = [
        "🧘 Méditer",
        "📚 Lire",
        "🏃 Courir",
        "💧 Boire de l'eau",
        "✍️ Écrire",
        "🎸 Guitare",
        "🌙 Dormir tôt",
        "🥦 Manger des légumes",
        "🇪🇸 Espagnol",
        "🚶 10 000 pas",
    ];
    let row = |reverse: bool| {
        div()
            .class("marquee__track")
            .class_if("marquee__track--reverse", move || reverse)
            .children(
                items
                    .iter()
                    .chain(items.iter())
                    .map(|t| span().class("marquee__item").text(*t)),
            )
    };
    div()
        .class("marquee")
        .attr("aria-hidden", "true")
        .child(row(false))
        .child(row(true))
}

fn features() -> Element {
    let list = [
        (
            "✚",
            "Créer une habitude",
            "Un nom, une fréquence (chaque jour ou quelques fois par semaine), une couleur. C'est tout.",
            "violet",
        ),
        (
            "✓",
            "Cocher la journée",
            "Un seul geste : « fait ». Vous pouvez corriger un oubli en touchant le jour dans l'historique.",
            "cyan",
        ),
        (
            "🔥",
            "Suivre la série",
            "La série continue en jours ou en semaines. Aujourd'hui pas encore coché ne la casse pas.",
            "pink",
        ),
        (
            "▦",
            "Voir l'historique",
            "Cinq semaines d'un coup d'œil, avec les jours manqués en rouge et les jours de repos en gris.",
            "amber",
        ),
        (
            "⇪",
            "Exporter",
            "Copiez votre bilan en texte, téléchargez-le en .txt ou en image PNG pour le partager.",
            "mint",
        ),
        (
            "⚡",
            "Installable",
            "Une PWA en Rust compilé en WebAssembly : sur l'écran d'accueil, hors ligne, sans serveur ni compte.",
            "violet",
        ),
    ];
    section()
        .class("section features")
        .id("fonctionnalites")
        .child(heading(
            "01",
            "Fonctionnalités",
            "Tout ce qu'il faut. Rien de plus.",
            "Le MVP tient en cinq gestes, pensés d'abord pour le pouce.",
        ))
        .child(
            ul().class("features__grid")
                .children(
                    list.iter()
                        .enumerate()
                        .map(|(i, (icon, title, text, tone))| {
                            li().class(format!("feature glass tone-{tone}"))
                                .attr("data-reveal", "")
                                .attr("data-spotlight", "")
                                .style(format!("--d:{}ms", i * 70))
                                .child(div().class("feature__shine").attr("aria-hidden", "true"))
                                .child(
                                    div()
                                        .class("feature__icon")
                                        .attr("aria-hidden", "true")
                                        .child(shape("shape--sm"))
                                        .child(span().class("feature__glyph").text(*icon)),
                                )
                                .child(h3().text(*title))
                                .child(p().text(*text))
                        }),
                ),
        )
}

fn streaks() -> Element {
    section()
        .class("section streaks")
        .id("series")
        .child(
            div()
                .class("streaks__grid")
                .child(
                    div()
                        .class("streaks__text")
                        .child(heading(
                            "02",
                            "Séries",
                            "Chaque jour, une brique de plus.",
                            "Chaque jour fait tombe, rebondit et se pose sur la pile : une animation 2D dessinée en Rust dans un canvas. Ajoutez-en un.",
                        ))
                        .child(
                            ul().class("ticks")
                                .attr("data-reveal", "")
                                .child(li().text("Quotidienne : la série compte les jours d'affilée."))
                                .child(li().text("Hebdomadaire : elle compte les semaines où l'objectif est atteint."))
                                .child(li().text("Le record reste affiché, même après une pause.")),
                        ),
                )
                .child(
                    div()
                        .class("streaks__stage glass")
                        .attr("data-reveal", "")
                        .child(island("tower", "", islands::tower)),
                ),
        )
}

fn steps() -> Element {
    let list = [
        (
            "1",
            "Nommez-la",
            "« Méditer 10 min ». Choisissez chaque jour, ou 3 fois par semaine.",
        ),
        (
            "2",
            "Cochez-la",
            "Un toucher sur « Marquer comme fait ». La flamme s'allume.",
        ),
        (
            "3",
            "Tenez-la",
            "La série grandit, l'historique se remplit. Les jours manqués restent visibles, sans culpabiliser.",
        ),
    ];
    section()
        .class("section steps")
        .child(heading(
            "03",
            "Comment ça marche",
            "Trois étapes, trente secondes.",
            "Survolez ou touchez une carte : chaque étape a son détail.",
        ))
        .child(
            ol().class("steps__list")
                .children(list.iter().enumerate().map(|(i, (n, title, text))| {
                    li().class("step glass")
                        .attr("data-reveal", "")
                        .attr("tabindex", "0")
                        .style(format!("--d:{}ms", i * 120))
                        .child(
                            div()
                                .class("step__front")
                                .child(span().class("step__num").text(*n))
                                .child(h3().text(*title))
                                .child(small().text("Toucher pour voir le détail")),
                        )
                        .child(div().class("step__back").child(p().text(*text)))
                })),
        )
}

/// Une habitude d'exemple, pour la démonstration de l'historique.
fn sample_habit(today: Day) -> Habit {
    let mut h = Habit::new("demo", "Méditer", 7, 0, today.shift(-34));
    for i in 0..35 {
        if ![4, 11, 12, 23, 30].contains(&i) {
            h.set_done(today.shift(-34 + i), true);
        }
    }
    h
}

fn history() -> Element {
    let today = islands::today();
    let h = sample_habit(today);
    let first = today.week_start().shift(-28);
    section()
        .class("section history-demo")
        .child(heading(
            "04",
            "Historique",
            "Les trous se voient. C'est le but.",
            "Cinq semaines en un coup d'œil : les jours faits brillent, les jours manqués restent marqués.",
        ))
        .child(
            div()
                .class("heatmap glass")
                .attr("data-reveal", "")
                .attr("data-spotlight", "")
                .child(
                    div()
                        .class("heatmap__head")
                        .child(strong().text("🧘 Méditer"))
                        .child(span().class("badge").text(format!("🔥 {} jours", h.streak(today)))),
                )
                .child(div().class("heatmap__days").attr("aria-hidden", "true").children(
                    crate::habits::WEEKDAYS.map(|d| span().text(d)),
                ))
                .child(
                    div()
                        .class("heatmap__grid")
                        .attr("role", "img")
                        .attr("aria-label", format!("{} jours manqués sur 30", h.missed(today, 30)))
                        .children((0..35).map(|i| {
                            let cell = h.cell(first.shift(i), today);
                            span().class("heat")
                                .class(cell.class())
                                .style(format!("--i:{i};--row:{};--col:{}", i / 7, i % 7))
                                .attr("aria-hidden", "true")
                                .text(if cell == Cell::Missed { "×" } else { "" })
                        })),
                )
                .child(
                    p().class("heatmap__legend")
                        .child(span().class("key is-done"))
                        .text("fait")
                        .child(span().class("key is-missed"))
                        .text("manqué")
                        .child(span().class("key is-pending"))
                        .text("aujourd'hui"),
                ),
        )
}

fn ios() -> Element {
    let points = [
        (
            "SwiftUI",
            "Une interface native, fluide à 120 Hz, avec Liquid Glass, pour iOS 27.",
        ),
        (
            "SwiftData",
            "Les habitudes sont enregistrées sur l'iPhone, sans compte.",
        ),
        (
            "Swift 6.4",
            "Concurrence stricte : le code est vérifié dès la compilation.",
        ),
        (
            "Partage",
            "Le bilan s'exporte en texte ou en image, comme sur le web.",
        ),
    ];
    section()
        .class("section ios")
        .id("ios")
        .child(
            div()
                .class("ios__grid")
                .child(
                    div()
                        .class("ios__text")
                        .child(heading(
                            "05",
                            "iPhone",
                            "Et dans votre poche, en Swift.",
                            "L'app iOS est écrite en Swift avec SwiftUI. Même logique de séries que la version web, même simplicité.",
                        ))
                        .child(ul().class("ios__points").children(points.iter().enumerate().map(
                            |(i, (title, text))| {
                                li().attr("data-reveal", "")
                                    .style(format!("--d:{}ms", i * 80))
                                    .child(strong().text(*title))
                                    .child(span().text(*text))
                            },
                        ))),
                )
                .child(phone()),
        )
}

/// Le jour de la semaine et la date, en français : « Mardi 10 novembre ».
fn long_date(day: Day) -> String {
    const NAMES: [&str; 7] = [
        "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche",
    ];
    const MONTHS: [&str; 12] = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let (_, m, d) = day.ymd();
    format!(
        "{} {d} {}",
        NAMES[day.weekday() as usize],
        MONTHS[(m - 1) as usize]
    )
}

/// Un iPhone à plat qui flotte et montre l'app ; un doigt touche l'écran de temps en temps.
fn phone() -> Element {
    let rows = [
        ("Méditer", "🔥 12", "#7c5cff", true),
        ("Lire 20 pages", "🔥 5", "#22d3ee", true),
        ("Courir", "🔥 3 sem.", "#34d399", false),
        ("Boire de l'eau", "🔥 21", "#fbbf24", true),
    ];
    div()
        .class("phone-stage")
        .attr("aria-hidden", "true")
        .attr("data-reveal", "")
        .child(div().class("phone-stage__glow"))
        .child(
            div()
                .class("phone")
                .child(div().class("phone__island"))
                .child(
                    div()
                        .class("phone__screen")
                        .child(p().class("phone__date").text(long_date(islands::today())))
                        .child(p().class("phone__title").text("Aujourd'hui"))
                        .child(div().class("phone__ring").child(span().text("3/4")))
                        .children(rows.iter().enumerate().map(
                            |(i, (name, streak, color, done))| {
                                div()
                                    .class(if *done {
                                        "phone__row is-done"
                                    } else {
                                        "phone__row"
                                    })
                                    .style(format!("--c:{color};--i:{i}"))
                                    .child(span().class("phone__check"))
                                    .child(span().class("phone__name").text(*name))
                                    .child(span().class("phone__streak").text(*streak))
                            },
                        )),
                )
                .child(span().class("phone__tap")),
        )
}

fn privacy() -> Element {
    let numbers = [
        ("0", "", "compte à créer"),
        ("0", "", "serveur nécessaire"),
        ("100", " %", "hors ligne"),
        ("7", "/7", "jours par semaine"),
    ];
    section()
        .class("section privacy")
        .child(heading(
            "06",
            "Vie privée",
            "Vos habitudes ne regardent que vous.",
            "Sur le web, tout est dans le localStorage du navigateur ; sur iPhone, dans SwiftData. Rien n'est envoyé nulle part.",
        ))
        .child(ul().class("numbers").children(numbers.iter().enumerate().map(
            |(i, (value, suffix, label))| {
                li().class("number glass")
                    .attr("data-reveal", "")
                    .style(format!("--d:{}ms", i * 90))
                    .child(
                        strong()
                            .attr("data-count", *value)
                            .attr("data-suffix", *suffix)
                            .text(format!("{value}{suffix}")),
                    )
                    .child(span().text(*label))
            },
        )))
}

fn faq() -> Element {
    let list = [
        (
            "Est-ce gratuit ?",
            "Oui. L'app web et l'app iPhone sont gratuites, sans publicité.",
        ),
        (
            "Où sont mes données ?",
            "Uniquement sur votre appareil : le localStorage du navigateur pour la PWA, SwiftData pour l'iPhone. Effacer les données du site les efface pour de bon : pensez à l'export.",
        ),
        (
            "Comment l'installer sur mon téléphone ?",
            "Ouvrez l'app web, puis « Ajouter à l'écran d'accueil » dans le menu de partage (Safari) ou d'installation (Chrome). Elle marche ensuite hors ligne.",
        ),
        (
            "Que se passe-t-il si j'oublie un jour ?",
            "La série d'une habitude quotidienne repart de zéro, mais le record reste. Si vous aviez juste oublié de cocher, touchez le jour dans l'historique pour le corriger.",
        ),
        (
            "Et les habitudes « 3 fois par semaine » ?",
            "La série compte alors les semaines où l'objectif est atteint. Les jours vides d'une semaine réussie sont des jours de repos, pas des jours manqués.",
        ),
        (
            "Avec quoi est-ce construit ?",
            "Le site et la PWA sont en Rust avec le framework Paquets (pages rendues par le serveur, îlots WebAssembly, animations 2D dans un canvas). L'app iPhone est en Swift avec SwiftUI et SwiftData.",
        ),
    ];
    section()
        .class("section faq")
        .id("faq")
        .child(heading(
            "07",
            "FAQ",
            "Questions fréquentes.",
            "Et si la vôtre n'y est pas, le code est ouvert.",
        ))
        .child(div().class("faq__list").children(list.iter().map(|(q, r)| {
            details()
                .class("faq__item glass")
                .attr("data-reveal", "")
                .child(
                    summary()
                        .child(span().text(*q))
                        .child(span().class("faq__plus").attr("aria-hidden", "true")),
                )
                .child(p().text(*r))
        })))
}

fn cta() -> Element {
    section()
        .class("section cta")
        .child(
            div()
                .class("cta__card glass")
                .attr("data-reveal", "")
                .child(div().class("cta__shapes").attr("aria-hidden", "true").children(
                    (1..=4).map(|i| shape(&format!("shape--float shape--f{i}"))),
                ))
                .child(h2().text("Votre première série commence aujourd'hui."))
                .child(p().text("Ouvrez l'app, créez une habitude, cochez la journée. Demain, vous aurez une série."))
                .child(
                    a().class("btn btn--primary btn--lg")
                        .href("/app/")
                        .child(span().text("Ouvrir Routia"))
                        .child(span().class("btn__arrow").attr("aria-hidden", "true").text("→")),
                ),
        )
}

// ----- L'application ----------------------------------------------------------------

pub fn tracker(ctx: &Ctx) -> Page {
    let content = main()
        .class("app-page")
        .child(
            header()
                .class("app-page__head")
                .child(p().class("eyebrow").text("Routia · app web"))
                .child(h1().class("app-page__title").text("Mes habitudes"))
                .child(p().class("app-page__lead").text(
                    "Tout reste dans ce navigateur. Ajoutez Routia à l'écran d'accueil pour l'avoir hors ligne.",
                )),
        )
        .child(island("tracker", "", islands::tracker));
    layout(
        ctx,
        "Mes habitudes — Routia",
        "Créez vos habitudes, cochez la journée et suivez votre série. Hors ligne, sans compte.",
        "/app/",
        content,
    )
}

// ----- Page introuvable -------------------------------------------------------------

pub fn not_found(ctx: &Ctx) -> Page {
    let content = main()
        .class("lost")
        .child(div().class("lost__shape").child(shape("shape--lg")))
        .child(h1().text("404"))
        .child(p().text("Cette page a cassé sa série. Pas vous."))
        .child(
            a().class("btn btn--primary")
                .href("/")
                .text("Retour à l'accueil"),
        );
    layout(
        ctx,
        "Page introuvable — Routia",
        "Cette page n'existe pas.",
        "/404",
        content,
    )
    .status(404)
}
