import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = fileURLToPath(new URL("../../", import.meta.url));
const smokeWorkspacePath = path.join(
  repoRoot,
  ".dev",
  "desktop-smoke",
  "workspace",
);

let tauriDriver;
let shuttingDown = false;

function cargoTargetDirPath() {
  const result = spawnSync(
    "cargo",
    ["metadata", "--format-version", "1", "--no-deps"],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );

  if (result.error) {
    throw new Error(
      `Failed to query cargo metadata for smoke target dir: ${result.error.message}`,
    );
  }
  if (result.status !== 0) {
    throw new Error(
      `cargo metadata failed while resolving smoke target dir: ${result.stderr}`,
    );
  }

  const metadata = JSON.parse(result.stdout);
  return metadata.target_directory;
}

const tauriBinaryPath = path.join(
  cargoTargetDirPath(),
  "debug",
  process.platform === "win32" ? "fricon-ui.exe" : "fricon-ui",
);

function runChecked(command, args, description) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
  });

  if (result.error) {
    throw new Error(`Failed to run ${description}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${description} failed with status ${result.status}`);
  }
}

function closeTauriDriver() {
  shuttingDown = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on("exit", cleanup);
  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);
  process.on("SIGHUP", cleanup);
  process.on("SIGBREAK", cleanup);
}

export const config = {
  host: "127.0.0.1",
  port: 4444,
  specs: ["./tests/desktop-smoke/specs/**/*.smoke.test.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: tauriBinaryPath,
      },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },
  onPrepare: () => {
    // This helper binary runs without `custom-protocol`, so `tauri_build::is_dev()`
    // stays true and `crates/fricon-ui/build.rs` skips the frontend bundle work.
    // The packaged app build below is the step that intentionally pays that cost.
    runChecked(
      "cargo",
      [
        "run",
        "-p",
        "fricon-ui",
        "--bin",
        "create-smoke-workspace",
        "--",
        "--force",
        smokeWorkspacePath,
      ],
      "desktop smoke fixture creation",
    );
    runChecked(
      "cargo",
      [
        "build",
        "-p",
        "fricon-ui",
        "--bin",
        "fricon-ui",
        "--features",
        "custom-protocol",
      ],
      "desktop smoke app build",
    );
  },
  beforeSession: () => {
    tauriDriver = spawn("tauri-driver", [], {
      stdio: [null, process.stdout, process.stderr],
      env: {
        ...process.env,
        FRICON_WORKSPACE: smokeWorkspacePath,
      },
    });

    tauriDriver.on("error", (error) => {
      console.error("tauri-driver error:", error);
      process.exit(1);
    });
    tauriDriver.on("exit", (code) => {
      if (!shuttingDown) {
        console.error("tauri-driver exited with code:", code);
        process.exit(1);
      }
    });
  },
  afterSession: () => {
    closeTauriDriver();
  },
};

onShutdown(() => {
  closeTauriDriver();
});
