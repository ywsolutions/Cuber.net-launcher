// Точка подключения сервисов. Друзья и Party пока на заглушках (этап 5).

import { inTauri } from "../api";
import { MockFriendsService, MockGameLauncher, MockPartyService, MockProfileService } from "./mock";
import { TauriGameLauncher, TauriProfileService } from "./real";
import { NoFriendsService, NoPartyService } from "./social-stub";
import type { FriendsService, GameLauncher, PartyService, ProfileService } from "./types";

export interface Services {
  profiles: ProfileService;
  launcher: GameLauncher;
  friends: FriendsService;
  party: PartyService;
}

export function createServices(localName: () => string): Services {
  return {
    // Профили и запуск — настоящие (в браузере без Tauri — заглушки для вёрстки).
    profiles: inTauri ? new TauriProfileService() : new MockProfileService(),
    launcher: inTauri ? new TauriGameLauncher() : new MockGameLauncher(),
    // Друзья и Party: в лаунчере пусто до этапа 5; в браузере — демо-данные для вёрстки.
    friends: inTauri ? new NoFriendsService() : new MockFriendsService(),
    party: inTauri ? new NoPartyService() : new MockPartyService(localName),
  };
}
