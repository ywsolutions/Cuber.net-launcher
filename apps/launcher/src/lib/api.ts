// Обёртка над командами ядра (Rust).
// Если интерфейс открыт в обычном браузере (`npm run dev` без Tauri),
// используются заглушки — так удобно верстать без пересборки Rust.

import { invoke, isTauri } from "@tauri-apps/api/core";

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

const inTauri = isTauri();

let mockConfig: Config = {
  theme: "system",
  after_launch: "minimize",
  default_memory_mb: 4096,
  last_nickname: null,
};

export async function appInfo(): Promise<AppInfo> {
  if (!inTauri) return { version: "dev", dataDir: "(браузер)", platform: "web" };
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

/** Применяет тему к документу. */
export function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", theme);
}
