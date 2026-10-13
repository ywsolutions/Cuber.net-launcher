// Временные реализации сервисов с имитацией статусов (порт моков из дизайна).
// Заменяются реальными в services/index.ts.

import type { Friend, GameProfile, PartyMember, PartyMemberState, SocialRequest } from "../models";
import { Emitter } from "./emitter";
import type {
  FriendsEvents,
  FriendsService,
  GameLauncher,
  PartyEvents,
  PartyService,
  ProfileService,
} from "./types";

const LOCAL_ID = "local";
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const rand = (min: number, max: number) => min + Math.floor(Math.random() * (max - min));

const PROFILES: GameProfile[] = [
  { id: "p-survival", name: "Cuber Survival", version: "1.20.1", loader: "Fabric" },
  { id: "p-creative", name: "Cuber Creative", version: "1.21", loader: "Vanilla" },
  { id: "p-hardcore", name: "Hardcore SMP", version: "1.19.4", loader: "Forge" },
];

const FRIENDS: Friend[] = [
  { id: "f-kotik", name: "Kotik_Craft", presence: "in_game", activity: "Cuber Survival" },
  { id: "f-redstone", name: "Redstone_Ru", presence: "online", activity: null },
  { id: "f-dima", name: "Diamond_Dima", presence: "in_game", activity: "Hardcore SMP" },
  { id: "f-lina", name: "EnderLina", presence: "online", activity: null },
  { id: "f-max", name: "MaxBuilder", presence: "online", activity: null },
  { id: "f-owl", name: "NightOwl", presence: "online", activity: null },
  { id: "f-golem", name: "SnowGolem", presence: "in_game", activity: "Cuber Creative" },
  { id: "f-masha", name: "IronMasha", presence: "online", activity: null },
  { id: "f-kid", name: "Creeper_Kid", presence: "online", activity: null },
  { id: "f-bob", name: "BlockyBob", presence: "offline", activity: null },
  { id: "f-aqua", name: "AquaSteve", presence: "offline", activity: null },
];

export class MockProfileService implements ProfileService {
  private list = PROFILES.map((p) => ({ ...p }));

  async profiles() {
    return this.list.map((p) => ({ ...p }));
  }

  async create(name: string, version: string) {
    const p = { id: `p-${Date.now()}`, name, version, loader: "Vanilla" };
    this.list.unshift(p);
    return { ...p };
  }

  async versions() {
    return { latest: "1.21.1", versions: ["1.21.1", "1.21", "1.20.6", "1.20.4", "1.20.1", "1.19.4", "1.18.2", "1.16.5", "1.12.2", "1.8.9", "1.7.10"] };
  }

  async openFolder() {}
}

/** Имитирует подготовку сборки и запуск (~3 секунды), игра «закрывается» через 5 с. */
export class MockGameLauncher implements GameLauncher {
  private exitHandlers = new Set<(id: string, crashed: boolean) => void>();

  async launch(profile: GameProfile, onProgress: (p: { fraction: number; label: string }) => void) {
    const steps = 80;
    for (let i = 1; i <= steps; i++) {
      await sleep(40);
      onProgress({ fraction: i / steps, label: `Загрузка ${Math.floor((i / steps) * 100)}%` });
    }
    setTimeout(() => this.exitHandlers.forEach((h) => h(profile.id, false)), 5000);
  }

  onExit(handler: (id: string, crashed: boolean) => void) {
    this.exitHandlers.add(handler);
    return () => this.exitHandlers.delete(handler);
  }
}

/** Друзья: статусы периодически меняются, отправленная заявка «принимается» через пару секунд. */
export class MockFriendsService implements FriendsService {
  private list: Friend[] = FRIENDS.map((f) => ({ ...f }));
  private reqs: SocialRequest[] = [
    { id: "r-pixel", kind: "friend", fromId: "f-pixel", fromName: "Pixel_Fox" },
    { id: "r-dima", kind: "party_invite", fromId: "f-dima", fromName: "Diamond_Dima" },
  ];
  private events = new Emitter<FriendsEvents>();

  constructor() {
    setTimeout(() => setInterval(() => this.shufflePresence(), 9000), 8000);
    setTimeout(() => {
      const request: SocialRequest = { id: "r-ivy", kind: "friend", fromId: "f-ivy", fromName: "Sunny_Ivy" };
      this.reqs.push(request);
      this.events.emit("requestReceived", request);
    }, 15000);
  }

  on<K extends keyof FriendsEvents>(event: K, handler: FriendsEvents[K]) {
    return this.events.on(event, handler);
  }

  async friends() {
    return this.list.map((f) => ({ ...f }));
  }

  async requests() {
    return this.reqs.map((r) => ({ ...r }));
  }

  async sendFriendRequest(playerName: string) {
    setTimeout(() => {
      this.addFriend({
        id: "f-" + playerName.toLowerCase(),
        name: playerName,
        presence: "online",
        activity: null,
      });
    }, 2500);
  }

  async accept(request: SocialRequest) {
    this.removeRequest(request.id);
    if (request.kind === "friend") {
      this.addFriend({ id: request.fromId, name: request.fromName, presence: "online", activity: null });
    }
  }

  async decline(request: SocialRequest) {
    this.removeRequest(request.id);
  }

  private addFriend(friend: Friend) {
    if (this.list.some((f) => f.id === friend.id)) return;
    this.list.push(friend);
    this.events.emit("friendUpdated", { ...friend });
  }

  private removeRequest(id: string) {
    const before = this.reqs.length;
    this.reqs = this.reqs.filter((r) => r.id !== id);
    if (this.reqs.length !== before) this.events.emit("requestRemoved", id);
  }

  private shufflePresence() {
    const i = rand(0, this.list.length);
    const f = this.list[i];
    let next: Friend;
    if (f.presence === "offline") next = { ...f, presence: "online", activity: null };
    else if (f.presence === "online")
      next =
        Math.random() < 0.5
          ? { ...f, presence: "in_game", activity: PROFILES[rand(0, PROFILES.length)].name }
          : { ...f, presence: "offline", activity: null };
    else next = { ...f, presence: "online", activity: null };
    this.list[i] = next;
    this.events.emit("friendUpdated", { ...next });
  }
}

/** Party: приглашённые проходят «Подключается» → «Подготавливает сборку» → «Готов к игре». */
export class MockPartyService implements PartyService {
  private list: PartyMember[] = [];
  private generation = 0;
  private events = new Emitter<PartyEvents>();

  constructor(private localName: () => string) {
    // Стартуем уже в Party, чтобы сразу было видно динамическую панель.
    this.list = [
      { id: LOCAL_ID, name: localName(), state: "ready", isLocal: true, isLeader: true },
      { id: "f-kotik", name: "Kotik_Craft", state: "ready", isLocal: false, isLeader: false },
      { id: "f-redstone", name: "Redstone_Ru", state: "preparing", isLocal: false, isLeader: false },
    ];
    void this.simulateJoin("f-redstone", this.generation, true);
  }

  on<K extends keyof PartyEvents>(event: K, handler: PartyEvents[K]) {
    return this.events.on(event, handler);
  }

  isInParty() {
    return this.list.length > 0;
  }

  members() {
    return this.list.map((m) => (m.isLocal ? { ...m, name: this.localName() } : { ...m }));
  }

  async invite(friendId: string, friendName: string) {
    let created = false;
    if (this.list.length === 0) {
      this.generation++;
      this.list.push({ id: LOCAL_ID, name: this.localName(), state: "ready", isLocal: true, isLeader: true });
      created = true;
    }
    if (this.list.some((m) => m.id === friendId)) return;

    const invited: PartyMember = {
      id: friendId,
      name: friendName,
      state: "connecting",
      isLocal: false,
      isLeader: false,
    };
    this.list.push(invited);
    if (created) this.events.emit("changed");
    this.events.emit("memberUpdated", { ...invited });
    void this.simulateJoin(friendId, this.generation, false);
  }

  async join(invite: SocialRequest) {
    this.generation++;
    const gen = this.generation;
    this.list = [
      { id: invite.fromId, name: invite.fromName, state: "ready", isLocal: false, isLeader: true },
      { id: "x-ivy", name: "Sunny_Ivy", state: "ready", isLocal: false, isLeader: false },
      { id: "x-torch", name: "Mr_Torch", state: "preparing", isLocal: false, isLeader: false },
      { id: LOCAL_ID, name: this.localName(), state: "connecting", isLocal: true, isLeader: false },
    ];
    this.events.emit("changed");
    void this.simulateJoin("x-torch", gen, true);
    void this.simulateJoin(LOCAL_ID, gen, false);
  }

  async leave() {
    this.generation++;
    this.list = [];
    this.events.emit("changed");
  }

  private async simulateJoin(id: string, gen: number, skipConnecting: boolean) {
    if (!skipConnecting) {
      await sleep(rand(1000, 1900));
      this.setState(id, gen, "preparing");
    }
    await sleep(rand(1800, 4200));
    this.setState(id, gen, "ready");
  }

  private setState(id: string, gen: number, state: PartyMemberState) {
    if (gen !== this.generation) return;
    const i = this.list.findIndex((m) => m.id === id);
    if (i < 0) return;
    this.list[i] = { ...this.list[i], state };
    const m = this.list[i];
    this.events.emit("memberUpdated", m.isLocal ? { ...m, name: this.localName() } : { ...m });
  }
}
