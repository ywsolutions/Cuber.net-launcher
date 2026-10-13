<script lang="ts">
  import { avatarUrl } from "./avatar";

  let { name, size = 30, radius = 8 }: { name: string; size?: number; radius?: number } = $props();
</script>

<img
  src={avatarUrl(name)}
  alt=""
  width={size}
  height={size}
  style:border-radius="{radius}px"
  draggable="false"
/>

<style>
  img {
    display: block;
    flex: none;
    image-rendering: pixelated;
  }
</style>
