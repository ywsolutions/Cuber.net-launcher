// Друзья и Party до их настоящей реализации (этап 5): пустые списки,
// никаких выдуманных людей. Попытка добавить друга объясняет, что функция в пути.

import type { SocialRequest } from "../models";
import type { FriendsService, PartyService } from "./types";

export const SOCIAL_SOON = "Друзья и Party появятся в одном из ближайших обновлений";

const noop = () => () => {};

export class NoFriendsService implements FriendsService {
  async friends() {
    return [];
  }
  async requests() {
    return [];
  }
  async sendFriendRequest(): Promise<void> {
    throw new Error(SOCIAL_SOON);
  }
  async accept(_r: SocialRequest) {}
  async decline(_r: SocialRequest) {}
  on = noop;
}

export class NoPartyService implements PartyService {
  isInParty() {
    return false;
  }
  members() {
    return [];
  }
  async invite(): Promise<void> {
    throw new Error(SOCIAL_SOON);
  }
  async join() {}
  async leave() {}
  on = noop;
}
