//! cuber-cli — установка и запуск Minecraft из командной строки.
//!
//! Нужна для проверки ядра без интерфейса (в том числе в CI):
//!
//! ```text
//! cuber-cli versions
//! cuber-cli install 1.20.1
//! cuber-cli launch 1.20.1 --nick Steve --wait 90 --expect "Setting user"
//! ```

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use cuber_core::Paths;
use cuber_launch::mojang::rules::Env;
use cuber_launch::{launch, Installer, LaunchOptions, Player, Progress, Stage};

#[derive(Parser)]
#[command(
    name = "cuber-cli",
    version,
    about = "Cuber.net: установка и запуск Minecraft"
)]
struct Cli {
    /// Папка данных (по умолчанию — как у лаунчера, или CUBER_HOME).
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Список версий Minecraft.
    Versions {
        /// Показать и снапшоты.
        #[arg(long)]
        all: bool,
    },
    /// Скачать всё нужное для версии.
    Install { version: String },
    /// Установить (если нужно) и запустить версию.
    Launch {
        version: String,
        #[arg(long, default_value = "CuberTest")]
        nick: String,
        #[arg(long, default_value_t = 2048)]
        memory: u32,
        /// Подождать столько секунд, проверить игру и закрыть её.
        #[arg(long)]
        wait: Option<u64>,
        /// Строка, которая должна появиться в выводе игры (можно несколько).
        #[arg(long)]
        expect: Vec<String>,
        /// Считать успехом, если ожидаемые строки появились, даже если игра потом
        /// закрылась (для машин без видеокарты, где окно игры не создаётся).
        #[arg(long)]
        lenient: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("CUBER_LOG")
                .unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let paths = match cli.home {
        Some(home) => Paths::at(home),
        None => Paths::discover()?,
    };
    paths.ensure()?;
    let installer = Installer::new(paths.clone())?;

    match cli.command {
        Command::Versions { all } => {
            let manifest = installer.version_manifest().await?;
            println!("Последний релиз: {}", manifest.latest.release);
            for v in manifest
                .versions
                .iter()
                .filter(|v| all || v.kind == "release")
            {
                println!("{:<24} {}", v.id, v.kind);
            }
        }
        Command::Install { version } => {
            let game_dir = paths
                .profiles()
                .join(format!("cli-{version}"))
                .join("minecraft");
            let started = Instant::now();
            let prepared = installer
                .prepare(&version, &game_dir, printer())
                .await
                .with_context(|| format!("установка {version}"))?;
            println!(
                "Готово за {:.1} с: Java {} ({}), {} файлов в classpath",
                started.elapsed().as_secs_f64(),
                prepared.java_major,
                prepared.java.display(),
                prepared.classpath.len()
            );
        }
        Command::Launch {
            version,
            nick,
            memory,
            wait,
            expect,
            lenient,
        } => {
            if !cuber_launch::offline::is_valid_nickname(&nick) {
                bail!("недопустимый ник: {nick}");
            }
            let game_dir = paths
                .profiles()
                .join(format!("cli-{version}"))
                .join("minecraft");
            let prepared = installer.prepare(&version, &game_dir, printer()).await?;
            let opts = LaunchOptions {
                player: Player::offline(&nick),
                game_dir: game_dir.clone(),
                memory_mb: memory,
                java_override: None,
                extra_jvm_args: vec![],
                resolution: None,
            };
            let cmd = launch::build(&prepared, &opts, &Env::current())?;
            let log = paths.root().join("cli-logs").join(format!("{version}.log"));
            println!("Запуск: {} {}", cmd.java.display(), cmd.args.join(" "));
            let mut child = launch::spawn(&cmd, &log)?;
            println!("Игра запущена, pid {}, лог: {}", child.id(), log.display());

            let Some(wait) = wait else {
                let status = child.wait()?;
                println!("Игра завершилась: {status}");
                return Ok(());
            };

            let deadline = Instant::now() + Duration::from_secs(wait);
            let mut exited = None;
            while Instant::now() < deadline {
                if let Some(status) = child.try_wait()? {
                    exited = Some(status);
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            let output = std::fs::read_to_string(&log).unwrap_or_default();
            let missing: Vec<&String> = expect
                .iter()
                .filter(|e| !output.contains(e.as_str()))
                .collect();
            if exited.is_none() {
                let _ = child.kill();
                let _ = child.wait();
            }
            println!("──── последние строки вывода игры ────");
            let lines: Vec<&str> = output.lines().collect();
            for line in &lines[lines.len().saturating_sub(40)..] {
                println!("{line}");
            }
            println!("──────────────────────────────────────");
            let crashed = output.contains("---- Minecraft Crash Report ----")
                || output.contains("Exception in thread \"main\"");
            if lenient && missing.is_empty() && !expect.is_empty() {
                println!("ПРОВЕРКА ПРОЙДЕНА (мягкая): игра дошла до {expect:?}");
                return Ok(());
            }
            match (exited, missing.is_empty(), crashed) {
                (None, true, false) => println!("ПРОВЕРКА ПРОЙДЕНА: игра работала {wait} с"),
                (Some(status), true, false) => {
                    bail!("игра завершилась раньше времени: {status}")
                }
                (_, false, _) => bail!("в выводе игры нет ожидаемого: {missing:?}"),
                (_, _, true) => bail!("игра упала (см. вывод выше)"),
            }
        }
    }
    Ok(())
}

/// Печать прогресса не чаще раза в секунду.
fn printer() -> impl Fn(Progress) + Send + Sync + 'static {
    let last = Arc::new(Mutex::new((
        Instant::now() - Duration::from_secs(5),
        None::<Stage>,
    )));
    move |p: Progress| {
        let mut last = last.lock().unwrap();
        let stage_changed = last.1 != Some(p.stage);
        if !stage_changed && last.0.elapsed() < Duration::from_secs(1) {
            return;
        }
        *last = (Instant::now(), Some(p.stage));
        match p.stage {
            Stage::Downloading => println!(
                "  загрузка: {:.1} / {:.1} МБ ({:.0}%)",
                p.done as f64 / 1e6,
                p.total as f64 / 1e6,
                p.fraction() * 100.0
            ),
            other => println!("  {other:?}"),
        }
    }
}
