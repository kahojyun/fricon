## Crate-Specific Constraints

- This crate is mid-refactor toward runtime-agnostic feature services. Existing uses of Tokio scheduling, Tokio channels, or broadcast senders inside feature services/workflows are temporary compatibility debt. Do not copy those patterns into new code, and remove them when touching the affected flow if practical.
- Follow the existing vertical slice boundaries. Prefer extending the owning domain/feature and keeping feature-local layers over introducing shared catch-all modules.
- Domain/feature modules own business models, errors, and service/repository ports.
- Adapter layers own transport and persistence details. Do not move business types into database or transport modules.
- Keep dependency flow one-way: composition/app -> services/use cases -> feature core plus feature-defined ports -> adapters/infrastructure.
- Core/service code must stay runtime- and transport-agnostic: no Diesel/schema modules, transport types, or Tokio scheduling/channel primitives. Model those needs as feature ports, implement them in application/adapter code, and keep `spawn_blocking`, task tracking, and channel lifecycle outside feature service code.
- Use publisher/input ports for workflow notifications or streamed inputs when they are part of the use case. Return values carry the primary domain result; runtime-specific stream/channel types stop at the application/transport boundary.
- Feature/service errors describe business failures and feature-port failures only. Map Tokio, transport, framework, and adapter-specific failures at application/adapter boundaries.
- For simple CRUD-heavy features, prefer a thin local adapter over a shared generic repository abstraction. Simplify adapter thickness, not feature boundaries; service/business code must remain unaware of Diesel models and schema modules.
