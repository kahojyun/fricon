from collections.abc import Iterable, Mapping, Sequence
from datetime import datetime
from typing import Any, Literal, TypeAlias, TypeVar, final

import numpy.typing as npt
import pandas as pd
import polars as pl
import pyarrow as pa
from _typeshed import StrPath
from numpy import floating
from typing_extensions import Self

__all__ = [
    "Dataset",
    "DatasetManager",
    "DatasetWriter",
    "ServerHandle",
    "Trace",
    "Workspace",
    "main",
    "main_gui",
    "serve_workspace",
]

def main() -> int: ...
def main_gui() -> int: ...
def serve_workspace(path: StrPath) -> tuple[Workspace, ServerHandle]: ...

@final
class ServerHandle:
    def shutdown(self, timeout: float | None = None) -> None: ...
    @property
    def is_running(self) -> bool: ...

@final
class Workspace:
    @staticmethod
    def connect(path: StrPath) -> Workspace: ...
    @property
    def dataset_manager(self) -> DatasetManager: ...

@final
class DatasetManager:
    def create(
        self,
        name: str,
        *,
        description: str | None = ...,
        tags: Iterable[str] | None = ...,
    ) -> DatasetWriter: ...
    def open(
        self,
        dataset_id: str | int,
    ) -> Dataset: ...
    def list_all(
        self,
        *,
        limit: int | None = ...,
        offset: int | None = ...,
    ) -> pd.DataFrame: ...

_ScalarT_co = TypeVar("_ScalarT_co", float, complex, covariant=True)
_ArrowAnyArray: TypeAlias = pa.Array[Any]  # pyright: ignore[reportExplicitAny]
_NumpyAnyArray: TypeAlias = npt.NDArray[Any]  # pyright: ignore[reportExplicitAny]

@final
class Trace:
    @staticmethod
    def variable_step(
        x: Sequence[float] | npt.NDArray[floating],
        y: Sequence[_ScalarT_co] | _ArrowAnyArray | _NumpyAnyArray,
    ) -> Trace: ...
    @staticmethod
    def fixed_step(
        x0: float,
        step: float,
        y: Sequence[_ScalarT_co] | _ArrowAnyArray | _NumpyAnyArray,
    ) -> Trace: ...

_ColumnType: TypeAlias = (
    float
    | complex
    | Sequence[float]
    | Sequence[complex]
    | Trace
    | _ArrowAnyArray
    | _NumpyAnyArray
)

@final
class DatasetWriter:
    def write(self, **kwargs: _ColumnType) -> None: ...
    def write_dict(self, values: Mapping[str, _ColumnType]) -> None: ...
    @property
    def dataset(self) -> Dataset: ...
    def close(self) -> None: ...
    def __enter__(self) -> Self: ...
    def __exit__(
        self, exc_type: object, exc_value: object, traceback: object
    ) -> None: ...

@final
class Dataset:
    def to_polars(self) -> pl.LazyFrame: ...
    def to_arrow(self) -> pa.Table: ...
    def add_tags(self, *tag: str) -> None: ...
    def remove_tags(self, *tag: str) -> None: ...
    def update_metadata(
        self,
        *,
        name: str | None = None,
        description: str | None = None,
        favorite: bool | None = None,
    ) -> None: ...
    @property
    def name(self) -> str: ...
    @property
    def description(self) -> str: ...
    @property
    def favorite(self) -> bool: ...
    @property
    def tags(self) -> list[str]: ...
    @property
    def id(self) -> int: ...
    @property
    def uid(self) -> str: ...
    @property
    def path(self) -> str: ...
    @property
    def created_at(self) -> datetime: ...
    @property
    def status(self) -> Literal["writing", "completed", "aborted"]: ...
