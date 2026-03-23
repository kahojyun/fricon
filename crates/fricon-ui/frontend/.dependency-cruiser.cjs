/** @type {import("dependency-cruiser").IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "app-routes-no-feature-internals",
      comment:
        "App and route modules must import features through their public barrels only.",
      severity: "error",
      from: { path: "^src/(?:app|routes)/" },
      to: { path: "^src/features/[^/]+/(?:api|hooks|model|ui)(?:/|$)" },
    },
    {
      name: "features-no-app-or-routes",
      comment: "Feature modules must not depend on app or route modules.",
      severity: "error",
      from: { path: "^src/features/([^/]+)/" },
      to: { path: "^src/(?:app|routes)/" },
    },
    {
      name: "features-no-cross-feature-dependencies",
      comment:
        "Feature modules must not depend on other features, whether by alias or relative escape.",
      severity: "error",
      from: { path: "^src/features/([^/]+)/" },
      to: {
        path: "^src/features/",
        pathNot: "^src/features/$1/",
      },
    },
    {
      name: "features-non-api-no-bindings",
      comment:
        "Feature modules may only import generated bindings from feature-local api modules.",
      severity: "error",
      from: {
        path: "^src/features/[^/]+/",
        pathNot: "^src/features/[^/]+/api(?:/|$)",
      },
      to: { path: "^src/shared/lib/bindings\\.ts$" },
    },
    {
      name: "app-routes-no-bindings",
      comment:
        "App and route modules must consume backend access through shared or feature APIs, not generated bindings directly.",
      severity: "error",
      from: { path: "^src/(?:app|routes)/" },
      to: { path: "^src/shared/lib/bindings\\.ts$" },
    },
    {
      name: "shared-no-bindings-outside-tauri",
      comment:
        "Shared modules may only import generated bindings from the minimal shared Tauri bridge.",
      severity: "error",
      from: {
        path: "^src/shared/",
        pathNot: "^src/shared/lib/tauri\\.ts$",
      },
      to: { path: "^src/shared/lib/bindings\\.ts$" },
    },
    {
      name: "runtime-no-direct-tauri-apis-outside-bridges",
      comment:
        "Runtime modules must access Tauri APIs and plugins through feature-local api modules or the shared bridge files.",
      severity: "error",
      from: {
        path: "^src/",
        pathNot:
          "^(?:src/features/[^/]+/api(?:/|$)|src/shared/lib/bindings\\.ts$|src/shared/lib/tauri\\.ts$|src/.+(?:\\.test\\.(?:ts|tsx)|/test-utils\\.tsx?)$)",
      },
      to: { path: "^@tauri-apps/(?:api(?:/|$)|plugin-)" },
    },
  ],
  options: {
    doNotFollow: {
      path: "node_modules",
      dependencyTypes: [
        "npm",
        "npm-dev",
        "npm-optional",
        "npm-peer",
        "npm-bundled",
        "npm-no-pkg",
      ],
    },
    tsConfig: {
      fileName: "./tsconfig.json",
    },
    enhancedResolveOptions: {
      extensions: [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"],
    },
    skipAnalysisNotInRules: true,
  },
};
