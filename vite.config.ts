import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: host || "127.0.0.1",
    port: 1420,
    strictPort: true,
    hmr: host
      ? {
          host,
          port: 1421,
        }
      : "localhost",
    watch: {
      // Ignore heavy folders to keep dev server responsive
      ignored: [
        "**/.git/**",
        "**/node_modules/**",
        "**/target/**"
      ]
    }
  },
  build: {
    target: ["es2021", "chrome100"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
