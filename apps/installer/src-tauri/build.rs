//! Встраивает лаунчер в установщик.
//!
//! Путь к собранному cuber-launcher.exe передаётся через CUBER_PAYLOAD.
//! Без него (разработка, проверки CI) установщик собирается с пустой
//! «начинкой» и честно сообщает, что ставить нечего.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=CUBER_PAYLOAD");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("payload.bin");
    match std::env::var("CUBER_PAYLOAD") {
        Ok(path) if !path.is_empty() => {
            println!("cargo:rerun-if-changed={path}");
            std::fs::copy(&path, &out)
                .unwrap_or_else(|e| panic!("не удалось прочитать CUBER_PAYLOAD={path}: {e}"));
        }
        _ => {
            std::fs::write(&out, []).unwrap();
        }
    }
    tauri_build::build()
}
