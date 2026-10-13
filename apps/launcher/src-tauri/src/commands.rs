//! Команды, которые вызывает интерфейс через `invoke(...)`.
//!
//! Ошибки возвращаются строкой: интерфейсу нужен только понятный текст.

use serde::Serialize;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

use cuber_core::Config;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    data_dir: String,
    platform: &'static str,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: cuber_core::VERSION,
        data_dir: state.paths.root().display().to_string(),
        platform: std::env::consts::OS,
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> CmdResult<Config> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(state: State<'_, AppState>, config: Config) -> CmdResult<Config> {
    config
        .save(&state.paths.config_file())
        .map_err(|e| e.to_string())?;
    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config.clone();
    tracing::info!("настройки сохранены");
    Ok(config)
}

#[tauri::command]
pub fn open_data_dir(app: tauri::AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    app.opener()
        .open_path(state.paths.root().display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}
