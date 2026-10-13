// Модели данных интерфейса. Совпадают с моделями из дизайна (WPF-прототипа).

export type PartyMemberState = "connecting" | "preparing" | "ready";
export type FriendPresence = "offline" | "online" | "in_game";
export type SocialRequestKind = "friend" | "party_invite";

/** Участник текущей Party. */
export interface PartyMember {
  id: string;
  name: string;
  state: PartyMemberState;
  isLocal: boolean;
  isLeader: boolean;
}

/** Друг. `activity` — название сборки, если друг в игре. */
export interface Friend {
  id: string;
  name: string;
  presence: FriendPresence;
  activity: string | null;
}

/** Входящая заявка в друзья или приглашение в Party. */
export interface SocialRequest {
  id: string;
  kind: SocialRequestKind;
  fromId: string;
  fromName: string;
}

/** Игровой профиль (сборка). */
export interface GameProfile {
  id: string;
  name: string;
  version: string;
  loader: string;
}
