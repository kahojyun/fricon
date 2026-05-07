# Compatibility Policy

## Status

Accepted reset policy, draft implementation details.

## Policy Summary

v0.2 is a clean reset from pre-v0.2 workspace/dataset-first compatibility.

Do not preserve v0.1 public APIs, workspace layout, IPC contracts, desktop
navigation, or archive formats when they conflict with the v0.2 data-library
and measurement-centered model.

## Pre-v0.2 Compatibility

Pre-v0.2 local workspaces are prototype/test data unless a later ADR names a
specific migration target.

Default policy:

- no requirement to open old workspaces in v0.2
- no requirement to migrate v0.1 dataset catalogs automatically
- no requirement to maintain old Python workspace/dataset APIs
- no requirement to preserve current gRPC/protobuf contracts
- reusable implementation ideas may be adapted without compatibility promises

If a real legacy migration need appears, create a focused ADR that defines:

- supported source versions
- migration or import route
- data loss and unsupported cases
- validation and recovery checks

## v0.2+ Data Durability Promise

Once v0.2 records real lab data, Fricon must protect recorded data.

The promise is:

- explicit data-library format versioning
- explicit migrations or recovery paths
- backup/checkpoint before risky migrations where practical
- fail-before-write diagnostics for incompatible clients
- clear user guidance about what to update or migrate
- readable/exportable data whenever practical, even if mutating APIs change

The promise is not:

- strict v0.x Python API stability
- strict third-party protocol stability
- support for direct shared-folder multi-machine database access
- silent migration during app launch or active measurement

## Compatibility Dimensions

| Dimension | Gate |
| --- | --- |
| Data-library format | Storage format version and migration state. |
| Local runtime/API | API/protocol version and capability negotiation. |
| Python SDK | Required capability checks before writes. |
| Desktop/CLI | Bundled with compatible local runtime for normal local flow. |
| Export bundle | Format version and reader compatibility. |
| Feature support | Capability flags for optional or future features. |

## Update Safety

Updates and migrations must account for:

- active measurements
- open dataset writers
- imports/exports
- data-library migrations or repairs
- future managed tasks and resource leases

When busy, Fricon should offer "install when idle" or "remind later" rather
than silently stopping the measurement environment.
