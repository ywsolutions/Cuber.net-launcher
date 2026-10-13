<script lang="ts">
  import { closeWindow, minimizeWindow, startDragging } from "../api";
  import type { LauncherState } from "../state/launcher.svelte";
  import Avatar from "../ui/Avatar.svelte";
  import Icon from "../ui/Icon.svelte";
  import ProfileMenu from "./ProfileMenu.svelte";

  let { app, popup = $bindable() }: { app: LauncherState; popup?: HTMLElement } = $props();

  let profileButton: HTMLElement | undefined = $state();
  const profile = $derived(app.selectedProfile);

  // Перетаскивание окна за свободное место на панели.
  function onBarMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button, input, a, [role=button]")) return;
    startDragging();
  }

  // Клик вне меню профилей закрывает его.
  function onWindowMouseDown(e: MouseEvent) {
    if (!app.profileMenuOpen) return;
    const t = e.target as Node;
    if (popup?.contains(t) || profileButton?.contains(t)) return;
    app.profileMenuOpen = false;
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card bar" onmousedown={onBarMouseDown}>
  <!-- Профиль -->
  <div class="profile-wrap">
    <button
      bind:this={profileButton}
      class="profile"
      class:checked={app.profileMenuOpen}
      title="Сменить профиль"
      onclick={() => (app.profileMenuOpen = !app.profileMenuOpen)}
    >
      <Avatar name={app.nickname} size={36} radius={9} />
      <span class="meta">
        <span class="pname">{profile?.name ?? "Нет профиля"}</span>
        <span class="sub">{profile ? `${profile.version} · ${profile.loader}` : "—"}</span>
      </span>
      <span class="dim"><Icon name="chevron-down" size={12} /></span>
    </button>

    {#if app.profileMenuOpen}
      <div class="card popup" bind:this={popup}>
        <ProfileMenu {app} />
      </div>
    {/if}
  </div>

  <!-- Друзья -->
  <button
    class="chip"
    class:checked={app.panel === "friends"}
    title="Друзья"
    onclick={() => app.togglePanel("friends")}
  >
    <Icon name="users" size={15} />
    <span>Друзья</span>
    {#if app.friends.requests.length > 0}
      <span class="count">{app.friends.requests.length}</span>
    {/if}
  </button>

  <!-- Настройки -->
  <button
    class="chip square"
    class:checked={app.panel === "settings"}
    title="Настройки"
    onclick={() => app.togglePanel("settings")}
  >
    <Icon name="sliders" size={16} />
  </button>

  <!-- Играть -->
  <button
    class="play"
    disabled={app.launching || app.selectedRunning || !profile}
    title={profile ? undefined : "Сначала создайте профиль"}
    onclick={() => app.play()}
  >
    {#if !app.launching && !app.selectedRunning}<Icon name="play" size={13} />{/if}
    <span>{app.playText}</span>
    {#if app.launching}
      <span class="progress"><span style:transform="scaleX({app.launchProgress})"></span></span>
    {/if}
  </button>

  <!-- Окно -->
  <div class="window-btns">
    <button class="icon-btn tiny" title="Свернуть" onclick={minimizeWindow}>
      <Icon name="minus" size={10} />
    </button>
    <button class="icon-btn tiny" title="Закрыть" onclick={closeWindow}>
      <Icon name="x" size={10} />
    </button>
  </div>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    height: 64px;
    padding: 0 6px 0 10px;
    gap: 6px;
  }

  .profile-wrap {
    position: relative;
    flex: 1;
    min-width: 0;
    margin-left: -4px;
  }

  .profile {
    display: flex;
    align-items: center;
    width: 100%;
    height: 46px;
    padding: 4px;
    border-radius: 11px;
    text-align: left;
    transition: background 120ms;
  }

  .profile:hover {
    background: var(--surface);
  }

  .profile.checked {
    background: var(--hover);
  }

  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    margin: 0 6px 0 10px;
  }

  .pname {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }


  .profile .dim {
    margin-right: 4px;
  }

  .popup {
    position: absolute;
    left: -2px;
    bottom: calc(100% + 14px);
    width: 270px;
    padding: 6px;
    z-index: 10;
    animation: fade 140ms ease;
  }


  @keyframes fade {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }






  .chip {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 38px;
    padding: 0 12px;
    border-radius: 10px;
    background: var(--surface);
    color: var(--text-dim);
    font-size: 12.5px;
    transition: background 120ms, color 120ms;
  }

  .chip:hover {
    background: var(--hover);
    color: var(--text);
  }

  .chip.checked {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .chip.square {
    width: 38px;
    padding: 0;
    justify-content: center;
  }

  .count {
    position: absolute;
    top: -5px;
    right: -5px;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--red);
    color: #fff;
    font-size: 10px;
    font-weight: 700;
    line-height: 16px;
    text-align: center;
  }

  .play {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 124px;
    height: 40px;
    margin-left: 2px;
    border-radius: 11px;
    background: var(--play);
    color: var(--on-accent);
    font-size: 14px;
    font-weight: 600;
    overflow: hidden;
    transition: transform 150ms ease;
  }

  .play:hover:not(:disabled) {
    background: var(--play-hover);
  }

  .play:active:not(:disabled) {
    transform: scale(0.96);
    transition-duration: 80ms;
  }

  .play:disabled {
    background: var(--surface);
    color: var(--text);
    cursor: default;
  }

  .progress {
    position: absolute;
    left: 12px;
    right: 12px;
    bottom: 6px;
    height: 3px;
    border-radius: 1.5px;
    background: rgba(255, 255, 255, 0.13);
  }

  .progress span {
    display: block;
    height: 100%;
    border-radius: 1.5px;
    background: var(--accent);
    transform-origin: left center;
    transition: transform 60ms linear;
  }

  .window-btns {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-left: -2px;
  }

  .tiny {
    width: 20px;
    height: 20px;
    border-radius: 6px;
  }
</style>
