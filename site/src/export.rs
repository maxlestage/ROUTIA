//! `routia-site export <dossier>` : tout le site en fichiers statiques, pour GitHub Pages.
//! Chaque page vient du serveur lui-même, via le client de test en mémoire de Paquets
//! (`app.test()`) : le site statique et le serveur sont identiques.
//!
//! GitHub Pages sert un dépôt sous un sous-chemin (`/ROUTIA`) : `BASE_PATH` le préfixe à
//! chaque adresse absolue des pages, du manifeste et du service worker.

use std::io;
use std::path::{Path, PathBuf};

use crate::server::{app, site_dir};

/// Les pages à écrire : (chemin demandé, fichier écrit, statut attendu).
const PAGES: [(&str, &str, u16); 3] = [
    ("/", "index.html", 200),
    ("/app/", "app/index.html", 200),
    ("/404", "404.html", 404),
];

/// Les autres réponses du serveur à garder telles quelles (avec le préfixe).
const FILES: [&str; 4] = [
    "/manifest.webmanifest",
    "/sw.js",
    "/sitemap.xml",
    "/robots.txt",
];

/// `/ROUTIA/` → `/ROUTIA` ; `/` ou rien → rien.
pub fn normalize_base(base: &str) -> String {
    let trimmed = base.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

/// Chaque adresse absolue entre guillemets (`"/…"`, mais pas `"//…"`) derrière `base`,
/// pour le manifeste et le service worker (du JSON et du JavaScript).
pub fn with_base(text: &str, base: &str) -> String {
    prefix_after(text, "\"", base)
}

/// Les adresses d'une page : les attributs `href`, `src` et `action`, et ce que Paquets
/// écrit lui-même (le chargeur du bundle `/pkg/…`, l'enregistrement de `/sw.js`). Les
/// autres textes (`data-suffix="/7"`, les balises `"/>`) restent tels quels.
pub fn html_with_base(html: &str, base: &str) -> String {
    let mut out = html.to_owned();
    for marker in ["href=\"", "src=\"", "action=\""] {
        out = prefix_after(&out, marker, base);
    }
    out.replace("\"/pkg/", &format!("\"{base}/pkg/")).replace(
        "register(\"/sw.js\")",
        &format!("register(\"{base}/sw.js\")"),
    )
}

/// Ajoute `base` après chaque `marker` suivi d'une adresse absolue (`/…`, pas `//…`).
fn prefix_after(text: &str, marker: &str, base: &str) -> String {
    if base.is_empty() {
        return text.to_owned();
    }
    let already = format!("{base}/");
    let mut out = String::with_capacity(text.len() + 256);
    let mut rest = text;
    while let Some(i) = rest.find(marker) {
        let end = i + marker.len();
        out.push_str(&rest[..end]);
        let after = &rest[end..];
        if after.starts_with('/') && !after.starts_with("//") && !after.starts_with(&already) {
            out.push_str(base);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Écrit le site dans `out`. Rend le nombre de fichiers écrits.
pub async fn export(out: &Path, base: &str) -> io::Result<usize> {
    let base = normalize_base(base);
    let out = std::path::absolute(out)?;
    // Paquets trouve `public/` et `pkg/` (le bundle) dans le dossier courant.
    let dir = std::path::absolute(site_dir())?;
    std::env::set_current_dir(&dir)?;
    if !dir.join("pkg").is_dir() {
        eprintln!("attention : pkg/ manque, les pages ne seront pas interactives (voir build.sh)");
    }
    let mut app = app();
    // Pas de serveur à protéger, et l'export demande toutes les pages d'un coup.
    app.security(None);
    let client = app.test().await;

    let mut written = 0;
    let mut write = |path: &str, bytes: &[u8]| -> io::Result<()> {
        let file = out.join(path.trim_start_matches('/'));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)?;
        written += 1;
        Ok(())
    };

    for (path, file, status) in PAGES {
        let res = client.get(path).await;
        if res.status().as_u16() != status {
            return Err(io::Error::other(format!(
                "GET {path} : {} au lieu de {status}",
                res.status()
            )));
        }
        write(file, html_with_base(&res.text(), &base).as_bytes())?;
    }
    for path in FILES {
        let res = client.get(path).await;
        if res.status() != 200 {
            return Err(io::Error::other(format!("GET {path} : {}", res.status())));
        }
        let text = res.text();
        // Le plan du site et robots.txt ont déjà des adresses complètes (SITE_URL).
        let text = if path.ends_with(".xml") || path.ends_with(".txt") {
            text
        } else {
            with_base(&text, &base)
        };
        write(path, text.as_bytes())?;
    }
    for folder in ["public", "pkg"] {
        let root = dir.join(folder);
        let prefix = if folder == "pkg" { "pkg/" } else { "" };
        for file in files(&root) {
            let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
            let url = format!("{prefix}{}", relative.to_string_lossy().replace('\\', "/"));
            // Les fichiers de wasm-bindgen inutiles pour lancer le bundle.
            if url.ends_with(".gitignore") || url.ends_with("package.json") {
                continue;
            }
            write(&url, &std::fs::read(&file)?)?;
        }
    }
    // Les fichiers sont servis tels quels : pas de Jekyll sur GitHub Pages.
    write(".nojekyll", b"")?;
    Ok(written)
}

/// Tous les fichiers sous `dir`, récursivement, dans l'ordre.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_page_urls_only() {
        let html = r##"<meta charset="utf-8"/><a href="/app/">x</a><link href="/site.css?v=1"/><a href="https://x.dev/a">y</a><a href="#ios">z</a><img src="//cdn/x.png"/><strong data-suffix="/7">7/7</strong><script>navigator.serviceWorker.register("/sw.js")</script><script type="module">import init from "/pkg/a.js";init({module_or_path:"/pkg/a_bg.wasm"});</script>"##;
        let out = html_with_base(html, "/ROUTIA");
        assert!(out.contains(r#"<meta charset="utf-8"/>"#));
        assert!(out.contains(r#"href="/ROUTIA/app/""#));
        assert!(out.contains(r#"href="/ROUTIA/site.css?v=1""#));
        assert!(out.contains(r#"href="https://x.dev/a""#));
        assert!(out.contains(r##"href="#ios""##));
        assert!(out.contains(r#"src="//cdn/x.png""#));
        assert!(out.contains(r#"data-suffix="/7""#));
        assert!(out.contains(r#"register("/ROUTIA/sw.js")"#));
        assert!(out.contains(r#"from "/ROUTIA/pkg/a.js""#));
        assert!(out.contains(r#"module_or_path:"/ROUTIA/pkg/a_bg.wasm""#));
        // Une deuxième passe ne double pas le préfixe des attributs.
        assert_eq!(prefix_after(&out, "href=\"", "/ROUTIA"), out);
        assert_eq!(html_with_base(html, ""), html);
        let json = r#"{"start_url":"/app/","icons":[{"src":"/assets/i.png"}]}"#;
        assert_eq!(
            with_base(json, "/ROUTIA"),
            r#"{"start_url":"/ROUTIA/app/","icons":[{"src":"/ROUTIA/assets/i.png"}]}"#
        );
    }

    #[test]
    fn normalizes_the_base_path() {
        assert_eq!(normalize_base(""), "");
        assert_eq!(normalize_base("/"), "");
        assert_eq!(normalize_base("ROUTIA/"), "/ROUTIA");
        assert_eq!(normalize_base("/ROUTIA"), "/ROUTIA");
    }
}
