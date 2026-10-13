// Реальные сервисы поверх команд ядра (Rust).

import {
  createProfile,
  launchProfile,
  listProfiles,
  listVersions,
  onEvent,
  openProfileDir,
  type GameExitedEvent,
  type LaunchProgressEvent,
} from "../api";
import type { GameProfile } from "../models";
import type { GameLauncher, LaunchProgress, ProfileService, Unsubscribe } from "./types";

const STAGE_LABEL: Record<LaunchProgressEvent["stage"], string> = {
  resolving: "Подготовка",
  java: "Java",
  checking: "Проверка",
  downloading: "Загрузка",
  unpacking: "Распаковка",
  launching: "Запуск",
};

export class TauriProfileService implements ProfileService {
  profiles() {
    return listProfiles();
  }

  create(name: string, version: string) {
    return createProfile(name, version);
  }

  async versions(snapshots = false) {
    const v = await listVersions(snapshots);
    return { latest: v.latest, versions: v.versions.map((x) => x.id) };
  }

  openFolder(profile: GameProfile) {
    return openProfileDir(profile.id);
  }
}

export class TauriGameLauncher implements GameLauncher {
  async launch(profile: GameProfile, onProgress: (p: LaunchProgress) => void) {
    const unlisten = await onEvent<LaunchProgressEvent>("launch-progress", (e) => {
      if (e.profileId !== profile.id) return;
      const label =
        e.stage === "downloading"
          ? `Загрузка ${Math.floor(e.fraction * 100)}%`
          : STAGE_LABEL[e.stage];
      onProgress({ fraction: e.fraction, label });
    });
    try {
      await launchProfile(profile.id);
    } finally {
      unlisten();
    }
  }

  onExit(handler: (profileId: string, crashed: boolean) => void): Unsubscribe {
    const p = onEvent<GameExitedEvent>("game-exited", (e) => handler(e.profileId, e.crashed));
    return () => void p.then((u) => u());
  }
}
