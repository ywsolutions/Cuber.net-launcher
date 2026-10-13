import { appInfo, saveConfig, getConfig, type AppInfo, type Config } from "../api";
import type { GameProfile } from "../models";
import { DEFAULT_NICKNAME } from "../nickname";
import { createServices, type Services } from "../services";
import { FriendsState } from "./friends.svelte";
import { PartyState } from "./party.svelte";

/** Выдвижная панель над основной. Одновременно открыта максимум одна. */
export type Panel = "none" | "friends" | "settings";

const PLAY_IDLE = "Играть";
export const MEMORY_MIN_GB = 2;
export const MEMORY_MAX_GB = 16;

export class LauncherState {
  config = $state<Config | null>(null);
  info = $state<AppInfo | null>(null);
  error = $state<string | null>(null);

  profiles = $state<GameProfile[]>([]);
  selectedProfile = $state<GameProfile | null>(null);
  profileMenuOpen = $state(false);
  panel = $state<Panel>("none");

  launching = $state(false);
  launchProgress = $state(0);
  playText = $state(PLAY_IDLE);

  nickname = $derived(this.config?.last_nickname || DEFAULT_NICKNAME);
  memoryGb = $derived(
    Math.min(
      MEMORY_MAX_GB,
      Math.max(MEMORY_MIN_GB, Math.round((this.config?.default_memory_mb ?? 6144) / 1024)),
    ),
  );
  minimizeOnLaunch = $derived(this.config?.after_launch !== "keep_open");

  readonly services: Services;
  readonly party: PartyState;
  readonly friends: FriendsState;

  constructor() {
    this.services = createServices(() => this.nickname);
    this.party = new PartyState(this.services.party);
    this.friends = new FriendsState(this.services.friends, this.services.party, this.party);
  }

  async init() {
    try {
      const [config, info, profiles] = await Promise.all([
        getConfig(),
        appInfo(),
        this.services.profiles.profiles(),
      ]);
      this.config = config;
      this.info = info;
      this.profiles = profiles;
      this.selectedProfile ??= profiles[0] ?? null;
      this.party.renameLocal(this.nickname);
      await this.friends.load();
    } catch (e) {
      this.error = String(e);
    }
  }

  togglePanel(panel: Exclude<Panel, "none">) {
    this.panel = this.panel === panel ? "none" : panel;
    this.profileMenuOpen = false;
  }

  closePanels() {
    this.panel = "none";
    this.profileMenuOpen = false;
  }

  selectProfile(profile: GameProfile) {
    this.selectedProfile = profile;
    this.profileMenuOpen = false;
  }

  async updateConfig(patch: Partial<Config>) {
    if (!this.config) return;
    const previous = this.config;
    this.config = { ...previous, ...patch };
    try {
      this.config = await saveConfig(this.config);
    } catch (e) {
      this.config = previous;
      throw e;
    }
  }

  setMemoryGb(gb: number) {
    return this.updateConfig({ default_memory_mb: gb * 1024 });
  }

  setMinimizeOnLaunch(on: boolean) {
    return this.updateConfig({ after_launch: on ? "minimize" : "keep_open" });
  }

  async setNickname(name: string) {
    await this.updateConfig({ last_nickname: name });
    this.party.renameLocal(name);
  }

  async play() {
    const profile = this.selectedProfile;
    if (!profile || this.launching) return;

    this.launching = true;
    this.profileMenuOpen = false;
    this.playText = "Запуск 0%";
    try {
      await this.services.launcher.launch(profile, this.memoryGb, (p) => {
        this.launchProgress = p;
        this.playText = `Запуск ${Math.round(p * 100)}%`;
      });
      this.playText = "В игре";
      await new Promise((r) => setTimeout(r, 2500));
    } catch {
      this.playText = "Ошибка";
      await new Promise((r) => setTimeout(r, 2000));
    } finally {
      this.launching = false;
      this.launchProgress = 0;
      this.playText = PLAY_IDLE;
    }
  }
}
