# PyO3 Official Sources

Last checked: 2026-02-24 (US local date).
Version snapshot: guide pages show v0.28.2; migration sections currently cover through the 0.28 line.

## Core Docs

- Guide home: https://pyo3.rs/latest/
- Module guide (v0.28.2): https://pyo3.rs/v0.28.2/module.html
- Getting started tutorial: https://pyo3.rs/latest/getting-started
- Migration guide: https://pyo3.rs/latest/migration.html
- Changelog: https://pyo3.rs/latest/changelog.html

## API References

- `#[pymodule]` attribute docs: https://docs.rs/pyo3/latest/pyo3/attr.pymodule.html
- `Python` attach/detach APIs: https://docs.rs/pyo3/latest/pyo3/marker/struct.Python.html

## Notes For This Skill

- Prefer the `latest` docs above for active guidance.
- For module style, follow the v0.28.2 module page and prefer declarative `#[pymodule] mod` + `#[pymodule_export]`.
- Re-check changelog and migration sections before large refactors.
