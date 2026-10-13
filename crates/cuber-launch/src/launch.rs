//! Сборка командной строки и запуск процесса игры.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::error::{Error, IoContext, Result};
use crate::install::Prepared;
use crate::mojang::rules::Env;
use crate::offline::Player;

pub const LAUNCHER_NAME: &str = "cuber";

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub player: Player,
    /// Папка игры профиля: миры, моды, настройки.
    pub game_dir: PathBuf,
    pub memory_mb: u32,
    /// Своя Java вместо скачанной (необязательно).
    pub java_override: Option<PathBuf>,
    pub extra_jvm_args: Vec<String>,
    /// Размер окна игры (если задан).
    pub resolution: Option<(u32, u32)>,
}

/// Готовая команда: что запустить и с какими аргументами.
#[derive(Debug, Clone)]
pub struct LaunchCommand {
    pub java: PathBuf,
    pub args: Vec<String>,
    pub game_dir: PathBuf,
}

pub fn build(prepared: &Prepared, opts: &LaunchOptions, env: &Env) -> Result<LaunchCommand> {
    let v = &prepared.version;
    let main_class = v
        .main_class
        .clone()
        .ok_or_else(|| Error::Launch("в описании версии нет mainClass".into()))?;

    let sep = if cfg!(windows) { ";" } else { ":" };
    let classpath = prepared
        .classpath
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(sep);

    let mut env = env.clone();
    env.features
        .insert("has_custom_resolution".into(), opts.resolution.is_some());

    let vars = variables(prepared, opts, &classpath, sep);
    let sub = |s: &str| substitute(s, &vars);

    let mut args: Vec<String> = Vec::new();

    // Память и параметры от нас — до параметров версии.
    args.push(format!("-Xmx{}M", opts.memory_mb));
    args.push(format!("-Xms{}M", opts.memory_mb.min(1024)));
    args.extend(opts.extra_jvm_args.iter().cloned());

    match v.arguments.as_ref().filter(|a| !a.jvm.is_empty()) {
        Some(a) => {
            for arg in &a.jvm {
                args.extend(arg.values(&env).into_iter().map(&sub));
            }
        }
        None => {
            // Старые версии (до 1.13) не описывают JVM-аргументы — стандартный набор.
            if env.os == "windows" {
                args.push("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".into());
            }
            args.push(sub("-Djava.library.path=${natives_directory}"));
            args.push(sub("-Dminecraft.launcher.brand=${launcher_name}"));
            args.push(sub("-Dminecraft.launcher.version=${launcher_version}"));
            args.push("-cp".into());
            args.push(classpath.clone());
        }
    }
    if let Some(logging) = &prepared.logging_arg {
        args.push(logging.clone());
    }

    args.push(main_class);

    if let Some(a) = v.arguments.as_ref().filter(|a| !a.game.is_empty()) {
        for arg in &a.game {
            args.extend(arg.values(&env).into_iter().map(&sub));
        }
    } else if let Some(legacy) = &v.minecraft_arguments {
        args.extend(legacy.split_whitespace().map(&sub));
        if let Some((w, h)) = opts.resolution {
            args.extend([
                "--width".into(),
                w.to_string(),
                "--height".into(),
                h.to_string(),
            ]);
        }
    }

    Ok(LaunchCommand {
        java: opts
            .java_override
            .clone()
            .unwrap_or_else(|| prepared.java.clone()),
        args,
        game_dir: opts.game_dir.clone(),
    })
}

fn variables(
    p: &Prepared,
    o: &LaunchOptions,
    classpath: &str,
    sep: &str,
) -> HashMap<&'static str, String> {
    let s = |p: &Path| p.to_string_lossy().into_owned();
    let uuid = o.player.uuid.simple().to_string();
    let (w, h) = o.resolution.unwrap_or((854, 480));
    HashMap::from([
        ("auth_player_name", o.player.name.clone()),
        ("auth_uuid", uuid.clone()),
        ("auth_access_token", o.player.access_token.clone()),
        (
            "auth_session",
            format!("token:{}:{}", o.player.access_token, uuid),
        ),
        ("auth_xuid", "0".into()),
        ("clientid", "0".into()),
        ("user_type", o.player.user_type.clone()),
        ("user_properties", "{}".into()),
        ("version_name", p.version.id.clone()),
        (
            "version_type",
            p.version.kind.clone().unwrap_or_else(|| "release".into()),
        ),
        ("game_directory", s(&o.game_dir)),
        ("assets_root", s(&p.assets_root)),
        ("game_assets", s(&p.game_assets)),
        ("assets_index_name", p.asset_index_id.clone()),
        ("natives_directory", s(&p.natives_dir)),
        ("library_directory", s(&p.libraries_dir)),
        ("classpath", classpath.to_string()),
        ("classpath_separator", sep.to_string()),
        ("launcher_name", LAUNCHER_NAME.into()),
        ("launcher_version", env!("CARGO_PKG_VERSION").into()),
        ("resolution_width", w.to_string()),
        ("resolution_height", h.to_string()),
    ])
}

/// Подставить ${имя}. Неизвестные переменные оставляем как есть.
fn substitute(s: &str, vars: &HashMap<&'static str, String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let key = &after[..end];
                match vars.get(key) {
                    Some(v) => out.push_str(v),
                    None => out.push_str(&rest[start..start + 3 + end]),
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Запустить игру. Вывод игры пишется прямо в файл: если лаунчер закроют,
/// игра продолжит работать (с каналом-трубой она упала бы при записи в лог).
pub fn spawn(cmd: &LaunchCommand, log_file: &Path) -> Result<Child> {
    std::fs::create_dir_all(&cmd.game_dir).at(&cmd.game_dir)?;
    if let Some(dir) = log_file.parent() {
        std::fs::create_dir_all(dir).at(dir)?;
    }
    let log = std::fs::File::create(log_file).at(log_file)?;
    let log_err = log.try_clone().at(log_file)?;

    let mut command = Command::new(&cmd.java);
    command
        .args(&cmd.args)
        .current_dir(&cmd.game_dir)
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(log_err);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .spawn()
        .map_err(|e| Error::Launch(format!("{}: {e}", cmd.java.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitution() {
        let vars = HashMap::from([("a", "1".to_string()), ("bb", "two".to_string())]);
        assert_eq!(substitute("x=${a},y=${bb}", &vars), "x=1,y=two");
        assert_eq!(substitute("${unknown}-${a}", &vars), "${unknown}-1");
        assert_eq!(substitute("plain", &vars), "plain");
        assert_eq!(substitute("broken ${a", &vars), "broken ${a");
    }
}
