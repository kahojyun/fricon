from __future__ import annotations

from dev_bootstrap import ProjectPaths, bootstrap_checkout


def main() -> None:
    print("Bootstrapping Fricon checkout...")
    paths = ProjectPaths.discover()
    python_path, target_dir = bootstrap_checkout(paths)

    print(f"Repo root: {paths.root}")
    print(f"Environment file: {paths.dotenv_path}")
    print(f"Workspace path: {paths.fricon_workspace_path}")
    print(
        f"Workspace database (configured by setup-dev): {paths.workspace_database_path}"
    )
    print(f"Cargo worktree config: {paths.cargo_worktree_path}")
    print(f"PYO3_PYTHON: {python_path}")
    if target_dir is None:
        print("Shared target dir: not configured")
    else:
        print(f"Shared target dir: {target_dir}")
    print("")
    print("Next steps:")
    print("- Install only the dependencies your task needs")
    print("- Run uv sync / pnpm install lazily based on the slice you change")
    print("- Run narrower checks instead of full-workspace setup when possible")


if __name__ == "__main__":
    main()
