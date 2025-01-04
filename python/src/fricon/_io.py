from __future__ import annotations

import polars as pl
import pyarrow as pa


def read_arrow(path: str) -> pa.Table:
    with pa.memory_map(path, "rb") as source:
        return pa.ipc.open_file(source).read_all()


def read_polars(path: str) -> pl.DataFrame:
    return pl.read_ipc(path)
