# pyright: reportExplicitAny=false
# pyright: reportAny=false
# pyright: reportUnknownMemberType=false
# pyright: reportUnknownVariableType=false
from __future__ import annotations

import re
from pathlib import Path

import polars as pl
import pyarrow as pa

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
    return pa.concat_tables(tables)


def read_polars(dir_path: str) -> pl.LazyFrame:
    files = _collect_chunk_files(dir_path)
    if not files:
        msg = f"no chunk files found in {dir_path}"
        raise FileNotFoundError(msg)
    return pl.scan_ipc(files)
