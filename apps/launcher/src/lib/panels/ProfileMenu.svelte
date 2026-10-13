<script lang="ts">
  import type { LauncherState } from "../state/launcher.svelte";
  import Avatar from "../ui/Avatar.svelte";
  import Icon from "../ui/Icon.svelte";

  let { app }: { app: LauncherState } = $props();

  // Режим: список профилей или форма нового.
  let creating = $state(false);
  let versions = $state<string[]>([]);
  let loadingVersions = $state(false);
  let version = $state("");
  let name = $state("");
  let nameTouched = $state(false);
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function startCreate() {
    creating = true;
    error = null;
    nameTouched = false;
    if (versions.length > 0) return;
    loadingVersions = true;
    try {
      const v = await app.services.profiles.versions();
      versions = v.versions;
      version = v.latest;
      name = `Minecraft ${v.latest}`;
    } catch (e) {
      error = `Не удалось получить список версий: ${e}`;
    } finally {
      loadingVersions = false;
    }
  }

  function onVersionChange() {
    // Пока имя не трогали — подставляем версию в имя.
    if (!nameTouched) name = `Minecraft ${version}`;
  }

  async function create() {
    if (!name.trim() || !version || busy) return;
    busy = true;
    error = null;
    try {
      await app.createProfile(name.trim(), version);
      creating = false;
      app.profileMenuOpen = false;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if !creating}
  <div class="section-label">Профили</div>
  <div class="list scroll">
    {#each app.profiles as p (p.id)}
      <div class="item">
        <button class="list-btn" onclick={() => app.selectProfile(p)}>
          <Avatar name={app.nickname} size={28} radius={7} />
          <span class="meta">
            <span class="pname">{p.name}</span>
            <span class="sub">
              {p.version} · {p.loader}{app.running.includes(p.id) ? " · в игре" : ""}
            </span>
          </span>
          <span class="check" class:visible={p.id === app.selectedProfile?.id}>
            <Icon name="check" size={13} stroke={2.4} />
          </span>
        </button>
        <button
          class="icon-btn folder"
          title="Открыть папку профиля"
          onclick={() => app.services.profiles.openFolder(p)}
        >
          <Icon name="folder" size={13} />
        </button>
      </div>
    {:else}
      <div class="empty">Профилей пока нет</div>
    {/each}
  </div>
  <div class="divider"></div>
  <button class="list-btn add" onclick={startCreate}>
    <span class="add-icon"><Icon name="plus" size={14} stroke={2.4} /></span>
    <span>Новый профиль</span>
  </button>
{:else}
  <form
    class="form"
    onsubmit={(e) => {
      e.preventDefault();
      void create();
    }}
  >
    <div class="form-head">
      <span class="section-label flat">Новый профиль</span>
      <button type="button" class="icon-btn small" title="Назад" onclick={() => (creating = false)}>
        <Icon name="x" size={11} />
      </button>
    </div>

    <label for="pname">Название</label>
    <input
      id="pname"
      class="input"
      maxlength="40"
      spellcheck="false"
      autocomplete="off"
      bind:value={name}
      oninput={() => (nameTouched = true)}
    />

    <label for="pversion">Версия Minecraft</label>
    {#if loadingVersions}
      <div class="input placeholder">Загрузка списка…</div>
    {:else}
      <select id="pversion" class="input" bind:value={version} onchange={onVersionChange}>
        {#each versions as v (v)}
          <option value={v}>{v}</option>
        {/each}
      </select>
    {/if}

    {#if error}<div class="error">{error}</div>{/if}

    <button class="create" type="submit" disabled={busy || loadingVersions || !version}>
      {busy ? "Создаю…" : "Создать"}
    </button>
    <div class="hint">Файлы игры скачаются при первом запуске.</div>
  </form>
{/if}

<style>
  .section-label {
    margin: 4px 0 4px 8px;
  }

  .section-label.flat {
    margin: 0;
  }

  .list {
    max-height: 236px;
  }

  .item {
    position: relative;
  }

  .list-btn {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 6px;
    border-radius: 9px;
    text-align: left;
    transition: background 120ms;
  }

  .list-btn:hover {
    background: var(--hover);
  }

  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    margin: 0 30px 0 10px;
  }

  .pname {
    font-size: 12.5px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .check {
    margin-right: 4px;
    color: var(--accent);
    visibility: hidden;
  }

  .check.visible {
    visibility: visible;
  }

  /* Папка появляется при наведении на строку. */
  .folder {
    position: absolute;
    right: 28px;
    top: 50%;
    transform: translateY(-50%);
    opacity: 0;
  }

  .item:hover .folder {
    opacity: 1;
  }

  .empty {
    padding: 8px 10px;
    font-size: 12px;
    color: var(--text-faint);
  }

  .divider {
    height: 1px;
    margin: 4px 6px;
    background: var(--stroke);
  }

  .add {
    gap: 10px;
    color: var(--text-dim);
    font-size: 12.5px;
  }

  .add:hover {
    color: var(--text);
  }

  .add-icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  .form {
    display: flex;
    flex-direction: column;
    padding: 4px 6px 6px;
  }

  .form-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 2px 0 8px 2px;
  }

  .small {
    width: 24px;
    height: 24px;
  }

  label {
    margin: 6px 0 5px 2px;
    font-size: 12px;
    color: var(--text-dim);
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

  .placeholder {
    display: flex;
    align-items: center;
    color: var(--text-faint);
  }

  .error {
    margin-top: 8px;
    font-size: 11.5px;
    color: var(--red);
  }

  .create {
    height: 34px;
    margin-top: 12px;
    border-radius: 9px;
    background: var(--play);
    color: var(--on-accent);
    font-weight: 600;
  }

  .create:hover:not(:disabled) {
    background: var(--play-hover);
  }

  .create:disabled {
    background: var(--surface);
    color: var(--text-dim);
    cursor: default;
  }

  .hint {
    margin-top: 6px;
    font-size: 11px;
    color: var(--text-faint);
    text-align: center;
  }
</style>
