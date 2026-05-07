# Deferred Module Boundary Questions

## Status

Deferred. Domain context boundaries are still upstream of concrete module
architecture.

## Upstream Domain Inputs

`domain/context-map.md` owns bounded-context routing and anti-corruption rules.
Concrete crate, module, package, and frontend feature boundaries should be
derived later from accepted product/domain inputs and ADRs.

## Deferred Boundary Questions

- Which Rust modules should own data-library, measurement, dataset artifact,
  sample context, provenance, export, and compatibility behavior?
- Which boundaries are real domain boundaries versus implementation folders?
- Which ports/adapters are needed for storage, transport, filesystem, runtime,
  and Desktop shell integration?
- Which frontend code must remain browser-capable, and which code is specific
  to Tauri shell behavior?
- Which current v0.1 modules can be adapted without preserving old public
  semantics?
- Which boundaries deserve traits because there are multiple plausible
  implementations?

## Guardrails To Revalidate

These were useful v0.1 implementation guidelines and remain likely defaults,
but the concrete v0.2 module architecture should revalidate them:

- Keep storage schema details behind storage adapters.
- Keep transport DTOs from becoming hidden owners of domain decisions.
- Keep feature events separate from UI shell commands and transport event DTOs.
- Keep frontend feature code browser-capable where practical.
- Keep Tauri-specific file dialogs, launch behavior, updater, diagnostics, and
  shell integration behind shell adapters.
- Add traits only for real boundaries or multiple plausible implementations.

## Candidate Inputs For Later Architecture Work

- `crates/fricon/src/workspace.rs`: current workspace implementation, not a
  v0.2 product concept.
- `crates/fricon/src/dataset/**`: possible reusable dataset payload and
  semantic-validation ideas, with ownership redesign pressure.
- `crates/fricon/src/transport/**`: current transport background, not an
  accepted v0.2 service contract.
- `crates/fricon-py/**`: current Python binding background; exact v0.2 Python
  API syntax is deferred.
- `crates/fricon-ui/frontend/src/features/datasets/**`: possible reusable chart
  and table code after model changes.
- `crates/fricon-ui/src/features/**`: current shell integration background.
