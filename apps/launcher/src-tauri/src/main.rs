// В релизной сборке на Windows не открываем лишнее консольное окно.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use cuber_core::{logging, Config, Paths};

use crate::state::AppState;

fn main() {
    let paths = Paths::discover().expect("не удалось определить папку данных");
    if let Err(err) = paths.ensure() {
        eprintln!("Cuber: не удалось создать папку данных: {err}");
        std::process::exit(1);
    }

    // Без логов лаунчер всё равно должен работать, поэтому ошибка здесь не фатальна.
    let _log_guard = match logging::init(&paths.logs()) {
        Ok(guard) => Some(guard),
        Err(err) => {
            eprintln!("Cuber: логирование отключено: {err}");
            None
        }
    };

    tracing::info!(
        version = cuber_core::VERSION,
        data = %paths.root().display(),
        "запуск Cuber"
    );

    let config = Config::load_or_recover(&paths.config_file());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(paths, config))
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::get_config,
            commands::save_config,
            commands::open_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("ошибка при запуске окна Cuber");
}
