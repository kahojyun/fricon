import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import { reactRefresh } from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import { defineConfig, globalIgnores } from "eslint/config";
import reactX from "eslint-plugin-react-x";
import reactDom from "eslint-plugin-react-dom";
import prettier from "eslint-config-prettier";

export default defineConfig([
  globalIgnores(["dist"]),
  js.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  reactHooks.configs.flat.recommended,
  reactRefresh.configs.vite({
    extraHOCs: [
      "createFileRoute",
      "createRootRoute",
      "createRootRouteWithContext",
    ],
  }),
  reactX.configs["recommended-typescript"],
  reactDom.configs.recommended,
  {
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        projectService: true,
      },
    },
  },
  {
    files: ["src/components/ui/*"],
    rules: {
      "react-refresh/only-export-components": "off",
    },
  },
  {
    files: ["**/*.test.ts", "**/*.test.tsx", "src/test/**/*"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.vitest,
      },
    },
  },
  prettier,
]);
