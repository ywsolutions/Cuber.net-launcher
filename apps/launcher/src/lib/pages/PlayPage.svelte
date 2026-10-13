<script lang="ts">
  import { untrack } from "svelte";
  import { saveConfig, type Config } from "../api";
  import { nicknameError } from "../nickname";

  let { config, onChange }: { config: Config; onChange: (c: Config) => void } = $props();

  // Берём сохранённый ник один раз при открытии страницы.
  let nickname = $state(untrack(() => config.last_nickname ?? ""));
  let error = $derived(nicknameError(nickname));
  let canPlay = $derived(nickname.length > 0 && !error);

  async function rememberNickname() {
    if (!canPlay || nickname === config.last_nickname) return;
    onChange(await saveConfig({ ...config, last_nickname: nickname }));
  }
</script>

<header>
  <h1>Играть</h1>
  <p class="dim">Профили и запуск Minecraft появятся на следующем этапе.</p>
</header>

<section class="card">
  <label for="nick">Ник</label>
  <div class="row">
    <input
      id="nick"
      placeholder="Например, Steve"
      maxlength="16"
      spellcheck="false"
      autocomplete="off"
      bind:value={nickname}
      onblur={rememberNickname}
    />
    <button class="primary" disabled title="Запуск игры — этап 1">Играть</button>
  </div>
  {#if error}
    <p class="hint error">{error}</p>
  {:else}
    <p class="hint dim">Офлайн-режим: лицензия не нужна.</p>
  {/if}
</section>

<style>
  header {
    margin-bottom: 20px;
  }

  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 18px;
    max-width: 480px;
  }

  label {
    display: block;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .row {
    display: flex;
    gap: 10px;
  }

  .row input {
    flex: 1;
  }

  .hint {
    margin: 8px 0 0;
    font-size: 12.5px;
  }

  .error {
    color: var(--danger);
  }
</style>
