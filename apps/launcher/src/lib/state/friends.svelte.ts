import type { Friend, FriendPresence, SocialRequest } from "../models";
import { isValidNickname } from "../nickname";
import type { FriendsService, PartyService } from "../services/types";
import type { PartyState } from "./party.svelte";

const RANK: Record<FriendPresence, number> = { in_game: 0, online: 1, offline: 2 };

export function friendStatusText(f: Friend): string {
  if (f.presence === "in_game") return f.activity ? `В игре · ${f.activity}` : "В игре";
  if (f.presence === "online") return "В сети";
  return "Не в сети";
}

export class FriendsState {
  friends = $state<Friend[]>([]);
  requests = $state<SocialRequest[]>([]);
  newName = $state("");
  /** Короткое сообщение под полем ввода (ошибка или «заявка отправлена»). */
  notice = $state<string | null>(null);

  /** Порядок: в игре → в сети → не в сети, внутри — по нику. */
  sorted = $derived(
    [...this.friends].sort(
      (a, b) =>
        RANK[a.presence] - RANK[b.presence] ||
        a.name.localeCompare(b.name, undefined, { sensitivity: "base" }),
    ),
  );
  onlineCount = $derived(this.friends.filter((f) => f.presence !== "offline").length);

  private noticeTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(
    private service: FriendsService,
    private partyService: PartyService,
    private party: PartyState,
  ) {
    service.on("friendUpdated", (f) => this.upsert(f));
    service.on("requestReceived", (r) => {
      if (!this.requests.some((x) => x.id === r.id)) this.requests = [...this.requests, r];
    });
    service.on("requestRemoved", (id) => {
      this.requests = this.requests.filter((r) => r.id !== id);
    });
  }

  async load() {
    const [friends, requests] = await Promise.all([this.service.friends(), this.service.requests()]);
    this.friends = friends;
    this.requests = requests;
  }

  isInParty(friend: Friend) {
    return this.party.contains(friend.id);
  }

  canInvite(friend: Friend) {
    return friend.presence !== "offline" && !this.isInParty(friend);
  }

  async add() {
    const name = this.newName.trim();
    if (!isValidNickname(name)) {
      this.showNotice("Ник: 3–16 символов — латиница, цифры и _");
      return;
    }
    if (this.friends.some((f) => f.name.toLowerCase() === name.toLowerCase())) {
      this.showNotice(`${name} уже в друзьях`);
      return;
    }
    try {
      await this.service.sendFriendRequest(name);
    } catch (e) {
      this.showNotice(e instanceof Error ? e.message : String(e));
      return;
    }
    this.newName = "";
    this.showNotice(`Заявка отправлена: ${name}`);
  }

  async accept(request: SocialRequest) {
    if (request.kind === "party_invite") await this.partyService.join(request);
    await this.service.accept(request);
  }

  decline(request: SocialRequest) {
    return this.service.decline(request);
  }

  async invite(friend: Friend) {
    if (this.canInvite(friend)) await this.partyService.invite(friend.id, friend.name);
  }

  private upsert(friend: Friend) {
    const i = this.friends.findIndex((f) => f.id === friend.id);
    if (i < 0) this.friends = [...this.friends, friend];
    else this.friends[i] = friend;
  }

  private showNotice(text: string) {
    clearTimeout(this.noticeTimer);
    this.notice = text;
    this.noticeTimer = setTimeout(() => (this.notice = null), 3000);
  }
}
