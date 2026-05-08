# US-015: Declare Common Scan Shapes Quickly

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-002: New measurement logging replacement.

## Story

As a Measurement Method Author, I want concise Python-native scan plans or
helpers for common scan and trace shapes so that Fricon data has reliable
plotting semantics without making every script write raw schema by hand.

## Success Criteria

- Scan-plan authoring covers common 1D, 2D, N-D, fixed-trace, and
  variable-trace cases.
- Dict/literal-friendly plans are acceptable if they keep axes, setters,
  measured values, labels, and units readable in ordinary Python.
- The accepted simplification shape is informed by real scripts/notebooks and
  feedback, not only by borrowed framework API names.
- Schema can represent regular grids, partial grids, irregular/adaptive points,
  repeated points, and missing expected points where meaningful.
- Variable-length trace scenarios preserve each trace's own coordinate values,
  measured values, and per-trace settings instead of forcing padding,
  resampling, or a fake shared grid at write time.
- Advanced users can use raw schema when scan-plan helpers are too narrow.
- Live and historical plots rely on schema semantics instead of column order
  guesses.

## Not In Scope

- Visual sweep builder as the primary acquisition model.
- Managed execution plans or device control.
- A fixed framework-specific helper name or QCoDeS-style parameter-object
  model.

## Related Capabilities

CAP-006, CAP-028.
