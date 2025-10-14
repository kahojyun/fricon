from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from typing import final


@final
class Project:
    DEV_FOLDER = ".dev"
    TEST_DB = "testdb.sqlite3"

    def __init__(self):
        self.root = self._get_project_root()
        self.dev_folder = self.root / self.DEV_FOLDER
        self.dotenv_path = self.root / ".env"
        self.database_path = self.dev_folder / self.TEST_DB
        self.cargo_config_path = self.root / ".cargo" / "config.toml"

    def _get_project_root(self) -> Path:
        """Find project root with __file__."""
        this_file = Path(__file__)  # <project-root>/scripts/setup-dev.py
        return this_file.parent.parent

    def write_dotenv(self) -> None:
        database_path_resolved = self.database_path.resolve()
        # Convert Windows backslashes to forward slashes for better compatibility
        database_path_unix = database_path_resolved.as_posix()
        new_db_url_line = f"DATABASE_URL=sqlite://{database_path_unix}\n"

        lines: list[str] = []
        if self.dotenv_path.exists():
            with self.dotenv_path.open("r") as f:
                lines = f.readlines()

        updated_lines: list[str] = []
        db_url_found = False
        for line in lines:
            if line.startswith("DATABASE_URL="):
                updated_lines.append(new_db_url_line)
                db_url_found = True
            else:
                updated_lines.append(line)

        if not db_url_found:
            updated_lines.append(new_db_url_line)

        with self.dotenv_path.open("w") as f:
            f.writelines(updated_lines)

    def create_dev_folder(self) -> None:
        try:
            self.dev_folder.mkdir()
        except FileExistsError:
            print(f"Folder {self.DEV_FOLDER} already exists. Ignoring.")
            return
        gitignore = self.dev_folder / ".gitignore"
        with gitignore.open("w") as f:
            _ = f.write("*\n")

    def diesel_setup(self) -> None:
        fricon_path = self.root / "crates" / "fricon"
        try:
            print("Setting up database...")
            _ = subprocess.run(
                ["diesel", "setup"],
                cwd=fricon_path,
                check=True,
            )
            print("Running database migrations...")
            _ = subprocess.run(
                ["diesel", "migration", "run"],
                cwd=fricon_path,
                check=True,
            )
            print("Database setup completed successfully!")
        except FileNotFoundError:
            print(
                "ERROR: `diesel` not found in $PATH. Please install diesel_cli:\n"
                + "  cargo install diesel_cli --no-default-features --features sqlite\n"
                + "For more information, see CONTRIBUTING.md or README.md."
            )
            raise
        except subprocess.CalledProcessError as e:
            print(f"ERROR: Database setup failed with exit code {e.returncode}")
            print("Make sure you have SQLite3 development libraries installed.")
            raise

    def get_venv_python_path(self) -> Path:
        """Get the absolute path to the Python executable in uv venv."""
        # uv venv creates .venv folder in project root
        venv_path = self.root / ".venv"

        # Check if venv exists
        if not venv_path.exists():
            print("Warning: .venv directory not found.")
            print("Please run 'uv sync --dev' to create the virtual environment.")

        # Determine python executable name based on platform
        if sys.platform == "win32":
            python_exe = "python.exe"
            python_path = venv_path / "Scripts" / python_exe
        else:
            python_exe = "python"
            python_path = venv_path / "bin" / python_exe

        # Return the venv path, not the resolved system path
        # This ensures PYO3_PYTHON points to the venv python executable
        return python_path

    def setup_cargo_config(self) -> None:
        """Setup .cargo/config.toml with PYO3_PYTHON environment variable."""
        python_path = self.get_venv_python_path()

        # Ensure .cargo directory exists
        cargo_dir = self.cargo_config_path.parent
        cargo_dir.mkdir(exist_ok=True)

        # Create config content
        config_content = f"""[env]
PYO3_PYTHON = "{python_path.as_posix()}"
"""

        # Write config file
        with self.cargo_config_path.open("w") as f:
            _ = f.write(config_content)

        print(f"Cargo config updated with PYO3_PYTHON = {python_path}")


def main() -> None:
    print("Setting up Fricon development environment...")
    project = Project()

    print("1. Setting up environment variables...")
    project.write_dotenv()

    print("2. Creating development directory...")
    project.create_dev_folder()

    print("3. Setting up Cargo configuration...")
    project.setup_cargo_config()

    print("4. Setting up database...")
    project.diesel_setup()

    print("\nDevelopment environment setup completed!")
    print(f"Database path: {project.database_path}")
    print(f"Environment file: {project.dotenv_path}")
    print(f"Cargo config: {project.cargo_config_path}")
    print("\nNext steps:")
    print("- Run 'uv sync --dev' to set up Python environment")
    print("- Run 'cargo build' to build Rust components")
    print("- See CONTRIBUTING.md for detailed development instructions")


if __name__ == "__main__":
    main()
