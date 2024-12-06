from time import perf_counter
from zoneinfo import ZoneInfo

import pyarrow as pa

from fricon import connect


async def main() -> None:
    addr = "http://[::1]:22777"
    client = await connect(addr)

    t0 = perf_counter()

    writer = await client.create_dataset("test", tags=["aaa", "bb"])
    for i in range(10):
        data = {
            "i": [i],
            "s": [f"{i}"],
            "b": [i % 2 == 0],
        }
        batch = pa.record_batch(data)  # pyright: ignore[reportUnknownMemberType]
        sink = pa.BufferOutputStream()
        with pa.ipc.new_stream(sink, batch.schema) as batch_writer:
            batch_writer.write_batch(batch)  # pyright: ignore[reportUnknownMemberType]
        writer.write(sink.getvalue().to_pybytes())
    await writer.aclose()

    t1 = perf_counter()

    print(writer.uid)
    print(t1 - t0)

    info = await client.get_dataset(writer.uid)
    print(info.name)
    print(info.tags)
    print(info.path)
    print(info.created_at.astimezone(ZoneInfo("Asia/Shanghai")))

    try:
        _ = await client.get_dataset("nonexistent")
    except RuntimeError as e:
        print(e)


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
