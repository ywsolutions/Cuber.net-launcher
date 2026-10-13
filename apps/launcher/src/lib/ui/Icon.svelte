<script lang="ts" module>
  // Линейные иконки 24×24. В дизайне были Segoe Fluent Icons — они есть только
  // на Windows, поэтому здесь свои SVG: одинаково выглядят на всех системах.
  const PATHS = {
    users:
      "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75",
    "user-plus":
      "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8M19 8v6M22 11h-6",
    plus: "M12 5v14M5 12h14",
    check: "M20 6 9 17l-5-5",
    x: "M18 6 6 18M6 6l12 12",
    leave: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9",
    "chevron-down": "m6 9 6 6 6-6",
    sliders:
      "M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3M14 2v4M8 10v4M16 18v4",
    minus: "M5 12h14",
    folder:
      "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
  } as const;

  export type IconName = keyof typeof PATHS | "play";
</script>

<script lang="ts">
  let { name, size = 16, stroke = 2 }: { name: IconName; size?: number; stroke?: number } =
    $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "play"}
    <path d="M7 4.5v15l12-7.5z" fill="currentColor" stroke-width="1.5" />
  {:else}
    <path d={PATHS[name]} />
  {/if}
</svg>

<style>
  svg {
    display: block;
    flex: none;
  }
</style>
