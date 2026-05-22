import { defineConfig } from "vite";

export default defineConfig({
  root: "tauri/src",
  base: "./",
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: "tauri/dist",
    emptyOutDir: true,
  },
});