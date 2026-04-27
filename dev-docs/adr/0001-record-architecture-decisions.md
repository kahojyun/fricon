# ADR 0001: Record Architecture Decisions

## Status

Accepted

## Context

Fricon has several design choices that affect Rust crates, Python bindings,
desktop UI behavior, workspace storage, and compatibility policy. Existing
developer docs already separate current implementation notes, proposals,
maintenance checklists, and AI-assisted architecture guidance.

As the project grows, durable decisions need a stable home so future human and
AI contributors do not have to infer settled direction from old proposals,
discussion notes, or implementation details.

## Decision

The project will keep architecture decision records under `dev-docs/adr/`.

ADRs should be used selectively for durable, cross-cutting decisions. They
should not replace implementation plans, current implementation notes, or
release guidance.

## Consequences

Future durable decisions can be linked from roadmap, architecture, maintenance,
and implementation documents without duplicating full rationale.

Contributors should avoid treating every local code choice as an ADR-worthy
decision. The ADR directory should stay small enough to remain useful as an
orientation tool.

When an ADR becomes outdated, add a new ADR that supersedes it instead of
silently rewriting history.

## Alternatives Considered

Keeping all rationale in topic-specific notes would avoid another directory,
but it would make durable decisions harder to discover once proposals and
implementation plans accumulate.

Using issues or pull requests as the only decision log would preserve history,
but it would be too scattered for low-context contributors and AI agents.
