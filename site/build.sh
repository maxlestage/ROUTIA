#!/usr/bin/env bash
# Construit le bundle du navigateur (pkg/) puis le serveur. À lancer depuis site/.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --lib --target wasm32-unknown-unknown --profile wasm-release
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir pkg target/wasm32-unknown-unknown/wasm-release/routia_site.wasm
cargo build --release
