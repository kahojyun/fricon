from __future__ import annotations

import subprocess
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

    def _get_project_root(self) -> Path:
        """Find project root with __file__."""
        this_file = Path(__file__)  # <project-root>/scripts/setup-dev.py
        return this_file.parent.parent

    def write_dotenv(self) -> None:
        database_path_resolved = self.database_path.resolve()
        new_db_url_line = f"DATABASE_URL=sqlite://{database_path_resolved}\n"

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
            _ = subprocess.run(
                ["diesel", "setup"],
                cwd=fricon_path,
                check=True,
            )
            _ = subprocess.run(
                ["diesel", "migration", "run"],
                cwd=fricon_path,
                check=True,
            )
        except FileNotFoundError:
            print(
                "`diesel` not found in $PATH. Please check development requirements in "
                + "README.md."
            )
            raise


def main() -> None:
    project = Project()
    project.write_dotenv()
    project.create_dev_folder()
    project.diesel_setup()


if __name__ == "__main__":
    main()
