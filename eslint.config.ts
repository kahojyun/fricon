// https://eslint.vuejs.org/user-guide/#example-configuration-with-typescript-eslint-and-prettier
import { defineConfig } from "eslint/config";
import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginVue from "eslint-plugin-vue";
import globals from "globals";
import typescriptEslint from "typescript-eslint";

export default defineConfig(
  {
    ignores: [
      "*.d.ts",
      "**/coverage",
      "**/dist",
      "**/.venv",
      "**/target",
      "**/site",
    ],
  },
  {
    extends: [
      eslint.configs.recommended,
      ...typescriptEslint.configs.recommendedTypeChecked,
      ...typescriptEslint.configs.stylisticTypeChecked,
      ...eslintPluginVue.configs["flat/recommended"],
    ],
    files: ["**/*.{ts,vue}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: globals.browser,
      parserOptions: {
        extraFileExtensions: [".vue"],
        projectService: true,
        parser: typescriptEslint.parser,
      },
    },
    rules: {
      // your rules
    },
  },
  eslintConfigPrettier,
);
