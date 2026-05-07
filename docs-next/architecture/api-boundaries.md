# Deferred API Boundary Questions

## Status

Deferred. Product and domain analysis are still upstream of API architecture.

## Upstream Product Inputs

- The Python SDK is a primary user experience.
- Notebook-friendly, ordinary-Python use with a visible reusable context/handle
  is an accepted product experience direction.
- Exact Python names, signatures, context-manager syntax, decorator syntax,
  writer object model, and capture mechanics are deferred.
- Python reopen/export should use stable public APIs, not storage paths.
- Mutating clients must fail compatibility checks before writes.

## Deferred API Questions

- What local service or equivalent local authority coordinates mutating
  operations?
- Which transport should Desktop, Python SDK, and CLI use?
- Which parts of the service contract are JSON/control, streaming events, or
  binary payload transfer?
- Which capability/version negotiation is required before measurement creation,
  dataset writing, export, migration, and future managed execution?
- How should Python SDK examples preserve low ceremony without freezing helper
  names too early?
- Which read APIs expose semantic tables, axes, grids, partial grids,
  irregular/adaptive points, repeated points, and traces?
- Which API surfaces, if any, receive pre-1.0 compatibility promises?

## Candidate Inputs For Later ADRs

These are historical or plausible directions only:

- current v0.1 gRPC/protobuf code
- browser-capable HTTP/event-stream APIs
- binary Arrow-compatible dataset payload endpoints
- explicit write-session operations
- server-side summaries, paging, and downsampling for UI reads

Do not promote any candidate to accepted architecture without an ADR.
