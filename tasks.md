# Implementation Tasks (Dataset Features Phase 1)

Legend: ID, Description, ReqRefs, Depends, Est (S/M/L), Status(Pending)

| ID  | Description                                                                       | ReqRefs   | Depends   | Est | Status      |
| --- | --------------------------------------------------------------------------------- | --------- | --------- | --- | ----------- |
| A1  | Add `config.rs` module with `DatasetConfig` + load/save + validation              | 8,12,16   | -         | M   | Done        |
| A2  | Integrate config save/load path resolution in `DatasetManager`                    | 6,8       | A1        | S   | Done        |
| A3  | Add AppEvent variants (RowsAppended, ConfigUpdated)                               | 18,19     | -         | S   | Done        |
| A4  | Implement WriteSessionRegistry (new struct in dataset_manager or submodule)       | 10        | A3        | M   | Pending     |
| A5  | Implement DatasetWriter struct (public API in core crate)                         | 10,17     | A4        | M   | Pending     |
| A6  | Expose create_writer binding in `fricon-py` (PyO3)                                | 10        | A5        | M   | Pending     |
| A7  | Implement row -> Arrow builders + flush logic                                     | 10,17,18  | A5        | M   | Pending     |
| A8  | Emit RowsAppended event on flush                                                  | 18,19     | A7        | S   | Pending     |
| A9  | Finalize logic: finish writer, update status, run index inference if needed       | 5,9,10,17 | A7        | M   | Pending     |
| A10 | Aborted finalize path (exception propagation)                                     | 5,10,15   | A7        | S   | Pending     |
| A11 | Index inference module + perfect/partial selection                                | 9         | A9        | M   | Pending     |
| A12 | Manual override API `set_index_columns`                                           | 9,10      | A1        | S   | Pending     |
| A13 | Visualization views CRUD in core (save_view/delete/set_default)                   | 12        | A1        | M   | Pending     |
| A14 | Core trait `DatasetViewAccess` + impl (config + full slice)                       | 6,8,10,18 | A1        | M   | Pending     |
| A15 | gRPC: extend proto for GetConfig/SaveConfig/GetDatasetSlice (delegating to trait) | 12,13,18  | A14       | M   | Pending     |
| A16 | Server handlers + integration tests (gRPC parity with trait)                      | 12,13,18  | A15       | M   | Pending     |
| A17 | Python bindings for views + index override + to_pandas partial read               | 10,12,18  | A6,A15    | M   | Pending     |
| A18 | Code snippet generator (workspace comment)                                        | 16        | -         | S   | Pending     |
| A19 | GUI: direct in-process access (render views, role selection UI skeleton)          | 12,13,18  | A14       | L   | Pending     |
| A20 | GUI: auto-refresh on RowsAppended (full reload using trait)                       | 18        | A19,A8    | M   | Pending     |
| A21 | Tests: DatasetWriter unit tests                                                   | 10,17     | A5,A7     | M   | Pending     |
| A22 | Tests: Index inference cases                                                      | 9         | A11       | S   | Pending     |
| A23 | Tests: Config + view validation                                                   | 8,12      | A1,A13    | S   | In Progress |
| A24 | Tests: Real-time events (flush -> event)                                          | 18,19     | A8        | S   | Pending     |
| A25 | Tests: Trait vs gRPC parity (config + full slice)                                 | 6,8,18    | A16       | S   | Pending     |
| A26 | Tests: Python end-to-end create -> write -> visualize                             | 10,12,18  | A17       | M   | Pending     |
| A27 | Docs: Update README + API docs (Python usage snippet)                             | 10,16     | A6,A18    | S   | Pending     |
| A28 | Changelog entry                                                                   | all       | after all | S   | Pending     |
| A29 | Technical debt placeholder issues (inference perf, tail API, mmap)                | 9,18      | A11,A14   | S   | Pending     |

Dependency Graph (simplified):
Core foundation (A1,A3) -> Writer infra (A4-A7) -> Flush/events (A8) -> Finalize + inference (A9,A11) -> View & config core (A13,A14) -> gRPC (A15,A16) & Python (A17) -> GUI direct (A19,A20) -> Tests (A21-A26) -> Docs/Changelog (A27,A28) -> Debt (A29).

## Execution Strategy

Confidence high; proceed full implementation. Parallelizable clusters:

- Cluster 1: Config + Events (A1,A3)
- Cluster 2: Writer Core (A4-A7, A8, A9, A10)
- Cluster 3: Inference (A11, A21) can start once writer flush shape known.
- Cluster 4: Views + gRPC (A13,A14,A15) -> Python (A16)
- Cluster 5: GUI (A18,A19)
- Cluster 6: Tests + Docs (A20-A27)

## Acceptance Criteria Summary

- Create -> write rows -> finalize produces Completed dataset with Arrow file & config.json.
- Real-time events observed for appended rows in tests.
- Index columns inferred (perfect or partial) when absent after finalize or first viz trigger.
- Views CRUD persists & validates roles.
- Python snippet includes workspace comment line.
- GUI receives RowsAppended and re-renders with new data.
