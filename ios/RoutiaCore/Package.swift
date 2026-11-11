// swift-tools-version: 6.4
// La logique de Routia sans interface : dates, séries, historique, export.
// Elle tourne sur iOS, macOS et Linux (`swift test`), et suit les mêmes règles que le
// site en Rust (`site/src/habits.rs`).

import PackageDescription

let package = Package(
    name: "RoutiaCore",
    platforms: [.iOS(.v26), .macOS(.v26)],
    products: [
        .library(name: "RoutiaCore", targets: ["RoutiaCore"]),
    ],
    targets: [
        .target(name: "RoutiaCore"),
        .testTarget(name: "RoutiaCoreTests", dependencies: ["RoutiaCore"]),
    ],
    swiftLanguageModes: [.v6]
)
