// В релизной сборке на Windows не открываем лишнее консольное окно.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod game;
mod state;
mod window;

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
    let state = match AppState::new(paths, config) {
        Ok(state) => state,
        Err(err) => {
            tracing::error!(%err, "не удалось подготовить загрузчик");
            eprintln!("Cuber: {err}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(|app| {
            // Окно создаётся скрытым: ставим его на место, а показывает его
            // интерфейс, когда подгонит высоту под содержимое — без мигания.
            if let Some(main) = window::main_window(app.handle()) {
                if let Err(err) = window::place_initially(&main, window::MAX_HEIGHT) {
                    tracing::warn!(%err, "не удалось расположить окно");
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Moved(position) = event {
                window::on_moved(window, *position);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::get_config,
            commands::save_config,
            commands::open_data_dir,
            commands::fit_window,
            commands::restore_window,
            commands::selftest_enabled,
            commands::selftest_log,
            game::list_profiles,
            game::create_profile,
            game::list_versions,
            game::open_profile_dir,
            game::launch_profile,
        ])
        .run(tauri::generate_context!())
        .expect("ошибка при запуске окна Cuber");
}
