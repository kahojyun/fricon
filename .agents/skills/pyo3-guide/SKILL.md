---
name: pyo3-guide
description: Use the latest PyO3 tutorial and migration guidance to write or refactor Rust-Python bindings with current APIs. Trigger when adding or updating `#[pyclass]`, `#[pymethods]`, `#[pyfunction]`, `#[pymodule]`, conversion traits, GIL attach/detach code, or when modernizing pre-0.27/0.28 PyO3 patterns.
---

# PyO3 Guide

## Overview

Follow current PyO3 idioms first, then patch old code only where needed. Keep edits minimal but prefer deprecation-free APIs from the latest official docs.

## Source Of Truth

- Read `references/official-sources.md` first.
- Treat `https://pyo3.rs/latest/` as the canonical guide entrypoint.
- Re-check migration and changelog when touching compatibility-sensitive code.

## Workflow

1. Identify current PyO3 usage in the target diff (`#[pymodule] mod`, `#[pymodule_export]`, `Python::with_gil`, conversion traits, `Bound` usage).
2. Map old patterns to current APIs using the migration checklist below.
3. Apply the smallest safe rewrite that removes deprecated patterns.
4. Run repo-appropriate checks (at least `cargo check`; add Python tests if bindings behavior changed).

## High-Impact Migrations

1. Prefer GIL APIs `Python::attach` / `Python::detach`.
Replace old `Python::with_gil` / `allow_threads` / nested attach-detach combinations.

2. Prefer `Bound<'py, T>` and `Borrowed<'a, 'py, T>` in function boundaries.
Avoid re-introducing legacy GIL reference patterns (`&PyAny`, `PyTryFrom` style conversions).

3. Prefer declarative module style for this repository.
Use `#[pymodule] mod _core { ... }` with `#[pymodule_export]` (as in `crates/fricon-py/src/lib.rs`) instead of rewriting into function-style module initialization.

4. Use `#[pymodule_init]` only for custom module init code.
Keep regular symbol export declarative via `#[pymodule_export]`.

5. Prefer `IntoPyObject` / `IntoPyObjectExt`.
Replace old `ToPyObject` / `IntoPy` return-conversion idioms when touching conversion code.

6. Update extraction patterns for `FromPyObject`.
Use current trait signatures and derive attributes (`#[pyo3(item)]`, `#[pyo3(attribute)]`, optional `annotation` for clearer type hints).

7. Replace deprecated synchronization helpers.
Use `PyOnceLock` and `MutexExt` instead of deprecated `GILOnceCell` and `GILProtected`.

## Rewrite Pattern Snippets

```rust
// Old
Python::with_gil(|py| {
    let obj = value.to_object(py);
    // ...
});

// New
Python::attach(|py| {
    let obj = value.into_pyobject(py)?;
    // ...
    Ok::<_, PyErr>(())
})?;
```

```rust
#[pymodule]
mod _core {
    #[pymodule_export]
    use super::{MyClass, my_fn};
}
```

## Constraints

- Keep guidance concise and actionable.
- Do not invent APIs from memory; verify against official sources when unsure.
- Prefer forward-compatible rewrites over compatibility shims unless the user asks for legacy support.
