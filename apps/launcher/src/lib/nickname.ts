// Ник в Minecraft: 3–16 символов, латиница, цифры и подчёркивание.
// Такие же правила проверяет ванильный сервер.
const NICK_RE = /^[A-Za-z0-9_]{3,16}$/;

export const DEFAULT_NICKNAME = "Player";

export function isValidNickname(nick: string): boolean {
  return NICK_RE.test(nick);
}

export function nicknameError(nick: string): string | null {
  if (nick.length === 0) return null;
  if (nick.length < 3) return "Минимум 3 символа";
  if (nick.length > 16) return "Максимум 16 символов";
  if (!NICK_RE.test(nick)) return "Только латиница, цифры и _";
  return null;
}
