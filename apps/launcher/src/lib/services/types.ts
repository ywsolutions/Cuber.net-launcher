// Контракты, через которые интерфейс получает данные.
// Сейчас за ними стоят моки (mock.ts). На следующих этапах их заменят реальные
// реализации поверх команд Rust — без изменений в компонентах и состоянии.

import type { Friend, GameProfile, PartyMember, SocialRequest } from "../models";

export type Unsubscribe = () => void;

export interface PartyEvents {
  /** Party создана, покинута или полностью сменилась — перечитать `members()`. */
  changed: () => void;
  /** Участник добавлен или изменил статус. */
  memberUpdated: (member: PartyMember) => void;
  /** Участник покинул Party (аргумент — id). */
  memberLeft: (id: string) => void;
}

export interface PartyService {
  isInParty(): boolean;
  members(): PartyMember[];
  invite(friendId: string, friendName: string): Promise<void>;
  join(invite: SocialRequest): Promise<void>;
  leave(): Promise<void>;
  on<K extends keyof PartyEvents>(event: K, handler: PartyEvents[K]): Unsubscribe;
}

export interface FriendsEvents {
  friendUpdated: (friend: Friend) => void;
  requestReceived: (request: SocialRequest) => void;
  /** Заявка обработана (аргумент — id заявки). */
  requestRemoved: (id: string) => void;
}

export interface FriendsService {
  friends(): Promise<Friend[]>;
  requests(): Promise<SocialRequest[]>;
  sendFriendRequest(playerName: string): Promise<void>;
  accept(request: SocialRequest): Promise<void>;
  decline(request: SocialRequest): Promise<void>;
  on<K extends keyof FriendsEvents>(event: K, handler: FriendsEvents[K]): Unsubscribe;
}

export interface ProfileService {
  profiles(): Promise<GameProfile[]>;
}

export interface GameLauncher {
  /** Готовит сборку и запускает игру. Прогресс — от 0 до 1. */
  launch(
    profile: GameProfile,
    memoryGb: number,
    onProgress: (progress: number) => void,
  ): Promise<void>;
}
