//! Le serveur : une `paquets::App` avec les pages, le plan du site, la PWA et les
//! fichiers statiques (`public/`) et le bundle (`pkg/`), trouvés par Paquets dans le
//! dossier courant.

use std::io;
use std::path::{Path, PathBuf};

use paquets::prelude::*;

use crate::pages;

/// Les pages du site, pour le plan.
pub const ROUTES: [&str; 2] = ["/", "/app/"];

/// Le dossier `site/` : `SITE_DIR`, le dossier courant s'il a `public/`, sinon celui
/// de la crate.
pub fn site_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SITE_DIR") {
        return PathBuf::from(dir);
    }
    if Path::new("public").is_dir() {
        return PathBuf::from(".");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// FNV-1a, pour distinguer les versions de la feuille de style (`?v=`).
pub fn version(bytes: &[u8]) -> String {
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{h:016x}")[..10].to_owned()
}

/// L'adresse publique : `SITE_URL`, sinon celle de la requête.
fn origin(req: &Request) -> String {
    if let Ok(url) = std::env::var("SITE_URL") {
        let url = url.trim().trim_end_matches('/');
        if !url.is_empty() {
            return url.to_owned();
        }
    }
    let header = |name: &str| {
        req.header(name)
            .map(|v| v.split(',').next().unwrap_or("").trim().to_owned())
    };
    let host = header("x-forwarded-host")
        .or_else(|| header("host"))
        .filter(|h| {
            !h.is_empty()
                && h.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
        })
        .unwrap_or_else(|| "localhost".into());
    let proto = if header("x-forwarded-proto").as_deref() == Some("https") {
        "https"
    } else {
        "http"
    };
    format!("{proto}://{host}")
}

fn sitemap(req: &Request) -> Response {
    let origin = origin(req);
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for path in ROUTES {
        xml.push_str(&format!("<url><loc>{origin}{path}</loc></url>\n"));
    }
    xml.push_str("</urlset>\n");
    Response::new()
        .header("content-type", "application/xml; charset=utf-8")
        .send(xml)
}

fn robots(req: &Request) -> Response {
    Response::new()
        .header("content-type", "text/plain; charset=utf-8")
        .send(format!(
            "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
            origin(req)
        ))
}

/// L'application, prête à tourner ou à être testée.
pub fn app() -> App {
    let css = std::fs::read(site_dir().join("public/site.css")).unwrap_or_default();
    let css_version: &'static str = Box::leak(version(&css).into_boxed_str());
    let mut app = App::new();
    app.get("/", move |req: Request| async move {
        pages::home(&pages::Ctx::new(origin(&req), css_version))
            .into_response()
            .header("cache-control", "no-cache")
    });
    app.get("/app/", move |req: Request| async move {
        pages::tracker(&pages::Ctx::new(origin(&req), css_version))
            .into_response()
            .header("cache-control", "no-cache")
    });
    app.get("/sitemap.xml", |req: Request| async move { sitemap(&req) });
    app.get("/robots.txt", |req: Request| async move { robots(&req) });
    // Installable sur l'écran d'accueil, utilisable hors ligne.
    app.pwa(
        Pwa::new("Routia")
            .short_name("Routia")
            .description("Le tracker d'habitudes minimaliste : une habitude à la fois.")
            .start_url("/app/")
            .theme_color("#0b0820")
            .background_color("#0b0820")
            .categories(["health", "lifestyle", "productivity"])
            .icon("/assets/icon-192.png", "192x192", "image/png")
            .icon("/assets/icon-512.png", "512x512", "image/png")
            .maskable_icon("/assets/icon-maskable-512.png", "512x512", "image/png"),
    );
    app.fallback(move |req: Request| async move {
        pages::not_found(&pages::Ctx::new(origin(&req), css_version)).into_response()
    });
    app
}

/// `cargo run` : le serveur, sur `PORT` (Heroku) ou 3000.
pub fn run() -> io::Result<()> {
    let dir = site_dir();
    if dir != Path::new(".") {
        std::env::set_current_dir(&dir)?;
    }
    app().run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_change_with_the_content() {
        assert_eq!(version(b"a").len(), 10);
        assert_ne!(version(b"a"), version(b"b"));
    }
}
