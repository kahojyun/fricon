# Dataset Feature Design (Phase 2)

Status: Draft
Date: 2025-08-30
Related: `requirements.md` (v0.1)
Scope: Implementation design for dataset writer, config management, visualization views, index inference, real-time updates.

## 1. Architecture Overview

The Tauri GUI (crate `fricon-ui`) and the core server (`fricon`) run inside the **same process**. Python clients interact **out-of-process** via gRPC over the IPC socket. This yields a dual-access model:

| Access Path       | Consumers        | Transport | Serialization Layer |
| ----------------- | ---------------- | --------- | ------------------- |
| In-process direct | fricon-ui (Rust) | None      | Native Rust types   |
| IPC gRPC          | Python, CLI      | gRPC/IP   | Protobuf            |

Design principle: plotting / visualization logic (role selection, value transforms, heatmap matrix shaping) resides **only in `fricon-ui`**. Core provides _data & config primitives_ (no chart semantics). This separation keeps the core lean and stable while allowing UI iteration.

New / extended concerns (core scope):

- DatasetWriter (row-wise streaming, unchanged conceptually).
- Config management (`config.json`) decoupled from DB metadata (core owns persistence / validation of generic structure; UI owns visualization semantics built atop generic view schema).
- Real-time event emission for partial dataset visualization (AppEvent extension).
- Index column deferred inference (core, data-centric only).
- In-process dataset slice & tail reading helper (no gRPC overhead for UI).

Visualization concern layering:

- Core: stores neutral view definitions (chart_type enum limited to Line/Heatmap; roles map).
- UI: interprets roles to produce plot data structures (e.g., series arrays, heatmap grids).

### Updated Component Map

| Component                  | Layer            | Role                                                        |
| -------------------------- | ---------------- | ----------------------------------------------------------- |
| DatasetManager             | Core             | Metadata CRUD & lifecycle                                   |
| WriteSessionRegistry       | Core             | Active write sessions                                       |
| BatchWriter                | Core             | Arrow batch writing                                         |
| ConfigService              | Core             | Load/save/validate config.json                              |
| IndexInference             | Core             | Determine index columns                                     |
| DatasetSliceProvider (new) | Core             | Read full / tail slices for Arrow file (partial safe)       |
| AppEvent bus               | Core             | Broadcast dataset & config events                           |
| gRPC DatasetService        | IPC              | Wraps core for out-of-process clients                       |
| fricon-py bindings         | IPC              | Python exposure (via gRPC + maybe direct FFI later)         |
| Visualization Layer        | UI (`fricon-ui`) | Turn views + data slices into plot models                   |
| UI State Store             | UI               | Subscribe to AppEvent & request slices via direct core call |

### In-Process Access Strategy

`fricon-ui` gets an `AppHandle` reference during startup. It uses new trait(s):

```rust
pub trait DatasetViewAccess {
    fn get_config(&self, uuid: Uuid) -> Result<DatasetConfig, Error>;
    fn save_config(&self, uuid: Uuid, cfg: &DatasetConfig) -> Result<(), Error>;
    fn read_full(&self, uuid: Uuid) -> Result<arrow::record_batch::RecordBatchReader, Error>;
    fn read_tail(&self, uuid: Uuid, rows: usize) -> Result<Vec<RecordBatch>, Error>; // optional later
}
```

Implementation lives in core; UI depends only on the trait. gRPC service internally calls the same trait, avoiding duplicated logic.

## 2. Data Structures

Rust (core additions):

```rust
// config.rs (new)
pub struct DatasetConfig {
    pub version: u32,
    pub index_columns: Option<Vec<String>>,
    pub views: BTreeMap<String, ViewDefinition>,
    pub default_view: String,
}

pub struct ViewDefinition {
    pub chart_type: ChartType, // Line | Heatmap
    pub roles: ViewRoles,
    pub style: Option<ViewStyle>,
    pub filters: Option<Vec<FilterClause>>, // maybe empty in MVP
}

pub enum ChartType { Line, Heatmap }

pub struct ViewRoles { pub x: String, pub y: Vec<String>, pub z: Option<String>, pub color: Option<String> }
```

Write session:

```rust
struct ActiveWriteSession {
    dataset_id: i32,
    uuid: Uuid,
    schema: ArrowSchemaRef,
    arrow_writer: BatchWriter<BufWriter<File>>,
    pending_rows: Vec<RowBuffer>, // logical rows before batch flush
    flushed_row_count: u64,
    last_flush_instant: Instant,
    scan_params_spec: Option<Vec<String>>,
}
```

Python mapping: `DatasetWriter` keeps a PyO3 wrapper referencing an internal Arc<Mutex<ActiveWriteSession>>; flush calls via GIL release.

## 3. Lifecycle & Sequence Flows

### Create + Write (Python)

1. `create_writer()` -> DB insert (status Pending) -> directory create -> DB status set to Writing; `ActiveWriteSession` created and stored.
2. First `add_row` defines schema: convert dict -> Arrow arrays builder set.
3. Rows accumulate until flush condition (#rows >= FLUSH_BATCH_SIZE default e.g. 1024, or explicit flush, or context exit).
4. Flush: build RecordBatch, call existing `BatchWriter.write()`.
5. After each flush emit `AppEvent::DatasetRowsAppended { uuid, appended_row_count }`.
6. Context exit success: finish writer, update DB status Completed, run deferred index inference if needed, save config.
7. Context exit error: abort writer (drop file open handle), DB status Aborted.

### Real-Time Visualization

GUI subscribes directly (broadcast receiver). On `RowsAppended` it invokes in-process `DatasetSliceProvider::read_full()` (MVP) without gRPC. Python keeps using gRPC `GetDatasetSlice`.

### Deferred Index Inference

Triggered either at finalize OR first view needing index columns when none set. Implementation: read Arrow file schema + full scan (iterator reading record batches) and compute uniqueness prefixes.

## 4. File Layout

Per dataset directory:

```
dataset.arrow
metadata.json (optional future portability)
config.json
```

No preview/table or stats files in MVP.

## 5. gRPC / IPC Surface (Additions)

Still required for out-of-process (Python / CLI) only. UI bypasses these.

- `GetConfig(uuid)`
- `SaveConfig(uuid, Config)`
- `GetDatasetSlice(uuid, mode=FULL|TAIL, tail_rows=Optional<u32>)` (MVP: FULL only)

We extend existing Dataset service; no separate visualization service. Implementation delegates to `DatasetViewAccess` trait.

## 6. Python API Contracts

Contract: `create_writer(name, description=None, tags=None, index_columns=None, scan_params_spec=None) -> DatasetWriter`
DatasetWriter:

```python
with create_writer("exp1", scan_params_spec=["temp","voltage"]) as w:
    w.add_row({"temp":1.0,"voltage":2.0,"value":0.5})
    w.add_rows([{...}, {...}])
handle = w.handle  # only after context
df = handle.to_pandas()
```

Errors: ValueError(schema mismatch), RuntimeError(after finalize), KeyError(not found).

## 7. Batch & Flush Strategy

- Parameter: FLUSH_BATCH_SIZE (env or config) default 1024 rows.
- Manual flush for low-latency viz: `w.flush()` exposed.
- Each flush builds Arrow arrays for all columns; columns discovered on first row.
- Single writer per dataset enforced by registry (HashMap<Uuid, ActiveWriteSession>). Attempts to create second writer => error.

## 8. Index Inference Algorithm

Input: Column order C[0..n-1], dataset rows (iterator). Goal: smallest prefix P such that unique tuples coverage ratio=1.0, else highest ratio.
Pseudo:

```
best = (ratio=0.0, prefix_len=None)
for L in 1..=n:
  seen = HashSet<Tuple(C[0..L-1])>
  for row in rows: insert tuple; if seen.len == total_rows break early
  ratio = seen.len / total_rows
  if ratio==1.0: return prefix(L, perfect=true)
  if ratio > best.ratio: best=(ratio,L)
return prefix(best.L, perfect=false)
```

Complexity: O(n \* rows). Acceptable for MVP (assume moderate row counts). Potential future optimization: sampling for large datasets.

## 9. Config Management

- Load: if missing -> default {version:1,index_columns:None,views:{},default_view:"main" (empty view placeholder)}
- Save: atomic write via write to `config.json.tmp` then rename.
- Validation: view roles per chart type; ensure default_view exists.

## 10. Real-Time Events

Extend `AppEvent` enum with:

```rust
DatasetRowsAppended { uuid: String, appended: u64 }
ConfigUpdated { uuid: String }
```

Emission points: flush (RowsAppended), successful config save (ConfigUpdated), finalize (already existing Completed event), abort.

## 11. Error Handling Matrix

| Scenario                         | Layer     | Error Type (Python) | Action                          |
| -------------------------------- | --------- | ------------------- | ------------------------------- |
| Add row after finalize           | Writer    | RuntimeError        | Reject                          |
| Schema mismatch                  | Writer    | ValueError          | Reject row, keep session active |
| Missing scan param               | Writer    | ValueError          | Reject                          |
| Duplicate writer                 | Creation  | RuntimeError        | Abort creation                  |
| Config validation fail           | Config    | ValueError          | No write                        |
| Index inference file read fail   | Inference | OSError             | Log + skip inference            |
| Partial dataset read (truncated) | Reader    | (no error)          | Return partial data             |

## 12. Testing Strategy

Add explicit in-process vs gRPC coverage:

- In-process path: UI-mimic tests call trait directly (no serialization).
- gRPC path: integration tests ensure parity with direct calls (golden responses).
  Unit:
- DatasetWriter: first-row schema lock, add_row/add_rows, flush triggers event stub.
- Index inference: perfect unique, partial unique, override manual.
- Config validation (line vs heatmap required roles).
- Real-time events emission counts.

Integration:

- Python: create_writer -> add rows -> finalize -> to_pandas.
- GUI (stub): simulate RowsAppended and verify slice retrieval API returns new data.

Property / Edge:

- Empty writer (no rows) finalize -> Completed with zero rows (allowed?) or enforce at least one row (decision: allow; index inference may produce empty -> no index_columns).
- Aborted mid-write still readable partial.

## 13. Performance & Future Optimizations

Deferred; baseline acceptable. Future: delta tail API; parquet; sampling inference.

## 14. Risks & Mitigations

| Risk                                                   | Impact              | Mitigation                                                  |
| ------------------------------------------------------ | ------------------- | ----------------------------------------------------------- |
| Full-scan inference on large dataset                   | Latency             | Add sampling threshold later                                |
| Frequent flush for real-time -> small batches overhead | Write amplification | Encourage manual flush or adaptive flush size               |
| Partial file corruption on crash mid-write             | Read failure        | Arrow stream writer expected; consider temp filename future |
| GUI refresh flood                                      | UI lag              | Debounce updates client-side                                |

## 15. Requirement Mapping

See `requirements.md` Traceability; all new components align with sections 6–19.

## 16. Open Decisions

- Whether to introduce a memory-mapped Arrow reader for faster tail reads (post-MVP).
- Tail API batching strategy (lines vs bytes) postponed.
- Tail vs full dataset fetch for incremental refresh: MVP chooses full; tail optimization postponed (Task D5).
- Empty dataset on finalize allowed (treat as Completed). Documented; can revisit if UX confusion.

## 17. Confidence

High (>=90%). Requirements clearly scoped; technical approach leverages existing Arrow writer and event system.
