#!/usr/bin/env python3
"""Run all checks locally based on CI configuration."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def run_command(cmd: list[str]) -> bool:
    """Run a command and return success status."""
    print(f"Running: {' '.join(cmd)}")

    # Use shell=True for more realistic output behavior
    shell_cmd = " ".join(cmd)
    try:
        _ = subprocess.run(
            shell_cmd,
            shell=True,
            check=True,
            text=True,
        )
        print("✓ Success")
        return True
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed with return code {e.returncode}")
        return False
    except FileNotFoundError as e:
        print(f"✗ Command not found: {e}")
        return False


def main():
    """Run all checks."""
    project_root = Path(__file__).parent.parent
    print(f"Project root: {project_root}")

    checks = [
        # Dependency installation
        ["uv", "sync", "--locked", "--all-groups"],
        ["uv", "run", "maturin", "develop"],
        # Format checks
        ["uv", "run", "ruff", "format", "--check"],
        ["uv", "run", "ruff", "check"],
        ["cargo", "+nightly", "fmt", "--all", "--check"],
        ["pnpm", "run", "check"],
        # Build and test checks
        ["cargo", "build", "--workspace", "--locked"],
        ["cargo", "test", "--workspace"],
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
        # Python checks
        ["uv", "run", "pytest"],
        ["uv", "run", "basedpyright"],
        ["uv", "run", "stubtest", "fricon._core"],
        # Documentation
        ["uv", "run", "mkdocs", "build", "-s"],
        # Dependency checks
        ["cargo", "deny", "--workspace", "--all-features", "check"],
    ]

    failed_checks: list[str] = []

    for cmd in checks:
        print(f"\n{'=' * 60}")
        if not run_command(cmd):
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
