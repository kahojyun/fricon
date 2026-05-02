# Fricon v0.2 Product Direction

## Status

Proposed v0.2 product direction.

This is not current behavior. Use this document to align planning, ADRs, and
implementation slices before changing public docs or API contracts.

## Product Positioning

Fricon v0.2 should be a local lab data library and automation foundation for
experimental science.

It should help experimentalists:

- explore quickly from Python scripts and notebooks
- record experiment history without manual folder discipline
- track samples, cooldowns, parameters, code, and datasets together
- visualize sample parameters and measurement results
- maintain large parameter sets without silent drift
- run repetitive calibration work with explicit proposals and history
- recover historical context when data, code, or parameters have evolved

Fricon should not make users operate a complex LIMS, hosted service, or
multi-user lab administration system before they can collect data.

## Main User Pain

The current lab pattern Fricon should replace is:

```text
new sample or setup
  -> create a new data-vault folder
  -> sometimes copy an experiment code directory
  -> edit JSON parameters locally
  -> run scripts and save data
  -> later struggle to know which sample, cooldown, code, and parameters
     produced which result
```

This creates fragmented data history, duplicated code, drifting parameters, and
weak calibration provenance.

Fricon v0.2 should replace that pattern with:

```text
one Fricon data library
  -> sample and session records
  -> experiment runs
  -> dataset artifacts
  -> code and environment summaries
  -> parameter snapshots and proposals
  -> analysis and calibration history
```

## Primary User Model

The normal measurement flow should be:

```text
select or create sample
start or select sample session / cooldown
run experiment from Python
watch produced datasets
annotate run and mark quality
analyze in Python or UI
promote useful calibration results through parameter proposals
```

The first-screen product should eventually be organized around current lab work:

- active sample/session
- recent experiment runs
- live datasets
- important notes and quality state
- parameter/calibration status

Dataset browsing remains important, but datasets are outputs inside a broader
measurement history.

## Core User-Visible Concepts

### Data Library

The user should normally have one main Fricon data library, not many workspaces.
The library is the local data root and catalog.

Projects, campaigns, samples, sessions, tags, and saved views should organize
the library without encouraging users to split data and copied code into many
long-lived roots.

### Sample

A sample is the measured physical object, device under test, wafer, chip, batch,
preparation, or specimen.

Samples need:

- stable identity
- display name and aliases
- structured custom fields
- notes, tags, and lifecycle state
- links to sample sessions, experiment runs, datasets, and parameter history
- optional 2D layout or coordinate map

Do not force all labs into a full inventory system. Sample records should be
lightweight but real.

### Sample Session / Cooldown

A sample session records a period or condition under which a sample is used:

- cooldown cycle
- mounting or wiring configuration
- room-temperature characterization session
- wafer probing session
- campaign under a particular setup

The same sample may have multiple sessions. Parameter drift across cooldowns
belongs on sessions and parameter history, not usually on separate sample
identities.

Create a new sample only when the physical identity has meaningfully changed.
If processing creates a new object, record lineage from the old sample to the
new one rather than hiding the change in names or folders.

### Experiment Run

An experiment run is the default record for measurement work.

It should link:

- sample and session
- parameter snapshot or legacy parameter JSON
- code and environment summary
- produced datasets
- notes, tags, quality, and attachments
- continuation or recovery decisions

### Dataset Artifact

A dataset is a data artifact produced or consumed by work. It can be measured,
processed, imported, or simulated.

Datasets must remain directly openable from Python and the desktop UI, but they
should not be the only organizing object.

### Parameter Profile And Snapshot

Large parameter sets should be versioned explicitly.

Fricon should support:

- mutable refs or profiles for current working parameter sets
- immutable snapshots for run facts
- diffs between snapshots
- proposals from analysis or calibration
- validation and explicit promotion to refs

Calibration should not silently mutate important profiles during measurement.

### Code And Environment Summary

Fricon should reduce the need to copy code directories when sample or experiment
context changes.

Start with passive summaries:

- entry point or script path
- Git commit, dirty state, or file hash summary when available
- Python/fricon versions
- lock file or environment summary when practical

Do not make v0.2 a full Git client or environment manager.

## Sample Parameter Visualization

Sample visualization should be treated as a first-class product need.

User story:

> As an experimentalist, I want to define sample parameters and visualize them
> on a 2D layout so that I can choose measurement targets, compare device
> variation, and understand drift across sessions.

Capabilities to design toward:

- JSON/table editor for sample fields
- typed custom fields where useful
- 2D coordinate map or layout
- color by parameter, measurement result, quality, or calibration state
- link plotted points to experiment runs and datasets
- compare values across sessions/cooldowns
- show drift or history for selected sample points

This is likely as important as generic dataset browsing for many labs.

## v0.2 Must-Have User Stories

### Create A Local Data Library

As an experimentalist, I want one Fricon data library so that data, sample
records, parameters, code summaries, and run history do not fragment into many
folders.

### Register A Sample

As an experimentalist, I want to create a lightweight sample record with custom
fields so that measurement data is tied to the object I measured.

### Start A Sample Session

As an experimentalist, I want to record a cooldown or measurement session for a
sample so that drift and context changes are tracked without pretending each
cooldown is a new sample.

### Run An Exploratory Experiment

As an experimentalist, I want to run a measurement from Python with minimal
boilerplate so that Fricon records the run, datasets, sample/session context,
and basic provenance.

### Watch And Inspect Data

As an experimentalist, I want live and historical table/chart views so that I
can decide whether a measurement is working.

### Annotate Once At The Right Level

As an experimentalist, I want to put notes, tags, and quality on the experiment
or sample/session by default so that I do not have to annotate every dataset.

### Reopen Data From Python

As an analyst, I want stable IDs and read snippets so that I can reopen
experiment outputs without knowing storage paths.

### Track Parameter Evolution

As an experimentalist, I want parameter snapshots, diffs, and proposals so that
large parameter sets do not drift through untracked JSON edits.

### Avoid Code Directory Copies

As an experimentalist, I want Fricon to record code and environment summaries
for runs so that I do not copy experiment code directories just to preserve
history.

### Visualize Sample Parameters

As an experimentalist, I want a 2D sample map colored by parameters or results
so that I can choose devices/regions and compare behavior across sessions.

### Calibrate With Reviewable Automation

As an experimentalist, I want scheduled or repeated calibration to produce
analysis results and parameter proposals so that tedious updates are automated
without silently mutating important parameter profiles.

## Managed Experiment Framework Direction

Simple experiments should remain ordinary Python. Users should not need to
learn a declarative framework before they can collect data.

For repeated or automation-heavy experiments, Fricon may provide an optional
managed experiment framework where users declare how parameter snapshots,
run-local inputs, and scan points resolve into desired device state and dataset
outputs.

User story:

> As an experimentalist with a repeated scan, I want to declare the intended
> device state for each scan point so that Fricon can preview the run, debug it
> with dummy devices, resume after interruption, and attach post-processing or
> calibration hooks.

The managed framework should help with:

- previewing the device state that will be applied before touching hardware
- inspecting differences between current device state and requested state
- using dummy or simulated devices for dry runs
- restart and resume after an interrupted scan
- explicit scan point identity and post-processing hooks
- safer automatic calibration workflows
- stronger provenance for device apply, readback, and output datasets

This should be a recommended integration path for managed runs, not the only
valid experiment style. Existing imperative experiment code should still be able
to record interactive runs and datasets, but advanced retry, resume, dry-run,
and automatic calibration behavior may require the managed declarative API.

## Later User Stories

- Remote monitoring from another machine.
- Managed submitted experiments with queue and resource leases.
- Analysis-run UI for derived datasets and reports.
- Calibration workflow templates and history views.
- Import of legacy LabRAD/Data Vault history.
- Device identity and readback verification.
- AI-assisted metadata cleanup, reports, and calibration explanations.

## Product Non-Goals For v0.2

- Multi-user lab administration.
- Hosted SaaS.
- Full permissions/roles UI.
- Complete notebook state capture.
- Automatic code rewrite or environment management.
- Broad hardware driver framework.
- Generic workflow DAG engine as the first automation layer.
