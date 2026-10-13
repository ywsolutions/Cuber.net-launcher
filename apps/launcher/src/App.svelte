<script lang="ts">
  import { onMount } from "svelte";
  import { appInfo, applyTheme, getConfig, type AppInfo, type Config } from "./lib/api";
  import Sidebar, { type Page } from "./lib/Sidebar.svelte";
  import PlayPage from "./lib/pages/PlayPage.svelte";
  import ContentPage from "./lib/pages/ContentPage.svelte";
  import FriendsPage from "./lib/pages/FriendsPage.svelte";
  import SettingsPage from "./lib/pages/SettingsPage.svelte";

  let page = $state<Page>("play");
  let config = $state<Config | null>(null);
  let info = $state<AppInfo | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [config, info] = await Promise.all([getConfig(), appInfo()]);
      applyTheme(config.theme);
    } catch (e) {
      error = String(e);
    }
  });

  function onConfigChange(next: Config) {
    config = next;
    applyTheme(next.theme);
  }
</script>

<div class="layout">
  <Sidebar bind:page version={info?.version ?? ""} />

  <main>
    {#if error}
      <p class="error">Не удалось загрузить настройки: {error}</p>
    {:else if !config || !info}
      <p class="dim">Загрузка…</p>
    {:else if page === "play"}
      <PlayPage {config} onChange={onConfigChange} />
    {:else if page === "content"}
      <ContentPage />
    {:else if page === "friends"}
      <FriendsPage />
    {:else}
      <SettingsPage {config} {info} onChange={onConfigChange} />
    {/if}
  </main>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    height: 100%;
  }

  main {
    overflow-y: auto;
    padding: 28px 32px;
  }

  .error {
    color: var(--danger);
  }
</style>
