#!/usr/bin/env python3
"""
Development utilities for Fricon project.
This script provides common development tasks in a convenient interface.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path
from typing import NoReturn


def run_command(cmd: list[str], cwd: Path | None = None, description: str = "") -> bool:
    """Run a command and return True if successful."""
    if description:
        print(f"→ {description}")
    print(f"  Running: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, cwd=cwd, check=True)
        return result.returncode == 0
    except subprocess.CalledProcessError as e:
        print(f"  ✗ Command failed with exit code {e.returncode}")
        return False
    except FileNotFoundError:
        print(f"  ✗ Command not found: {cmd[0]}")
        return False


def get_project_root() -> Path:
    """Get the project root directory."""
    return Path(__file__).parent.parent


def setup_env(args: argparse.Namespace) -> bool:
    """Set up development environment."""
    root = get_project_root()
    setup_script = root / "scripts" / "setup-dev.py"
    
    print("Setting up development environment...")
    return run_command([sys.executable, str(setup_script)], description="Running setup script")


def build_rust(args: argparse.Namespace) -> bool:
    """Build Rust components."""
    root = get_project_root()
    
    print("Building Rust components...")
    cmd = ["cargo", "build"]
    if args.release:
        cmd.append("--release")
    if args.package:
        cmd.extend(["-p", args.package])
    
    return run_command(cmd, cwd=root, description="Building Rust crates")


def build_python(args: argparse.Namespace) -> bool:
    """Build Python components."""
    root = get_project_root()
    
    print("Building Python components...")
    success = True
    
    # Install Python dependencies
    success &= run_command(["uv", "sync", "--dev"], cwd=root, description="Installing Python dependencies")
    
    # Build Python extension
    success &= run_command(["uv", "run", "maturin", "develop"], cwd=root, description="Building Python extension")
    
    return success


def build_frontend(args: argparse.Namespace) -> bool:
    """Build frontend components."""
    root = get_project_root()
    frontend_dir = root / "crates" / "fricon-ui" / "frontend"
    
    print("Building frontend components...")
    success = True
    
    # Install dependencies
    success &= run_command(["pnpm", "install"], cwd=frontend_dir, description="Installing frontend dependencies")
    
    # Build
    cmd = ["pnpm", "run", "dev" if args.dev else "build"]
    success &= run_command(cmd, cwd=frontend_dir, description="Building frontend")
    
    return success


def test_rust(args: argparse.Namespace) -> bool:
    """Run Rust tests."""
    root = get_project_root()
    
    print("Running Rust tests...")
    cmd = ["cargo", "test"]
    if args.package:
        cmd.extend(["-p", args.package])
    
    return run_command(cmd, cwd=root, description="Running Rust tests")


def test_python(args: argparse.Namespace) -> bool:
    """Run Python tests."""
    root = get_project_root()
    
    print("Running Python tests...")
    cmd = ["uv", "run", "pytest"]
    if args.coverage:
        cmd.extend(["--cov=fricon"])
    
    return run_command(cmd, cwd=root, description="Running Python tests")


def test_frontend(args: argparse.Namespace) -> bool:
    """Run frontend tests."""
    root = get_project_root()
    frontend_dir = root / "crates" / "fricon-ui" / "frontend"
    
    print("Running frontend tests...")
    return run_command(["pnpm", "run", "test"], cwd=frontend_dir, description="Running frontend tests")


def lint_all(args: argparse.Namespace) -> bool:
    """Run all linting checks."""
    root = get_project_root()
    frontend_dir = root / "crates" / "fricon-ui" / "frontend"
    
    print("Running linting checks...")
    success = True
    
    # Rust
    success &= run_command(["cargo", "fmt", "--all", "--check"], cwd=root, description="Checking Rust formatting")
    success &= run_command(["cargo", "clippy", "--all-targets", "--all-features"], cwd=root, description="Running Rust linter")
    
    # Python
    success &= run_command(["uv", "run", "ruff", "check", "."], cwd=root, description="Running Python linter")
    success &= run_command(["uv", "run", "ruff", "format", "--check", "."], cwd=root, description="Checking Python formatting")
    
    # Frontend
    success &= run_command(["pnpm", "run", "lint"], cwd=frontend_dir, description="Running frontend linter")
    
    return success


def fix_all(args: argparse.Namespace) -> bool:
    """Fix all auto-fixable issues."""
    root = get_project_root()
    frontend_dir = root / "crates" / "fricon-ui" / "frontend"
    
    print("Fixing auto-fixable issues...")
    success = True
    
    # Rust
    success &= run_command(["cargo", "fmt", "--all"], cwd=root, description="Formatting Rust code")
    
    # Python
    success &= run_command(["uv", "run", "ruff", "check", ".", "--fix"], cwd=root, description="Fixing Python issues")
    success &= run_command(["uv", "run", "ruff", "format", "."], cwd=root, description="Formatting Python code")
    
    # Frontend
    success &= run_command(["pnpm", "run", "format"], cwd=frontend_dir, description="Formatting frontend code")
    
    return success


def clean_all(args: argparse.Namespace) -> bool:
    """Clean build artifacts."""
    root = get_project_root()
    frontend_dir = root / "crates" / "fricon-ui" / "frontend"
    
    print("Cleaning build artifacts...")
    success = True
    
    # Rust
    success &= run_command(["cargo", "clean"], cwd=root, description="Cleaning Rust artifacts")
    
    # Python
    cache_dirs = [".pytest_cache", "__pycache__", "*.egg-info", ".coverage"]
    for pattern in cache_dirs:
        import glob
        for path in glob.glob(str(root / "**" / pattern), recursive=True):
            try:
                import shutil
                if Path(path).is_dir():
                    shutil.rmtree(path)
                else:
                    Path(path).unlink()
                print(f"  Removed: {path}")
            except Exception as e:
                print(f"  Failed to remove {path}: {e}")
    
    # Frontend
    node_modules = frontend_dir / "node_modules"
    dist_dir = frontend_dir / "dist"
    for dir_path in [node_modules, dist_dir]:
        if dir_path.exists():
            import shutil
            shutil.rmtree(dir_path)
            print(f"  Removed: {dir_path}")
    
    return success


def main() -> NoReturn:
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Fricon development utilities")
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Setup command
    setup_parser = subparsers.add_parser("setup", help="Set up development environment")
    setup_parser.set_defaults(func=setup_env)
    
    # Build commands
    build_parser = subparsers.add_parser("build", help="Build components")
    build_subparsers = build_parser.add_subparsers(dest="build_target", help="Build targets")
    
    rust_parser = build_subparsers.add_parser("rust", help="Build Rust components")
    rust_parser.add_argument("--release", action="store_true", help="Build in release mode")
    rust_parser.add_argument("--package", "-p", help="Build specific package")
    rust_parser.set_defaults(func=build_rust)
    
    python_parser = build_subparsers.add_parser("python", help="Build Python components")
    python_parser.set_defaults(func=build_python)
    
    frontend_parser = build_subparsers.add_parser("frontend", help="Build frontend components")
    frontend_parser.add_argument("--dev", action="store_true", help="Start development server")
    frontend_parser.set_defaults(func=build_frontend)
    
    # Test commands
    test_parser = subparsers.add_parser("test", help="Run tests")
    test_subparsers = test_parser.add_subparsers(dest="test_target", help="Test targets")
    
    test_rust_parser = test_subparsers.add_parser("rust", help="Run Rust tests")
    test_rust_parser.add_argument("--package", "-p", help="Test specific package")
    test_rust_parser.set_defaults(func=test_rust)
    
    test_python_parser = test_subparsers.add_parser("python", help="Run Python tests")
    test_python_parser.add_argument("--coverage", action="store_true", help="Generate coverage report")
    test_python_parser.set_defaults(func=test_python)
    
    test_frontend_parser = test_subparsers.add_parser("frontend", help="Run frontend tests")
    test_frontend_parser.set_defaults(func=test_frontend)
    
    # Lint and fix commands
    lint_parser = subparsers.add_parser("lint", help="Run linting checks")
    lint_parser.set_defaults(func=lint_all)
    
    fix_parser = subparsers.add_parser("fix", help="Fix auto-fixable issues")
    fix_parser.set_defaults(func=fix_all)
    
    # Clean command
    clean_parser = subparsers.add_parser("clean", help="Clean build artifacts")
    clean_parser.set_defaults(func=clean_all)
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        sys.exit(1)
    
    if not hasattr(args, 'func'):
        if args.command == "build" and not args.build_target:
            build_parser.print_help()
        elif args.command == "test" and not args.test_target:
            test_parser.print_help()
        else:
            parser.print_help()
        sys.exit(1)
    
    success = args.func(args)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()