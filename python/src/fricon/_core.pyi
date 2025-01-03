from collections.abc import Iterable, Mapping, Sequence
from datetime import datetime
from typing import TypeVar, final

import pandas as pd
import polars as pl
import pyarrow as pa
from _typeshed import StrPath
from typing_extensions import Self, TypeAlias

__all__ = [
    "Dataset",
    "DatasetManager",
    "DatasetWriter",
    "Trace",
    "Workspace",
    "complex128",
    "main",
    "trace_",
]

def main() -> int: ...
def complex128() -> pa.DataType: ...
def trace_(item: pa.DataType, fixed_step: bool) -> pa.DataType: ...
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
        schema: pa.Schema | None = ...,
        index: Sequence[str] | None = ...,
    ) -> DatasetWriter: ...
    def open(
        self,
        dataset_id: str | int,
    ) -> Dataset: ...
    def list_all(self) -> pd.DataFrame: ...

_ScalarT_co = TypeVar("_ScalarT_co", str, bool, complex, covariant=True)

@final
class Trace:
    @staticmethod
    def variable_step(xs: Sequence[float], ys: Sequence[_ScalarT_co]) -> Trace: ...
    @staticmethod
    def fixed_step(
        x0: float,
        dx: float,
        ys: Sequence[_ScalarT_co],
    ) -> Trace: ...
    @property
    def data_type(self) -> pa.DataType: ...

_ColumnType: TypeAlias = (
    str | bool | complex | Sequence[str] | Sequence[bool] | Sequence[complex] | Trace
)

@final
class DatasetWriter:
    def write(self, **kwargs: _ColumnType) -> None: ...
    def write_dict(self, values: Mapping[str, _ColumnType]) -> None: ...
    @property
    def id(self) -> int: ...
    def close(self) -> None: ...
    def __enter__(self) -> Self: ...
    def __exit__(
        self, exc_type: object, exc_value: object, traceback: object
    ) -> None: ...

@final
class Dataset:
    # @staticmethod
    # def open(path: StrPath) -> Dataset: ...
    def to_pandas(self) -> pd.DataFrame: ...
    def to_polars(self) -> pl.DataFrame: ...
    def to_arrow(self) -> pa.Table: ...
    name: str
    description: str
    tags: list[str]
    favorite: bool
    @property
    def id(self) -> int: ...
    @property
    def uid(self) -> str: ...
    @property
    def path(self) -> str: ...
    @property
    def created_at(self) -> datetime: ...
    @property
    def schema(self) -> pa.Schema: ...
    @property
    def index(self) -> list[str]: ...
    def close(self) -> None: ...
    def __enter__(self) -> Self: ...
    def __exit__(
        self, exc_type: object, exc_value: object, traceback: object
    ) -> None: ...
