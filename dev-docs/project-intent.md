# Project Intent

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
- parameter management
- device management
- local experiment workspace management
- a desktop UI for browsing, managing, and inspecting collected data
- Python APIs for scripting and automation

The product should make common scientific measurement workflows easier while
remaining scriptable for users who already use Python in their research.

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

### Make Advanced Workflows Explicit

Experiment execution, parameter management, and device management should become
explicit product concepts as they mature. Avoid hiding those semantics inside
dataset naming conventions or incidental metadata.

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
- Prefer feature-local changes that preserve clear ownership.
- Treat Python API and desktop UI behavior as user-facing contracts.
- Treat internal Rust module boundaries as changeable when doing so improves
  the architecture.

## Open Product Questions

These questions are intentionally unresolved:

- What experiment execution model should Fricon support first: script-driven,
  UI-driven, or a hybrid?
- How should parameters be represented: flat key-value sets, typed schemas,
  sweep definitions, or versioned experiment configurations?
- What level of device abstraction is useful without overbuilding a hardware
  framework?
- Which data formats and array shapes should be first-class beyond the current
  dataset model?
- What remote-client use cases are worth supporting without introducing
  multi-user product complexity?
