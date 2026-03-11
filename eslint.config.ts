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
  globalIgnores([
    "dist",
    "**/dist/**",
    ".venv/**",
    "site/**",
    "target/**",
    "**/node_modules/**",
    "**/src/shared/lib/bindings.ts",
    "crates/fricon-ui/frontend/src/shared/ui/**",
  ]),
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
    files: ["**/*.test.ts", "**/*.test.tsx", "**/src/shared/test/**/*"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.vitest,
      },
    },
  },
  {
    files: ["crates/fricon-ui/frontend/src/features/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@/app/**", "@/routes/**", "@/features/**"],
              message:
                "Feature files must use relative imports within the feature and may not import app, routes, or other features.",
            },
          ],
        },
      ],
    },
  },
  {
    files: [
      "crates/fricon-ui/frontend/src/app/**/*.{ts,tsx}",
      "crates/fricon-ui/frontend/src/routes/**/*.{ts,tsx}",
    ],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: [
                "@/features/*/api/**",
                "@/features/*/hooks/**",
                "@/features/*/model/**",
                "@/features/*/ui/**",
              ],
              message:
                "App and routes must import features through their public barrel exports.",
            },
          ],
        },
      ],
    },
  },
  prettier,
]);
