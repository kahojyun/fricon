# pyright: reportExplicitAny=false
# pyright: reportAny=false
# pyright: reportUnknownMemberType=false
# pyright: reportUnknownVariableType=false
from __future__ import annotations

from typing import TYPE_CHECKING, Any

import polars as pl
import pyarrow as pa

from ._core import complex128

if TYPE_CHECKING:
    import numpy.typing as npt


def read_arrow(path: str) -> pa.Table:
    with pa.memory_map(path, "rb") as source:
        return pa.ipc.open_file(source).read_all()


def read_polars(path: str) -> pl.DataFrame:
    return pl.read_ipc(path)


def arrow_to_numpy(arr: pa.Array[Any] | pa.ChunkedArray[Any]) -> npt.NDArray[Any]:
    """Convert Arrow array to numpy array.

    If the Arrow array is of custom `complex128` type, it will be converted to
    a numpy array of complex numbers. Otherwise, the Arrow array will be
    converted with [`pyarrow.Array.to_numpy`][]

    Parameters:
        arr: Arrow array.

    Returns:
        Numpy array.
    """
    if isinstance(arr, pa.ChunkedArray):
        arr = arr.combine_chunks()
    if arr.type == complex128():
        if not isinstance(arr, pa.StructArray):
            msg = "arr must be a StructArray of complex128 type"
            raise AssertionError(msg)
        re = arr.field("real").to_numpy()
        im = arr.field("imag").to_numpy()
        return re + 1j * im
    return arr.to_numpy()
