# Post-MVP Future Systems Research

## Status

Draft research synthesis.

## Review Date

2026-05-06.

## Purpose

Capture background lessons for post-MVP parameter systems, managed code
snapshots, runner capture, setup state, calibration history, and generated run
history.

This research informs `product/future-stories-and-requirements.md`; it does
not change accepted MVP scope.

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

## Product Lessons

- Parameter drift is a standalone product problem. Post-MVP parameter work
  should treat named parameter profiles, immutable snapshots, proposals,
  overrides, and diffs as first-class concepts.
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
- Calibration should first produce evidence and reviewed parameter/setup
  proposals: source measurements, analysis attempts, code context, fitted
  values, affected parameter paths, diffs, review outcome, and rollback target
  where practical.
- Setup/device/calibration state should start with store, bind, search, and
  diff. Applying settings to devices comes later and needs safety ADRs.
- Detailed confidence-label schemes are secondary. They should not block the
  core code/parameter/calibration evidence model, and they can be added later
  where a concrete workflow needs them.
- Local-first experiment history aligns better with Fricon than cloud-first ML
  tracking, model registries, or hyperparameter leaderboards.
- Borrow lifecycle, event, and schema rigor from Bluesky, but avoid exposing
  facility-scale document-stream complexity as the normal Fricon API.
- Borrow QCoDeS station snapshot ideas, but avoid opaque giant snapshot dumps
  as the main user experience. Diffs and named groups matter.
- Borrow ELN/logbook links and audit primitives, but avoid becoming a full
  ELN, LIMS, or compliance platform.
