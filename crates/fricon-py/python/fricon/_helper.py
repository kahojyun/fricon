# pyright: reportExplicitAny=false
# pyright: reportAny=false
# pyright: reportUnknownMemberType=false
# pyright: reportUnknownVariableType=false
from __future__ import annotations

import re
from pathlib import Path
from typing import TYPE_CHECKING, Any

import polars as pl
import pyarrow as pa

from ._core import complex128

if TYPE_CHECKING:
    import numpy.typing as npt


_CHUNK_PATTERN = re.compile(r"data_chunk_(\d+)\.arrow$")


def _collect_chunk_files(dir_path: str) -> list[Path]:
    p = Path(dir_path)
    if not p.is_dir():
        msg = f"expected dataset directory, got: {dir_path}"
        raise ValueError(msg)
    base_dir = p
    chunk_files: list[tuple[int, Path]] = []
    for f in base_dir.iterdir():
        if not f.is_file():
            continue
        m = _CHUNK_PATTERN.match(f.name)
        if m:
            chunk_files.append((int(m.group(1)), f))
    chunk_files.sort(key=lambda pair: pair[0])
    return [f for _, f in chunk_files]


def read_arrow(dir_path: str) -> pa.Table:
    files = _collect_chunk_files(dir_path)
    if not files:
        msg = f"no chunk files found in {dir_path}"
        raise FileNotFoundError(msg)
    tables: list[pa.Table] = []
    for f in files:
        with pa.memory_map(str(f), "rb") as source:
            tables.append(pa.ipc.open_file(source).read_all())
    if len(tables) == 1:
        return tables[0]
    return pa.concat_tables(tables, promote=True)


def read_polars(dir_path: str) -> pl.DataFrame:
    files = _collect_chunk_files(dir_path)
    if not files:
        msg = f"no chunk files found in {dir_path}"
        raise FileNotFoundError(msg)
    dfs = [pl.read_ipc(str(f)) for f in files]
    if len(dfs) == 1:
        return dfs[0]
    return pl.concat(dfs, how="vertical_relaxed")


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
