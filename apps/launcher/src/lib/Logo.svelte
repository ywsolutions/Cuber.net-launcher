<script lang="ts">
  let { size = 24 }: { size?: number } = $props();
</script>

<!-- Изометрический куб: верх светлый, грани темнее. -->
<svg width={size} height={size} viewBox="0 0 64 64" aria-label="Cuber">
  <path d="M32 6 56 19 32 32 8 19z" fill="#7ad687" />
  <path d="M8 19 32 32v26L8 45z" fill="#3fa34d" />
  <path d="M56 19 32 32v26l24-13z" fill="#2b7a37" />
</svg>
