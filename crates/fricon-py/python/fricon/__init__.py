"""Fricon client library."""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError, version

from ._core import (
    Column,
    Dataset,
    DatasetManager,
    DatasetWriter,
    FriconDatasetError,
    IndexAxis,
    Trace,
    Workspace,
)

try:
    __version__ = version("fricon")
except PackageNotFoundError:
    __version__ = "0+unknown"

__all__ = [
    "Column",
    "Dataset",
    "DatasetManager",
    "DatasetWriter",
    "FriconDatasetError",
    "IndexAxis",
    "Trace",
    "Workspace",
    "__version__",
]
