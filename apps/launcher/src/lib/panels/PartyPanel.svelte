<script lang="ts">
  import type { PartyMember } from "../models";
  import type { PartyState } from "../state/party.svelte";
  import Avatar from "../ui/Avatar.svelte";
  import Icon from "../ui/Icon.svelte";
  import Reveal from "../ui/Reveal.svelte";

  let { party }: { party: PartyState } = $props();

  const STATUS: Record<PartyMember["state"], string> = {
    connecting: "Подключается",
    preparing: "Подготавливает сборку",
    ready: "Готов к игре",
  };
</script>

<div class="card party">
  <!-- Заголовок: клик сворачивает/разворачивает только список участников -->
  <div
    class="head"
    role="button"
    tabindex="0"
    title="Свернуть / развернуть список"
    onclick={() => party.toggleExpanded()}
    onkeydown={(e) => (e.key === "Enter" || e.key === " ") && party.toggleExpanded()}
  >
    <span class="badge-icon"><Icon name="users" size={14} /></span>
    <span class="title">
      <b>Party</b> <span class="faint">·</span> <span class="dim">{party.countText}</span>
    </span>
    <span class="ready" class:all={party.allReady}>{party.readyText}</span>
    <button
      class="icon-btn"
      title="Покинуть Party"
      onclick={(e) => {
        e.stopPropagation();
        void party.leave();
      }}
    >
      <Icon name="leave" size={14} />
    </button>
    <span class="chevron" class:collapsed={!party.expanded}>
      <Icon name="chevron-down" size={14} />
    </span>
  </div>

  <!-- Список участников: до 5 строк, дальше — прокрутка -->
  <Reveal open={party.expanded} anchor="top">
    <div class="divider"></div>
    <div class="list scroll">
      {#each party.members as m (m.id)}
        <div class="row row-appear member">
          <Avatar name={m.name} size={26} radius={7} />
          <span class="who">
            <span class="name">{m.name}</span>
            {#if m.isLeader}<span class="tag leader">лидер</span>{/if}
            {#if m.isLocal}<span class="tag">вы</span>{/if}
          </span>
          <span class="status {m.state}">
            <span class="dot"></span>
            {STATUS[m.state]}
          </span>
        </div>
      {/each}
    </div>
  </Reveal>
</div>

<style>
  .party {
    padding: 6px;
    margin-bottom: 8px;
  }

  .head {
    display: flex;
    align-items: center;
    height: 36px;
    cursor: pointer;
    border-radius: 9px;
    outline: none;
  }

  .badge-icon {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    margin: 0 10px 0 6px;
    border-radius: 7px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  .title {
    flex: 1;
    font-size: 13px;
  }

  .title b {
    font-weight: 600;
  }

  .ready {
    margin-right: 4px;
    padding: 2px 7px;
    border-radius: 6px;
    font-size: 11px;
    background: var(--surface);
    color: var(--text-dim);
    transition: background 200ms, color 200ms;
  }

  .ready.all {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .chevron {
    display: grid;
    place-items: center;
    width: 28px;
    color: var(--text-dim);
    transition: transform 200ms ease;
  }

  .chevron.collapsed {
    transform: rotate(180deg);
  }

  .divider {
    height: 1px;
    margin: 4px 6px;
    background: var(--stroke);
  }

  .list {
    max-height: 200px;
  }

  .member {
    height: 40px;
  }

  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tag {
    padding: 1px 5px;
    border-radius: 5px;
    background: var(--surface);
    font-size: 10px;
    color: var(--text-dim);
  }

  .tag.leader {
    color: var(--amber);
  }

  .member:hover .tag {
    background: var(--hover);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 11.5px;
    white-space: nowrap;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }

  .status.connecting,
  .status.preparing {
    color: var(--text-dim);
  }

  .connecting .dot {
    background: var(--amber);
    animation: pulse 1.4s ease-in-out infinite;
  }

  .preparing .dot {
    background: var(--blue);
    animation: pulse 1.4s ease-in-out infinite;
  }

  @keyframes pulse {
    50% {
      opacity: 0.25;
    }
  }
</style>
