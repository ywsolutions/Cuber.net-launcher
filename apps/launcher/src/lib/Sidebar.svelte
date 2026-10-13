<script lang="ts" module>
  export type Page = "play" | "content" | "friends" | "settings";
</script>

<script lang="ts">
  import Logo from "./Logo.svelte";

  let { page = $bindable(), version }: { page: Page; version: string } = $props();

  const items: { id: Page; label: string; icon: string }[] = [
    { id: "play", label: "Играть", icon: "M8 5v14l11-7z" },
    {
      id: "content",
      label: "Контент",
      icon: "M12 2 3 7v10l9 5 9-5V7zm0 2.3L18.6 8 12 11.7 5.4 8zM5 9.7l6 3.3v6.7l-6-3.3zm8 10v-6.7l6-3.3v6.7z",
    },
    {
      id: "friends",
      label: "Друзья",
      icon: "M16 11a3 3 0 1 0-3-3 3 3 0 0 0 3 3m-8 0a3 3 0 1 0-3-3 3 3 0 0 0 3 3m0 2c-2.3 0-7 1.2-7 3.5V19h14v-2.5C15 14.2 10.3 13 8 13m8 0c-.3 0-.6 0-1 .1a4.2 4.2 0 0 1 2 3.4V19h6v-2.5c0-2.3-4.7-3.5-7-3.5",
    },
    {
      id: "settings",
      label: "Настройки",
      icon: "M19.4 13a7.6 7.6 0 0 0 0-2l2.1-1.6-2-3.5-2.5 1a7.4 7.4 0 0 0-1.7-1L15 3h-4l-.4 2.9a7.4 7.4 0 0 0-1.7 1l-2.5-1-2 3.5L6.6 11a7.6 7.6 0 0 0 0 2l-2.1 1.6 2 3.5 2.5-1a7.4 7.4 0 0 0 1.7 1L11 21h4l.4-2.9a7.4 7.4 0 0 0 1.7-1l2.5 1 2-3.5zM13 15.5a3.5 3.5 0 1 1 3.5-3.5 3.5 3.5 0 0 1-3.5 3.5",
    },
  ];
</script>

<nav>
  <div class="brand">
    <Logo size={26} />
    <span>Cuber</span>
  </div>

  <ul>
    {#each items as item (item.id)}
      <li>
        <button class:active={page === item.id} onclick={() => (page = item.id)}>
          <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
            <path d={item.icon} fill="currentColor" />
          </svg>
          {item.label}
        </button>
      </li>
    {/each}
  </ul>

  <div class="version dim">{version ? `v${version}` : ""}</div>
</nav>

<style>
  nav {
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: 18px 12px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 700;
    font-size: 17px;
    padding: 0 8px 20px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  li button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    border: none;
    background: transparent;
    color: var(--text-dim);
    text-align: left;
    padding: 9px 10px;
  }

  li button:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  li button.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .version {
    margin-top: auto;
    padding: 0 8px;
    font-size: 12px;
  }
</style>
