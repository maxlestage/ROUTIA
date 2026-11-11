//! Routia, écrit avec Paquets : la logique des habitudes et les îlots partagés par le
//! serveur et le navigateur, le serveur (natif) et le bundle du navigateur (wasm32).

pub mod habits;
pub mod islands;

#[cfg(not(target_arch = "wasm32"))]
pub mod export;
#[cfg(not(target_arch = "wasm32"))]
pub mod pages;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(target_arch = "wasm32")]
mod client;
