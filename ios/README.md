# Routia pour iPhone

SwiftUI, SwiftData, Swift 6.4 (concurrence stricte), Liquid Glass. Compilée avec Xcode 27.2
(SDK iOS 27), elle tourne à partir d'iOS 26.

| Dossier | Contenu |
|---|---|
| `RoutiaCore/` | Paquet Swift sans interface : dates, séries, historique, export texte. `swift test` (Swift Testing), aussi sous Linux. |
| `Routia/` | L'app : `Model/HabitRecord.swift` (SwiftData), `Views/` (écran du jour, carte d'habitude, historique, création, export en texte et en image). |
| `project.yml` | Le projet Xcode, généré par [XcodeGen](https://github.com/yonaskolb/XcodeGen). |

```sh
brew install xcodegen
xcodegen generate
open Routia.xcodeproj
```

Animations, toutes en 2D : anneau de progression, flamme qui respire (`symbolEffect`),
carte qui rebondit quand la journée est cochée (`keyframeAnimator`), cartes qui
s'estompent au défilement (`scrollTransition`), anneaux qui se remplissent sur l'écran vide,
retour haptique.
