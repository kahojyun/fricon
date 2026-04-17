from __future__ import annotations

from dev_bootstrap import (
    ProjectPaths,
    bootstrap_checkout,
    configure_workspace_database_url,
    ensure_workspace_exists,
)


class Project:
    def __init__(self):
        self.paths = ProjectPaths.discover()
        self.root = self.paths.root
        self.dev_folder = self.paths.dev_folder
        self.dotenv_path = self.paths.dotenv_path
        self.workspace_path = self.paths.fricon_workspace_path
        self.database_path = self.paths.workspace_database_path
        self.cargo_config_path = self.paths.cargo_worktree_path


def main() -> None:
    print("Setting up Fricon development environment...")
    project = Project()

    print("1. Bootstrapping checkout-local configuration...")
    python_path, target_dir = bootstrap_checkout(project.paths)
    print(f"PYO3_PYTHON = {python_path}")
    print(f"Shared target dir = {target_dir}")

    print("2. Ensuring development workspace exists...")
    created_workspace = ensure_workspace_exists(project.paths)
    if created_workspace:
        print(f"Created workspace via `fricon init`: {project.workspace_path}")
    else:
        print(f"Workspace already exists: {project.workspace_path}")

    configure_workspace_database_url(project.paths)
    print(f"Configured DATABASE_URL for workspace database: {project.database_path}")

    print("\nDevelopment environment setup completed!")
    print(f"Workspace path: {project.workspace_path}")
    print(f"Database path: {project.database_path}")
    print(f"Environment file: {project.dotenv_path}")
    print(f"Cargo config: {project.cargo_config_path}")
    print("\nNext steps:")
    print("- Run 'uv sync --dev' only if this task needs Python dependencies")
    print("- Run 'pnpm install' only if this task needs frontend dependencies")
    print(
        "- Run 'uv run maturin develop' before Python tests when bindings may be stale"
    )
    print("- See CONTRIBUTING.md for detailed development instructions")


if __name__ == "__main__":
    main()
