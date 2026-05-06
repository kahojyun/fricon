# Personas

## Status

Accepted.

## P-001 Experimentalist

Researchers with entry-level Python ability who run measurement scripts or
notebooks on lab computers.

Needs:

- low-boilerplate Python recording
- live feedback while a measurement is running
- reliable recovery from interrupted runs
- easy reopening and export for later analysis
- no requirement to understand Rust, IPC, storage layout, or database internals

Constraints:

- may use Windows lab computers
- may work on locked-down or offline machines
- may pin Python environments with `uv.lock`, virtual environments, or lab
  setup scripts
- may not update Desktop, service, CLI, and Python SDK together

## P-002 Lab Maintainer

The person who helps keep lab measurement code, setup profiles, and computers
usable.

Needs:

- clear installation and diagnostics
- explicit split between Fricon install, data-library location, measurement
  code source, and Python environment
- support for sharing approved measurement code without copying random folders
- exportable support bundles that are local and redacted by default

## P-003 Analyst

The person who reopens completed measurements for notebooks, reports, or HPC
analysis.

Needs:

- stable IDs and Python reopen snippets
- direct read access to export bundles
- metadata and units preserved well enough to understand the data away from the
  acquisition computer

## P-004 Future Automation Author

A later user or maintainer who builds calibration, managed execution, or
AI-assisted workflows.

Needs:

- explicit measurement, artifact, parameter, code, and event boundaries
- audit records for accepted/rejected mutating actions
- no hidden mutation of completed dataset facts
