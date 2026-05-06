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

## Desktop Shape

First screen becomes a measurement console:

- active sample/session context
- active measurements
- recent measurements
- produced datasets
- table/plot actions
- partial/failed/trash shortcuts
- diagnostics when service or compatibility checks fail

## Storage Shape

Storage ADR must decide physical layout. The design requires only:

- stable data-library and record identities
- measurement catalog records
- dataset payload chunks and semantic schema
- event timeline records
- compatibility version and migration state
- active writer state sufficient for recovery

## Open Design Questions

1. Minimal lifecycle state enum and legal transitions.
2. Data-library storage layout and pre-v0.2 import stance.
3. Dataset artifact representation for fixed arrays and variable-length traces.
4. HTTP/WebSocket/binary API shape and local service discovery.
5. Actor/token storage and local operator profile scope.
