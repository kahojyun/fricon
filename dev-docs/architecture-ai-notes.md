# Architecture Notes From AI/Repo Discussion

## Scope

This note summarizes the architectural conclusions from a review and comparison of `fricon` and `crates.io`, with a focus on long-term maintainability in an AI-heavy development workflow.

## High-Level Takeaways

- `fricon` is closer to a good fit for AI-assisted development than `crates.io` because it has stronger explicit architectural rules and clearer vertical slice ownership.
- `crates.io` is a pragmatic, DB-centric modular monolith. That makes local edits fast, but it relies more heavily on reviewer discipline to prevent long-term drift.
- `fricon` benefits from stronger boundaries because its core workflows span more than the database: metadata in SQLite, dataset files on disk, write sessions, UI/runtime integration, and multiple user-facing entry points.
- For AI-heavy development, written boundaries have unusually high value because AI is weak at preserving unwritten architectural conventions over long periods.

## Layering Guidance

- Prefer feature-local layered slices.
- A feature may contain:
    - core/domain modules
    - service/use-case orchestration
    - thin local adapters
- Do not require every feature to have the same amount of abstraction.
- Reduce abstraction thickness for simple CRUD-heavy features, but do not remove the persistence boundary.

Recommended dependency direction inside a feature:

```text
composition/app -> services + adapters
services -> core types + feature-defined ports
adapters -> core types + infrastructure
```

Rules of thumb:

- Core/service code should not directly depend on Diesel, database schema modules, or transport/runtime-specific types.
- Only adapter modules should directly touch persistence, transport, runtime, or filesystem details.
- Shared code should be extracted only after it is clearly stable across multiple features.

## Diesel and Repository Pattern

### `crates.io`

- `crates.io` uses Diesel more directly.
- Database row structs and domain-ish models are often the same type.
- Diesel queries appear both in database models and in controllers.
- This style is productive for a DB-centric backend, but it couples business flow tightly to persistence concerns.

### `fricon`

- `fricon` uses a more explicit repository/port style around the dataset domain.
- This is justified because the core business flow is not just database I/O.
- The database stores metadata, while the real dataset content and write lifecycle also involve filesystem and in-memory write-session concerns.
- The main benefit of the repository boundary is not “future database swapping”; it is keeping orchestration logic from collapsing into persistence code.

Conclusion:

- Repository abstractions are not always worth adding in ORM-based systems.
- In `fricon`, they are worthwhile for the core dataset domain because the workflow crosses multiple infrastructure boundaries.
- For simpler features, a thin local adapter is usually a better fit than a full repository trait.

## ORM Guidance

- ORMs such as EF Core or SQLAlchemy already provide strong persistence abstractions.
- That does not mean they automatically solve application architecture problems.
- Their abstractions are strong at database access, tracking, and unit-of-work concerns.
- They do not automatically solve boundary control, orchestration clarity, or cross-resource coordination.

Useful rule:

- If the complexity is mostly database CRUD, direct ORM usage is often enough.
- If the complexity is orchestration across database, files, sessions, runtime, or external adapters, a persistence port/repository boundary becomes more valuable.

Avoid:

- generic repository layers that just wrap ORM CRUD methods
- leaking ORM-specific query abstractions into higher layers

Prefer:

- feature-specific ports
- application services/use cases
- thin adapters at the edge

## Port, Adapter, and Service Terminology

- A **port** is a contract defined by the core/application side.
- An **adapter** is the implementation of that contract against infrastructure.
- A **service** orchestrates a use case by calling ports and coordinating multiple dependencies.

Useful interpretation:

- Repository traits in `fricon` are better thought of as persistence ports than as generic service objects.
- They are injectable dependencies, but semantically they represent infrastructure capabilities required by the feature.

## Tokio and Runtime Dependencies

- Tokio can be treated as part of the application platform in practice.
- Using Tokio directly in app/composition, transport, and runtime glue is generally fine.
- The real issue is not whether Tokio is used, but whether Tokio types become part of the core business language.

Guidance:

- It is acceptable for app-level orchestration, gRPC glue, background tasks, and runtime coordination to use Tokio types directly.
- It is better to avoid exposing `tokio::sync` types as the primary API of feature core modules unless stream/concurrency semantics are truly part of the business contract.

## Event Design

Current issue identified:

- `AppEvent` currently mixes feature-level notifications and app/UI-shell requests.

Recommended distinction:

- Feature events:
    - example: dataset created/updated
- App/shell commands:
    - example: show main window
- Transport/UI events:
    - adapter-specific event shapes for Tauri/frontend

Suggested structure:

- feature-local event type for each feature
- optional small publisher trait for service/core boundaries
- app-level bus implementation that aggregates and forwards events

Publisher trait guidance:

- Use a small publisher trait when feature/application code needs to publish notifications.
- Feature code can usually treat such notifications as fire-and-forget.
- App/adapter code should own logging, metrics, delivery failures, and transport-specific concerns.
- Only promote publication failure back into business logic if delivery is itself part of the business contract.

## Guidance for `app` and gRPC Layers

- `app` should act as the composition root.
- `app` should create infrastructure objects, instantiate adapters, inject them into services, and expose stable handles.
- gRPC services should be inbound adapters only.
- gRPC handlers should parse requests, validate transport-level input, call services, and map errors to transport status codes.
- gRPC handlers should not directly query Diesel or own business orchestration.

## Practical Sweet Spot

Current recommendation for `fricon`:

- Keep the current feature-local layered slice direction.
- Keep strong edge boundaries around persistence and transport.
- Avoid copying full repository patterns into every simple feature.
- Allow thin local adapters for CRUD-heavy features.
- Do not allow service/business code to talk to Diesel directly.

This is the current sweet spot:

- explicit enough for AI to follow
- not so heavy that every feature turns into ceremony

## Anti-Decay Heuristics

Watch for these signs of healthy structure:

- Most changes stay within one feature slice.
- Services read like business steps instead of persistence scripts.
- Diesel-specific code stays concentrated in adapter/persistence modules.
- Feature APIs are defined in domain/application terms instead of infrastructure terms.

Watch for these signs of drift:

- services start importing Diesel or schema modules directly
- feature boundaries become inconsistent across different modules
- “shared” helpers appear too early and start mixing multiple features
- repository traits become generic wrappers with no domain meaning
- event types start mixing feature semantics and UI shell control concerns

## Summary

The main architectural lesson is not “use more layers”. It is:

- keep boundaries explicit
- keep abstractions proportional to real workflow complexity
- keep infrastructure at the edge
- optimize for local reasoning, especially for AI contributors

For `fricon`, that means preserving feature-local layered slices with thin adapters and one-way dependency flow, while resisting both uncontrolled direct Diesel usage and unnecessary abstraction growth.
