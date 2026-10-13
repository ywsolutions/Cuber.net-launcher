# Cuber.net

Лёгкий минималистичный лаунчер для Minecraft Java Edition: профили, моды с Modrinth,
друзья и Party, совместная игра через встроенную виртуальную сеть — без Hamachi,
рекламы и лишних фоновых процессов.

> Проект на раннем этапе разработки. Сейчас готов каркас: окно, настройки, папки данных.

## Стек

- **Ядро:** Rust, [Tauri 2](https://tauri.app)
- **Интерфейс:** Svelte 5 + TypeScript + Vite
- **Сеть (позже):** QUIC/P2P через [iroh](https://iroh.computer), резервный relay
- **Сервер (позже):** Rust (axum) + PostgreSQL

## Структура

```
apps/launcher/          приложение
├─ src/                 интерфейс (Svelte)
└─ src-tauri/           окно и команды Tauri (Rust)
crates/
└─ cuber-core/          пути данных, настройки, логирование
assets/                 исходники логотипа и иконок
```

Новые модули (`cuber-launch`, `cuber-content`, `cuber-net` и др.) добавляются в `crates/`
по мере этапов.

## Разработка

Нужны: [Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) 22+,
и [системные зависимости Tauri](https://tauri.app/start/prerequisites/)
(на Windows — WebView2 и Build Tools for Visual Studio).

```bash
cd apps/launcher
npm install
npm run tauri dev      # запуск с горячей перезагрузкой
npm run tauri build    # релизная сборка и установщик
```

Только интерфейс в браузере, без Rust: `npm run dev` и откройте http://localhost:1420
(команды ядра заменяются заглушками).

Проверки перед коммитом:

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cd apps/launcher && npm run check
```

### Переменные окружения

| Переменная  | Назначение                                                 |
|-------------|------------------------------------------------------------|
| `CUBER_HOME`| Своя папка данных вместо системной (удобно при разработке) |
| `CUBER_LOG` | Уровень логов, синтаксис `RUST_LOG`, например `debug`      |

### Где лежат данные

- Windows: `%APPDATA%\Cuber\Cuber\data`
- Linux: `~/.local/share/cuber`
- macOS: `~/Library/Application Support/net.Cuber.Cuber`

### Иконки

Исходник — `assets/logo.svg`. Пересоздать иконки:

```bash
cd apps/launcher && npx tauri icon ../../assets/logo.svg -o src-tauri/icons
```

## Лицензия

[GPL-3.0](LICENSE)
