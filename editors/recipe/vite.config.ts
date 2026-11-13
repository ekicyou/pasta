import { defineConfig } from "vite";
import jsLicenses from "./scripts/js-licenses.mjs";

// Tauri の窓から読む静的な資産だけを dist/ へ出す（外へつながない）
export default defineConfig({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "dist", target: "es2022" },
  plugins: [jsLicenses()],
  // worker は別の束ね。集めた根は本体の plugin が書く
  worker: { plugins: () => [jsLicenses({ write: false })] },
});
