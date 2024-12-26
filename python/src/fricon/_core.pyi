from collections.abc import Iterable, Sequence
from datetime import datetime
from typing import Generic, TypeVar, final, overload

import pandas as pd
import polars as pl
import pyarrow as pa
from _typeshed import StrPath
from typing_extensions import Self

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
def trace_(item: pa.DataType) -> pa.DataType: ...
@final
class Workspace:
    @staticmethod
    def open(path: StrPath) -> Workspace: ...
    @property
    def dataset_manager(self) -> DatasetManager: ...

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
        uid: str,
    ) -> Dataset: ...
    def list_all(self) -> pd.DataFrame: ...

_ScalarT_co = TypeVar("_ScalarT_co", int, float, str, bool, complex, covariant=True)

class Trace(Generic[_ScalarT_co]):
    @overload
    def __new__(cls, ys: Sequence[_ScalarT_co]) -> Self: ...
    @overload
    def __new__(cls, xs: Sequence[float], ys: Sequence[_ScalarT_co]) -> Self: ...
    @overload
    def __new__(cls, x0: float, dx: float, ys: Sequence[_ScalarT_co]) -> Self: ...

class DatasetWriter:
    @overload
    def write(
        self, **kwargs: _ScalarT_co | Trace[_ScalarT_co] | Sequence[_ScalarT_co]
    ) -> None: ...
    @overload
    def write(
        self,
        kwargs: dict[str, _ScalarT_co | Trace[_ScalarT_co] | Sequence[_ScalarT_co]],
    ) -> None: ...
    def to_dataset(self) -> Dataset: ...
    def __enter__(self) -> Self: ...
    def __exit__(
        self, exc_type: object, exc_value: object, traceback: object
    ) -> None: ...

class Dataset:
    name: str
    description: str
    tags: list[str]
    favorite: bool
    def to_pandas(self) -> pd.DataFrame: ...
    def to_polars(self) -> pl.DataFrame: ...
    def to_arrow(self) -> pa.Table: ...
    @staticmethod
    def open(path: StrPath) -> Dataset: ...
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
