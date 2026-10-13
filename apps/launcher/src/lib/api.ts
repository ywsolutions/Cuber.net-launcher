// Обёртка над командами ядра (Rust) и окном.
// Если интерфейс открыт в обычном браузере (`npm run dev` без Tauri),
// используются заглушки — так удобно верстать без пересборки Rust.

import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Theme = "system" | "dark" | "light";
export type AfterLaunch = "keep_open" | "minimize" | "close";

export interface Config {
  theme: Theme;
  after_launch: AfterLaunch;
  default_memory_mb: number;
  last_nickname: string | null;
}

export interface AppInfo {
  version: string;
  dataDir: string;
  platform: string;
}

export const inTauri = isTauri();

let mockConfig: Config = {
  theme: "dark",
  after_launch: "minimize",
  default_memory_mb: 6144,
  last_nickname: null,
};

export async function appInfo(): Promise<AppInfo> {
  if (!inTauri) return { version: "dev", dataDir: "%APPDATA%\\Cuber", platform: "web" };
  return invoke<AppInfo>("app_info");
}

export async function getConfig(): Promise<Config> {
  if (!inTauri) return { ...mockConfig };
  return invoke<Config>("get_config");
}

export async function saveConfig(config: Config): Promise<Config> {
  if (!inTauri) {
    mockConfig = { ...config };
    return { ...mockConfig };
  }
  return invoke<Config>("save_config", { config });
}

export async function openDataDir(): Promise<void> {
  if (!inTauri) return;
  return invoke("open_data_dir");
}

// ── Окно ────────────────────────────────────────────────────────────

/** Подогнать высоту окна (нижний край на месте); `show` — показать окно, если скрыто. */
export async function fitWindow(height: number, show = false): Promise<void> {
  if (!inTauri) return;
  return invoke("fit_window", { height, show });
}

/** Вернуть окну рассчитанные размер и позицию (после первого показа). */
export async function restoreWindow(): Promise<void> {
  if (!inTauri) return;
  return invoke("restore_window");
}

export function minimizeWindow() {
  if (inTauri) void getCurrentWindow().minimize();
}

export function closeWindow() {
  if (inTauri) void getCurrentWindow().close();
}

export function startDragging() {
  if (inTauri) void getCurrentWindow().startDragging();
}
