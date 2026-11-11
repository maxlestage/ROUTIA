# Routia 🎯

**Le tracker d'habitudes minimaliste : une habitude à la fois.**

| Fonctionnalité | Web (PWA) | iPhone |
|---|---|---|
| Créer une habitude (nom, fréquence, couleur) | ✔️ | ✔️ |
| Cocher la journée « fait » | ✔️ | ✔️ |
| Suivre la série (*streak*) et le record | ✔️ | ✔️ |
| Historique sur 5 semaines, jours manqués visibles, correction d'un oubli | ✔️ | ✔️ |
| Export en texte ou en image | ✔️ | ✔️ |

Aucun compte, aucun serveur pour les données : tout reste sur l'appareil
(`localStorage` sur le web, SwiftData sur iPhone).

## Le dépôt

| Dossier | Contenu | Technologies |
|---|---|---|
| [`site/`](site/) | Le site de présentation et l'app web installable (`/app/`) | Rust 1.99, [Paquets](https://github.com/maxlestage/Paquets) (pages rendues par le serveur, îlots WebAssembly, animations 2D en canvas et en CSS) |
| [`ios/`](ios/) | L'app iPhone | Swift 6.4, SwiftUI (Liquid Glass), SwiftData, Xcode 27.2 (SDK iOS 27), à partir d'iOS 26 |

Les deux suivent les mêmes règles de séries, testées des deux côtés
(`site/src/habits.rs` et `ios/RoutiaCore`) :

- **chaque jour** : la série compte les jours d'affilée ; aujourd'hui, pas encore coché, ne la casse pas ;
- **N fois par semaine** : la série compte les semaines (du lundi au dimanche) où l'objectif est atteint ; la semaine en cours ne la casse pas ;
- un jour vide est **manqué** pour une habitude quotidienne, ou dans une semaine ratée ; c'est un jour de **repos** dans une semaine réussie.

## Démarrer

```sh
# Site + app web, sur http://localhost:3000
cd site
cargo install wasm-bindgen-cli --version 0.2.130 --locked
rustup target add wasm32-unknown-unknown
./build.sh && ./target/release/routia-site

# App iPhone (macOS, Xcode 27)
cd ios
brew install xcodegen
xcodegen generate && open Routia.xcodeproj
```

## Vérifications, fusion automatique et déploiement

La CI (`.github/workflows/ci.yml`) vérifie chaque push :

- `site` : `cargo fmt`, `clippy` (serveur et WebAssembly), `cargo test`, le bundle, puis
  `e2e/overflow.mjs` ouvre chaque page de 320 à 1440 px dans Chromium et échoue si la page
  peut défiler à l'horizontale, si une ressource manque ou si le navigateur signale une
  erreur, sur le serveur comme sur l'export statique ;
- `ios-core` : `swift test` de RoutiaCore sous Linux (Swift 6.4) ;
- `ios-app` : la compilation de l'app avec Xcode 27.2 ;
- `merge` : quand tout est vert, la branche est **fusionnée automatiquement dans
  `master`** (sauf les branches `wip/…`), puis le déploiement est lancé.

Le site est publié sur **GitHub Pages** par `.github/workflows/pages.yml` (à chaque mise à
jour de `master`, ou à la main depuis l'onglet Actions) : le serveur écrit chaque page en
fichiers statiques (`routia-site export`), sous le chemin du dépôt.

À régler une fois dans GitHub : **Settings → Pages → Source : GitHub Actions**.
Adresse : <https://maxlestage.github.io/ROUTIA/>.
