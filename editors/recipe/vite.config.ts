import { defineConfig } from "vite";

// Tauri の窓から読む静的な資産だけを dist/ へ出す（外へつながない）
export default defineConfig({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "dist", target: "es2022" },
});
