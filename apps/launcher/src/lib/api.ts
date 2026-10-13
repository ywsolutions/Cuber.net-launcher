// Обёртка над командами ядра (Rust) и окном.
// Если интерфейс открыт в обычном браузере (`npm run dev` без Tauri),
// используются заглушки — так удобно верстать без пересборки Rust.

import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Theme = "system" | "dark" | "light";
export type AfterLaunch = "keep_open" | "minimize" | "close";

export interface Config {
  theme: Theme;
  after_launch: AfterLaunch;
  default_memory_mb: number;
  last_nickname: string | null;
  onboarded: boolean;
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
  onboarded: false,
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

// ── Профили и запуск ────────────────────────────────────────────────

export interface ProfileDto {
  id: string;
  name: string;
  version: string;
  loader: string;
}

export interface VersionsDto {
  latest: string;
  versions: { id: string; kind: string }[];
}

export type LaunchStage = "resolving" | "java" | "checking" | "downloading" | "unpacking" | "launching";

export interface LaunchProgressEvent {
  profileId: string;
  stage: LaunchStage;
  fraction: number;
  done: number;
  total: number;
}

export interface GameExitedEvent {
  profileId: string;
  code: number | null;
  crashed: boolean;
}

export const listProfiles = () => invoke<ProfileDto[]>("list_profiles");
export const createProfile = (name: string, version: string) =>
  invoke<ProfileDto>("create_profile", { profile: { name, version } });
export const listVersions = (snapshots = false) =>
  invoke<VersionsDto>("list_versions", { snapshots });
export const launchProfile = (id: string) => invoke<void>("launch_profile", { id });
export const openProfileDir = (id: string) => invoke<void>("open_profile_dir", { id });

export function onEvent<T>(name: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(name, (e) => handler(e.payload));
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

// ── Самопроверка интерфейса (CI) ────────────────────────────────────

export const selftestEnabled = () =>
  inTauri ? invoke<boolean>("selftest_enabled") : Promise.resolve(false);
export const selftestLog = (step: string) => invoke<void>("selftest_log", { step });
