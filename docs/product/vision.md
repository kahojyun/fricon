# Product Vision

## Status

High-confidence product input; derived scope pending revalidation.

## Thesis

Fricon is a local lab data library and automation foundation for scientific
measurement work.

The near-term product replaces the fragile Data Vault/Grapher-centered loop
for new interactive measurement work with a maintained local system for
recording, monitoring, reopening, and exporting experiment data. The long-term
product should become the lab's local experiment memory and reviewed action
layer: a system of record for measurement intent, effective parameters, code
provenance, setup state, analysis attempts, trust decisions, handoff, and
reviewed replay.

## Problem Statement

For the initial adoption slice, Fricon replaces the fragile
Data Vault/Grapher-centered measurement loop with a maintained local system for
recording, monitoring, reopening, and exporting new interactive experiment
data.

## Planning Language

This document uses planning horizons, not semantic-version labels.

Initial Adoption Slice means the first practical Fricon release path that lets
a lab start new measurement work with a useful write, watch, recover, reopen,
and export loop. It is not a claim that other capabilities are less important.
It comes first because it can provide standalone value while generating real
Fricon records and usage feedback.

Strategic Follow-On means product-core capabilities that remain central to
Fricon's thesis, but are sequenced after the initial adoption slice because
they depend on accepted product evidence, domain semantics, architecture
decisions, or safety and review boundaries. Parameter management, managed run,
calibration evidence, and reviewed automation belong here by dependency, not by
importance.

ADR-gated means a direction may be valuable, but should not become durable
product scope before its compatibility, safety, data semantics, or architecture
decision is recorded.

## Long-Term Motivation

Physical measurement work is hard to make reliable when data, parameters,
measurement code, setup state, notes, and later analysis live in separate tools
or informal files. Existing frameworks can help users collect data, but they
often leave the broader experiment record to conventions that are difficult to
inspect, compare, migrate, or automate. Fricon should replace not only the
write-to-logger step, but also the informal folder discipline around copied
measurement code, mutable local JSON configuration, setup sidecars, and later
analysis handoff.

The adoption pain is not that notebook-based exploration is inherently wrong.
Interactive scripts and notebooks can be the comfortable way to try an
experiment. The failure appears when useful work has to survive beyond one
person, one lab computer, one copied code folder, one Conda environment, or one
uninterrupted storage session. Multiple machines may contain similar-looking
measurement-code folders, waveform helpers, and analysis utilities whose real
differences are hard to identify. Collaboration then depends on personal
discipline instead of a maintained product model.

Fricon should give experimenters one local-first product model for defining,
running, inspecting, explaining, and reusing measurement work. The long-term
aim is not only to store results, but to make the relationship between a
measurement, its datasets, its Python code, its context, and its later
interpretation explicit enough for humans and future automation to trust.

The highest-value later lesson from legacy workflows is the code-and-parameter
management loop: copied code, mutable local configuration, generated sidecars,
and notebook-local analysis make calibration hard to trust. Fricon should first
make the recorded facts durable enough to explain, compare, hand off, repeat,
and then safely automate work.

The long-term center is not device control, sample visualization, or AI by
itself. Those capabilities matter when they serve trustworthy experiment
memory, reviewable changes, and safer reuse. Strategic follow-on ordering is a
draft hypothesis in `product/future-concepts.md`; future terminology lives in
`product/glossary.md`.

The product should stay close to how experimentalists already work: Python
scripts and notebooks remain first-class, local lab computers remain useful
without a server account model, and higher-provenance workflows grow from the
same core experience instead of becoming a separate system.

Staying close to current practice does not mean freezing current practice as
the ideal model. Later Fricon slices can introduce higher-provenance workflows,
but only after the relevant facts, review boundaries, and safety model are
explicit.

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
initial adoption concept without explicit product and architecture review.

## Initial Adoption Goal

The initial adoption goal is practical: replace the simple LabRAD Data
Vault/Grapher loop for new interactive measurements and portable later
analysis.

Success means a user can start new measurement work in Fricon, write data from
Python, watch live plots, run multiple independent experiments without
unnecessary global-session interference, recover partial results, reopen data
later, and export it to another computer for analysis without depending on old
storage paths or notebook-only reconstruction. The initial adoption slice does
not need to import old history, emulate LabRAD, or ship a LabRAD compatibility
layer; old LabRAD, QCoDeS, Labber, or folder-based history can remain where it
is while new work moves to Fricon through small explicit recording-code
rewrites.

## User Promise

Fricon should help a researcher answer:

- What measurement did I run?
- Which sample or session was active, if any?
- What datasets and attachments did it produce?
- Was the run finished, interrupted, failed, invalidated, or recovered?
- What notes, parameters, code provenance, and setup labels explain it?
- Which selected local configuration files or summaries were bound to it?
- Can I run another experiment without this one interfering with it?
- How do I inspect it live, reopen it from Python, export it, or recover it?
- Can I analyze the exported result on another computer without recreating the
  acquisition runtime?

Strategic follow-on slices should also help answer:

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

Strategic follow-on slices should extend that model:

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

For the first adoption slice, migration from Data Vault-style scripts should
mean a simple rewrite of the recording section, not emulation of LabRAD or its
unit system. Users should be able to keep experiment logic, instrument calls,
waveform generation, and analysis utilities outside Fricon while replacing the
writer and live-inspection path.

Common workflows should have appropriate simplifications, but exact helper
shapes are not part of the product vision until real usage feedback supports
them.

Detailed SDK usage guidance lives in `product/python-sdk-ux.md`. Historical
planning snippets should be read as non-binding UX sketches unless an ADR
accepts exact API syntax.

## Measurement Code Shape

At product level, measurement code means the user-authored Python that creates
or explains measurement work. In the initial adoption slice, this mainly
includes ordinary Python scripts, notebook cell flows, Data Vault-style
translated scripts, and copied lab working folders. Fricon should record honest
context for these forms without pretending it owns their execution.

The initial adoption code promise is provenance, not code management. Fricon
may record an unmanaged label, optional script or notebook path, copied-folder
or source-root label, user summary, selected local configuration snapshots, and
export privacy choice. Git state can be recorded when it is meaningful, but it
is not a first-slice success requirement because many lab folders are old
copies or machine-local working trees. Fricon should not claim automatic
notebook capture, approved code releases, deployment, immutable code snapshots,
or managed execution.

Strategic follow-on code management should grow toward configured measurement
code sources, approved update flows, importable managed-run entry points, and
code snapshots only after the product facts and review boundaries are clear.

## Initial Adoption Scope

To meet the initial adoption goal, the first adoption slice should include:

- one local data library per normal lab computer
- explicit measurements
- optional sample and sample-session context
- dataset artifacts that remain directly searchable and openable, even though
  the Desktop home is measurement-first
- declared scan datasets for common 1D, 2D, and N-D sweeps
- step or record datasets for irregular workflows such as minimizers, adaptive
  scans, and instrument-driven coarse/fine passes where each step may carry
  parameters and one or more scalar, array, or trace results
- selectable collections of logs or traces so users can compare coarse/fine
  passes or variable-length optimizer traces without hard-coding those
  experiment types into the product model
- low-ceremony scan-plan/schema authoring for common scans and traces, plus a
  raw schema escape hatch for advanced cases
- nonblocking live monitor views for current 1D line/scatter, 2D heatmap, and
  selected output channels or logs
- richer historical browsing and selector views that do not need to be live
  auto-refresh surfaces
- measurement lifecycle, notes, events, favorites/pins, trash/recover, and
  readable partial data semantics
- light attachments
- light contextual summaries for parameters, code provenance, setup,
  environment, and unmanaged procedure context
- selected run-bound local configuration copies, snapshots, references, or
  summaries, such as parameter files, registry files, wiring references,
  line/chip info, or demod/readout settings, without turning initial adoption
  into a full parameter registry
- Python reopen snippets through public APIs
- a portable Fricon package readable by a lightweight Python reader without
  running the acquisition-time local runtime or Desktop
- analysis-friendly reads into common Python objects such as NumPy, pandas, or
  Polars where appropriate
- backup/restore and migration checkpoints
- coherent install/update compatibility and guided setup diagnostics
- migration documentation for non-obvious script shapes, including N-D sweeps,
  VNA-like coarse/fine trace collections, minimizer-style irregular traces, and
  a realistic Data Vault-style recording rewrite

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
- Required metadata should stay limited to facts needed for later
  interpretation, slicing, and plotting. Labels, units, notes, sample/session
  context, operator labels, and free-form JSON are valuable examples and
  optional records, not reasons to block ordinary writes when absent.
- Live inspection should support repeated lab monitoring behavior without
  becoming part of the write acknowledgement path. Users may need to watch
  multiple independent measurements or views at once while acquisition keeps
  running. The Desktop should remain measurement-first; live monitors can be
  opened from measurement, dataset, or session context rather than assuming one
  global active experiment.
- Measurement-code and run-configuration ideas should stay focused on the user
  pain of copied folders, mutable local files, setup sidecars, scan helpers,
  plot presets, and export recipes. Initial adoption records honest context;
  approved code update and managed execution remain strategic follow-on.
- Fricon should not run user plotting code in the first slice. Users can reopen
  data from Python and build custom plots themselves; built-in live plotting
  should focus on common measurement monitor views.
- Mutating actions should be attributable enough for local lab history, but the
  first adoption slice must not turn this into accounts, roles, permissions, or
  remote collaboration.

## Strategic Follow-On Direction

At the vision level, strategic follow-on work should extend the facts recorded
during initial adoption into three user loops:

- Explain: run manifests, parameter/code/setup provenance, lifecycle evidence,
  analysis attempts, and failure or anomaly investigation.
- Compare: previous-good baselines, drift detection, operator handoff,
  analysis/calibration status, and visible differences before repeat work.
- Repeat: run-like-previous drafts, reviewed parameter proposals, routine
  recipes, and audited automation after the facts are trustworthy.

Implementation should not treat strategic follow-on ordering as accepted until
the product baseline is revalidated. The current strategic follow-on priority
hypothesis is:

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

## Non-Goals For Initial Adoption

- hosted SaaS
- account/team administration
- multi-user permissions
- distributed database semantics
- direct shared-folder access to one editable data library
- LabRAD Data Vault/Grapher compatibility server
- built-in LabRAD-dependent helper module or unit-system adapter
- built-in legacy Data Vault import or browser
- requiring full old-history migration before adopting Fricon for new data
- first-slice report or presentation generation
- running user plotting code inside Fricon
- broad device-driver framework
- generic workflow DAG engine
- visual sweep builder as the primary acquisition model
- automatic notebook state capture
- AI actions that mutate data-library state without explicit review and audit
