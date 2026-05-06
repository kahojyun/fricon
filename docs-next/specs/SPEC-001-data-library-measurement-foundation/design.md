# SPEC-001: Data Library Measurement Foundation Design

## Status

Draft.

## Design Summary

Build a thin but coherent v0.2 backbone:

```text
Python SDK / Desktop / CLI
  -> local service API
  -> data library
  -> measurement
  -> dataset artifacts
  -> live events and semantic reads
```

## Domain Records

First implementation records:

- DataLibrary
- Measurement
- DatasetArtifact
- Event/AuditRecord
- optional Sample and SampleSession links
- optional ParameterSnapshot
- optional CodeProvenanceSummary
- optional SetupProvenanceSummary
- optional ProcedureSummary

DatasetArtifact remains a first-class searchable/openable record. Measurement
is the primary navigation context, but datasets are not hidden children because
future analysis, import, and export workflows need direct artifact handles.
DatasetArtifact may have internal stream-like payload groups in storage/export
or advanced read APIs, but v0.2 should keep streams out of the normal user
concept budget.

## Service Operations

Candidate operation groups:

- `library.open_or_create`
- `compatibility.negotiate`
- `measurements.create`
- `measurements.finish`
- `measurements.abort`
- `measurements.record_event`
- `datasets.create_writer`
- `datasets.append`
- `datasets.finish`
- `datasets.abort`
- `measurements.list_recent`
- `measurements.get`
- `datasets.read_semantic`
- `datasets.search`
- `datasets.get`
- `events.subscribe`

Exact route/transport shape requires an ADR.

## Python Shape

Draft user-facing shape:

```python
lib = fricon.library()
lib.use_context(sample="sample-a", session="cooldown-2026-05")

with lib.measurement("rabi q3") as meas:
    ds = meas.dataset(
        "rabi",
        scan={"independent": "amp", "dependent": "signal"},
    )
    ds.write(amp=0.1, signal=0.25)
```

Common scan and trace schema should also have Python-native scan-plan authoring.
Exact names and whether the accepted API is dict/literal data, a helper
function, small builder objects, or multiple entry points are unsettled. The
public direction is low ceremony for common workflows, then refinement from
real script and notebook feedback:

```python
with lib.measurement("rabi q3") as meas:
    rabi = meas.scan(
        "rabi",
        plan={"axes": [{"name": "amp"}], "measure": ["signal"]},
    )
    rabi.write(amp=0.1, signal=0.25)
```

The scan-plan path should not replace raw schema for irregular/adaptive,
multi-output, repeated-point, or trace-heavy cases. It also should not imply a
QCoDeS-compatible helper API or parameter-object model.

## Desktop Shape

First screen becomes a measurement console:

- active sample/session context
- active measurements
- recent measurements
- produced datasets
- dataset search/direct open entry points
- table/plot actions
- partial/failed/trash shortcuts
- diagnostics when service or compatibility checks fail

## Storage Shape

Storage ADR must decide physical layout. The design requires only:

- stable data-library and record identities
- measurement catalog records
- dataset payload chunks and semantic schema
- scan shape modes for regular grids, partial grids, irregular/adaptive
  points, repeated points, fixed-shape traces, and variable-length traces
- partial-read semantics for interrupted or incomplete data
- optional internal stream-like groups for advanced storage/export/read needs
- event timeline records
- compatibility version and migration state
- active writer state sufficient for recovery

## Open Design Questions

1. Minimal lifecycle state enum and legal transitions.
2. Data-library storage layout and pre-v0.2 import stance.
3. Dataset artifact representation for fixed arrays and variable-length traces.
4. Partial grid, irregular/adaptive, repeated-point, and trace read APIs.
5. HTTP/WebSocket/binary API shape and local service discovery.
6. Actor/token storage and local operator profile scope.
7. Passive setup summary shape and whether future device snapshots need a
   reserved artifact/reference hook.
8. Passive procedure summary shape.
9. Whether internal stream groups are required for first implementation or only
   reserved in the storage/export ADRs.
