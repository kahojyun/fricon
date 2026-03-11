## Crate-Specific Constraints

- Any blocking or potentially long-running Rust operation must release the Python GIL.
- Wrap `get_runtime().block_on(...)` and equivalent blocking waits with `py.detach(...)`.
- If a function needs to block and has no `Python<'_>` argument, add `py: Python<'_>` and use `py.detach(...)`.
- In `Drop` implementations, use `Python::try_attach(...)` and call `py.detach(...)` inside it, with a safe fallback when attach is unavailable.
