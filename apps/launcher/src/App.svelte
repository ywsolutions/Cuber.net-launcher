<script lang="ts">
  import { onMount } from "svelte";
  import { fitWindow, restoreWindow } from "./lib/api";
  import FriendsPanel from "./lib/panels/FriendsPanel.svelte";
  import MainBar from "./lib/panels/MainBar.svelte";
  import PartyPanel from "./lib/panels/PartyPanel.svelte";
  import SettingsPanel from "./lib/panels/SettingsPanel.svelte";
  import { LauncherState } from "./lib/state/launcher.svelte";
  import Icon from "./lib/ui/Icon.svelte";
  import Reveal from "./lib/ui/Reveal.svelte";

  const app = new LauncherState();

  // ── Окно «растёт вверх» ──────────────────────────────────────────
  // Весь интерфейс прижат к низу. Перед раскрытием чего-либо окно сразу
  // вырастает до максимума (чтобы анимации было куда расти), а когда всё
  // успокоилось — ужимается ровно по содержимому.
  const MAX_HEIGHT = 760;
  const TOP_SHADOW = 16;
  const SETTLE_MS = 300;

  let stack: HTMLElement | undefined = $state();
  let popup: HTMLElement | undefined = $state();
  let windowHeight = MAX_HEIGHT;
  let shown = false;
  let settleTimer: ReturnType<typeof setTimeout> | undefined;

  function requiredHeight() {
    if (!stack) return MAX_HEIGHT;
    let top = stack.getBoundingClientRect().top;
    if (popup) top = Math.min(top, popup.getBoundingClientRect().top);
    return Math.ceil(window.innerHeight - top + TOP_SHADOW);
  }

  function grow() {
    if (windowHeight >= MAX_HEIGHT) return;
    windowHeight = MAX_HEIGHT;
    void fitWindow(MAX_HEIGHT);
  }

  function settleSoon() {
    clearTimeout(settleTimer);
    settleTimer = setTimeout(() => {
      windowHeight = Math.min(MAX_HEIGHT, requiredHeight());
      const firstShow = !shown;
      shown = true;
      void fitWindow(windowHeight, firstShow).then(() => {
        // Часть систем при первом показе сбрасывает размер или сдвигает окно — возвращаем.
        if (firstShow) setTimeout(() => void restoreWindow(), 250);
      });
    }, SETTLE_MS);
  }

  // Любое раскрытие/скрытие или изменение списков — подготовить место и пересчитать.
  $effect(() => {
    void [
      app.panel,
      app.profileMenuOpen,
      app.party.inParty,
      app.party.expanded,
      app.party.members.length,
      app.friends.requests.length,
      app.friends.notice,
      app.launchError,
    ];
    grow();
    settleSoon();
  });

  onMount(() => {
    void app.init();
    const observer = new ResizeObserver(() => {
      if (requiredHeight() > windowHeight) grow();
      settleSoon();
    });
    if (stack) observer.observe(stack);
    return () => observer.disconnect();
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      app.closePanels();
      // Как в дизайне: после закрытия не оставляем рамку фокуса на кнопке.
      (document.activeElement as HTMLElement | null)?.blur();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="root">
  <div class="stack" bind:this={stack}>
    {#if app.error}
      <div class="card error">Не удалось загрузить настройки: {app.error}</div>
    {/if}

    {#if app.config}
      <Reveal open={app.party.inParty} bleed>
        <PartyPanel party={app.party} />
      </Reveal>

      <Reveal open={app.panel === "friends"} bleed>
        <FriendsPanel friends={app.friends} onClose={() => app.closePanels()} />
      </Reveal>

      <Reveal open={app.panel === "settings"} bleed>
        <SettingsPanel {app} onClose={() => app.closePanels()} />
      </Reveal>

      <!-- Ошибка запуска: над основной панелью, закрывается крестиком. -->
      <Reveal open={!!app.launchError} bleed>
        <div class="card launch-error" role="alert">
          <span class="msg">{app.launchError}</span>
          <button class="icon-btn" title="Скрыть" onclick={() => (app.launchError = null)}>
            <Icon name="x" size={12} />
          </button>
        </div>
      </Reveal>
    {/if}

    <MainBar {app} bind:popup />
  </div>
</div>

<style>
  .root {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  /* Поля — место под тени карточек. */
  .stack {
    margin: 0 16px 22px;
  }

  .launch-error {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 8px 10px 14px;
    margin-bottom: 8px;
    border-color: rgba(248, 113, 113, 0.35);
  }

  .launch-error .msg {
    flex: 1;
    padding-top: 4px;
    font-size: 12px;
    color: var(--red);
    user-select: text;
    word-break: break-word;
  }

  .error {
    padding: 10px 14px;
    margin-bottom: 8px;
    color: var(--red);
  }
</style>
