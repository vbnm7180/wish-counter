//! Скрипт сборки (build script) Cargo.
//!
//! Cargo запускает его перед компиляцией крейта. `tauri_build::build()`
//! читает `tauri.conf.json`, проверяет иконки и права (capabilities)
//! и встраивает их в итоговый `.exe`.

fn main() {
    tauri_build::build();
}
