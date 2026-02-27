import path from "path";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import { createViteLicensePlugin } from "rollup-license-plugin";

const host = process.env.TAURI_DEV_HOST;
const isLicenseBundle = process.env.npm_lifecycle_event === "bundle-licenses";

function formatLicenseNotices(
  packages: {
    name?: string;
    version?: string;
    license?: string;
    repository?: string;
    licenseText?: string;
  }[],
): string {
  return `${packages
    .slice()
    .sort((a, b) => {
      const aName = a.name ?? "";
      const bName = b.name ?? "";
      if (aName !== bName) {
        return aName.localeCompare(bName);
      }
      return (a.version ?? "").localeCompare(b.version ?? "");
    })
    .map(({ name, version, license, repository, licenseText }) => {
      const normalizedLicenseText = licenseText?.trim();

      return [
        `Package: ${name ?? "unknown"}@${version ?? "unknown"}`,
        `License: ${license ?? "UNKNOWN"}`,
        `Repository: ${repository ?? "N/A"}`,
        "",
        "License Text:",
        normalizedLicenseText && normalizedLicenseText.length > 0
          ? normalizedLicenseText
          : "N/A",
      ].join("\n");
    })
    .join("\n\n-----\n\n")}\n`;
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    tanstackRouter({ target: "react", autoCodeSplitting: true }),
    react({
      babel: {
        plugins: [["babel-plugin-react-compiler"]],
      },
    }),
    tailwindcss(),
    isLicenseBundle
      ? createViteLicensePlugin({
          outputFilename: false,
          additionalFiles: {
            "THIRD_PARTY_NPM.txt": formatLicenseNotices,
          },
        })
      : undefined,
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
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
