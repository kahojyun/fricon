# Python SDK Usage Guideline

## Status

Draft pending SDK experience revalidation.

## Purpose

Capture the Python SDK experience that changes how users think about Fricon,
without prematurely accepting exact API syntax, parameter-capture mechanics, or
runner internals.

Most Fricon users will define and execute experiments through Python scripts or
notebooks. The SDK is therefore a primary product surface. Historical code
sketches should be interpreted as UX sketches unless an ADR accepts exact
syntax.

This file is intentionally not a detailed API design. It records only the SDK
shapes that would be expensive to change after users build habits around them.

## Usage Stance

For initial adoption, Fricon should feel like ordinary Python with a small
number of explicit product concepts:

- a visible notebook context for the current local library and lab context
- an interactive unmanaged path for exploratory runs
- a lightweight way to bind selected local configuration context to a run
- Python-native scan-plan authoring for routine scans
- a dedicated trace-writing path for trace-valued records inside outer sweeps
- first-class complex values so SDK, readers, and plots can expose meaningful
  magnitude/phase or I/Q views from data semantics
- public reopen APIs for later analysis
- a simple migration path where Data Vault-style scripts rewrite the recording
  section instead of depending on a LabRAD compatibility layer

Importable managed-run entry points are a strategic follow-on SDK direction,
not part of the first-slice migration promise.

The notebook-friendly reusable context/handle style is part of the product
experience: a user should be able to establish the current local library and
optional lab context once, inspect it in a notebook, reset it when needed, and
pass it explicitly to measurement helpers. This does not accept a specific
entry-point name such as `fricon.library()` or `fc.open()`.

Notebook or process session association may be useful for monitor-window reuse
and runtime connection behavior, but it should not become a user-facing product
concept unless later design evidence requires it.

The main ergonomic constraint is low ceremony. Fricon should ask for structure
only where it changes user understanding: which context is active, whether a
run is unmanaged or managed, which selected local configuration helps explain
the run, what scan shape should be plotted, and how results are reopened later.

In this guideline, measurement code remains ordinary Python first: notebooks,
scripts, translated Data Vault-style scripts, importable functions, and local
lab folders are all plausible shapes. The initial adoption slice should record
honest provenance for those shapes. Managed-run entry points, code snapshots,
approved code sources, and update flows are strategic follow-on product
directions, not accepted SDK API mechanics.

Initial migration should be a small explicit rewrite of writer calls. Fricon
should not depend on LabRAD internals, LabRAD services, or LabRAD's unit system
for the first slice because local installations may diverge from upstream and
legacy unit objects may not map cleanly.

For common workflows, Fricon should provide appropriate simplification without
pretending the right simplification is known before use. The exact API shape
should be refined from real script and notebook experience, while this
guideline preserves the product intent.

## Expected Running Model

Users should be able to start from an ordinary Python session, not from a new
framework runtime. A notebook or script should first establish a visible Fricon
context for the local library and optional lab context. That context is cheap
to keep around, inspect, reset, and pass explicitly.

The default exploratory path is unmanaged: the user's Python process remains in
control, and Fricon records measurements, datasets, notes, lifecycle, and
honest provenance as the script runs. This keeps notebooks, debugging, and
manual device work natural.

Strategic follow-on managed runs should let users move selected code into
importable entry points while keeping the measurement logic normal Python. That
later path should not make managed execution mandatory for exploratory scripts.

Routine scans should not force users to build schemas by hand. A Python-native
scan plan should make common scan shapes concise while still leaving manual
writers and raw schema available for advanced or unusual data.

The product requirement is the low-ceremony scan-plan experience, not a
specific helper name or a QCoDeS-style parameter-object model. A plan may be
dict/literal-friendly, a small helper object, a function wrapper, or a mix of
these after usage feedback and API design are accepted.

Trace-valued records need their own low-ceremony path. The product expectation
is that a user can append an outer sweep record with one or more traces,
including explicit coordinate/value arrays or compact regular-coordinate
forms, without manually flattening every trace point into scalar table rows.
The exact names and object model remain deferred.

## Low-Ceremony Expectations

Every public example should be readable as ordinary Python. Fricon-specific
ceremony should be justified by one of these user-visible benefits:

- selecting or inspecting the current library and lab context
- creating a real measurement rather than a loose file or anonymous table
- declaring scan shape so live and historical plots know what the axes mean
- labeling unmanaged execution honestly, and later choosing managed execution
  only when that feature exists
- binding selected local configuration context when copied files or sidecars
  explain the run
- reopening results through stable public APIs and stable IDs
- copying or reusing stable IDs as the normal input users change in later
  analysis code
- leaving room for later export APIs without making export the first SDK
  success path

Boilerplate that exists only for transport, storage layout, local runtime
startup, local tokens, object graph construction, or future parameter machinery
should stay out of first-contact examples.

## Trace And Reopen Expectations

Trace support is part of the SDK experience, not only a storage concern. For a
VNA-style scan, users should be able to write an outer record such as voltage
and power plus trace payloads whose coordinates and values are known to
Fricon. Some traces have explicit coordinate arrays, while others are naturally
described by a start value, delta, and value array. A record may need more than
one trace.

Complex values should be preserved as complex values at product level. First
adoption can expose I/Q or magnitude/phase views for plotting and reading, but
those views should come from first-class complex semantics rather than naming
conventions.

Reader ergonomics should account for stable analysis code. Users may copy a
stable measurement or dataset ID from Desktop or CLI and paste it into an
existing reader call. Full reader snippets, export-reader snippets, and plot
snippets are still useful later, but they do not need to be the fastest path if
ID copy is reliable and discoverable.

Reader APIs should start with a generic, predictable view that users can wrap
for experiment-specific analysis. Trace-bearing datasets may later need a
record-centric nested table, an exploded trace table, an opt-in sample-level
long table, or a labeled array view when axes are rectangular enough. These
are reader UX pressures, not first-slice promises or an accepted internal
storage model.

IQ single-shot reads should make ndarray-style analysis easy when the data is
shape-compatible: sweep dimensions first, shot dimension last, with labels and
metadata available for prepared states, channels, and classification context.
Generic irregular or ragged step reads can cover minimizer output at first.
Fricon should not add minimizer-specific product behavior before managed
measurement or parameter workflows need it.

Trace readers should support common downstream actions without copying old
plotting pipelines: selecting traces from a sweep, drawing several traces in
one line plot, and building 2D heatmaps from sweep-plus-trace data. Combining
or comparing coarse/fine trace ranges can start as user-side helper logic and
move into Fricon later if repeated use justifies it.

## Illustrative Sketches

These sketches are non-binding. They show the desired user feel, not accepted
names, signatures, decorators, or object models.

Interactive notebook setup:

```python
import fricon as fc

ctx = fc.open("lab-library")
ctx.use_sample("sample-a")
ctx
```

Interactive unmanaged measurement:

```python
with ctx.measurement("gate sweep") as run:
    data = run.scan1d("gate", values=gate_values, y="current")

    for gate in gate_values:
        dac.set_gate(gate)
        data.append(gate=gate, current=dmm.read())
```

Python-native scan-plan sketch:

```python
result = fc.run_scan(
    ctx,
    title="gate-bias map",
    plan={
        "axes": [
            {"name": "gate", "values": gate_values, "set": dac.set_gate},
            {"name": "bias", "values": bias_values, "set": dac.set_bias},
        ],
        "measure": {"current": dmm.read},
    },
)
```

Strategic follow-on managed-run sketch:

```python
import fricon as fc

@fc.managed_run
def gate_sweep(ctx, gate_values):
    with ctx.measurement("gate sweep") as run:
        data = run.scan1d("gate", values=gate_values, y="current")
        for gate in gate_values:
            dac.set_gate(gate)
            data.append(gate=gate, current=dmm.read())
```

## Deferred Detail

Do not settle these in this guideline:

- exact Python names, decorator syntax, context-manager syntax, or function
  signatures
- parameter binding, parameter capture, override, run configuration, or snapshot
  mechanics
- code snapshot format, environment capture details, stdout/stderr handling, or
  managed-run lifecycle protocol
- dataset-writer object model, storage layout, runtime transport, or local
  token mechanics
- full scan-schema representation beyond the need for concise scan-plan UX and
  a raw escape hatch
- a QCoDeS compatibility layer, fixed borrowed helper name, or
  parameter-object model
- a LabRAD compatibility layer, LabRAD-dependent helper module, or LabRAD unit
  adapter
- scheduler, resource leases, queues, retries, workflow DAGs, or resume
  protocols

Those decisions belong in later ADRs, specs, or implementation design after the
product-level usage guideline is stable.
