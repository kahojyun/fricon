# Strategic Follow-On Future Systems Research

## Status

Draft research synthesis.

## Review Date

2026-05-06.

## Purpose

Capture background lessons for strategic follow-on parameter systems, managed
code snapshots, runner capture, setup state, calibration history, and generated
run history.

This research informs `product/future-concepts.md`; it does not change initial
adoption scope.

## Sources

Traditional measurement and control systems:

- LabRAD Data Vault/Grapher:
  https://sourceforge.net/p/labrad/wiki/QuickStartDataVaultAndGrapher/
- QCoDeS measurement and station snapshots:
  https://microsoft.github.io/Qcodes/examples/DataSet/Performing-measurements-using-qcodes-parameters-and-dataset.html
  https://microsoft.github.io/Qcodes/examples/basic_examples/Station.html
  https://microsoft.github.io/Qcodes/examples/DataSet/Working%20with%20snapshots.html
- Bluesky documents and event model:
  https://blueskyproject.io/bluesky/main/documents.html
  https://blueskyproject.io/event-model/main/explanations/data-model.html
- Tiled and Bluesky Tiled Plugins:
  https://blueskyproject.io/tiled/getting-started/what-is-tiled.html
  https://blueskyproject.io/bluesky-tiled-plugins/
- Labber:
  https://www.keysight.com/content/keysight/zz/en/products/all-instrument-software/labber-software.html
- Labbench and PyMeasure:
  https://pages.nist.gov/labbench/guide/02_getting_started/03%20data%20logging.html
  https://pymeasure.readthedocs.io/en/stable/tutorial/procedure.html

Code provenance, runner, and experiment-history systems:

- Sacred:
  https://sacred.readthedocs.io/en/latest/experiment.html
  https://sacred.readthedocs.io/en/stable/optional.html
- MLflow:
  https://mlflow.org/docs/latest/ml/tracking/
  https://mlflow.org/docs/latest/ml/projects/
- W&B:
  https://docs.wandb.ai/models/track
  https://docs.wandb.ai/guides/app/features/panels/code
- DVC:
  https://dvc.org/doc/use-cases/experiment-tracking
  https://dvc.org/doc/start/data-pipelines/data-pipelines
- Sumatra:
  https://sumatra.readthedocs.io/en/master/introduction.html
  https://sumatra.readthedocs.io/en/latest/reference/records.html
- ReproZip:
  https://docs.reprozip.org/en/0.6.x/packing.html
- Nextflow and Snakemake:
  https://www.nextflow.io/docs/latest/reports.html
  https://nextflow.io/docs/latest/tutorials/data-lineage.html
  https://snakemake.readthedocs.io/en/v9.17.0/snakefiles/reporting.html
  https://snakemake.readthedocs.io/en/latest/executing/provenance.html

Lab state, audit, and calibration references:

- EPICS Archiver Appliance:
  https://epicsarchiver.readthedocs.io/en/latest/developer/details.html
- Phoebus Olog:
  https://control-system-studio.readthedocs.io/en/latest/app/logbook/olog/ui/doc/index.html
- eLabFTW:
  https://www.elabftw.net/
  https://doc.elabftw.net/
- LabKey LIMS:
  https://www.labkey.com/products-services/lims-software/
- openBIS data model:
  https://openbis.readthedocs.io/en/20.10.12-plus/user-documentation/advance-features/openbis-data-modelling.html

Modern desired-state and reviewable-apply references:

- React render/state model:
  https://react.dev/learn/state-as-a-snapshot
  https://react.dev/learn/render-and-commit
  https://react.dev/learn/managing-state
- Terraform plan/apply and dependency graph:
  https://developer.hashicorp.com/terraform/cli/commands/plan
  https://developer.hashicorp.com/terraform/cli/commands/apply
  https://developer.hashicorp.com/terraform/internals/graph
- Kubernetes desired/current state and controller pattern:
  https://kubernetes.io/docs/concepts/overview/working-with-objects/
  https://kubernetes.io/docs/concepts/architecture/controller/
  https://kubernetes.io/docs/tasks/manage-kubernetes-objects/declarative-config

## Product Lessons

- Parameter drift is a standalone product problem. Strategic follow-on
  parameter work should treat named parameter profiles, immutable snapshots,
  proposals, overrides, and diffs as first-class concepts.
- Code and parameter drift are coupled in practice. Calibration cannot be
  reliably automated while fitted values depend on copied measurement folders,
  mutable config files, notebook-local analysis, or generated sidecars that are
  not visible in the experiment record.
- Sample visualization can stay convention-friendly: parameter table row keys
  and user-authored 2D map configs can provide useful target binding without
  forcing a full sample-component ontology early.
- Useful run history should be generated from captured facts. Users will not
  reliably hand-enter code, parameter, setup, calibration, and environment
  context after every run.
- Managed code snapshot and runner capture matter early because they create
  inspectable history for measurements, analysis, and calibration. They should
  be opt-in at first, not a forced replacement for Python scripts.
- Provenance level must be honest: unmanaged, observed, and managed snapshots
  are different product states.
- Experiment-parameter calibration should first produce evidence, fitted
  values, chain-scoped working-ref updates, health decisions, retries, and
  pause/review reasons. Durable named profile promotion can then carry source
  measurements, analysis attempts, code context, affected parameter paths,
  diffs, review outcome, and rollback target where practical.
- Keep Calibration as the field-standard product term, but qualify other
  meanings. Instrument calibration and setup/device reconciliation have
  different safety, readback, and audit semantics from qubit/gate/readout
  parameter calibration.
- Setup/device/calibration state should start with store, bind, search, and
  diff. Applying settings to devices comes later and needs safety ADRs.
- Broad confidence-label schemes are secondary. Calibration task
  health/confidence gates are different: they are a concrete automation
  requirement for deciding whether a bootstrap calibration should continue,
  retry, pause, or ask for review.
- Local-first experiment history aligns better with Fricon than cloud-first ML
  tracking, model registries, or hyperparameter leaderboards.
- Borrow lifecycle, event, and schema rigor from Bluesky, but avoid exposing
  facility-scale document-stream complexity as the normal Fricon API.
- Borrow QCoDeS station snapshot ideas, but avoid opaque giant snapshot dumps
  as the main user experience. Diffs and named groups matter.
- Borrow ELN/logbook links and audit primitives, but avoid becoming a full
  ELN, LIMS, or compliance platform.
- Borrow declarative desired-state thinking from modern UI and infrastructure
  systems, but translate it carefully for hardware. Instead of writing every
  device in every nested scan-loop body, a managed routine could compute the
  expected setup/device state from parameters, diff it against observed state,
  preview an apply plan, skip no-op writes, and group safe independent writes.
- Desired-state reconciliation should be a reviewable plan/apply workflow, not
  hidden magic. A saved plan should record the observed state it was based on,
  dependencies, expected mutations, readback checks, and failure handling
  before hardware or durable named parameter refs are changed.
- Separate spec/status-style facts. Desired state is intent; observed device
  state, readbacks, and apply execution status are evidence. Mixing those
  concepts recreates the old problem where generated config and mutable state
  become hard to trust.
- Prefer small specialized controllers or reconcilers over one monolithic
  workflow engine. Calibration, parameter proposals, setup state, device apply,
  and routine replay have different safety and audit semantics.
- Treat long-running or replayable automation like a state machine with durable
  events. Non-deterministic side effects such as device I/O, time-sensitive
  readings, random choices, and external services should happen at explicit
  activity boundaries with recorded inputs and outputs.
- Use Git-like review habits for lab state: propose, diff, review, apply, and
  rollback. This maps naturally to parameter proposals, calibration proposals,
  and setup/device apply plans without requiring a heavyweight permissions
  system.
- Use dataflow-style invalidation for derived facts. If a parameter snapshot,
  code snapshot, calibration record, or generated sidecar changes, derived
  analysis results and routine previews should be able to show what they depend
  on and whether they need to be recomputed or reviewed.
