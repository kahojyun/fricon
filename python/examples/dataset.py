# pyright: basic
from time import perf_counter

import pyarrow as pa
from fricon import connect

addr = "[::1]:22777"
client = connect(addr)

t0 = perf_counter()

with client.create_dataset("test", tags=["aaa", "bb"]) as writer:
    for i in range(10):
        data = {
            "i": [i],
            "s": [f"{i}"],
            "b": [i % 2 == 0],
        }
        batch = pa.record_batch(data)
        writer.write(batch)

t1 = perf_counter()
print(writer.uid)
print(t1 - t0)

dataset = client.get_dataset(writer.uid)
print(dataset.path)
print(dataset.created_at.astimezone())
