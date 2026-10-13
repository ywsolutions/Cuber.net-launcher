import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Настройки рассчитаны на Tauri: фиксированный порт и без очистки консоли,
// чтобы были видны ошибки Rust.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    target: "es2022",
    sourcemap: false,
  },
});
