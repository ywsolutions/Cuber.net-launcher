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

/// Подгоняет высоту окна под содержимое (нижний край остаётся на месте)
/// и при `show = true` показывает окно, если оно ещё скрыто.
///
/// Команда асинхронная: так она выполняется не в главном потоке окна,
/// и интерфейс не замирает, пока система двигает окно.
#[tauri::command]
pub async fn fit_window(window: tauri::WebviewWindow, height: f64, show: bool) -> CmdResult<()> {
    crate::window::fit_height(&window, height).map_err(|e| e.to_string())?;
    if show && !window.is_visible().unwrap_or(true) {
        window.show().map_err(|e| e.to_string())?;
        crate::window::mark_shown();
        // После показа — ещё раз на место: оконный менеджер мог сдвинуть окно.
        let _ = crate::window::reapply(&window);
        let _ = window.set_focus();
    }
    Ok(())
}

/// Возвращает окну рассчитанные размер и позицию. Вызывается вскоре после
/// первого показа: часть систем в момент показа сбрасывает размер или сдвигает окно.
#[tauri::command]
pub async fn restore_window(window: tauri::WebviewWindow) -> CmdResult<()> {
    crate::window::reapply(&window).map_err(|e| e.to_string())
}

// ── Самопроверка интерфейса (CI) ────────────────────────────────────
// Включается переменной CUBER_SELFTEST=<файл отчёта>. Интерфейс сам открывает
// и закрывает панели, а сюда пишет размеры окна — так в CI на Windows проверяется,
// что окно растёт вверх, нижний край стоит на месте и ничего не зависает.

#[tauri::command]
pub fn selftest_enabled() -> bool {
    std::env::var_os("CUBER_SELFTEST").is_some()
}

#[tauri::command]
pub async fn selftest_log(window: tauri::WebviewWindow, step: String) -> CmdResult<()> {
    let Some(path) = std::env::var_os("CUBER_SELFTEST") else {
        return Ok(());
    };
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let pos = window
        .outer_position()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let size = window
        .outer_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let line = format!(
        "{step}\tx={:.0}\ty={:.0}\tw={:.0}\th={:.0}\tbottom={:.0}\n",
        pos.x,
        pos.y,
        size.width,
        size.height,
        pos.y + size.height
    );
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(line.as_bytes()).map_err(|e| e.to_string())
}
