<script lang="ts">
  import { friendStatusText, type FriendsState } from "../state/friends.svelte";
  import Avatar from "../ui/Avatar.svelte";
  import Icon from "../ui/Icon.svelte";

  let { friends, onClose }: { friends: FriendsState; onClose: () => void } = $props();
</script>

<div class="card panel">
  <div class="head">
    <span>
      <b>Друзья</b> <span class="faint">·</span>
      <span class="dim small">{friends.onlineCount} в сети</span>
    </span>
    <button class="icon-btn small-btn" title="Закрыть (Esc)" onclick={onClose}>
      <Icon name="x" size={12} />
    </button>
  </div>

  <form
    class="add"
    onsubmit={(e) => {
      e.preventDefault();
      void friends.add();
    }}
  >
    <input
      class="input"
      placeholder="Добавить друга по нику"
      maxlength="16"
      spellcheck="false"
      autocomplete="off"
      bind:value={friends.newName}
    />
    <button class="accept-btn big" type="submit" title="Отправить заявку">
      <Icon name="user-plus" size={15} />
    </button>
  </form>
  {#if friends.notice}
    <div class="notice">{friends.notice}</div>
  {/if}

  <div class="lists scroll">
    {#if friends.requests.length > 0}
      <div class="section-label">Заявки и приглашения</div>
      {#each friends.requests as r (r.id)}
        <div class="row row-appear item">
          <Avatar name={r.fromName} size={30} />
          <span class="info">
            <span class="name">{r.fromName}</span>
            <span class="sub" class:invite={r.kind === "party_invite"}>
              {r.kind === "party_invite" ? "Приглашает в Party" : "Хочет добавить в друзья"}
            </span>
          </span>
          <button class="accept-btn" title="Принять" onclick={() => friends.accept(r)}>
            <Icon name="check" size={13} stroke={2.4} />
          </button>
          <button class="icon-btn" title="Отклонить" onclick={() => friends.decline(r)}>
            <Icon name="x" size={12} />
          </button>
        </div>
      {/each}
    {/if}

    <div class="section-label">Друзья</div>
    {#each friends.sorted as f (f.id)}
      <div class="row item" class:offline={f.presence === "offline"}>
        <span class="avatar">
          <Avatar name={f.name} size={30} />
          <span class="presence {f.presence}"></span>
        </span>
        <span class="info">
          <span class="name">{f.name}</span>
          <span class="sub">{friendStatusText(f)}</span>
        </span>
        {#if friends.isInParty(f)}
          <span class="in-party">в Party</span>
        {:else if friends.canInvite(f)}
          <button class="pill" title="Пригласить в Party" onclick={() => friends.invite(f)}>
            <Icon name="plus" size={10} stroke={2.6} /> Party
          </button>
        {/if}
      </div>
    {:else}
      <div class="empty">Пока никого — добавьте друга по нику</div>
    {/each}
  </div>
</div>

<style>
  .panel {
    padding: 10px;
    margin-bottom: 8px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 0 8px 4px;
    font-size: 14px;
  }

  .head b {
    font-weight: 600;
  }

  .small {
    font-size: 12px;
  }

  .small-btn {
    width: 26px;
    height: 26px;
  }

  .add {
    display: flex;
    gap: 6px;
  }

  .big {
    width: 34px;
    height: 34px;
    flex: none;
  }

  .notice {
    margin: 6px 0 0 6px;
    font-size: 11px;
    color: var(--text-dim);
  }

  .lists {
    max-height: 248px;
    margin-top: 4px;
  }

  .item {
    height: 44px;
  }

  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .sub.invite {
    color: var(--accent);
  }

  .avatar {
    position: relative;
    flex: none;
  }

  .presence {
    position: absolute;
    right: -3px;
    bottom: -3px;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    border: 2px solid var(--card);
    background: var(--text-faint);
  }

  .presence.online {
    background: var(--accent);
  }

  .presence.in_game {
    background: var(--blue);
  }

  .offline .avatar :global(img) {
    opacity: 0.45;
  }

  .offline .info {
    opacity: 0.55;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 9px;
    border-radius: 8px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11.5px;
    font-weight: 600;
    transition: background 120ms;
  }

  .pill:hover {
    background: var(--accent-soft-hover);
  }

  .in-party {
    margin-right: 6px;
    font-size: 11px;
    color: var(--accent);
  }

  .empty {
    margin: 4px 0 8px 6px;
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
