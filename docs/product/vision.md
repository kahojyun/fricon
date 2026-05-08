# Product Vision

## Status

Accepted clean-reset product baseline.

## Thesis

Fricon is a local lab data library and automation foundation for scientific
measurement work.

The near-term product proves a better local measurement write, watch, recover,
reopen, and export loop. The long-term product should become the lab's local
experiment memory and reviewed action layer: a system of record for measurement
intent, effective parameters, code provenance, setup state, analysis attempts,
trust decisions, handoff, and reviewed replay.

## Planning Language

This document uses MVP, post-MVP, and ADR-gated as product priority labels.
They are not semantic-version labels. Compatible improvements can still ship on
the same compatible release line when the storage, API, and compatibility
policies allow it.

## Long-Term Motivation

Physical measurement work is hard to make reliable when data, parameters,
measurement code, setup state, notes, and later analysis live in separate tools
or informal files. Existing frameworks can help users collect data, but they
often leave the broader experiment record to conventions that are difficult to
inspect, compare, migrate, or automate. Fricon should replace not only the
write-to-logger step, but also the informal folder discipline around copied
measurement code, mutable local JSON configuration, setup sidecars, and later
analysis handoff.

Fricon should give experimenters one local-first product model for defining,
running, inspecting, explaining, and reusing measurement work. The long-term
aim is not only to store results, but to make the relationship between a
measurement, its datasets, its Python code, its context, and its later
interpretation explicit enough for humans and future automation to trust.

The highest-value post-MVP lesson from legacy workflows is the code-and-parameter
management loop: copied code, mutable local configuration, generated sidecars,
and notebook-local analysis make calibration hard to trust. Fricon should first
make the recorded facts durable enough to explain, compare, hand off, repeat,
and then safely automate work.

The long-term center is not device control, sample visualization, or AI by
itself. Those capabilities matter when they serve trustworthy experiment
memory, reviewable changes, and safer reuse. Detailed post-MVP ordering lives
in `product/future-concepts.md`; future terminology is owned by
`product/glossary.md`.

The product should stay close to how experimentalists already work: Python
scripts and notebooks remain first-class, local lab computers remain useful
without a server account model, and higher-provenance workflows grow from the
same core experience instead of becoming a separate system.

Staying close to current practice does not mean freezing current practice as
the ideal model. Post-MVP Fricon can introduce higher-provenance workflows, but
only after the relevant facts, review boundaries, and safety model are explicit.

## Adoption Strategy

Fricon is intended for gradual lab adoption. A lab should be able to keep old
runners, notebooks, LabRAD/Data Vault history, local parameter files, generated
sidecars, and folder-based workflows while new measurement work moves into
Fricon.

This does not mean Fricon should preserve every legacy mechanism as a
first-class product model. Legacy paths, numeric IDs, copied folders, mutable
configuration files, generated sidecars, and notebook-local analysis should
enter Fricon as aliases, context, attachments, summaries, or evidence. The
canonical product model should stay centered on measurements, dataset
artifacts, lifecycle, scan schema, provenance, selected configuration context,
exports, and later reviewed parameter and calibration workflows.

Transition features should point toward the full Fricon workflow:

- legacy aliases become stable Fricon IDs
- mutable configuration files become effective snapshots, profiles, and
  reviewed proposals
- copied folders become code provenance or configured measurement-code sources
- unmanaged calibration scripts become calibration evidence and reviewed
  parameter changes
- notebooks, spreadsheets, and presentation decks become traceable analysis or
  handoff artifacts rather than the system of record

If a legacy need cannot fit one of these bridge forms, it should not become an
MVP product concept without explicit product and architecture review.

## MVP Goal

The MVP goal is practical: replace the simple LabRAD Data Vault/Grapher loop
for new measurements.

Success means a user can start new measurement work in Fricon, write data from
Python, watch it live, recover partial results, reopen it later, and export it
without depending on old storage paths or notebook-only reconstruction. The MVP
does not need to import old history or emulate LabRAD; old LabRAD, QCoDeS,
Labber, or folder-based history can remain where it is while new work moves to
Fricon.

## User Promise

Fricon should help a researcher answer:

- What measurement did I run?
- Which sample or session was active, if any?
- What datasets and attachments did it produce?
- Was the run finished, interrupted, failed, invalidated, or recovered?
- What notes, parameters, code provenance, and setup labels explain it?
- Which selected local configuration files or summaries were bound to it?
- How do I inspect it live, reopen it from Python, export it, or recover it?

After the MVP, Fricon should also help answer:

- What changed since the previous good run?
- Why did this run succeed, fail, or become questionable?
- Which parameter, setup, code, analysis, or calibration facts explain the
  difference?
- Which code source and parameter snapshot did this calibration depend on?
- Did each calibration task look healthy, need retry, pause for review, or
  produce fitted parameters that should update a parameter snapshot/profile?
- For a managed routine, what setup or device state was desired, what was
  already current, what did Fricon plan to change, and what actually happened?
- What did we conclude from this measurement?
- Can I repeat this work safely, and what will change if I do?

## Primary Mental Model

```text
I selected an active sample/session when it mattered.
I ran a measurement from Python.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

Post-MVP, the product should extend that model:

```text
I can choose a previous-good run or routine.
Fricon shows the effective parameters, code, setup, and expected artifacts.
I review what will change before anything durable is mutated.
Afterward, Fricon records the outcome, evidence, and handoff state.
```

## Python SDK Experience

The Python SDK is a primary user experience, not only an implementation API.
Most experimentalists will define and run measurements through Python scripts
or notebooks, so SDK ergonomics are product requirements.

At the vision level, the SDK should feel like ordinary Python with low
ceremony: a visible notebook context, natural interactive unmanaged runs,
importable managed-run entry points for higher provenance, Python-native
scan-plan authoring for routine scans, and public reopen/export APIs for later
analysis.

Common workflows should have appropriate simplifications, but exact helper
shapes are not part of the product vision until real usage feedback supports
them.

Detailed SDK usage guidance lives in `product/python-sdk-ux.md`. Historical
planning snippets should be read as non-binding UX sketches unless an ADR
accepts exact API syntax.

## Measurement Code Shape

At product level, measurement code means the user-authored Python that creates
or explains measurement work. In the MVP, this mainly includes ordinary Python
scripts, notebook cell flows, Data Vault-style translated scripts, and copied
lab working folders. Fricon should record honest context for these forms
without pretending it owns their execution.

The MVP code promise is provenance, not code management. Fricon may record an
unmanaged label, optional script or notebook path, Git summary, dirty-state
signal, copied-folder/source-root label, user summary, and export privacy
choice. It should not claim automatic notebook capture, approved code releases,
deployment, immutable code snapshots, or managed execution.

Post-MVP code management should grow toward configured measurement code
sources, approved update flows, importable managed-run entry points, and code
snapshots only after the product facts and review boundaries are clear.

## MVP Scope

To meet the MVP goal, the MVP should include:

- one local data library per normal lab computer
- explicit measurements
- optional sample and sample-session context
- dataset artifacts that remain directly searchable and openable, even though
  the Desktop home is measurement-first
- table-shaped scan and trace data with explicit scan schema for plotted data
- scan modes for regular grids, partial grids, irregular or adaptive points,
  repeated points, and fixed-shape or variable-length traces
- low-ceremony scan-plan/schema authoring for common 1D/2D/N-D scans and
  traces, plus a raw schema escape hatch for advanced cases
- nonblocking live table and chart inspection
- measurement lifecycle, notes, events, favorites/pins, trash/recover, and
  readable partial data semantics
- light attachments
- light contextual summaries for parameters, code provenance, setup,
  environment, and unmanaged procedure context
- selected run-bound local configuration snapshots or summaries, such as
  parameter files, registry files, wiring references, line/chip info, or
  demod/readout settings, without turning MVP into a full parameter registry
- Python reopen snippets through public APIs
- a near-term measurement export spec for portable bundles and common analysis
  formats
- backup/restore and migration checkpoints
- coherent install/update compatibility and guided setup diagnostics

## Product Pressure Checks

Keep these concrete user pressures at product level until architecture or spec
work starts:

- Lab computers are often Windows, offline, locked down, or updated on a lab
  schedule. Fricon should make install, launch, update, and recovery feel like
  one coherent local product, not a set of separately assembled services and
  packages.
- Setup diagnostics should help users distinguish stopped local runtime
  components, wrong library, locked library, old SDK, incompatible components,
  migration-required state, and unsafe update timing before they read logs.
- Export is an analysis workflow, not an archive dump. A user should be able to
  open a measurement bundle directly from Python, inspect a simple manifest or
  index preview, and choose whether sensitive paths, code, environment, setup,
  or sample details are included.
- Dataset artifact semantics should be checked against real measurement shapes,
  including adaptive or instrument-tuned traces where each trace may have its
  own coordinate values, settings, and length.
- Live inspection should support repeated lab monitoring behavior without
  becoming part of the write acknowledgement path. Users may need to watch
  multiple measurements or views at once while acquisition keeps running.
- Measurement-code and run-configuration ideas should stay focused on the user
  pain of copied folders, mutable local files, setup sidecars, scan helpers,
  plot presets, and export recipes. The MVP records honest context; approved
  code update and managed execution remain post-MVP.
- Mutating actions should be attributable enough for local lab history, but the
  MVP must not turn this into accounts, roles, permissions, or remote
  collaboration.

## Post-MVP Direction

At the vision level, post-MVP work should extend the MVP facts into three user
loops:

- Explain: run manifests, parameter/code/setup provenance, lifecycle evidence,
  analysis attempts, and failure or anomaly investigation.
- Compare: previous-good baselines, drift detection, operator handoff,
  analysis/calibration status, and visible differences before repeat work.
- Repeat: run-like-previous drafts, reviewed parameter proposals, routine
  recipes, and audited automation after the facts are trustworthy.

Implementation should follow the accepted priority ledger:

- parameter system first
- managed run second
- calibration chains and reviewable automation third

Sample visualization should stay lightweight until product evidence says
otherwise. Treat a sample visualizer as a view over parameter snapshots and
snapshot query results, not as proof that Fricon understands the sample's
physical shape. User-authored 2D sample-map configs/DSLs may live near sample
records for discoverability, but schema evolution, invalid visualizers, and
visualizer migration policy are later design questions.

Lower-priority or ADR-gated directions include read-only LAN monitoring, richer
sample-map authoring, device communication, resumable execution, user-facing
stream concepts, and AI-assisted automation. Mutation-capable calibration or
device apply remains ADR-gated.

## Non-Goals For MVP

- hosted SaaS
- account/team administration
- multi-user permissions
- distributed database semantics
- direct shared-folder access to one editable data library
- LabRAD Data Vault/Grapher compatibility server
- built-in legacy Data Vault import or browser
- requiring full old-history migration before adopting Fricon for new data
- broad device-driver framework
- generic workflow DAG engine
- visual sweep builder as the primary acquisition model
- automatic notebook state capture
- AI actions that mutate data-library state without explicit review and audit
