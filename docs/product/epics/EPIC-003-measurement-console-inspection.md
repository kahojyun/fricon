# EPIC-003: Measurement Console And Inspection

## Status

Draft pending product-analysis revalidation.

## Product Goal

Fricon Desktop opens to current lab work: active/recent measurements, live
datasets, quick plots, notes, favorites, partial/failed runs, and direct reopen
actions.

## Initial Adoption Scope

- Measurement-first console.
- Active and recent measurement list.
- Live monitor views for line/scatter, basic heatmap, selected outputs, simple
  trace inspection, complex-aware magnitude/phase or I/Q views, and IQ scatter
  where dataset semantics support them.
- Historical inspection for sweep-plus-trace heatmaps and selected trace
  comparison or overlay, including coarse/fine trace ranges where available.
- Selector or index views for choosing datasets, scan parameters, timestamps,
  and trace records.
- Ability to keep multiple relevant measurement, monitor, plot, or trace views
  visible while acquisition continues.
- Dataset direct-open entry points with stable-ID copy through right-click
  menus and keyboard shortcuts.
- Favorites, notes, lifecycle flags, and basic search shortcuts.

## Not Initial Adoption

- Publication plotting.
- Running user plotting code inside Fricon.
- Generic dashboard builder.
- Rich saved views and comparison workflows beyond the minimal console.
- Running fitting, classifier tuning, or publication plotting pipelines inside
  the first-slice Desktop.

## Key Stories

- US-005: Watch and inspect live data.
- US-007: Annotate at the right level.
- US-013: Find and open a dataset artifact directly.
