//! Окно лаунчера: прозрачное, без рамки, «растёт вверх».
//!
//! Весь интерфейс прижат к нижнему краю окна. Когда открывается Party или
//! выдвижная панель, интерфейс просит увеличить высоту окна, а нижний край
//! остаётся на месте — основная панель визуально не двигается.
//! Окно всегда ровно по размеру содержимого, поэтому пустые прозрачные
//! области не перехватывают клики по рабочему столу.
//!
//! Геометрию храним сами: у скрытого окна некоторые системы (GTK на Linux)
//! сообщают нулевой размер и позицию.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

/// Ширина окна, логические пиксели (как в дизайне).
pub const WIDTH: f64 = 520.0;
/// Максимальная высота окна.
pub const MAX_HEIGHT: f64 = 760.0;
/// Минимальная высота: основная панель с полями под тень.
pub const MIN_HEIGHT: f64 = 100.0;

/// Левый край, нижний край и высота окна в логических пикселях.
#[derive(Debug, Clone, Copy)]
struct Geometry {
    x: f64,
    bottom: f64,
    height: f64,
}

static GEOMETRY: Mutex<Option<Geometry>> = Mutex::new(None);

/// Когда окно впервые показано. Сразу после показа оконный менеджер может
/// сам сдвинуть окно — такие перемещения не считаем пользовательскими.
static SHOWN_AT: Mutex<Option<Instant>> = Mutex::new(None);
const SETTLE_AFTER_SHOW: Duration = Duration::from_millis(1000);

/// Отметить первый показ окна.
pub fn mark_shown() {
    *SHOWN_AT.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
}

/// Окно перемещено (пользователь перетащил его). Запоминаем новое место,
/// чтобы дальше расти и уменьшаться от него.
pub fn on_moved(window: &tauri::Window, position: PhysicalPosition<i32>) {
    let settled = SHOWN_AT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_some_and(|t| t.elapsed() >= SETTLE_AFTER_SHOW);
    if !settled {
        return;
    }
    let Ok(scale) = window.scale_factor() else {
        return;
    };
    let pos = position.to_logical::<f64>(scale);
    if let Some(g) = GEOMETRY.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        g.x = pos.x;
        g.bottom = pos.y + g.height;
    }
}

/// Ставит окно по центру по горизонтали, нижний край — примерно на 70 % высоты
/// рабочей области экрана (но не ниже неё и не выше, чем позволяет высота окна).
pub fn place_initially(window: &WebviewWindow, height: f64) -> tauri::Result<()> {
    let monitor = match window.current_monitor()? {
        Some(m) => Some(m),
        None => window.primary_monitor()?,
    };
    let Some(monitor) = monitor else {
        return Ok(());
    };

    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let left = area.position.x as f64 / scale;
    let top = area.position.y as f64 / scale;
    let width = area.size.width as f64 / scale;
    let area_height = area.size.height as f64 / scale;

    let geometry = Geometry {
        x: (left + (width - WIDTH) / 2.0).round(),
        bottom: (top + area_height)
            .min((top + MAX_HEIGHT).max(top + area_height * 0.7))
            .round(),
        height: height.clamp(MIN_HEIGHT, MAX_HEIGHT).round(),
    };
    apply(window, geometry, true)?;
    *GEOMETRY.lock().unwrap_or_else(|e| e.into_inner()) = Some(geometry);
    Ok(())
}

/// Меняет высоту окна, сохраняя положение нижнего края.
pub fn fit_height(window: &WebviewWindow, height: f64) -> tauri::Result<()> {
    let height = height.clamp(MIN_HEIGHT, MAX_HEIGHT).round();
    let mut guard = GEOMETRY.lock().unwrap_or_else(|e| e.into_inner());
    let Some(mut geometry) = *guard else {
        drop(guard);
        return place_initially(window, height);
    };

    let growing = height > geometry.height;
    let changed = (height - geometry.height).abs() >= 1.0;
    geometry.height = height;
    if changed {
        apply(window, geometry, growing)?;
    }
    *guard = Some(geometry);
    Ok(())
}

/// Ещё раз выставляет запомненную позицию (после показа окна оконный менеджер мог её сдвинуть).
pub fn reapply(window: &WebviewWindow) -> tauri::Result<()> {
    let geometry = *GEOMETRY.lock().unwrap_or_else(|e| e.into_inner());
    match geometry {
        Some(g) => apply(window, g, true),
        None => Ok(()),
    }
}

/// При росте сначала двигаем вверх, при уменьшении — сначала уменьшаем:
/// так в промежуточном кадре основная панель не прыгает.
fn apply(window: &WebviewWindow, g: Geometry, move_first: bool) -> tauri::Result<()> {
    let position = LogicalPosition::new(g.x, g.bottom - g.height);
    let size = LogicalSize::new(WIDTH, g.height);

    if cfg!(target_os = "linux") {
        // GTK у нерастягиваемого окна жёстко фиксирует размер и игнорирует set_size.
        // Поэтому на Linux окно формально растягиваемое, но минимальный и максимальный
        // размер всегда равны текущему — пользователь его не изменит.
        window.set_resizable(true)?;
        window.set_min_size(Some(size))?;
        window.set_max_size(Some(size))?;
    }
    if move_first {
        window.set_position(position)?;
        window.set_size(size)?;
    } else {
        window.set_size(size)?;
        window.set_position(position)?;
    }
    Ok(())
}

/// Главное окно приложения.
pub fn main_window(app: &tauri::AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}
