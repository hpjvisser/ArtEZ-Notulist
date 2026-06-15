import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri verwacht een vaste poort en faalt als die niet beschikbaar is.
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Voorkom dat Vite Rust-fouten verbergt.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Negeer de Rust-backend; tauri zorgt voor herbouw.
      ignored: ["**/src-tauri/**"],
    },
  },
}));
