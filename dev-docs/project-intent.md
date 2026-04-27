# Project Intent

## Status

Canonical project direction and AI-assisted development guardrails.

## Purpose

Fricon aims to become an easy-to-use scientific experiment measurement
management system.

The project should help researchers record, organize, inspect, and eventually
execute scientific measurement workflows on their own computers without requiring
them to operate a multi-user service or a complex lab information system.

## Target Users

The primary target users are researchers who have entry-level Python data
analysis ability.

Assumptions:

- They can install Python packages and run Python scripts or notebooks.
- They are comfortable with basic tabular or array-like data concepts.
- They may not be software engineers.
- They should not need to understand Fricon's internal Rust, IPC, database, or
  file-layout implementation details.

## Long-Term Product Direction

Fricon is expected to provide:

- data recording
- experiment execution
- parameter management, including future history and versioning workflows
- device management
- local experiment workspace management
- a desktop UI for browsing, managing, and inspecting collected data
- Python APIs for scripting and automation
- reproducibility support for experiment code, environments, parameters, and
  generated datasets

The product should make common scientific measurement workflows easier while
remaining scriptable for users who already use Python in their research.

## Current Product Route

The current route is dataset-first, Python-led, and local-first.

Fricon should first become reliable for recording, organizing, and inspecting
scientific measurement datasets. Dataset semantics should be explicit enough
that later experiment, parameter, and device concepts do not have to be hidden
inside dataset names, incidental metadata, or chart heuristics.

Initial experiment support should lean on Python scripts as the execution
entry point. The desktop UI should browse, inspect, and eventually assist those
workflows, but should not become the primary experiment execution engine before
the Python-led model is clear.

Device management should remain a later foundation. Near-term design may keep
room for device identity and configuration, but should avoid building a broad
driver framework or hardware orchestration layer before real workflows require
one.

## Runtime Model

Fricon is local-first. The main supported runtime model is:

- data lives on the user's local computer
- the desktop UI and Python API operate against a local workspace
- the local server process coordinates workspace operations

A remote client may be considered in the future, but it should build on the
local-first model rather than forcing the project into a hosted service shape.

## Non-Goals

Fricon does not currently aim to support:

- multi-user collaboration
- server-hosted SaaS operation
- account management, teams, roles, or permissions
- centralized lab administration
- distributed database semantics
- web-first deployment as the primary experience

These may become integration concerns someday, but they should not drive the
core architecture now.

## Design Principles

### Keep The User Model Simple

Users should think in terms of workspaces, datasets, experiments, parameters,
and devices. Internal concepts such as SQLite tables, Arrow chunk files, IPC
protocol versions, and Rust module boundaries belong in developer notes, not in
public user documentation.

### Prefer Local Reliability Over Distributed Flexibility

Because the product is local-first and single-user, prioritize predictable local
state, understandable recovery paths, and clear workspace compatibility over
distributed coordination patterns.

### Keep Python Ergonomic

The Python API is a primary user surface. It should support straightforward
data collection scripts without forcing users to predefine every low-level
schema detail.

Desktop and documentation workflows should help users get back to Python code.
For example, dataset detail views may eventually provide Python read snippets
that reopen selected datasets through the public API without exposing internal
storage paths.

### Make Advanced Workflows Explicit

Experiment execution, parameter management, and device management should become
explicit product concepts as they mature. Avoid hiding those semantics inside
dataset naming conventions or incidental metadata.

The first explicit experiment model should be Python-led: user scripts perform
measurement work while Fricon records datasets, run metadata, and parameters.
UI-led execution can be introduced later if the Python-led workflow proves too
limited.

Experiment reproducibility may eventually include automatic code history and
environment management. Git-backed code history, including a workspace-managed
bare repository, and environment tools such as `uv` or `pixi` are plausible
directions, but they should be designed as explicit product capabilities rather
than hidden side effects of dataset writes.

### Keep Architecture Proportional

Use explicit boundaries where workflows cross storage, runtime, IPC, device, or
UI concerns. Avoid general-purpose abstractions that only prepare for future
possibilities without simplifying current product work.

## AI-Assisted Development Guidance

This document is a constraint for AI-assisted changes:

- Do not expand the project toward multi-user SaaS unless explicitly requested.
- Do not expose internal storage or protocol details in `docs/`.
- Keep user-facing docs focused on workflows and stable product concepts.
- Put implementation details, architectural notes, and maintenance rules in
  `dev-docs/`.
- Preserve the dataset-first, Python-led product route unless a planning or ADR
  document explicitly changes it.
- Prefer feature-local changes that preserve clear ownership.
- Treat Python API and desktop UI behavior as user-facing contracts.
- Treat internal Rust module boundaries as changeable when doing so improves
  the architecture.

## Open Product Questions

These questions are intentionally unresolved:

- How should parameters be represented: flat key-value sets, typed schemas,
  sweep definitions, or versioned experiment configurations?
- How should parameter history and version comparison be represented in the
  Python API and desktop UI?
- How should experiment code history be captured without surprising users or
  turning Fricon into a general Git client?
- What level of automatic `uv` or `pixi` environment management is useful
  without making experiment setup opaque?
- What level of device abstraction is useful without overbuilding a hardware
  framework?
- Which data formats and array shapes should be first-class beyond the current
  dataset model?
- What remote-client use cases are worth supporting without introducing
  multi-user product complexity?
