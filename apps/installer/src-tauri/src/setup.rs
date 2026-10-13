//! Установка и удаление на Windows. На других системах — понятная ошибка
//! (там лаунчер распространяется своими средствами).

use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "Cuber";
#[cfg_attr(not(windows), allow(dead_code))]
pub const EXE_NAME: &str = "Cuber.exe";
#[cfg_attr(not(windows), allow(dead_code))]
pub const UNINSTALLER_NAME: &str = "Uninstall Cuber.exe";
#[cfg(windows)]
const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Cuber";

/// Запущен как деинсталлятор: по имени файла или флагу --uninstall.
pub fn is_uninstall_mode() -> bool {
    std::env::args().any(|a| a == "--uninstall")
        || std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
            .is_some_and(|n| n.starts_with("uninstall"))
}

/// %LOCALAPPDATA%\Programs\Cuber
pub fn default_dir() -> PathBuf {
    let base = directories::BaseDirs::new()
        .map(|d| d.data_local_dir().to_path_buf())
        .unwrap_or_else(std::env::temp_dir);
    base.join("Programs").join(APP_NAME)
}

/// Папка установленного Cuber из записи в реестре.
pub fn installed_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        let key = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(UNINSTALL_KEY)
            .ok()?;
        let dir: String = key.get_value("InstallLocation").ok()?;
        let dir = PathBuf::from(dir);
        dir.join(EXE_NAME).exists().then_some(dir)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(not(windows))]
pub fn install(_dir: &Path, _payload: &[u8], _desktop: bool) -> Result<(), String> {
    Err("Установщик работает только в Windows".into())
}

#[cfg(not(windows))]
pub fn uninstall(_remove_data: bool) -> Result<(), String> {
    Err("Установщик работает только в Windows".into())
}

#[cfg(not(windows))]
pub fn launch(_dir: &Path) -> Result<(), String> {
    Err("Установщик работает только в Windows".into())
}

#[cfg(windows)]
pub use win::{install, launch, uninstall};

#[cfg(windows)]
mod win {
    use super::*;
    use std::os::windows::process::CommandExt;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS};
    use winreg::RegKey;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const DETACHED_PROCESS: u32 = 0x0000_0008;

    fn err(what: &str, e: impl std::fmt::Display) -> String {
        format!("{what}: {e}")
    }

    fn start_menu_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| {
            d.config_dir()
                .join(r"Microsoft\Windows\Start Menu\Programs")
        })
    }

    fn desktop_dir() -> Option<PathBuf> {
        directories::UserDirs::new().and_then(|u| u.desktop_dir().map(Path::to_path_buf))
    }

    fn shortcut(target: &Path, link: &Path) -> Result<(), String> {
        if let Some(parent) = link.parent() {
            std::fs::create_dir_all(parent).map_err(|e| err("папка ярлыков", e))?;
        }
        let mut lnk = mslnk::ShellLink::new(target).map_err(|e| err("ярлык", e))?;
        lnk.set_working_dir(target.parent().map(|p| p.display().to_string()));
        lnk.set_icon_location(Some(target.display().to_string()));
        lnk.create_lnk(link).map_err(|e| err("ярлык", e))
    }

    pub fn install(dir: &Path, payload: &[u8], desktop: bool) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| err("не удалось создать папку", e))?;

        // Лаунчер: пишем рядом и подменяем — так при ошибке старая версия не пострадает.
        let exe = dir.join(EXE_NAME);
        let tmp = dir.join("Cuber.exe.new");
        std::fs::write(&tmp, payload).map_err(|e| err("не удалось записать лаунчер", e))?;
        if let Err(e) = std::fs::rename(&tmp, &exe) {
            let _ = std::fs::remove_file(&tmp);
            return Err(if e.kind() == std::io::ErrorKind::PermissionDenied {
                "Cuber сейчас запущен — закройте его и нажмите «Установить» ещё раз".into()
            } else {
                err("не удалось заменить лаунчер", e)
            });
        }

        // Деинсталлятор — копия этого же установщика.
        let me = std::env::current_exe().map_err(|e| err("путь установщика", e))?;
        let uninstaller = dir.join(UNINSTALLER_NAME);
        if me != uninstaller {
            std::fs::copy(&me, &uninstaller).map_err(|e| err("деинсталлятор", e))?;
        }

        // Ярлыки.
        if let Some(menu) = start_menu_dir() {
            shortcut(&exe, &menu.join("Cuber.lnk"))?;
        }
        if let Some(desktop_dir) = desktop_dir() {
            let link = desktop_dir.join("Cuber.lnk");
            if desktop {
                shortcut(&exe, &link)?;
            } else {
                let _ = std::fs::remove_file(link);
            }
        }

        // «Приложения и возможности».
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(UNINSTALL_KEY)
            .map_err(|e| err("реестр", e))?;
        let size_kb = (payload.len() as u64
            + std::fs::metadata(&uninstaller)
                .map(|m| m.len())
                .unwrap_or(0))
            / 1024;
        let set =
            |name: &str, value: &str| key.set_value(name, &value).map_err(|e| err("реестр", e));
        set("DisplayName", "Cuber")?;
        set("DisplayVersion", env!("CARGO_PKG_VERSION"))?;
        set("Publisher", "YOLO")?;
        set("DisplayIcon", &exe.display().to_string())?;
        set("InstallLocation", &dir.display().to_string())?;
        set(
            "UninstallString",
            &format!("\"{}\" --uninstall", uninstaller.display()),
        )?;
        set(
            "URLInfoAbout",
            "https://github.com/ywsolutions/Cuber.net-launcher",
        )?;
        key.set_value("EstimatedSize", &(size_kb as u32))
            .map_err(|e| err("реестр", e))?;
        key.set_value("NoModify", &1u32)
            .map_err(|e| err("реестр", e))?;
        key.set_value("NoRepair", &1u32)
            .map_err(|e| err("реестр", e))?;
        Ok(())
    }

    pub fn uninstall(remove_data: bool) -> Result<(), String> {
        let dir = installed_dir()
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(Path::to_path_buf))
            })
            .ok_or("не найдена папка Cuber")?;

        // Сам лаунчер (если запущен — просим закрыть).
        let exe = dir.join(EXE_NAME);
        if exe.exists() {
            std::fs::remove_file(&exe).map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    "Cuber сейчас запущен — закройте его и попробуйте снова".to_string()
                } else {
                    err("не удалось удалить лаунчер", e)
                }
            })?;
        }

        // Ярлыки и запись в реестре.
        if let Some(menu) = start_menu_dir() {
            let _ = std::fs::remove_file(menu.join("Cuber.lnk"));
        }
        if let Some(desktop) = desktop_dir() {
            let _ = std::fs::remove_file(desktop.join("Cuber.lnk"));
        }
        if let Ok(uninstall) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            KEY_ALL_ACCESS,
        ) {
            let _ = uninstall.delete_subkey_all("Cuber");
        }

        // Игровые данные — только по явной просьбе.
        if remove_data {
            // Данные лаунчера: %APPDATA%\Cuber (миры, профили, настройки, Java).
            if let Some(base) = directories::BaseDirs::new() {
                let _ = std::fs::remove_dir_all(base.config_dir().join(APP_NAME));
            }
        }

        // Свою папку (с этим exe внутри) удалит cmd после нашего выхода.
        let script = format!(
            "ping -n 3 127.0.0.1 >NUL & rmdir /S /Q \"{}\"",
            dir.display()
        );
        std::process::Command::new("cmd")
            .args(["/C", &script])
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .spawn()
            .map_err(|e| err("очистка папки", e))?;
        Ok(())
    }

    pub fn launch(dir: &Path) -> Result<(), String> {
        std::process::Command::new(dir.join(EXE_NAME))
            .current_dir(dir)
            .creation_flags(DETACHED_PROCESS)
            .spawn()
            .map(|_| ())
            .map_err(|e| err("не удалось запустить Cuber", e))
    }
}
