## Crate-Specific Constraints

- Follow the existing vertical slice boundaries. Prefer extending the owning domain/feature instead of introducing shared catch-all modules.
- Domain/feature modules own business models, errors, and repository/service traits.
- Adapter layers own transport and persistence details. Do not move business types into database or transport modules.
- Keep dependency flow one-way: application/composition -> feature/domain -> adapters/infrastructure.
- If an API depends on Tokio runtime context for spawning, listening, or reactor-backed I/O, make that requirement explicit in the signature rather than relying on ambient runtime state.
