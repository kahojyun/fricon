# Internal Rust Doc Comment Guideline

## Goal

Higher-signal comments in boundary-heavy and workflow-heavy code, while keeping
obvious local code lightly documented. Optimized for safe maintenance —
especially with AI assistance.

## When to document

| Code type                                         | Level of documentation                                          |
| ------------------------------------------------- | --------------------------------------------------------------- |
| Module (`//!`) with cross-layer coordination      | **Always** — ownership, invariants, sequencing, extension notes |
| Public/`pub(crate)` trait or struct at a boundary | **Always** — owns, reads/mutates, key invariants                |
| Multi-step workflow method                        | **Always** — preconditions, step sequence, rollback semantics   |
| Simple CRUD passthrough                           | One-liner or skip                                               |
| Obvious local helper                              | Skip                                                            |

## What to cover

Prefer contract-oriented comments over decorative section headers or
line-by-line narration.

For internal modules and important private helpers, prefer compact Rustdoc that
explains:

- **Ownership** — what the unit owns, what collaborators it delegates to.
- **Reads / mutates** — what state (database, filesystem, events) the function
  touches.
- **Invariants** — what must be true before and after the call.
- **Side effects** — events published, files written, etc.
- **Sequencing & rollback** — step order for multi-step workflows, what happens
  on failure at each step.
- **Error conditions** — which error variants are returned and when.
- **Extension notes** — "if you change X, also update Y" guidance where a field
  or format change requires coordinated updates in multiple locations.

## Style

- Use `//!` for module-level docs, `///` for items.
- Keep comments terse: a sentence or two per bullet. Use Rustdoc sections
  (`# Ownership`, `# Invariants`, etc.) only when there are multiple concerns
  worth separating.
- Do not restate what the code already says. Focus on _why_ and _what contract_,
  not _how_.
- When a trait method has a behavioral contract that implementations must follow,
  document it on the trait, not on each impl.

## Example

```rust
//! Dataset catalog service — orchestrates repository, filesystem, and event
//! side effects for dataset lifecycle operations.
//!
//! # Ownership
//!
//! This service owns the high-level dataset lifecycle. It coordinates:
//! - **Repository**: database state.
//! - **Storage**: live and graveyard filesystem layouts.
//! - **Events**: downstream notifications after successful state changes.
//!
//! # Extension notes
//!
//! Adding a field to `DatasetRecord` may require updates in
//! `ExportedMetadata`, `compute_diffs`, and the repository adapter.
```

```rust
/// Permanently delete a dataset that is already in trash.
///
/// # Preconditions
///
/// The dataset must be trashed (`trashed_at` set) and not yet deleted.
///
/// # Sequencing
///
/// 1. Move live directory → graveyard (filesystem).
/// 2. Mark record deleted (database) and publish `Updated` event.
/// 3. Best-effort graveyard cleanup. Failures are logged;
///    `garbage_collect_deleted_datasets` will retry later.
```
