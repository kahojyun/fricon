# Product Vision

## Status

High-confidence product input; derived scope pending revalidation.

## Thesis

Fricon is a local measurement data system for scientific experiment work.

The first product target is a maintained replacement for the fragile
Data Vault/Grapher-centered loop around new interactive measurements. The
long-term product should become the lab's local experiment memory and reviewed
action layer: a system of record for measurement intent, effective parameters,
code provenance, setup state, analysis attempts, trust decisions, handoff, and
reviewed replay.

## Problem Statement

For initial adoption, Fricon helps users record, monitor, and reopen new
interactive experiment data without relying on unmaintained LabRAD
Data Vault/Grapher behavior or notebook-only reconstruction. Export remains
important, but should follow from a reliable reopen path rather than lead the
first usable slice.

## Planning Language

This document uses planning horizons, not semantic-version labels.

Initial Adoption Slice means the first practical Fricon release path that lets
a lab start new measurement work with a useful write, watch, checkpoint-safe
read, and reopen loop. It is not a claim that other capabilities are less
important. It comes first because it can provide standalone value while
generating real Fricon records and usage feedback. Portable export can follow
once local reopen semantics are reliable.

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
often leave the broader experiment record to folder conventions, copied code,
mutable JSON files, sidecars, notebooks, and operator memory.

Notebook-based exploration is not the problem. It is often the most comfortable
way to try an experiment. The failure appears when useful work has to survive
beyond one person, one lab computer, one copied code folder, one Conda
environment, or one uninterrupted storage session.

Fricon should give experimenters a local-first product model for recording,
inspecting, explaining, and reusing measurement work. The long-term center is
trustworthy experiment memory and reviewed reuse, not device control, sample
visualization, or AI by itself.

The product should stay close to current practice: Python scripts and notebooks
remain first-class, local lab computers remain useful without a server account
model, and higher-provenance workflows grow from recorded facts and explicit
review boundaries.

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
artifacts, lifecycle, scan schema, provenance, selected software-visible
context, later exports, and later reviewed parameter and calibration workflows.

Transition features should point toward later Fricon workflows:

- legacy aliases become stable Fricon IDs
- selected mutable configuration files become run-bound context first, and may
  later become effective snapshots, profiles, or reviewed proposals
- copied folders become honest code provenance first, and may later become
  configured measurement-code sources
- unmanaged calibration scripts remain ordinary measurements first, and may
  later become calibration evidence or reviewed parameter changes
- notebooks, spreadsheets, and reports may become traceable analysis or
  handoff artifacts later, but they are not first-slice reporting scope

If a legacy need cannot fit one of these bridge forms, it should not become an
initial adoption concept without explicit product and architecture review.

## Initial Adoption Goal

The initial adoption goal is practical: replace the simple LabRAD Data
Vault/Grapher loop for new interactive measurements and portable later
analysis.

Success means a user can start new measurement work in Fricon, write data from
Python, watch live plots, run multiple independent experiments without
unnecessary global-session interference, keep already-written checkpoints
readable after user interruption or notebook-kernel failure, and reopen data
later without depending on old storage paths or notebook-only reconstruction.
The initial adoption slice does not need to import old history, emulate LabRAD,
ship a LabRAD compatibility layer, guarantee hard-crash recovery beyond later
durable-write decisions, or provide polished export before local reopen works
well. Old LabRAD, QCoDeS, Labber, or folder-based history can remain where it
is while new work moves to Fricon through small explicit recording-code
rewrites.

## User Promise

Fricon should help a researcher answer:

- What measurement did I run?
- Which sample or session was active, if any?
- What datasets and attachments did it produce?
- Was the run finished, interrupted, failed, invalidated, or recovered?
- What notes, user-supplied attributes, code provenance, and software-visible
  context explain it?
- Which selected local configuration files or summaries were bound to it as
  evidence, without pretending Fricon understands all physical setup context?
- Can I run another experiment without this one interfering with it?
- How do I inspect it live, reopen it from Python, or read already-written data
  after an interruption?
- Later, can I export or hand off the result without recreating the acquisition
  runtime?

Strategic follow-on slices can later help answer:

- What changed since the previous good run?
- Why did this run succeed, fail, or become questionable?
- Which parameter, setup, code, analysis, or calibration facts explain the
  difference?
- Which code source and parameter snapshot did this calibration depend on?
- Did calibration evidence look healthy, need retry, pause for review, or
  justify a proposed parameter update?
- For a managed routine, what setup or device state was desired, what was
  already current, what did Fricon plan to change, and what actually happened?
- What did we conclude from this measurement?
- Can I repeat this work safely, and what will change if I do?

## Primary Mental Model

```text
I ran a measurement from Python.
It produced datasets.
Fricon helps me monitor, inspect, and reopen them.
```

Optional sample or session context can replace folder-path habits for grouping
data by sample, cooldown, mount, or campaign. It should be settable once near
the start of a notebook and useful in plots and browsing, but it is not the
center of the first-slice workflow.

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

For initial adoption, the SDK should feel like ordinary Python with low
ceremony: a visible notebook context, natural interactive unmanaged runs,
Python-native scan/schema authoring, and public reopen APIs for later analysis.
Importable managed-run entry points and polished export flows are follow-on
directions, not part of the first usable migration promise.

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

Strategic follow-on code management can grow toward configured measurement
code sources, approved update flows, importable managed-run entry points, and
code snapshots after the product facts and review boundaries are clear.

## Initial Adoption Scope

To meet the initial adoption goal, the first adoption slice should include:

- one local data library per normal lab computer
- explicit measurements
- optional sample and sample-session context for grouping and display
- dataset artifacts that remain directly searchable and openable, even though
  the Desktop home is measurement-first
- declared scan datasets for common 1D, 2D, and N-D sweeps
- step or record datasets for irregular workflows, adaptive scans, and
  instrument-driven coarse/fine passes where each step may carry parameters
  and one or more scalar, array, or trace results. Minimizers should start as
  ordinary ragged or step records rather than special first-slice product
  concepts
- selectable trace or output collections so users can compare coarse/fine
  passes or variable-length optimizer traces without hard-coding those
  experiment types into the product model
- low-ceremony scan-plan/schema authoring for common scans and traces, plus a
  raw schema escape hatch for advanced cases
- nonblocking live monitor views for current 1D line/scatter, simple 2D
  heatmap, IQ scatter, and selected output or trace channels. Live monitor
  controls should stay simple; detailed row selection, overlays, and richer
  analysis belong in a fuller viewer or Python scripts
- richer historical browsing and selector views that do not need to be live
  auto-refresh surfaces. These can grow over time, but are an obvious
  scope-expansion risk
- measurement lifecycle, notes, events, favorites/pins, trash/recover, and
  checkpoint-safe readable data after user interruption or notebook-kernel
  failure
- light attachments
- light user-supplied attributes or notes for physical setup context, and
  honest software-visible context such as code provenance, selected files,
  environment labels, and unmanaged procedure summaries
- selected run-bound local configuration copies, such as parameter files,
  registry files, wiring references, line/chip info, or demod/readout settings.
  Fricon should preserve and return these files in their original user-supplied
  form, with simple text preview or a way to open them in an external editor
  where practical, without turning initial adoption into a parameter parser,
  registry, or source of truth for physical setup
- Python reopen through stable IDs and public APIs
- a generic reader path good enough for users to build experiment-specific
  helpers around. First-slice reader UX should support core tasks before
  promising many polished framework-specific views
- analysis-friendly local reads into common Python objects where appropriate
- backup/restore and migration checkpoints
- coherent install/update compatibility and guided setup diagnostics
- migration documentation for non-obvious script shapes, including N-D sweeps,
  VNA-like coarse/fine trace collections, generic irregular step records, and a
  realistic Data Vault-style recording rewrite

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
- Export is an analysis workflow, not an archive dump, but it can follow after
  local reopen is solid. Early users can reopen data from Python and export the
  analysis-specific format or metadata they need themselves.
- Generic export formats should be demand-driven. A Fricon package plus
  lightweight reader remains the likely later baseline, but should not outrank
  reliable local reopen in the first usable slice.
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
  plot presets, and export recipes. Initial adoption records honest
  software-visible context and user-supplied notes or attributes; approved code
  update, parameter systems, and managed execution remain strategic follow-on.
- Fricon should not run user plotting code in the first slice. Users can reopen
  data from Python and build custom plots themselves; built-in live plotting
  should focus on common measurement monitor views.
- Mutating actions should be attributable enough for local lab history, but the
  first adoption slice must not turn this into accounts, roles, permissions, or
  remote collaboration.

## Strategic Follow-On Direction

Strategic follow-on work should extend the facts recorded during initial
adoption into three user loops:

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
- hard-crash or power-loss recovery beyond accepted durable-write behavior
- parsing arbitrary setup, wiring, registry, or parameter files into trusted
  Fricon-owned truth
- polished portable export before local reopen works well
- AI actions that mutate data-library state without explicit review and audit
