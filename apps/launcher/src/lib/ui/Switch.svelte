<script lang="ts">
  let {
    checked,
    label,
    onchange,
  }: { checked: boolean; label: string; onchange: (value: boolean) => void } = $props();
</script>

<button class="switch" role="switch" aria-checked={checked} onclick={() => onchange(!checked)}>
  <span class="label">{label}</span>
  <span class="track" class:on={checked}><span class="knob"></span></span>
</button>

<style>
  .switch {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    text-align: left;
    font-size: 12.5px;
  }

  .track {
    position: relative;
    flex: none;
    width: 34px;
    height: 20px;
    border-radius: 10px;
    background: var(--pressed);
    transition: background 150ms;
  }

  .track.on {
    background: var(--accent);
  }

  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    transition: transform 150ms ease;
  }

  .on .knob {
    transform: translateX(14px);
  }
</style>
