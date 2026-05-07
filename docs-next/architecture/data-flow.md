# Deferred Runtime Flow Questions

## Status

Deferred. Product stories and domain lifecycles are still upstream of runtime
architecture.

## Upstream Flow Owners

- `product/story-map.md` owns the user route.
- `domain/lifecycle-model.md` owns lifecycle states and transition rules.
- `domain/invariants.md` owns hard data and provenance rules.
- Later specs should own implementation-slice acceptance and validation.

## Deferred Runtime Questions

- What is the exact write-session lifecycle for measurement creation, dataset
  declaration, append, finish, interruption, and abort?
- Which operations are synchronous acknowledgements versus eventual live
  preview events?
- How are committed append positions or event sequence IDs represented?
- How do live readers coalesce or drop previews without dropping committed
  data?
- How do Python reopen APIs request semantic tables, axes, grids, partial
  grids, irregular/adaptive points, repeated points, and traces?
- Which export operations require an idle state, read-safe state, checkpoint,
  or privacy preview?
- How do updates and migrations report blockers and recovery guidance?

## Accepted Flow Constraints

- Mutating clients fail compatibility checks before writes.
- Live inspection must not block acquisition writes.
- Committed data is not dropped because a live consumer is slow or broken.
- Partial/interrupted measurements remain readable.
- Dataset artifacts remain directly searchable and openable after a
  measurement flow.
