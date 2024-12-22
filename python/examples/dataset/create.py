"""Demonstrate how to create a dataset using fricon."""

from __future__ import annotations

import pyarrow as pa

from fricon import DatasetManager, Workspace, complex128


def simple(manager: DatasetManager):
    """When no schema is provided, the schema is inferred from the first write."""
    with manager.create("example", description="test", tags=["tagA", "tagB"]) as writer:
        for i in range(10):
            # supports primitive types and 1d arrays
            writer.write(a=i, b=i * 2, c=[1, 2, 3])

    d = writer.to_dataset()
    assert d.name == "example"
    assert d.description == "test"
    assert set(d.tags) == {"tagA", "tagB"}
    assert d.uid is not None


def with_schema(manager: DatasetManager):
    """When a schema is provided, the schema is used.

    .. note::

        Although arrow supports nested types, we should avoid using them in the
        schema so that visualization tools can work with the data.
    """
    # Arrow doesn't have complex128, so we need to import it from fricon.
    # complex128 is a struct with two float64 fields named "real" and "imag".
    schema = pa.schema([("a", pa.int64()), ("b", pa.int64()), ("c", complex128())])
    with manager.create("example", schema=schema) as writer:
        for i in range(10):
            writer.write(a=i, b=i * 2, c=1j)


if __name__ == "__main__":
    ws = Workspace.open(".dev/ws")
    manager = ws.dataset_manager
    simple(manager)
    with_schema(manager)
