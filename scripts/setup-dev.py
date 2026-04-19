from __future__ import annotations

from dev_bootstrap import (
    ProjectPaths,
    bootstrap_checkout,
    configure_workspace_database_url,
    ensure_workspace_exists,
)


def main() -> None:
    print("Setting up Fricon development environment...")
    paths = ProjectPaths.discover()

    print("1. Bootstrapping checkout-local configuration...")
    python_path, target_dir = bootstrap_checkout(paths)
    print(f"PYO3_PYTHON = {python_path}")
    if target_dir is None:
        print("Shared target dir = not configured")
    else:
        print(f"Shared target dir = {target_dir}")

    print("2. Ensuring development workspace exists...")
    created_workspace = ensure_workspace_exists(paths)
    if created_workspace:
        print(f"Created workspace via `fricon init`: {paths.fricon_workspace_path}")
    else:
        print(f"Workspace already exists: {paths.fricon_workspace_path}")

    configure_workspace_database_url(paths)
    print(
        "Configured DATABASE_URL for workspace database: "
        f"{paths.workspace_database_path}"
    )

    print("\nDevelopment environment setup completed!")
    print(f"Workspace path: {paths.fricon_workspace_path}")
    print(f"Database path: {paths.workspace_database_path}")
    print(f"Environment file: {paths.dotenv_path}")
    print(f"Cargo config: {paths.cargo_worktree_path}")
    print("\nNext steps:")
    print("- Run 'uv sync --dev' only if this task needs Python dependencies")
    print("- Run 'pnpm install' only if this task needs frontend dependencies")
    print(
        "- Run 'uv run maturin develop' before Python tests when bindings may be stale"
    )
    print("- See CONTRIBUTING.md for detailed development instructions")


if __name__ == "__main__":
    main()
