//! `cargo run` : le site sur http://localhost:3000 (`PORT` sinon). Lancez-le depuis
//! `site/`, après avoir construit le bundle du navigateur (`./build.sh`).
//!
//! `cargo run -- export dist` : tout le site en fichiers statiques dans `dist/`, pour
//! GitHub Pages. `BASE_PATH` (`/ROUTIA`) préfixe chaque adresse, `SITE_URL`
//! (`https://maxlestage.github.io/ROUTIA`) donne les adresses complètes.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("export") => {
            let dir = args.next().unwrap_or_else(|| "dist".into());
            let base = std::env::var("BASE_PATH").unwrap_or_default();
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            let files = runtime.block_on(routia_site::export::export(
                std::path::Path::new(&dir),
                &base,
            ))?;
            println!("{files} fichiers écrits dans {dir}");
            Ok(())
        }
        Some(other) => {
            eprintln!("commande inconnue {other:?} : rien, ou `export <dossier>`");
            std::process::exit(2);
        }
        None => routia_site::server::run(),
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
