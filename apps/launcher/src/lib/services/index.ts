// Точка подключения сервисов. Когда появятся реальные реализации
// (запуск игры — этап 1, друзья и Party — этап 5), они подменяются здесь.

import { MockFriendsService, MockGameLauncher, MockPartyService, MockProfileService } from "./mock";
import type { FriendsService, GameLauncher, PartyService, ProfileService } from "./types";

export interface Services {
  profiles: ProfileService;
  launcher: GameLauncher;
  friends: FriendsService;
  party: PartyService;
}

export function createServices(localName: () => string): Services {
  return {
    profiles: new MockProfileService(),
    launcher: new MockGameLauncher(),
    friends: new MockFriendsService(),
    party: new MockPartyService(localName),
  };
}
