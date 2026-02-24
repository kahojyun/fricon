# fricon-py Agent Rules

- In `crates/fricon-py`, any blocking or potentially long-running Rust operation must release the Python GIL.
- Wrap `get_runtime().block_on(...)` and similar blocking waits with `py.detach(...)`.
- If a function currently has no `Python<'_>` argument and needs to block, add `py: Python<'_>` and use `py.detach(...)`.
- For `Drop` implementations, use `Python::try_attach(...)` and call `py.detach(...)` inside it; provide a safe fallback when attach is unavailable.
