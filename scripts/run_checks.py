#!/usr/bin/env python3
"""Run all checks locally based on CI configuration."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def run_command(
    cmd: list[str], cwd: Path | None = None, env: dict[str, str] | None = None
) -> bool:
    """Run a command and return success status."""
    print(f"Running: {' '.join(cmd)}")
    if cwd:
        print(f"Working directory: {cwd}")

    try:
        process = subprocess.Popen(
            cmd,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            env=env,
        )

        # Stream output in real-time
        assert process.stdout is not None, (
            "stdout should be available when PIPE is used"
        )
        while True:
            output = process.stdout.readline()
            if output == "" and process.poll() is not None:
                break
            if output:
                print(output.strip())

        return_code = process.poll()
        if return_code == 0:
            print("✓ Success")
            return True
        else:
            print(f"✗ Failed with return code {return_code}")
            return False
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed: {e}")
        return False
    except FileNotFoundError as e:
        print(f"✗ Command not found: {e}")
        return False


def main():
    """Run all checks."""
    project_root = Path(__file__).parent.parent
    print(f"Project root: {project_root}")

    # Enable color output for tools that support it
    env = os.environ.copy()
    env["CARGO_TERM_COLOR"] = "always"

    checks = [
        # Dependency installation
        (["uv", "sync", "--locked", "--all-groups"], None),
        # Format checks
        (["uv", "run", "ruff", "format", "--check"], None),
        (["uv", "run", "ruff", "check"], None),
        (["cargo", "+nightly", "fmt", "--all", "--check"], None),
        (["pnpm", "run", "check"], None),
        # Build and test checks
        (["cargo", "build", "--workspace", "--locked"], None),
        (["cargo", "test", "--workspace"], None),
        (
            [
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            None,
        ),
        # Python checks
        (["uv", "run", "pytest"], None),
        (["uv", "run", "basedpyright"], None),
        (["uv", "run", "stubtest", "fricon._core"], None),
        # Documentation
        (["uv", "run", "mkdocs", "build", "-s"], None),
        # Dependency checks
        (["cargo", "deny", "--workspace", "--all-features", "check"], None),
    ]

    failed_checks: list[str] = []

    for cmd, cwd in checks:
        print(f"\n{'=' * 60}")
        # Use color environment for cargo commands
        cmd_env = env if cmd[0] == "cargo" else None
        if not run_command(cmd, cwd, cmd_env):
            failed_checks.append(" ".join(cmd))

    print(f"\n{'=' * 60}")
    if failed_checks:
        print(f"❌ {len(failed_checks)} check(s) failed:")
        for check in failed_checks:
            print(f"  - {check}")
        sys.exit(1)
    else:
        print("✅ All checks passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
