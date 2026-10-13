import type { PartyMember } from "../models";
import { plural } from "../russian";
import type { PartyService } from "../services/types";

// Сколько держим строки участников после выхода, пока панель плавно уезжает.
const LEAVE_FADE_MS = 320;

export class PartyState {
  members = $state<PartyMember[]>([]);
  inParty = $state(false);
  /** Раскрыт ли список участников. Меняется только действием пользователя. */
  expanded = $state(true);

  readyCount = $derived(this.members.filter((m) => m.state === "ready").length);
  allReady = $derived(this.members.length > 0 && this.readyCount === this.members.length);
  countText = $derived(
    `${this.members.length} ${plural(this.members.length, "игрок", "игрока", "игроков")}`,
  );
  readyText = $derived(`${this.readyCount}/${this.members.length} готовы`);

  constructor(private service: PartyService) {
    service.on("changed", () => this.sync());
    service.on("memberUpdated", (m) => this.upsert(m));
    service.on("memberLeft", (id) => this.remove(id));
    this.sync();
  }

  contains(id: string) {
    return this.members.some((m) => m.id === id);
  }

  toggleExpanded() {
    this.expanded = !this.expanded;
  }

  leave() {
    return this.service.leave();
  }

  /** Обновить ник локального игрока в списке после смены в настройках. */
  renameLocal(name: string) {
    this.members = this.members.map((m) => (m.isLocal ? { ...m, name } : m));
  }

  private sync() {
    if (!this.service.isInParty()) {
      this.inParty = false;
      setTimeout(() => {
        if (!this.inParty) this.members = [];
      }, LEAVE_FADE_MS);
      return;
    }
    this.members = this.service.members();
    this.inParty = true;
  }

  private upsert(member: PartyMember) {
    const i = this.members.findIndex((m) => m.id === member.id);
    if (i < 0) this.members = [...this.members, member];
    else this.members[i] = member;
  }

  private remove(id: string) {
    this.members = this.members.filter((m) => m.id !== id);
  }
}
