"""Set `FRICON_DOCS_TAG` environment variable in GitHub Actions environment."""

import os
from importlib.metadata import version

from packaging.version import parse


def main() -> None:
    fricon_version = parse(version("fricon"))
    docs_tag = f"{fricon_version.major}.{fricon_version.minor}"
    github_env = os.getenv("GITHUB_ENV")
    if github_env is None:
        msg = "Should be run in GitHub Actions environment"
        raise RuntimeError(msg)
    with open(github_env, "a", encoding="utf-8") as f:
        _ = f.write(f"FRICON_DOCS_TAG={docs_tag}\n")


if __name__ == "__main__":
    main()
