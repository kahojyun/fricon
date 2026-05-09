# Project Context For Agents

## Status

Draft pending greenfield product-analysis baseline.

## Assumption

Read `docs/README.md` first. It owns the documentation reading order,
source-of-truth map, and current product-analysis status.

This file is only an agent overlay: it records operating reminders that are
easy to forget during AI-assisted work. Do not duplicate the README here.

## Current Working Model

The current high-confidence product inputs are `product/vision.md` and
`product/personas.md`. Use `product/product-analysis-progress.md` to understand
which greenfield analysis steps are complete before relying on the current
story map, capability map, or strategic follow-on backlog material.

Primary user model:

```text
I ran an interactive measurement from Python.
It produced datasets.
Fricon helps me monitor, inspect, and reopen them.
```

## Hard Boundaries

- Do not preserve prior workspace/dataset compatibility by default.
- Do not treat draft capabilities, stories, or backlog items as
  implementation-ready.
- Do not reintroduce old product story, epic, capability, or future-story IDs.
  Fresh IDs are allowed later only after the new boundaries are accepted enough
  to need stable references.
- Do not make datasets own measurement, sample, parameter, code, or lifecycle
  meaning.
- Do not make Desktop the durable data backend.
- Do not bypass local runtime compatibility checks for mutating Python APIs.
- Do not introduce SaaS, accounts, teams, roles, or distributed database
  behavior for the initial adoption slice.
- Do not add LabRAD-dependent compatibility helpers, Data Vault parsers, or
  unit-system adapters to the initial adoption slice.
- Do not make first-slice adoption depend on report generation, user plotting
  code execution, code deployment, Conda environment management, or calibration
  automation.
- Do not implement strategic follow-on runner, device, calibration, or AI
  mutation systems before their ADRs/specs exist.

## Good Defaults

- Prefer measurement-scoped dataset writers in public examples.
- Use initial adoption, strategic follow-on, and ADR-gated for product
  priority. Do not use `v0.3` or `v0.4` as shorthand for feature horizons.
- Start product analysis from user journeys and story backbone before deriving
  capabilities.
- Treat high-impact Python SDK ergonomics as product requirements, not only
  implementation details.
- Keep product-level SDK docs focused on usage guidelines and non-binding
  sketches; exact syntax, capture mechanics, and object models belong in later
  ADRs/specs.
- For common workflows, provide appropriate simplification, but do not freeze
  the exact simplification shape before real script/notebook feedback.
- Keep sample/session context optional and correctable.
- Do not turn sample target binding into a heavy physical-component ontology
  unless a later product decision requires it.
- For strategic follow-on parameter, calibration, run-manifest,
  sample-visualizer, and setup/device reconciliation details, read
  `product/future-concepts.md` and `product/glossary.md` as draft backlog and
  terminology context.
- Do not implement future concepts early, but do use them as pressure when
  shaping domain and architecture boundaries. Early measurement, dataset,
  identity, lifecycle, event, provenance, note, attachment, and API models
  should not make later parameter, managed-run, calibration, setup/device,
  manifest, export, or reviewed-automation systems require a large conceptual
  rewrite.
- Make live views noncritical consumers.
- Keep code provenance honest: unmanaged means unmanaged.
- Treat Git metadata as optional context for initial adoption, not a
  reproducibility promise or a required provenance source.
- Use events for lifecycle, notes, corrections, and audit history.
- Use ADRs for storage, local runtime/API, export format, and compatibility
  decisions before durable implementation.
