# Le site de Routia

Écrit en Rust avec [Paquets](https://github.com/maxlestage/Paquets) : le serveur est une
`paquets::App`, les pages sont des `paquets::Page` construites avec les vues d'Active, et
les parties interactives sont des îlots compilés en WebAssembly. Tout est en 2D : deux
canvas dessinés en Rust (API Canvas 2D) et des animations CSS.

| Partie | Où |
|---|---|
| Règles des habitudes (dates, séries, historique, export) | `src/habits.rs` (testé) |
| Îlots : anneaux animés, tour de briques, tracker | `src/islands.rs` |
| Serveur, routes, PWA | `src/server.rs` |
| Pages (accueil, app, 404) | `src/pages.rs` |
| Export statique pour GitHub Pages | `src/export.rs` |
| Navigateur : canvas 2D, animations, stockage, export | `src/client/` |
| Styles (mobile first), logo | `public/` |

## Routes

| Chemin | Page |
|---|---|
| `/` | présentation : anneaux qui se remplissent et jours en orbite (canvas 2D), tour de briques qui tombent et rebondissent, cartes avec halo et verso qui se dévoile, iPhone qui flotte, animations au défilement |
| `/app/` | le tracker, installable et hors ligne (`manifest.webmanifest`, service worker) |
| `/sitemap.xml`, `/robots.txt` | pour les moteurs de recherche |

## Construire et lancer

```sh
./build.sh                      # bundle (pkg/) puis serveur
./target/release/routia-site    # http://localhost:3000, ou $PORT
```

Ou avec Docker : `docker build -t routia . && docker run -p 3000:3000 routia`.

## GitHub Pages

```sh
./build.sh
BASE_PATH=/ROUTIA SITE_URL=https://maxlestage.github.io/ROUTIA \
  ./target/release/routia-site export dist
```

Chaque page est rendue par l'app elle-même (client de test en mémoire de Paquets), puis
écrite dans `dist/` avec `404.html`, le manifeste, le service worker, `public/` et `pkg/`.
Toutes les adresses passent derrière `BASE_PATH`. Le workflow `pages.yml` fait de même et
publie `dist/`.

## Mobile first, sans débordement

Les règles de base de `public/site.css` visent un écran de 320 px ; les requêtes
`min-width` ajoutent la mise en page des grands écrans. Rien ne doit dépasser la largeur
de l'écran : `node e2e/overflow.mjs http://localhost:3000` le vérifie de 320 à 1440 px
(la CI le lance à chaque push, sur le serveur et sur l'export statique).
