//! Установка и запуск Minecraft Java Edition.
//!
//! - [`install::Installer`] — скачивает версию, библиотеки, ассеты и Java
//!   с официальных серверов Mojang, с докачкой и проверкой SHA-1;
//! - [`launch`] — собирает командную строку и запускает игру;
//! - [`profile`] — изолированные игровые профили;
//! - [`offline`] — игрок по нику (без лицензии).

pub mod error;
pub mod http;
pub mod install;
pub mod java;
pub mod launch;
pub mod mojang;
pub mod offline;
pub mod profile;
pub mod progress;

pub use error::{Error, Result};
pub use install::{Endpoints, Installer, Prepared};
pub use launch::{LaunchCommand, LaunchOptions};
pub use offline::Player;
pub use profile::{Loader, Profile, ProfileStore};
pub use progress::{Progress, Stage};
