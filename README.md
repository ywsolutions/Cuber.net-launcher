# Cuber.net

Лёгкий минималистичный лаунчер для Minecraft Java Edition: профили, моды с Modrinth,
друзья и Party, совместная игра через встроенную виртуальную сеть — без Hamachi,
рекламы и лишних фоновых процессов.

> Проект на раннем этапе разработки. Работает: установка и запуск Minecraft любой версии по нику,
> изолированные профили, интерфейс по дизайну, настройки. Друзья и Party пока на заглушках.

## Интерфейс

- Компактная плавающая панель внизу экрана: профиль, «Друзья», настройки, «Играть».
- Party-панель и выдвижные панели «Друзья» и «Настройки» появляются над ней и растут вверх;
  основная панель при этом не двигается. Открыта максимум одна выдвижная панель, Esc закрывает.
- Окно прозрачное, без рамки и всегда ровно по размеру содержимого — пустые области
  не перехватывают клики по рабочему столу.

## Стек

- **Ядро:** Rust, [Tauri 2](https://tauri.app)
- **Интерфейс:** Svelte 5 + TypeScript + Vite
- **Сеть (позже):** QUIC/P2P через [iroh](https://iroh.computer), резервный relay
- **Сервер (позже):** Rust (axum) + PostgreSQL

## Структура

```
apps/launcher/          приложение
├─ src/                 интерфейс (Svelte)
│  └─ lib/
│     ├─ panels/        основная панель, Party, друзья, настройки
│     ├─ state/         состояние интерфейса
│     ├─ services/      контракты данных и временные заглушки (моки)
│     └─ ui/            общие компоненты: иконки, аватары, анимация раскрытия
└─ src-tauri/           окно и команды Tauri (Rust)
crates/
├─ cuber-core/          пути данных, настройки, логирование
├─ cuber-launch/        установка и запуск Minecraft: версии, библиотеки, ассеты, Java, профили
└─ cuber-cli/           консольная утилита для проверки ядра без интерфейса
assets/                 исходники логотипа и иконок
```

Новые модули (`cuber-content`, `cuber-net` и др.) добавляются в `crates/`
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

### Проверка запуска без интерфейса

```bash
cargo run -p cuber-cli -- versions
cargo run -p cuber-cli -- launch 1.20.1 --nick Steve
```

В CI (`smoke.yml`) так запускается настоящий Minecraft нескольких версий на Linux
(в виртуальном дисплее), Windows и macOS; итог публикуется комментарием в PR.

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
