import subprocess
from pathlib import Path

DEV_FOLDER = ".dev"
TEST_DB = "testdb.sqlite3"
MIGRATIONS_PATH = "crates/fricon/migrations"


def get_project_root() -> Path:
    """Find project root with __file__."""
    this_file = Path(__file__)  # <project-root>/scripts/setup-dev.py
    return this_file.parent.parent


def write_dotenv() -> None:
    dotenv_path = get_project_root() / ".env"
    if dotenv_path.exists():
        raise RuntimeError("A .env file already exists.")
    with dotenv_path.open("w") as f:
        _ = f.write(f"DATABASE_URL=sqlite://{DEV_FOLDER}/{TEST_DB}\n")


def create_dev_folder() -> None:
    dev_folder = get_project_root() / DEV_FOLDER
    dev_folder.mkdir()  # Raises FileExistsError if already exists.
    gitignore = dev_folder / ".gitignore"
    with gitignore.open("w") as f:
        _ = f.write("*\n")


def diesel_setup() -> None:
    try:
        _ = subprocess.run(
            ["diesel", "setup"],
            cwd=get_project_root() / "crates" / "fricon",
            check=True,
        )
        _ = subprocess.run(
            ["diesel", "migration", "run"],
            cwd=get_project_root() / "crates" / "fricon",
            check=True,
        )
    except FileNotFoundError:
        print(
            "`diesel` not found in $PATH. Please check development requirements in "
            + "README.md."
        )
        raise


def main() -> None:
    write_dotenv()
    create_dev_folder()
    diesel_setup()


if __name__ == "__main__":
    main()
