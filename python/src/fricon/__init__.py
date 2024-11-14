"""Fricon client library."""

from __future__ import annotations

import queue
from datetime import datetime, timezone

import grpc
import pyarrow as pa

from fricon._proto.fricon.v1 import fricon_pb2 as pb
from fricon._proto.fricon.v1.fricon_pb2_grpc import DataStorageServiceStub


def connect(addr: str) -> Client:
    return Client(addr)


class Client:
    def __init__(self, addr: str) -> None:
        self._channel = grpc.insecure_channel(addr)

    def create_dataset(
        self, name: str, description: str = "", tags: list[str] | None = None
    ) -> DatasetWriter:
        if self._channel is None:
            raise RuntimeError("Server closed")
        return DatasetWriter(self._channel, name, description, tags)

    def get_dataset(self, uid: str) -> DatasetInfo:
        if self._channel is None:
            raise RuntimeError("Server closed")
        stub = DataStorageServiceStub(self._channel)
        req = pb.GetRequest(uid=uid)
        response: pb.GetResponse = stub.Get(req)
        metadata = response.metadata
        name = metadata.name
        description = metadata.description
        tags = list(metadata.tags)
        path = response.path
        timestamp = response.created_at.seconds
        created_at = datetime.fromtimestamp(timestamp, tz=timezone.utc)
        return DatasetInfo(name, description, tags, path, created_at)

    def close(self) -> None:
        if self._channel is not None:
            self._channel.close()
            self._channel = None


class DatasetWriter:
    def __init__(
        self, channel: grpc.Channel, name: str, description: str, tags: list[str] | None
    ) -> None:
        stub = DataStorageServiceStub(channel)

        req = pb.CreateRequest(
            metadata=pb.Metadata(name=name, description=description, tags=tags)
        )
        response: pb.CreateResponse = stub.Create(req)
        write_token = response.write_token

        write_metadata = (("fricon-token-bin", write_token),)
        self._queue = queue.Queue()
        self._write_future = stub.Write.future(
            iter(self._queue.get, None), metadata=write_metadata
        )
        self._schema = None
        self._uid = None

    @property
    def uid(self) -> str:
        if self._uid is None:
            raise RuntimeError("Dataset not finished")
        return self._uid

    def write(self, data: pa.RecordBatch | pa.Table) -> None:
        if self._queue is None or self._write_future is None:
            raise RuntimeError("Dataset already finished")
        if self._schema is None:
            self._schema = data.schema
        elif not self._schema.equals(data.schema):
            raise ValueError("Schema mismatch")

        sink = pa.BufferOutputStream()
        with pa.ipc.new_stream(sink, data.schema) as writer:
            writer.write(data)
        record_batch = sink.getvalue().to_pybytes()

        req = pb.WriteRequest(record_batch=record_batch)
        self._queue.put(req)
        if self._write_future.done():
            self._write_future.exception()

    def close(self) -> None:
        if self._queue is None or self._write_future is None:
            raise RuntimeError("Dataset already finished")
        self._queue.put(None)
        self._queue = None

        response: pb.WriteResponse = self._write_future.result()
        self._write_future = None
        self._uid = response.uid

    def __enter__(self) -> DatasetWriter:
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        self.close()


class DatasetInfo:
    def __init__(
        self,
        name: str,
        description: str,
        tags: list[str],
        path: str,
        created_at: datetime,
    ) -> None:
        self.name = name
        self.description = description
        self.tags = tags
        self.path = path
        self.created_at = created_at
