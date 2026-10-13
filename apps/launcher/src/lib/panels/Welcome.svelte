<script lang="ts">
  // Приветствие при первом запуске: ник и версия для первого профиля.
  import { onMount } from "svelte";
  import { closeWindow, minimizeWindow, startDragging } from "../api";
  import { isValidNickname, nicknameError } from "../nickname";
  import type { LauncherState } from "../state/launcher.svelte";
  import Avatar from "../ui/Avatar.svelte";
  import Icon from "../ui/Icon.svelte";

  let { app }: { app: LauncherState } = $props();

  let nick = $state("");
  let nickError = $derived(nicknameError(nick));
  let versions = $state<string[]>([]);
  let version = $state("");
  let versionsError = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let input: HTMLInputElement | undefined = $state();

  let canStart = $derived(isValidNickname(nick) && !busy);

  onMount(async () => {
    input?.focus();
    try {
      const v = await app.services.profiles.versions();
      versions = v.versions;
      version = v.latest;
    } catch {
      versionsError = true;
    }
  });

  async function start() {
    if (!canStart) return;
    busy = true;
    error = null;
    try {
      await app.completeOnboarding(nick, version || null);
    } catch (e) {
      error = String(e);
      busy = false;
    }
  }

  function onMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button, input, select, a")) return;
    startDragging();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card welcome" onmousedown={onMouseDown}>
  <div class="top">
    <div class="brand">
      <svg width="22" height="22" viewBox="0 0 64 64" aria-hidden="true">
        <path d="M32 6 56 19 32 32 8 19z" fill="#7ad687" />
        <path d="M8 19 32 32v26L8 45z" fill="#3fa34d" />
        <path d="M56 19 32 32v26l24-13z" fill="#2b7a37" />
      </svg>
      <span>Cuber</span>
    </div>
    <div class="window-btns">
      <button class="icon-btn tiny" title="Свернуть" onclick={minimizeWindow}>
        <Icon name="minus" size={10} />
      </button>
      <button class="icon-btn tiny" title="Закрыть" onclick={closeWindow}>
        <Icon name="x" size={10} />
      </button>
    </div>
  </div>

  <h1>Добро пожаловать!</h1>
  <p class="lead">Придумайте ник — под ним вас увидят в игре и друзья. Лицензия не нужна.</p>

  <form
    onsubmit={(e) => {
      e.preventDefault();
      void start();
    }}
  >
    <label for="w-nick">Ник</label>
    <div class="nick-row">
      <div class="avatar" class:empty={!nick}>
        <Avatar name={nick || "?"} size={34} radius={9} />
      </div>
      <input
        id="w-nick"
        bind:this={input}
        class="input"
        placeholder="Например, Steve"
        maxlength="16"
        spellcheck="false"
        autocomplete="off"
        bind:value={nick}
      />
    </div>
    <div class="hint" class:error={!!nickError}>
      {nickError ?? "3–16 символов: латиница, цифры и _"}
    </div>

    <label for="w-version">Версия Minecraft</label>
    {#if versionsError}
      <div class="input muted">Нет связи — профиль можно создать позже</div>
    {:else if versions.length === 0}
      <div class="input muted">Загрузка списка версий…</div>
    {:else}
      <select id="w-version" class="input" bind:value={version}>
        {#each versions as v (v)}
          <option value={v}>{v}{v === versions[0] ? " — последняя" : ""}</option>
        {/each}
      </select>
    {/if}
    <div class="hint">Потом можно создать сколько угодно профилей с разными версиями.</div>

    {#if error}<div class="hint error">{error}</div>{/if}

    <button class="start" type="submit" disabled={!canStart}>
      {busy ? "Готовлю…" : "Начать"}
    </button>
  </form>
</div>

<style>
  .welcome {
    padding: 14px 18px 18px;
    animation: appear 260ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  @keyframes appear {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }

  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 700;
  }

  .window-btns {
    display: flex;
    gap: 2px;
    margin-right: -8px;
  }

  .tiny {
    width: 24px;
    height: 24px;
    border-radius: 7px;
  }

  h1 {
    margin: 0 0 4px;
    font-size: 21px;
    font-weight: 650;
  }

  .lead {
    margin: 0 0 14px;
    color: var(--text-dim);
    font-size: 12.5px;
  }

  label {
    display: block;
    margin: 10px 0 6px 2px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .nick-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .avatar {
    transition: opacity 150ms;
  }

  .avatar.empty {
    opacity: 0.35;
  }

  .nick-row .input {
    flex: 1;
    height: 38px;
    font-size: 14px;
  }

  select.input {
    appearance: none;
    cursor: pointer;
    background-image: linear-gradient(45deg, transparent 50%, var(--text-dim) 50%),
      linear-gradient(135deg, var(--text-dim) 50%, transparent 50%);
    background-position:
      calc(100% - 15px) 15px,
      calc(100% - 10px) 15px;
    background-size: 5px 5px;
    background-repeat: no-repeat;
  }

  select.input option {
    background: var(--surface);
    color: var(--text);
  }

  .muted {
    display: flex;
    align-items: center;
    color: var(--text-faint);
  }

  .hint {
    margin: 5px 0 0 2px;
    font-size: 11px;
    color: var(--text-faint);
  }

  .hint.error {
    color: var(--red);
  }

  .start {
    width: 100%;
    height: 42px;
    margin-top: 18px;
    border-radius: 11px;
    background: var(--play);
    color: var(--on-accent);
    font-size: 14px;
    font-weight: 600;
    transition: transform 150ms;
  }

  .start:hover:not(:disabled) {
    background: var(--play-hover);
  }

  .start:active:not(:disabled) {
    transform: scale(0.98);
  }

  .start:disabled {
    background: var(--surface);
    color: var(--text-faint);
    cursor: default;
  }
</style>
