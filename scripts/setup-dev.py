from pathlib import Path
import subprocess


DEV_FOLDER = ".dev"
TEST_DB = "testdb.sqlite3"


def get_project_root():
    """Find project root with __file__."""
    this_file = Path(__file__)  # <project-root>/scripts/setup-dev.py
    return this_file.parent.parent


def write_dotenv():
    dotenv_path = get_project_root() / ".env"
    if dotenv_path.exists():
        raise RuntimeError("A .env file already exists.")
    with dotenv_path.open("w") as f:
        _ = f.write(f"DATABASE_URL=sqlite://{DEV_FOLDER}/{TEST_DB}\n")


def create_dev_folder():
    dev_folder = get_project_root() / DEV_FOLDER
    dev_folder.mkdir()  # Raises FileExistsError if already exists.
    gitignore = dev_folder / ".gitignore"
    with gitignore.open("w") as f:
        _ = f.write("*\n")


def sqlx_setup():
    try:
        _ = subprocess.run(["sqlx", "db", "setup"], cwd=get_project_root(), check=True)
    except FileNotFoundError:
        print(
            "`sqlx` not found in $PATH. Please check development requirements in "
            + "README.md."
        )
        raise


def main():
    write_dotenv()
    create_dev_folder()
    sqlx_setup()


if __name__ == "__main__":
    main()
