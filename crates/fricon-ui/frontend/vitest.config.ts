import { defineConfig, mergeConfig } from "vitest/config";

import viteConfig from "./vite.config.ts";

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: "jsdom",
      setupFiles: "./src/shared/test/setup.ts",
      css: true,
      globals: true,
      execArgv: ["--no-experimental-webstorage"],
    },
  }),
);
