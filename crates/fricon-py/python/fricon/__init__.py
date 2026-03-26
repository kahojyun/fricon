"""Fricon client library."""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError, version

from ._core import (
    Dataset,
    DatasetManager,
    DatasetWriter,
    FriconDatasetError,
    Trace,
    Workspace,
)

try:
    __version__ = version("fricon")
except PackageNotFoundError:
    __version__ = "0+unknown"

__all__ = [
    "Dataset",
    "DatasetManager",
    "DatasetWriter",
    "FriconDatasetError",
    "Trace",
    "Workspace",
    "__version__",
]
