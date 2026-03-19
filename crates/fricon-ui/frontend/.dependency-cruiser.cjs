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
