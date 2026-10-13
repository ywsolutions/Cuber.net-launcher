<script lang="ts">
  import { openDataDir, saveConfig, type AppInfo, type Config } from "../api";

  let {
    config,
    info,
    onChange,
  }: { config: Config; info: AppInfo; onChange: (c: Config) => void } = $props();

  let status = $state<{ ok: boolean; text: string } | null>(null);

  // Шаг 512 МБ, от 0,5 до 32 ГБ — слайдеру достаточно.
  const MEM_MIN = 512;
  const MEM_MAX = 32768;

  async function update(patch: Partial<Config>) {
    try {
      onChange(await saveConfig({ ...config, ...patch }));
      status = { ok: true, text: "Сохранено" };
    } catch (e) {
      status = { ok: false, text: String(e) };
    }
  }

  function formatMem(mb: number) {
    return mb >= 1024 ? `${(mb / 1024).toFixed(mb % 1024 ? 1 : 0)} ГБ` : `${mb} МБ`;
  }
</script>

<h1>Настройки</h1>
<p class="dim">Изменения сохраняются сразу.</p>

<div class="list">
  <div class="item">
    <div>
      <div class="name">Тема</div>
    </div>
    <select
      value={config.theme}
      onchange={(e) => update({ theme: e.currentTarget.value as Config["theme"] })}
    >
      <option value="system">Как в системе</option>
      <option value="dark">Тёмная</option>
      <option value="light">Светлая</option>
    </select>
  </div>

  <div class="item">
    <div>
      <div class="name">После запуска игры</div>
      <div class="desc dim">Свёрнутый лаунчер почти не тратит ресурсы.</div>
    </div>
    <select
      value={config.after_launch}
      onchange={(e) =>
        update({ after_launch: e.currentTarget.value as Config["after_launch"] })}
    >
      <option value="minimize">Свернуть</option>
      <option value="keep_open">Оставить открытым</option>
      <option value="close">Закрыть</option>
    </select>
  </div>

  <div class="item">
    <div>
      <div class="name">Память по умолчанию</div>
      <div class="desc dim">Для новых профилей. Сейчас: {formatMem(config.default_memory_mb)}</div>
    </div>
    <input
      type="range"
      min={MEM_MIN}
      max={MEM_MAX}
      step="512"
      value={config.default_memory_mb}
      onchange={(e) => update({ default_memory_mb: Number(e.currentTarget.value) })}
    />
  </div>

  <div class="item">
    <div>
      <div class="name">Папка данных</div>
      <div class="desc dim path">{info.dataDir}</div>
    </div>
    <button onclick={() => openDataDir()}>Открыть</button>
  </div>
</div>

{#if status}
  <p class="status" class:err={!status.ok}>{status.text}</p>
{/if}

<p class="about dim">Cuber v{info.version} · {info.platform}</p>

<style>
  .list {
    margin-top: 20px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    max-width: 640px;
  }

  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 14px 18px;
  }

  .item + .item {
    border-top: 1px solid var(--border);
  }

  .name {
    font-weight: 600;
  }

  .desc {
    font-size: 12.5px;
  }

  .path {
    user-select: text;
    word-break: break-all;
  }

  input[type="range"] {
    width: 200px;
    accent-color: var(--accent);
    padding: 0;
    background: none;
    border: none;
  }

  .status {
    font-size: 12.5px;
    color: var(--accent);
  }

  .status.err {
    color: var(--danger);
  }

  .about {
    margin-top: 28px;
    font-size: 12px;
  }
</style>
