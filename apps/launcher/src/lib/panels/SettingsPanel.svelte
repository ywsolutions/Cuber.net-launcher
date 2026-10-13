<script lang="ts">
  import { untrack } from "svelte";
  import { openDataDir } from "../api";
  import { nicknameError } from "../nickname";
  import { MEMORY_MAX_GB, MEMORY_MIN_GB, type LauncherState } from "../state/launcher.svelte";
  import Icon from "../ui/Icon.svelte";
  import Switch from "../ui/Switch.svelte";

  let { app, onClose }: { app: LauncherState; onClose: () => void } = $props();

  // Ник редактируется локально и сохраняется, только когда он корректен.
  let nick = $state(untrack(() => app.config?.last_nickname ?? ""));
  let nickError = $derived(nicknameError(nick));
  let memory = $state(untrack(() => app.memoryGb));
  let saveError = $state<string | null>(null);

  async function run(action: () => Promise<void>) {
    try {
      saveError = null;
      await action();
    } catch (e) {
      saveError = String(e);
    }
  }

  function commitNick() {
    if (nick && !nickError && nick !== app.config?.last_nickname) {
      void run(() => app.setNickname(nick));
    }
  }

  // Заполнение дорожки слайдера слева от бегунка.
  let fill = $derived(((memory - MEMORY_MIN_GB) / (MEMORY_MAX_GB - MEMORY_MIN_GB)) * 100);
</script>

<div class="card panel">
  <div class="head">
    <b>Настройки</b>
    <button class="icon-btn small-btn" title="Закрыть (Esc)" onclick={onClose}>
      <Icon name="x" size={12} />
    </button>
  </div>

  <div class="field">
    <label for="nick">Ник в игре</label>
    <input
      id="nick"
      class="input"
      placeholder="Player"
      maxlength="16"
      spellcheck="false"
      autocomplete="off"
      bind:value={nick}
      onblur={commitNick}
      onkeydown={(e) => e.key === "Enter" && commitNick()}
    />
    {#if nickError}
      <div class="hint error">{nickError}</div>
    {:else}
      <div class="hint">Офлайн-режим: лицензия не нужна</div>
    {/if}
  </div>

  <div class="line">
    <span>Память для игры</span>
    <b class="accent">{memory} ГБ</b>
  </div>
  <input
    class="slider"
    type="range"
    min={MEMORY_MIN_GB}
    max={MEMORY_MAX_GB}
    step="1"
    style:--fill="{fill}%"
    bind:value={memory}
    onchange={() => run(() => app.setMemoryGb(memory))}
  />

  <div class="gap"></div>
  <Switch
    label="Сворачивать лаунчер при запуске игры"
    checked={app.minimizeOnLaunch}
    onchange={(v) => run(() => app.setMinimizeOnLaunch(v))}
  />

  <button class="line folder" title="Открыть папку" onclick={() => openDataDir()}>
    <span>Папка игры</span>
    <span class="path">{app.info?.dataDir}</span>
  </button>

  {#if saveError}
    <div class="hint error">{saveError}</div>
  {/if}

  <div class="divider"></div>
  <div class="version">Cuber.net · {app.info?.version}</div>
</div>

<style>
  .panel {
    padding: 10px 14px 12px;
    margin-bottom: 8px;
    font-size: 12.5px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 -4px 10px 0;
    font-size: 14px;
  }

  .head b {
    font-weight: 600;
  }

  .small-btn {
    width: 26px;
    height: 26px;
  }

  .field {
    margin-bottom: 14px;
  }

  label {
    display: block;
    margin-bottom: 6px;
  }

  .hint {
    margin: 5px 0 0 2px;
    font-size: 11px;
    color: var(--text-faint);
  }

  .hint.error {
    color: var(--red);
  }

  .line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    text-align: left;
  }

  .accent {
    color: var(--accent);
    font-weight: 600;
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 20px;
    margin: 6px 0 0;
    background: transparent;
    cursor: pointer;
  }

  .slider::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
    background: linear-gradient(
      to right,
      var(--accent) var(--fill),
      var(--pressed) var(--fill)
    );
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    margin-top: -5px;
    border-radius: 50%;
    background: #fff;
  }

  .slider::-moz-range-track {
    height: 4px;
    border-radius: 2px;
    background: var(--pressed);
  }

  .slider::-moz-range-progress {
    height: 4px;
    border-radius: 2px;
    background: var(--accent);
  }

  .slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border: none;
    border-radius: 50%;
    background: #fff;
  }

  .gap {
    height: 14px;
  }

  .folder {
    margin-top: 14px;
    font-size: 12.5px;
  }

  .path {
    min-width: 0;
    font-size: 12px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder:hover .path {
    color: var(--text);
  }

  .divider {
    height: 1px;
    margin: 12px 0 8px;
    background: var(--stroke);
  }

  .version {
    font-size: 10.5px;
    color: var(--text-faint);
  }
</style>
