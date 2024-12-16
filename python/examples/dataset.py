# ruff: noqa: ERA001
# pyright: reportUnknownMemberType = false
from __future__ import annotations

from queue import Queue
from threading import Thread
from time import perf_counter
from zoneinfo import ZoneInfo

import numpy as np
import pyarrow as pa
from pyarrow.ipc import IpcWriteOptions
import pyarrow.parquet as pq

from fricon import connect


# def writer_main(q: Queue[pa.RecordBatch | None]) -> None:
#     item = q.get()
#     if item is None:
#         return
#     schema = item.schema
#     options = IpcWriteOptions(compression="zstd")
#     with pa.ipc.new_file("test.arrow", schema=schema, options=options) as writer:
#         while item is not None:
#             writer.write_batch(item)
#             item = q.get()


async def main() -> None:
    addr = "http://[::1]:22777"
    client = await connect(addr)
    rng = np.random.default_rng()

    t0 = perf_counter()

    # q: Queue[pa.RecordBatch | None] = Queue()
    # writer_thread = Thread(target=writer_main, args=(q,))
    # writer_thread.start()

    writer = await client.create_dataset("test", tags=["aaa", "bb"])
    schema = pa.schema(
        {"i": pa.int32()} | {f"q{j}": pa.list_(pa.float32(), 1024) for j in range(64)}
    )
    # writer = pa.ipc.new_file("test.arrow", schema=schema, options=None)
    for i in range(1000):
        data = {
            "i": [i],
        } | {f"q{j}": [rng.random(1024, dtype=np.float32)] for j in range(64)}
        batch = pa.record_batch(data, schema=schema)
        # writer.write_batch(batch)
        writer.write(batch)
    #     q.put(batch)
    # q.put(None)
    # writer_thread.join()
    t1 = perf_counter()
    await writer.aclose()
    # writer.close()

    # print(writer.uid)
    print(t1 - t0)
    #
    # info = await client.get_dataset(writer.uid)
    # print(info.name)
    # print(info.tags)
    # print(info.path)
    # print(info.created_at.astimezone(ZoneInfo("Asia/Shanghai")))


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
