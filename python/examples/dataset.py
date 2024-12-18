# pyright: reportUnknownMemberType = false
from __future__ import annotations

from time import perf_counter
from zoneinfo import ZoneInfo

import numpy as np
import pyarrow as pa

from fricon import connect


async def main() -> None:
    addr = "http://[::1]:22777"
    client = await connect(addr)
    rng = np.random.default_rng()

    t0 = perf_counter()

    writer = await client.create_dataset("test", tags=["aaa", "bb"])
    schema = pa.schema(
        {"i": pa.int32()} | {f"q{j}": pa.list_(pa.float32(), 1024) for j in range(64)}
    )
    for i in range(1000):
        data = {
            "i": [i],
        } | {f"q{j}": [rng.random(1024, dtype=np.float32)] for j in range(64)}
        batch = pa.record_batch(data, schema=schema)
        writer.write(batch)
    await writer.aclose()

    t1 = perf_counter()
    print(t1 - t0)

    print(writer.uid)

    info = await client.get_dataset(writer.uid)
    print(info.name)
    print(info.tags)
    print(info.path)
    print(info.created_at.astimezone(ZoneInfo("Asia/Shanghai")))


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
