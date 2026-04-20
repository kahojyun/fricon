/// <reference types="vitest/config" />

import { fileURLToPath } from "node:url";
import babel from "@rolldown/plugin-babel";
import { playwright } from "@vitest/browser-playwright";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";

const host = process.env.TAURI_DEV_HOST;
const isLicenseBundle = process.env.npm_lifecycle_event === "bundle-licenses";
const isVitest = process.env.VITEST === "true";
const isBrowserHeaded =
  process.env.VITEST_BROWSER_HEADED === "true" ||
  process.env.npm_lifecycle_event === "test:browser:headed";
const srcDir = fileURLToPath(new URL("./src", import.meta.url));

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    tanstackRouter({ target: "react", autoCodeSplitting: !isVitest }),
    react(),
    babel({
      presets: [reactCompilerPreset()],
    }),
    tailwindcss(),
  ],
  build: {
    license: isLicenseBundle ? { fileName: "THIRD_PARTY_NPM.md" } : false,
  },
  resolve: {
    alias: {
      "@": srcDir,
    },
  },
  test: {
    projects: [
      {
        resolve: {
          alias: {
            "@": srcDir,
          },
        },
        test: {
          name: "unit",
          include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
          exclude: ["src/**/*.browser.test.tsx"],
          environment: "jsdom",
          setupFiles: "./src/shared/test/setup.ts",
          css: true,
          globals: true,
          execArgv: ["--no-experimental-webstorage"],
        },
      },
      {
        resolve: {
          alias: {
            "@": srcDir,
          },
        },
        test: {
          name: "browser",
          include: ["src/**/*.browser.test.tsx"],
          setupFiles: "./src/shared/test/browser/setup.ts",
          css: true,
          globals: true,
          browser: {
            enabled: true,
            provider: playwright({
              launchOptions: {
                channel: "chromium",
              },
              persistentContext: true,
            }),
            headless: !isBrowserHeaded,
            instances: [{ browser: "chromium" }],
          },
        },
      },
    ],
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
