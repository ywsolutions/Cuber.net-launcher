<script lang="ts">
  // Плавное раскрытие/скрытие с анимацией высоты к «auto» (порт RevealDecorator).
  // Высота анимируется через grid-template-rows 0fr → 1fr, плюс лёгкий сдвиг и проявление.
  //
  // bleed — поля под тень карточки: overflow:hidden иначе обрезал бы её.
  // Поля компенсируются отрицательными отступами и не перехватывают клики.
  import type { Snippet } from "svelte";

  let {
    open,
    anchor = "bottom",
    bleed = false,
    children,
  }: { open: boolean; anchor?: "top" | "bottom"; bleed?: boolean; children: Snippet } = $props();
</script>

<div class="reveal" class:open class:top={anchor === "top"} class:bleed inert={!open}>
  <div class="clip">
    <div class="content">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .reveal {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows 220ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .reveal.open {
    grid-template-rows: 1fr;
  }

  .clip {
    min-height: 0;
    overflow: hidden;
  }

  .bleed > .clip {
    margin: 0 -16px -30px;
    padding: 0 16px 30px;
    pointer-events: none;
  }

  .bleed .content {
    pointer-events: auto;
  }

  .content {
    opacity: 0;
    transform: translateY(10px);
    transition:
      opacity 200ms ease,
      transform 220ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .top .content {
    transform: translateY(-6px);
  }

  .open .content {
    opacity: 1;
    transform: none;
  }
</style>
