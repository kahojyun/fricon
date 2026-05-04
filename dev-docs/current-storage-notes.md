# Current Workspace And Dataset Storage Notes

## Status

Current implementation note. Update this when workspace or dataset storage
layout changes.

## Purpose

This note captures current implementation details that should not be presented
as stable user-facing documentation. Public docs should describe workspaces and
datasets through the CLI, Python API, and desktop UI rather than through the
on-disk layout.

Dataset semantics proposal documents describe long-term direction. Use this
file as the current source of truth for implemented storage facts, including
the current v1 dataset semantic manifest sidecar.

## Workspace Layout

At the time of writing, a workspace contains:

```tree
workspace/
  .fricon_workspace.json
  fricon.sqlite3
  fricon.socket
  data/
    .graveyard/
    <uid[0:2]>/
      <uid>/
        dataset_manifest.json
        data_chunk_0.arrow
        data_chunk_1.arrow
        logical_index_chunk_0.arrow
  backup/
  log/
```

Notes:

- `.fricon_workspace.json` stores the workspace format version as an internal
  integer compatibility counter.
- Opening an older workspace may trigger a stepwise migration before the
  workspace is usable.
- `fricon.sqlite3` stores workspace catalog metadata.
- `fricon.socket` is runtime IPC state, not durable data.
- Dataset directories are currently sharded by the first two characters of the
  dataset UID.
- `data/.graveyard/` stores deleted dataset payload directories before they are
  permanently removed.

## Dataset Payload Storage

Current dataset payloads are chunked [Arrow IPC][] files under the dataset
directory. A dataset is modeled as one logical Arrow table split across
`data_chunk_<n>.arrow` files as needed.

New datasets created through ingest also store `dataset_manifest.json` beside
the chunk files. The manifest records v1 dataset semantic columns, realization
defaults, optional scan plans, and inference settings. New semantic datasets
physically materialize the Fricon-owned `__ds_record_id: uint64` system column
as the first Arrow column. Earlier transition snapshots may contain manifests
that declare `__ds_record_id` before the Arrow chunks materialized that column.

Datasets written with explicit logical scan indices store those indices in
append-only `logical_index_chunk_<n>.arrow` sidecar files. Logical-index chunks
are keyed by `__ds_record_id` and then contain one `uint64` column per scan
axis. These sidecars are not part of the visible user payload.

The physical storage layout is an implementation detail. User-facing docs may
mention Arrow-compatible tables, but should avoid promising exact file names,
chunking behavior, or directory structure.

## Dataset Metadata Ownership

Current dataset catalog metadata lives in SQLite. This includes dataset name,
description, favorite state, status, timestamps, and tags as represented by
`DatasetRecord` / `DatasetMetadata`.

Dataset payload facts live in Arrow chunk files. Dataset semantic defaults,
optional scan plans, and inference settings for new ingested datasets live
in `dataset_manifest.json`.

Dataset archives store catalog metadata in `metadata.json`, Arrow payload chunks
under `data/data_chunk_<n>.arrow`, logical-index chunks under
`logical_index/logical_index_chunk_<n>.arrow` when present, and
`dataset_manifest.json` as an optional root sidecar when the source dataset has
one. Dataset readers require manifests; manifest-free payloads are not
supported by the semantic read path in this PR.

## Write Buffering

Current Python dataset writes are buffered on the client and flushed
automatically when either:

- 16 rows have accumulated, or
- 200 ms have elapsed since the first buffered row.

Calling `finish()`, `abort()`, or dropping the writer flushes pending rows
before finalizing the dataset stream.

The public contract is that writes are buffered and finalization flushes pending
rows. The exact thresholds are implementation details unless they become part of
a documented performance contract.

## Maintenance Notes

For workspace format and dataset payload layout changes, use the canonical
checklists in `dev-docs/maintenance-checklist.md`.

[Arrow IPC]: https://arrow.apache.org/docs/format/Columnar.html#serialization-and-interprocess-communication-ipc
