# Release And Versioning

## Status

Canonical release and versioning policy.

## Purpose

This note is the canonical release and versioning policy for Fricon. Use it to
decide when a change needs release notes, how to describe the change, and which
versioning decisions must be made before merge.

For implementation-side coordination, use `dev-docs/maintenance-checklist.md`.

## Current Release Model

- Releases are managed by Knope.
- The repository has one unified project version.
- The canonical Git tag format is `v<version>`.
- Release preparation happens through the `release` branch and a prepared
  release PR.
- Merging the prepared release PR creates the GitHub release and tag.
- Publishing is currently PyPI-only.
- Versioned files are defined in `knope.toml`.

## Changeset Policy

Add a Knope changeset under `.changeset/` when a change is user-visible,
release-note-worthy, or intentionally changes versioned behavior.

Examples that usually need a changeset:

- new Python API capability
- changed Python API behavior
- new CLI or desktop UI capability
- changed dataset or workspace behavior visible to users
- user-visible bug fix
- compatibility change that users may need to know about

Examples that usually do not need a changeset:

- internal Rust-only refactor with no user-visible behavior change
- test-only change
- developer-doc-only maintenance note
- CI or tooling cleanup with no release impact
- changes to internal docs under `dev-docs/` that do not describe released
  behavior

When in doubt, prefer a concise patch changeset if a user could observe the
change after upgrading.

## Version Bump Guidance

Use normal semantic-versioning intent:

- `patch`: bug fixes, user-facing documentation corrections, small behavior
  fixes, and compatible polish
- `minor`: new user-facing Python, CLI, desktop UI, dataset, or workspace
  capability
- `major`: future stable-compatibility breaks once the project has stable
  compatibility promises

Knope handles `0.x` version behavior. Choose the release intent; do not
manually edit versioned files for normal changes.

## Changeset Format

AI agents should write changeset files directly rather than using the
interactive `knope document-change` CLI. Human contributors may use
`knope document-change` or write the file manually.

Use this template:

```md
---
default: patch
---

# Short user-facing title

Describe the user-visible change in release-note language.
```

Rules:

- Use `major`, `minor`, or `patch` instead of `patch` when the intended bump is
  not patch.
- Prefer user-facing language over implementation details.
- Do not place templates, README files, or helper Markdown files inside
  `.changeset/`; Knope treats Markdown files there as real changesets.
- `.changeset/.gitkeep` is acceptable.

## Compatibility Decisions

Some implementation changes require a version or compatibility decision even if
they do not directly edit release files:

- Workspace format changes require a `WORKSPACE_VERSION` decision.
- Rust IPC/gRPC contract changes require an `IPC_PROTOCOL_VERSION` decision.
- Python API behavior changes require updated stubs/tests and usually a
  changeset.
- Desktop UI or CLI behavior changes usually require user-facing release notes.

Use `dev-docs/maintenance-checklist.md` for the full coordinated checklist for
these changes.

## Release Workflow Summary

- The release preparation workflow is started manually from `main` and refreshes
  the prepared release PR from the `release` branch when Knope finds pending
  release notes.
- The release preparation workflow depends on the prepared release commit
  subject `chore: prepare release <version>`.
- Merging the prepared release PR creates the GitHub release and tag.
- Publishing remains PyPI-only unless the release workflow is changed.
