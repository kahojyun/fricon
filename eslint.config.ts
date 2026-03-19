import js from "@eslint/js";
import eslintReact from "@eslint-react/eslint-plugin";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import { reactRefresh } from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import { defineConfig, globalIgnores } from "eslint/config";
import prettier from "eslint-config-prettier";

const compilerIncompatibleHookImportPaths = [
  {
    name: "@tanstack/react-table",
    importNames: ["useReactTable"],
    message:
      "Keep compiler-incompatible table hooks in a leaf render component instead of a custom hook file.",
  },
  {
    name: "@tanstack/react-virtual",
    importNames: ["useVirtualizer", "useWindowVirtualizer"],
    message:
      "Keep compiler-incompatible virtualizer hooks in a leaf render component instead of a custom hook file.",
  },
];

export default defineConfig([
  globalIgnores([
    "dist",
    "**/dist/**",
    ".venv/**",
    "site/**",
    "target/**",
    "**/node_modules/**",
    "crates/fricon-ui/frontend/.dependency-cruiser.cjs",
    "**/src/shared/lib/bindings.ts",
    // shadcn/ui source files live here; keep repo-owned shared components elsewhere so they remain linted.
    "crates/fricon-ui/frontend/src/shared/ui/**",
  ]),
  js.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  eslintReact.configs["recommended-type-checked"],
  reactHooks.configs.flat.recommended,
  reactRefresh.configs.vite({
    extraHOCs: [
      "createFileRoute",
      "createRootRoute",
      "createRootRouteWithContext",
    ],
  }),
  {
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        projectService: true,
      },
    },
    rules: {
      "@typescript-eslint/no-deprecated": "error",
      // `react-hooks/todo` surfaces React Compiler bailout diagnostics that the
      // default recommended presets do not enable. Keep it on so unsupported
      // syntax such as some `try/catch/finally` patterns shows up in normal lint.
      "react-hooks/todo": "warn",
    },
  },
  {
    files: [
      "**/*.test.ts",
      "**/*.test.tsx",
      "**/test-utils.ts",
      "**/test-utils.tsx",
      "**/src/shared/test/**/*",
    ],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.vitest,
      },
    },
    rules: {
      "@eslint-react/component-hook-factories": "off",
      "react-refresh/only-export-components": "off",
    },
  },
  {
    files: ["crates/fricon-ui/frontend/src/**/use*.{ts,tsx}"],
    ignores: [
      "**/*.test.ts",
      "**/*.test.tsx",
      "**/test-utils.ts",
      "**/test-utils.tsx",
    ],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: compilerIncompatibleHookImportPaths,
        },
      ],
    },
  },
  prettier,
]);
