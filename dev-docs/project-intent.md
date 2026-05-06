# Project Intent

## Status

Canonical project direction and AI-assisted development guardrails.

## Purpose

Fricon aims to become an easy-to-use scientific measurement management system.

The project should help researchers record, organize, inspect, and eventually
execute scientific measurement workflows on their own computers without requiring
them to operate a multi-user service or a complex lab information system.

Installation, launch, and update workflows are part of that product promise.
Researchers should not need to manually reason about mismatched desktop, CLI,
Python SDK, and local-service versions before they can record data.
Lab scripts and notebooks may be pinned by `uv.lock`, virtual environments, or
shared lab setup, so the Python SDK cannot be assumed to update in lockstep
with the desktop app or local service.

## Target Users

The primary target users are researchers who have entry-level Python data
analysis ability.

Assumptions:

- They can install Python packages and run Python scripts or notebooks.
- They are comfortable with basic tabular or array-like data concepts.
- They may not be software engineers.
- Their lab computers may be Windows machines; macOS should remain supported
  for development and normal local use.
- They should not need to understand Fricon's internal Rust, IPC, database, or
  file-layout implementation details.

## Long-Term Product Direction

Fricon is expected to provide:

- data recording
- measurement execution
- parameter management, including future snapshot, history, diff, and
  versioning workflows
- device management
- local data-library management
- a desktop UI for browsing, managing, and inspecting collected data
- Python APIs for scripting and automation
- reproducibility support for measurement code, environments, parameters, and
  generated datasets
- repeatable setup for multiple lab computers, including shared measurement
  code sources and local environment guidance without requiring a central
  Fricon server
- workflow definitions above individual measurements for repeated calibration,
  optimization, and benchmark tasks
- AI-assisted automation for repetitive scientific data-management work, with
  explicit review and auditability for mutating actions

The product should make common scientific measurement workflows easier while
remaining scriptable for users who already use Python in their research.

The long-term product should go beyond replacing a logger. After the local
measurement loop is reliable, Fricon should become local experiment memory: a
system that records enough facts to explain what happened, compare against
previous-good runs, hand off state between operators, repeat work with visible
differences, and automate only through reviewed plans and durable audit records.

## Product Route

The product route is now the v0.2 measurement-library reset: Python-led,
local-first, and measurement-centered.

The current implementation is still workspace/dataset-first, but that is
implementation baseline, not the desired product direction. v0.2 should make a
large reset from a dataset catalog into a local data library for measurement
work.

Fricon should first become reliable for recording, organizing, inspecting, and
exporting new measurement records and their table-shaped datasets. Dataset
semantics still matter, but the v0.1 semantics baseline is now current
implementation to preserve and build on, not a separate development track that
should keep postponing the measurement model. In v0.2, explicit scan schema,
column meaning, units, and chart interpretation should support measurement
records.

The first adoption milestone is the v0.2 replacement slice: replacing a simple
LabRAD Grapher/Data Vault style measurement logger for new measurement work.
v0.2 should let users record new measurements from Python, inspect datasets in
Fricon Desktop, keep run-level context beside produced data, and reopen outputs
from Python without depending on the old logger. Importing or fully browsing
legacy LabRAD/Data Vault history is a follow-up migration concern, not a v0.2
requirement.

The v0.2 route should also acknowledge the multi-computer lab problem. Many
labs copy measurement code between acquisition computers because the old data
and code organization gives them no better path. Fricon should not solve this
by centralizing data libraries, but it should start recording where measurement
code came from and leave a path to experimenter-friendly setup and code-update
workflows across lab computers.

v0.2 is allowed to make broad breaking changes while Fricon is still
pre-adoption. Once real lab data is recorded, the project should protect data
durability and provide explicit migrations or recovery paths even if v0.x SDK,
CLI, UI, or service API shapes continue to change.

The primary v0.2 user mental model is:

```text
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

Interactive measurements should become the recommended path for measurement
work once the run API exists. Datasets remain independently addressable data
artifacts, not owned children that can only belong to measurements. This keeps
room for future analysis, import, simulation, and calibration activities that
consume existing datasets and produce new datasets, results, reports, or
parameter proposals.

Initial measurement support should lean on Python scripts as the execution
entry point. Fricon Desktop should browse, inspect, and eventually assist those
workflows, but should not become the primary measurement execution engine before
the Python-led model is clear.

Device management should remain a later foundation. Near-term design may keep
room for device identity and configuration, but should avoid building a broad
driver framework or hardware orchestration layer before real workflows require
one.

Workflow automation should be treated as a layer above individual measurements.
It can eventually coordinate scheduled calibration, optimization, benchmark,
and repeated measurement tasks, but should rely on clear measurement records,
parameter snapshots, provenance, and human approval boundaries.

AI-assisted workflows should be designed as assistive automation rather than
silent authority. AI may help draft snippets, summaries, reports, metadata
cleanup, parameter comparisons, and workflow proposals, but mutating
data-library, parameter, code, workflow, or execution state should remain
explicit, reviewable, and auditable. The initial AI posture is read and
suggest; AI-written annotations, analysis records, calibration actions, and
device or acquisition control require later product decisions.

The proposed v0.2 reset is captured in `v0.2/design.md`. It keeps the
Python-led and local-first constraints while broadening the durable product
model from a workspace/dataset catalog into a local data library centered on
samples, sample sessions, measurement records, dataset artifacts, analysis,
calibration, parameter snapshots, code provenance summaries, and auditability.

The first v0.2 shipped slice should remain narrower than that full model:
record-only measurement history, table-shaped datasets, optional
sample/session context, light attachments, optional parameter snapshots,
honest code provenance levels, guided diagnostics, trash/recover,
backup/restore, and measurement-centered exports. Analysis records, automatic
calibration, managed device communication, and declarative managed measurement
remain later layers.

## Runtime Model

Fricon is local-first. The v0.2 supported runtime model is local-only:

- data lives on the user's local computer
- Fricon Desktop and the Python API operate against one primary local data
  library
- the local service coordinates data-library operations and owns database
  access
- the desktop app may launch, supervise, and configure the local service, but
  should not be the durable data backend
- Python measurement scripts can run headlessly without first opening Fricon
  Desktop
- first-run setup asks where the data library should live and remembers it
- the local service uses a generated local token boundary for mutating access
- measurement code may come from a self-hosted Git service such as Gitea, a
  shared Git repository, read-only network mirror, package cache, or manually
  managed folder, but execution should use a local checkout, local environment,
  or future managed snapshot on the measurement computer
- network storage may be useful as an installer cache, code mirror, export
  destination, or backup destination, but not as the active shared
  database-backed data library

Remote clients, browser-served UI, and remote viewing may be considered after
the local replacement workflow is proven. The first remote phase should be
strict read-only monitoring, browsing, and export. Remote annotations and
remote acquisition writes should wait for stronger auth, actor, and audit
design. Remote access should build on the same local service/API boundary
rather than forcing the project into a hosted service shape or encouraging
multiple machines to open the same data library through a shared folder.

Distribution shape and client/server compatibility have technical
implementation details, but their user impact belongs in product planning.
v0.2 should define the supported compatibility envelope for Fricon Desktop,
CLI, Python SDK, local service, and data-library format. The v0.x promise is
durable recorded data, explicit migrations, and fail-before-write diagnostics
for incompatible clients, not strict API or protocol stability.
Fricon Desktop should install with a compatible local service sidecar for the
normal local workflow. Updates should be staged and applied only when the
service reports that measurements, dataset writers, imports, exports, and
migrations are idle.

## Non-Goals

Fricon does not currently aim to support:

- multi-user collaboration
- server-hosted SaaS operation
- account management, teams, roles, or permissions
- centralized lab administration
- a central Fricon server required only to distribute measurement code
- distributed database semantics
- web-first deployment as the primary experience
- regulated-lab compliance workflow as a v0.2 product goal
- direct multi-machine access to the same database-backed data library through
  a shared folder
- using a shared network folder as the primary editable measurement-code
  workflow
- a full Git forge or broad Git client for ordinary experimenter workflows
- remote mode or browser/PWA distribution in the v0.2 replacement slice

These may become integration concerns someday, but they should not drive the
core architecture now.

Regulated-lab guidance can be useful as a traceability stress test, especially
for audit events, actor labels, timing, checksums, and correction history, but
it should not turn the v0.2 product into a compliance system.

## Design Principles

### Keep The User Model Simple

Users should think in terms of data libraries, samples, sessions, measurements,
datasets, parameters, and devices as those product concepts land. `Experiment`
may remain an informal scientific term or a future grouping/template concept
above measurements, but it should not be the first public v0.2 acquisition
record name. Internal
concepts such as SQLite tables, Arrow chunk files, IPC protocol versions, and
Rust module boundaries belong in developer notes, not in public user
documentation.

### Prefer Local Reliability Over Distributed Flexibility

Because the product is local-first and single-user, prioritize predictable local
state, understandable recovery paths, and clear data-library compatibility over
distributed coordination patterns.

### Keep Python Ergonomic

The Python API is a primary user surface. It should support straightforward
data collection scripts without forcing users to predefine every low-level
schema detail. For datasets intended for live or historical plotting, however,
the acquisition code should provide explicit scan schema such as
setpoint/independent, measured/dependent, fixed/config, monitor/readback roles
and enough axis structure for slicing and display; the code knows this better
than a later chart guesser.

Live viewing is an operational aid for judging whether a run is sane. Live
plots, preview transforms, export preparation, and future analysis hooks should
not slow or fail acquisition writes.

Desktop and documentation workflows should help users get back to Python code.
For example, dataset detail views may eventually provide Python read snippets
that reopen selected datasets through the public API without exposing internal
storage paths.

Quality-of-life features should make common scientific work faster without
changing the user's mental model. Good candidates include preview/export
snippets, saved views, aliases, favorites, optional notes/tags, lifecycle flags,
compare views, and template measurements.

Acquisition should remain Python-first. Visual sweep builders or full
measurement-template systems should follow only if explicit scan schema and
managed-plan previews show that users need an additional authoring layer.

Export workflows should support researchers who move data to another computer
for analysis. Prefer measurement-centered portable exports that can be opened
directly from Python or a read-only viewer over workflows that require creating
and importing into a second local data library before analysis can begin. Include
a simple human-readable manifest/index preview so exported bundles remain
inspectable even before opening Fricon Desktop or Python. Treat exports as
analysis packages, not raw file dumps: preserve IDs, units, checksums,
notes/events, sample/session context, setup/method labels, timing metadata, and
loader snippets where practical.

### Preserve Provenance

Scientific workflows need enough traceability to explain where a result came
from. Fricon should be able to connect runs, datasets, parameters, code
versions, environments, device configuration, notes, imports, exports, and
future workflow definitions without exposing internal storage details as the
user model.

Data libraries should have durable source identity, such as a generated UUID
and user-editable display name, so exported measurements can record where they
came from.

Measurement records should also be able to carry an honest code provenance
level. Non-managed user-run Python may only be `unmanaged` or user-supplied
context. Future managed runs can link to immutable code snapshots resolved from
a configured source, such as a Gitea repository, Git remote, bare mirror, or
release bundle. This helps explain what ran on each lab computer without making
every user learn Git.

When records need correction, prefer appended correction or event history over
silent mutation of completed run facts.

### Leave Room For Core Scientific Entities

Some concepts may be implemented later but should influence early model
boundaries because they are expensive to retrofit. Dataset, run, and parameter
work should leave room for units and display metadata, sample or specimen
identity, dataset lineage, parameter snapshots, workflow definitions, local
automation approvals, setup/method labels, basic clock/timing source metadata,
and event or audit logs.

### Make Advanced Workflows Explicit

Measurement execution, parameter management, and device management should become
explicit product concepts as they mature. Avoid hiding those semantics inside
dataset naming conventions or incidental metadata.

The first explicit measurement model should be Python-led: user scripts perform
measurement work while Fricon records datasets, run metadata, and parameters.
UI-led execution can be introduced later if the Python-led workflow proves too
limited.

Measurement reproducibility may eventually include managed code snapshots,
shared measurement-code packages, and environment management. Git-backed code
history, approved release tags, self-hosted Git services such as Gitea,
read-only network mirrors, and environment tools such as `uv` or `pixi` are
plausible directions, but they should be designed as explicit product
capabilities rather than hidden side effects of dataset writes.

Workflow definitions may eventually orchestrate repeated measurement execution,
scheduled calibration, parameter optimization, and benchmark runs. These
capabilities should record workflow versions, triggers, inputs, outputs,
approval checkpoints, failures, and manual overrides.

AI model integration may eventually automate boring or repetitive work, but
should not bypass product boundaries. AI-generated changes to data,
parameters, code, workflow definitions, or execution plans should leave
auditable records and require user approval unless the operation is explicitly
designed as safe and reversible.

### Keep Architecture Proportional

Use explicit boundaries where workflows cross storage, runtime, IPC, device, or
UI concerns. Avoid general-purpose abstractions that only prepare for future
possibilities without simplifying current product work.

## AI-Assisted Development Guidance

This document is a constraint for AI-assisted changes:

- Do not expand the project toward multi-user SaaS unless explicitly requested.
- Do not expose internal storage or protocol details in `docs/`.
- Keep user-facing docs focused on workflows and stable product concepts.
- Put implementation details, architectural notes, and maintenance rules in
  `dev-docs/`.
- Preserve the Python-led, local-first, measurement-library reset route. Treat
  dataset-first behavior as current implementation baseline only, and treat
  `v0.2/design.md` as the canonical v0.2 planning reset when working on v0.2
  scope.
- Prefer feature-local changes that preserve clear ownership.
- Treat Python API and desktop UI behavior as user-facing contracts.
- Treat internal Rust module boundaries as changeable when doing so improves
  the architecture.

## Open Product Questions

These questions are intentionally unresolved:

- Which historical parameter-management ideas in
  `v0.2/archive/parameter-management-design.md` should be revisited in a future
  focused parameter design?
- How should parameter history, diff, and selected apply workflows be exposed in
  the Python API and desktop UI?
- Beyond v0.2 optional column unit, label, and display hints, what unit and
  display metadata belongs on dataset columns, parameters, or both?
- Beyond v0.2 flexible sample properties and generic sample sessions, how much
  sample/specimen structure is worth adding without overbuilding a lab
  inventory system?
- Which additional run facts beyond v0.2 metadata corrections need immutable
  event history?
- How much dataset lineage is needed for measured, processed, simulation, and
  imported datasets?
- How should measurement code history be captured without surprising users or
  turning Fricon into a general Git client?
- How should shared measurement-code sources or lab code packages be modeled so
  users can set up new acquisition computers without copying folders or
  learning a full Git workflow?
- Which reusable lab assets belong with a measurement-code source, such as
  templates, scan-schema helpers, setup profiles, plot presets, environment
  lock files, or calibration definitions?
- What level of automatic `uv` or `pixi` environment management is useful
  without making measurement setup opaque?
- What should a workflow definition contain beyond a Python entry point,
  parameters, schedules, approval checkpoints, and expected outputs?
- Which calibration, optimization, and benchmark tasks should be first-class
  workflow types?
- Beyond the initial read-and-suggest AI posture, which AI actions may mutate
  data-library, parameter, code, workflow, or execution state, and what approval
  or audit metadata should each class require?
- What AI model/provider/version and prompt-summary metadata is needed for
  reproducibility without storing unnecessary sensitive context?
- What level of device abstraction is useful without overbuilding a hardware
  framework?
- Which data formats and array shapes should be first-class beyond the v0.2
  table-shaped dataset model?
- After read-only LAN viewing, what remote-client use cases are worth
  supporting without introducing multi-user product complexity?
