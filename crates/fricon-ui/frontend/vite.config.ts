import { fileURLToPath } from "node:url";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import { createViteLicensePlugin } from "rollup-license-plugin";

const host = process.env.TAURI_DEV_HOST;
const isLicenseBundle = process.env.npm_lifecycle_event === "bundle-licenses";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    tanstackRouter({ target: "react", autoCodeSplitting: true }),
    react(),
    // @ts-expect-error The published types currently reject the documented presets-only shape.
    babel({
      presets: [reactCompilerPreset()],
    }),
    tailwindcss(),
    isLicenseBundle
      ? createViteLicensePlugin({
          outputFilename: "THIRD_PARTY_NPM.json",
        })
      : undefined,
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host ?? false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
  },
});
