//! Команды профилей, версий и запуска игры.
//!
//! Ход подготовки и состояние игры интерфейс получает событиями:
//! - `launch-progress` — { profileId, stage, fraction, done, total };
//! - `game-started`   — { profileId };
//! - `game-exited`    — { profileId, code, crashed }.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use cuber_core::AfterLaunch;
use cuber_launch::mojang::rules::Env;
use cuber_launch::{launch, LaunchOptions, Player, Profile, Progress};

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
    id: String,
    name: String,
    version: String,
    loader: String,
}

impl From<&Profile> for ProfileDto {
    fn from(p: &Profile) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            version: p.minecraft.clone(),
            loader: p.loader.display_name().into(),
        }
    }
}

#[derive(Serialize)]
pub struct VersionDto {
    id: String,
    kind: String,
}

#[derive(Serialize)]
pub struct VersionsDto {
    latest: String,
    versions: Vec<VersionDto>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    profile_id: String,
    stage: cuber_launch::Stage,
    fraction: f64,
    done: u64,
    total: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProfileEvent {
    profile_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ExitEvent {
    profile_id: String,
    code: Option<i32>,
    crashed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProfile {
    name: String,
    version: String,
}

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> CmdResult<Vec<ProfileDto>> {
    let list = state.profiles.list().map_err(|e| e.to_string())?;
    Ok(list.iter().map(ProfileDto::from).collect())
}

#[tauri::command]
pub fn create_profile(state: State<'_, AppState>, profile: NewProfile) -> CmdResult<ProfileDto> {
    let p = state
        .profiles
        .create(Profile::new(&profile.name, &profile.version))
        .map_err(|e| e.to_string())?;
    tracing::info!(id = p.id, version = p.minecraft, "создан профиль");
    Ok(ProfileDto::from(&p))
}

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>, snapshots: bool) -> CmdResult<VersionsDto> {
    let manifest = state
        .installer
        .version_manifest()
        .await
        .map_err(|e| e.to_string())?;
    Ok(VersionsDto {
        latest: manifest.latest.release.clone(),
        versions: manifest
            .versions
            .iter()
            .filter(|v| snapshots || v.kind == "release")
            .map(|v| VersionDto {
                id: v.id.clone(),
                kind: v.kind.clone(),
            })
            .collect(),
    })
}

#[tauri::command]
pub fn open_profile_dir(app: AppHandle, state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let profile = state.profiles.load(&id).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(
            state.profiles.game_dir(&profile.id).display().to_string(),
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

/// Подготовить (скачать недостающее) и запустить профиль.
/// Возвращается, когда процесс игры запущен; дальше — события.
#[tauri::command]
pub async fn launch_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    let profile = state.profiles.load(&id).map_err(|e| e.to_string())?;
    {
        let running = state.running.lock().map_err(|e| e.to_string())?;
        if running.contains(&profile.id) {
            return Err("этот профиль уже запущен".into());
        }
    }
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();

    let nickname = config
        .last_nickname
        .clone()
        .unwrap_or_else(|| "Player".into());
    if !cuber_launch::offline::is_valid_nickname(&nickname) {
        return Err(format!(
            "недопустимый ник «{nickname}» — исправьте его в настройках"
        ));
    }

    let game_dir = state.profiles.game_dir(&profile.id);
    let progress_app = app.clone();
    let profile_id = profile.id.clone();
    let on_progress = move |p: Progress| {
        let _ = progress_app.emit(
            "launch-progress",
            ProgressEvent {
                profile_id: profile_id.clone(),
                stage: p.stage,
                fraction: p.fraction(),
                done: p.done,
                total: p.total,
            },
        );
    };

    tracing::info!(
        id = profile.id,
        version = profile.minecraft,
        "подготовка к запуску"
    );
    let prepared = state
        .installer_for(&profile)
        .prepare(&profile.version_id(), &game_dir, on_progress)
        .await
        .map_err(|e| {
            tracing::error!(%e, "подготовка не удалась");
            e.to_string()
        })?;

    let opts = LaunchOptions {
        player: Player::offline(&nickname),
        game_dir,
        memory_mb: profile.memory_mb.unwrap_or(config.default_memory_mb),
        java_override: profile.java_path.clone(),
        extra_jvm_args: profile.jvm_args.clone(),
        resolution: None,
    };
    let cmd = launch::build(&prepared, &opts, &Env::current()).map_err(|e| e.to_string())?;
    let log = state.profiles.logs_dir(&profile.id).join("latest.log");
    let mut child = launch::spawn(&cmd, &log).map_err(|e| e.to_string())?;
    tracing::info!(pid = child.id(), id = profile.id, "игра запущена");

    state
        .running
        .lock()
        .map_err(|e| e.to_string())?
        .insert(profile.id.clone());
    let _ = state.profiles.mark_played(&profile.id);
    let _ = app.emit(
        "game-started",
        ProfileEvent {
            profile_id: profile.id.clone(),
        },
    );

    // Ждём выхода игры в отдельном потоке.
    let wait_app = app.clone();
    let id = profile.id.clone();
    std::thread::spawn(move || {
        let status = child.wait();
        let code = status.as_ref().ok().and_then(|s| s.code());
        let crashed = !status.map(|s| s.success()).unwrap_or(false);
        tracing::info!(?code, id, "игра завершилась");
        if let Some(state) = wait_app.try_state::<AppState>() {
            if let Ok(mut running) = state.running.lock() {
                running.remove(&id);
            }
        }
        let _ = wait_app.emit(
            "game-exited",
            ExitEvent {
                profile_id: id,
                code,
                crashed,
            },
        );
        // Если лаунчер был свёрнут на время игры — вернуть его.
        if let Some(w) = crate::window::main_window(&wait_app) {
            let _ = w.unminimize();
        }
    });

    // Что делать с лаунчером после запуска.
    match config.after_launch {
        AfterLaunch::KeepOpen => {}
        AfterLaunch::Minimize => {
            if let Some(w) = crate::window::main_window(&app) {
                // Небольшая пауза, чтобы интерфейс успел показать «В игре».
                tokio::time::sleep(Duration::from_millis(600)).await;
                let _ = w.minimize();
            }
        }
        AfterLaunch::Close => {
            tokio::time::sleep(Duration::from_millis(600)).await;
            app.exit(0);
        }
    }
    Ok(())
}
