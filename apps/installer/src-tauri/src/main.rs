//! Установщик Cuber: своё окно в стиле лаунчера вместо стандартного.
//!
//! - ставит лаунчер для текущего пользователя (без прав администратора)
//!   в %LOCALAPPDATA%\Programs\Cuber;
//! - создаёт ярлыки в меню «Пуск» и, по желанию, на рабочем столе;
//! - регистрируется в «Приложения и возможности» Windows;
//! - тот же exe, скопированный как «Uninstall Cuber.exe», удаляет лаунчер.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod setup;

use serde::Serialize;

/// Лаунчер, встроенный при сборке (см. build.rs).
static PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.bin"));

type CmdResult<T> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    version: &'static str,
    /// install | uninstall
    mode: &'static str,
    default_dir: String,
    /// Куда Cuber уже установлен (тогда установка — это обновление).
    installed_dir: Option<String>,
    has_payload: bool,
    supported: bool,
}

#[tauri::command]
fn info() -> Info {
    Info {
        version: env!("CARGO_PKG_VERSION"),
        mode: if setup::is_uninstall_mode() {
            "uninstall"
        } else {
            "install"
        },
        default_dir: setup::default_dir().display().to_string(),
        installed_dir: setup::installed_dir().map(|p| p.display().to_string()),
        has_payload: !PAYLOAD.is_empty(),
        supported: cfg!(windows),
    }
}

#[tauri::command]
async fn install(dir: String, desktop_shortcut: bool) -> CmdResult<()> {
    if PAYLOAD.is_empty() {
        return Err("Этот установщик собран без лаунчера внутри".into());
    }
    let dir = std::path::PathBuf::from(dir);
    tauri::async_runtime::spawn_blocking(move || setup::install(&dir, PAYLOAD, desktop_shortcut))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn uninstall(remove_data: bool) -> CmdResult<()> {
    tauri::async_runtime::spawn_blocking(move || setup::uninstall(remove_data))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn launch(app: tauri::AppHandle, dir: String) -> CmdResult<()> {
    setup::launch(std::path::Path::new(&dir))?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |p| {
        let _ = tx.send(p);
    });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
        .map(|p| {
            // Ставим в подпапку Cuber внутри выбранной папки.
            let path = p.to_string();
            let base = std::path::PathBuf::from(path);
            if base.file_name().is_some_and(|n| n == "Cuber") {
                base.display().to_string()
            } else {
                base.join("Cuber").display().to_string()
            }
        })
}

fn main() {
    // Тихий режим без окна (для администраторов и автотестов):
    //   Cuber-Setup.exe --silent [--dir <папка>] [--desktop]
    //   "Uninstall Cuber.exe" --uninstall --silent [--wipe]
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--silent") {
        let code = match run_silent(&args) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Cuber Setup: {e}");
                1
            }
        };
        std::process::exit(code);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            info,
            install,
            uninstall,
            launch,
            pick_folder
        ])
        .run(tauri::generate_context!())
        .expect("не удалось открыть окно установщика");
}

fn run_silent(args: &[String]) -> Result<(), String> {
    if setup::is_uninstall_mode() {
        return setup::uninstall(args.iter().any(|a| a == "--wipe"));
    }
    if PAYLOAD.is_empty() {
        return Err("установщик собран без лаунчера внутри".into());
    }
    let dir = args
        .iter()
        .position(|a| a == "--dir")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .or_else(setup::installed_dir)
        .unwrap_or_else(setup::default_dir);
    setup::install(&dir, PAYLOAD, args.iter().any(|a| a == "--desktop"))?;
    println!("установлено в {}", dir.display());
    Ok(())
}
