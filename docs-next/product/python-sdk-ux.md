# Python SDK Usage Guideline

## Status

Proposed product-level SDK usage guideline.

## Purpose

Capture the Python SDK experience that changes how users think about Fricon,
without prematurely accepting exact API syntax, parameter-capture mechanics, or
runner internals.

Most Fricon users will define and execute experiments through Python scripts or
notebooks. The SDK is therefore a primary product surface. Old code sketches in
`dev-docs/` should be interpreted as UX sketches unless an ADR accepts exact
syntax.

This file is intentionally not a detailed API design. It records only the SDK
shapes that would be expensive to change after users build habits around them.

## Usage Stance

Fricon should feel like ordinary Python with a small number of explicit product
concepts:

- a visible notebook context for the current local library and lab context
- an interactive unmanaged path for exploratory runs
- an importable decorated managed-run path for higher provenance
- concise doNd-style helpers for routine scans
- public reopen/export APIs for later analysis

The main ergonomic constraint is low ceremony. Fricon should ask for structure
only where it changes user understanding: which context is active, whether a
run is unmanaged or managed, what scan shape should be plotted, and how results
are reopened later.

## Expected Running Model

Users should be able to start from an ordinary Python session, not from a new
framework runtime. A notebook or script should first establish a visible Fricon
context for the local library and optional lab context. That context is cheap
to keep around, inspect, reset, and pass explicitly.

The default exploratory path is unmanaged: the user's Python process remains in
control, and Fricon records measurements, datasets, notes, lifecycle, and
honest provenance as the script runs. This keeps notebooks, debugging, and
manual device work natural.

When users want higher provenance, they should be able to move selected code
into an importable managed-run entry point. The same measurement logic should
remain normal Python that can be reviewed and tested, but Fricon can run it
under management when the user opts in.

Routine scans should not force users to build schemas by hand. doNd-style
helpers should make common scan shapes concise while still leaving manual
writers and raw schema available for advanced or unusual data.

## Low-Ceremony Expectations

Every public example should be readable as ordinary Python. Fricon-specific
ceremony should be justified by one of these user-visible benefits:

- selecting or inspecting the current library and lab context
- creating a real measurement rather than a loose file or anonymous table
- declaring scan shape so live and historical plots know what the axes mean
- choosing unmanaged versus managed execution honestly
- reopening or exporting results through stable public APIs

Boilerplate that exists only for transport, storage layout, service startup,
local tokens, object graph construction, or future parameter machinery should
stay out of first-contact examples.

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

doNd-style helper for a routine scan:

```python
result = fc.do2d(
    ctx,
    x=("gate", gate_values, dac.set_gate),
    y=("bias", bias_values, dac.set_bias),
    measure={"current": dmm.read},
    title="gate-bias map",
)
```

Importable managed-run entry point:

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
- parameter binding, parameter capture, override, or snapshot mechanics
- code snapshot format, environment capture details, stdout/stderr handling, or
  managed-run lifecycle protocol
- dataset-writer object model, storage layout, service transport, or local
  token mechanics
- full scan-schema representation beyond the need for concise helper UX and a
  raw escape hatch
- scheduler, resource leases, queues, retries, workflow DAGs, or resume
  protocols

Those decisions belong in later ADRs, specs, or implementation design after the
product-level usage guideline is stable.
