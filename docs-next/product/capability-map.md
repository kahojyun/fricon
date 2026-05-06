# Capability Map

## Status

Draft.

## ID Rules

Capability IDs are stable. Do not reuse or renumber them.

## Capabilities

| ID | Capability | v0.2 Target | Notes |
| --- | --- | --- | --- |
| CAP-001 | Local data library | In scope | One primary local data library per normal lab computer. |
| CAP-002 | Install and launch local Fricon | In scope | Desktop bundles compatible local service and CLI. |
| CAP-003 | Python measurement recording | In scope | Explicit but short measurement API. |
| CAP-004 | Optional sample/session context | In scope | Visible, correctable, not required for quick runs. |
| CAP-005 | Dataset artifact recording | In scope | Table-shaped facts, append while active, immutable after finish. |
| CAP-006 | Dataset scan semantics | In scope | Explicit roles and axes for plotted data. |
| CAP-007 | Live inspection | In scope | Noncritical table, line/scatter, heatmap, trace views. |
| CAP-008 | Lifecycle and partial recovery | In scope | Active, finished, interrupted, failed, aborted, trashed, recovered. |
| CAP-009 | Notes, markers, favorites, tags | In scope | Measurement-first annotation. |
| CAP-010 | Python reopen | In scope | Stable IDs and public SDK snippets. |
| CAP-011 | Measurement-centered export | In scope | Read-only bundles for offline analysis. |
| CAP-012 | Backup, restore, migration checkpoints | In scope | User-visible safety paths. |
| CAP-013 | Compatibility diagnostics | In scope | Client/service/library checks before writes. |
| CAP-014 | Code provenance summary | In scope | Honest unmanaged/user-supplied/future managed levels. |
| CAP-015 | Flexible parameter snapshot | In scope | Minimal snapshot without registry/profile UI. |
| CAP-016 | Light attachments | In scope | Small files, images, logs attached to measurements. |
| CAP-017 | Operator profile and audit actor | In scope | Lightweight local actor labels, not accounts. |
| CAP-018 | Read-only remote monitoring | Later | Candidate v0.3. |
| CAP-019 | Rich sample maps and saved views | Later | Candidate v0.3. |
| CAP-020 | Measurement-code source setup | Later | Configure/update lab code sources without becoming a Git host. |
| CAP-021 | Parameter profiles and proposals | Later | Candidate v0.4 automation foundation. |
| CAP-022 | Analysis and calibration records | Later | Consume artifacts, produce derived artifacts or proposals. |
| CAP-023 | Managed code snapshots and execution | Later | Requires ADR before implementation. |
| CAP-024 | Device boundary and managed device communication | Later | Reserve boundary, do not build broad framework in v0.2. |
| CAP-025 | AI-assisted reviewed automation | Later | Read/suggest first; mutating actions require review and audit. |
| CAP-026 | Dataset artifact discovery and direct open | In scope | Datasets are first-class searchable/openable artifacts even in a measurement-first UI. |
| CAP-027 | Passive setup and environment summary | In scope | Optional setup/device/environment facts without device control or reproducibility overclaiming. |
| CAP-028 | Scan schema authoring helpers | In scope | Short helpers for common scan/trace shapes plus raw schema for advanced cases. |
| CAP-029 | Passive procedure summary | In scope | Optional unmanaged script, external runner, or declared-plan summary without managed execution. |
