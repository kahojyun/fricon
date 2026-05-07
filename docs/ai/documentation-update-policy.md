# Documentation Update Policy

## Status

Accepted.

## Before Implementation

For v0.2+ system work:

- read the relevant product, domain, architecture, and ADR documents
- treat `docs/domain/` as a placeholder until the needed domain baseline is
  rebuilt from accepted product inputs
- confirm the spec status is Accepted or the user explicitly permits Draft work
- update docs first if implementation would contradict accepted direction
- record non-goals and compatibility impact in the spec

## During Implementation

Update docs in the same change when:

- a public concept, field, lifecycle state, or capability changes
- a module boundary changes
- a compatibility or migration rule changes
- a validation requirement changes

Keep docs token-efficient. Avoid large tables and broad duplicated lists. If a
table starts to grow, split the stable overview from the detailed story, spec,
or ADR that actually needs the detail.

## Numbered ID Management

Stable numbered IDs must be easy to allocate without scanning unrelated
priority lists.

- For substantial entries, prefer one file per ID in a directory whose filenames
  sort numerically, like `product/user-stories/US-001-...md`.
- For short entries kept in one file, keep the authoritative definitions in
  ascending ID order within that file.
- Put unstable ordering concerns such as priority, release slice, or milestone
  sequence in separate index sections that reference IDs.
- To allocate a new ID, list or search only the owning directory/file, choose
  the lowest unused number after the current maximum for that prefix, and never
  fill an old gap by reusing a removed ID.
- When a numbered entry moves between priority or milestone groups, update the
  index reference only; do not move the definition unless the owning file's ID
  order requires it.

## Compact Authoring

- Keep top-level maps short enough for future AI sessions to skim.
- Split detailed acceptance criteria into the owning story, spec, or ADR.
- Prefer ID indexes over duplicated tables when the same entries need multiple
  views.

## After Implementation

Gate D documentation sync requires:

- spec tasks reflect actual status
- traceability lists affected modules/files at the right granularity
- future user docs or the user-documentation plan are updated if implemented
  behavior is user-visible
- architecture docs or ADRs reflect significant deviations
- stale docs are listed with next actions if immediate update is not possible

## Staleness Policy

Mark stale documents explicitly:

- `Superseded` when replaced by a newer source of truth
- `Deprecated` when retained for history but not guiding new work
- `Draft` when exploratory

Do not silently delete superseded planning material unless the user asks for
cleanup.
