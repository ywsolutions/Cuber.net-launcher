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
  create(name: string, version: string): Promise<GameProfile>;
  /** Версии Minecraft (только релизы, если не просят снапшоты). */
  versions(snapshots?: boolean): Promise<{ latest: string; versions: string[] }>;
  openFolder(profile: GameProfile): Promise<void>;
}

export interface LaunchProgress {
  /** 0..1 */
  fraction: number;
  /** Короткий текст для кнопки: «Загрузка 45%». */
  label: string;
}

export interface GameLauncher {
  /** Готовит сборку (докачивает недостающее) и запускает игру.
   *  Завершается, когда процесс игры стартовал. */
  launch(profile: GameProfile, onProgress: (p: LaunchProgress) => void): Promise<void>;
  /** Игра профиля закрылась. */
  onExit(handler: (profileId: string, crashed: boolean) => void): Unsubscribe;
}
