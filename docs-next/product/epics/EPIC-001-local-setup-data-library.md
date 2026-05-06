# EPIC-001: Local Setup And Data Library

## Status

Accepted.

## Product Goal

An experimentalist can install Fricon, create or open one local data library,
and run Python scripts against it without assembling mismatched Desktop, CLI,
service, and SDK pieces.

## v0.2 Scope

- Coherent local Fricon release for Desktop, bundled service, bundled CLI, and
  Python SDK compatibility.
- First-run data-library location choice and durable library identity.
- Guided diagnostics for service, library, and version mismatch problems.
- Basic backup/restore and fail-before-write compatibility checks.

## Not v0.2

- Hosted service, multi-user administration, or direct shared-folder database
  access.
- Full auto-update polish or third-party protocol stability.

## Key Stories

- US-001: Install and launch Fricon on a lab computer.
- US-002: Create or open one local data library.
- US-010: Update without corrupting measurement work.
