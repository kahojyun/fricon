use std::{env, process::Command};

use tauri_utils::config::FrontendDist::Directory;

/// Find the pnpm executable by trying multiple possible names in order
fn find_pnpm_executable() -> &'static str {
    if cfg!(windows) {
        // Try multiple pnpm executable names on Windows
        let candidates = ["pnpm", "pnpm.cmd", "pnpm.exe"];

        for candidate in candidates {
            if Command::new(candidate)
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return candidate;
            }
        }

        // If none found, default to pnpm.cmd (most common on Windows)
        "pnpm.cmd"
    } else {
        "pnpm"
    }
}

fn main() {
    tauri_build::build();
    if !tauri_build::is_dev() {
        let target_triple = env::var("TARGET").unwrap();
        let target = tauri_utils::platform::Target::from_triple(&target_triple);
        let (config, _) =
            tauri_utils::config::parse(target, env::current_dir().unwrap().join("tauri.conf.json"))
                .expect("Failed to parse tauri config");
        let Directory(frontend_dist) = config.build.frontend_dist.unwrap() else {
            panic!("Invalid frontend distribution type");
        };
        let frontend_root = frontend_dist.parent().unwrap();
        let need_watch = [
            "src",
            "index.html",
            "package.json",
            "tsconfig.json",
            "tsconfig.app.json",
            "vite.config.ts",
        ];
        for item in need_watch {
            println!(
                "cargo::rerun-if-changed={}",
                frontend_root.join(item).display()
            );
        }
        let pnpm = find_pnpm_executable();
        Command::new(pnpm)
            .current_dir(frontend_root)
            .arg("install")
            .status()
            .expect("Failed to run pnpm install");
        Command::new(pnpm)
            .current_dir(frontend_root)
            .arg("run")
            .arg("build")
            .status()
            .expect("Failed to run pnpm build");
    }
}
