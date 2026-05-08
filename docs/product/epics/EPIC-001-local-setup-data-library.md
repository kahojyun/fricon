# EPIC-001: Local Setup And Data Library

## Status

Draft pending product-analysis revalidation.

## Product Goal

A Local Measurement System Maintainer can install Fricon, create or open one
local data library, and keep Python scripts runnable against it without
assembling mismatched Desktop, CLI, Python SDK, and local runtime pieces.

## MVP Scope

- Coherent local Fricon release for Desktop, CLI, Python SDK, and required
  local runtime components.
- First-run data-library location choice and durable library identity.
- Guided diagnostics for local runtime, library, and version mismatch problems.
- Basic backup/restore and fail-before-write compatibility checks.
- Product support for common lab-computer constraints, including Windows,
  offline or locked-down environments, pinned Python environments, and
  slow-to-update setups.

## Not MVP

- Hosted service, multi-user administration, or direct shared-folder database
  access.
- Full auto-update polish or third-party protocol stability.
- Full enterprise deployment management.

## Key Stories

- US-001: Install and launch Fricon on a lab computer.
- US-002: Create or open one local data library.
- US-010: Update without corrupting measurement work.
