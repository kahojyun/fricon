from __future__ import annotations

import re
import subprocess
import sys
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path


DEV_FOLDER = ".dev"
WORKSPACE_DIR = "ws"
WORKSPACE_DB = "fricon.sqlite3"
WORKSPACE_METADATA = ".fricon_workspace.json"
WORKSPACE_LOCK_FILE = ".fricon.lock"
MANAGED_ENV_KEYS = ("FRICON_WORKSPACE", "DATABASE_URL")


@dataclass(frozen=True)
class ProjectPaths:
    root: Path
    dev_folder: Path
    dotenv_path: Path
    fricon_workspace_path: Path
    workspace_database_path: Path
    workspace_metadata_path: Path
    cargo_worktree_path: Path

    @classmethod
    def discover(cls) -> ProjectPaths:
        root = Path(__file__).resolve().parent.parent
        dev_folder = root / DEV_FOLDER
        return cls(
            root=root,
            dev_folder=dev_folder,
            dotenv_path=root / ".env",
            fricon_workspace_path=dev_folder / WORKSPACE_DIR,
            workspace_database_path=dev_folder / WORKSPACE_DIR / WORKSPACE_DB,
            workspace_metadata_path=dev_folder / WORKSPACE_DIR / WORKSPACE_METADATA,
            cargo_worktree_path=root / ".cargo" / "config.worktree.toml",
        )


def bootstrap_checkout(paths: ProjectPaths) -> tuple[Path, Path | None]:
    ensure_dev_folder(paths)
    env_updates = {
        "FRICON_WORKSPACE": paths.fricon_workspace_path.resolve().as_posix(),
    }
    if is_valid_workspace(paths):
        env_updates["DATABASE_URL"] = (
            f"sqlite://{paths.workspace_database_path.resolve().as_posix()}"
        )

    update_managed_env_file(paths.dotenv_path, env_updates)
    python_path = resolve_pyo3_python(paths.root)
    target_dir = resolve_shared_target_dir(paths.root)
    write_worktree_cargo_config(paths.cargo_worktree_path, python_path, target_dir)
    return python_path, target_dir


def ensure_workspace_exists(paths: ProjectPaths) -> bool:
    if is_valid_workspace(paths):
        return False

    if paths.fricon_workspace_path.exists() and not can_initialize_workspace(paths):
        raise RuntimeError(
            "Workspace path exists but is not a valid Fricon workspace: "
            f"{paths.fricon_workspace_path}"
        )

    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "fricon-cli",
            "--",
            "init",
            paths.fricon_workspace_path.as_posix(),
        ],
        cwd=paths.root,
        check=True,
    )
    if not is_valid_workspace(paths):
        raise RuntimeError(
            "Workspace creation completed, but the resulting path is not a valid "
            f"Fricon workspace: {paths.fricon_workspace_path}"
        )
    return True


def configure_workspace_database_url(paths: ProjectPaths) -> None:
    if not is_valid_workspace(paths):
        raise RuntimeError(
            "Cannot configure DATABASE_URL before the workspace is valid: "
            f"{paths.fricon_workspace_path}"
        )

    update_managed_env_file(
        paths.dotenv_path,
        {
            "FRICON_WORKSPACE": paths.fricon_workspace_path.resolve().as_posix(),
            "DATABASE_URL": f"sqlite://{paths.workspace_database_path.resolve().as_posix()}",
        },
    )


def ensure_dev_folder(paths: ProjectPaths) -> None:
    paths.dev_folder.mkdir(exist_ok=True)
    gitignore_path = paths.dev_folder / ".gitignore"
    if not gitignore_path.exists():
        gitignore_path.write_text("*\n", encoding="utf-8")


def update_managed_env_file(path: Path, updates: Mapping[str, str]) -> None:
    lines = (
        path.read_text(encoding="utf-8").splitlines(keepends=True)
        if path.exists()
        else []
    )
    updated_lines: list[str] = []
    seen: set[str] = set()

    for line in lines:
        matching_key = next(
            (key for key in MANAGED_ENV_KEYS if line.startswith(f"{key}=")), None
        )
        if matching_key is None:
            updated_lines.append(line)
            continue

        if matching_key in seen:
            continue

        if matching_key in updates:
            updated_lines.append(f"{matching_key}={updates[matching_key]}\n")
        seen.add(matching_key)

    for key, value in updates.items():
        if key not in seen:
            if updated_lines and not updated_lines[-1].endswith("\n"):
                updated_lines.append("\n")
            updated_lines.append(f"{key}={value}\n")

    path.write_text("".join(updated_lines), encoding="utf-8")


def is_valid_workspace(paths: ProjectPaths) -> bool:
    return paths.workspace_metadata_path.is_file()


def can_initialize_workspace(paths: ProjectPaths) -> bool:
    workspace_root = paths.fricon_workspace_path
    if not workspace_root.exists():
        return True
    if not workspace_root.is_dir():
        return False

    ignored_entries = {WORKSPACE_LOCK_FILE}
    for entry in workspace_root.iterdir():
        if entry.name not in ignored_entries:
            return False
    return True


def resolve_pyo3_python(root: Path) -> Path:
    main_checkout_python = resolve_main_checkout_pyo3_python(root)
    if main_checkout_python is not None:
        return main_checkout_python

    if sys.platform == "win32":
        worktree_python = root / ".venv" / "Scripts" / "python.exe"
    else:
        worktree_python = root / ".venv" / "bin" / "python"

    if worktree_python.exists():
        return worktree_python

    return Path(sys.executable)


def resolve_main_checkout_pyo3_python(root: Path) -> Path | None:
    git_dir = resolve_git_path(root, "--git-dir")
    common_dir = resolve_git_path(root, "--git-common-dir")
    if git_dir is None or common_dir is None or git_dir == common_dir:
        return None

    main_checkout_root = common_dir.parent
    config_path = main_checkout_root / ".cargo" / "config.worktree.toml"
    configured_python = read_configured_pyo3_python(config_path)
    if configured_python is None or not configured_python.exists():
        return None

    return configured_python


def resolve_git_path(root: Path, arg: str) -> Path | None:
    try:
        output = subprocess.run(
            ["git", "rev-parse", "--path-format=absolute", arg],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except (FileNotFoundError, subprocess.CalledProcessError):
        return None

    if not output:
        return None

    return Path(output)


def read_configured_pyo3_python(path: Path) -> Path | None:
    if not path.is_file():
        return None

    match = re.search(
        r'^PYO3_PYTHON\s*=\s*"(?P<python>[^"]+)"\s*$',
        path.read_text(encoding="utf-8"),
        re.MULTILINE,
    )
    if match is None:
        return None

    return Path(match.group("python"))


def resolve_shared_target_dir(root: Path) -> Path | None:
    repo_target_dir = resolve_repo_target_dir(root)
    if repo_target_dir is None:
        return None

    try:
        repo_target_dir.mkdir(parents=True, exist_ok=True)
    except PermissionError:
        return None

    return repo_target_dir


def resolve_repo_target_dir(root: Path) -> Path | None:
    common_dir_path = resolve_git_path(root, "--git-common-dir")
    if common_dir_path is None:
        return None

    if common_dir_path.name == ".git":
        return common_dir_path.parent / "target"

    return None


def write_worktree_cargo_config(
    path: Path, python_path: Path, target_dir: Path | None
) -> None:
    path.parent.mkdir(exist_ok=True)
    lines = [
        "# Generated by scripts/bootstrap-checkout.py.\n",
        "# This file is checkout-local and safe to regenerate.\n\n",
        "[env]\n",
        f'PYO3_PYTHON = "{python_path.as_posix()}"\n',
    ]
    if target_dir is not None:
        lines.extend(
            [
                "\n",
                "[build]\n",
                f'target-dir = "{target_dir.resolve().as_posix()}"\n',
            ]
        )
    path.write_text("".join(lines), encoding="utf-8")
