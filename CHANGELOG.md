# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.2](https://github.com/kahojyun/fricon/compare/v0.1.0-alpha.1...v0.1.0-alpha.2) - 2026-02-20

### Added

- dataset detail page ([#158](https://github.com/kahojyun/fricon/pull/158))
- dataset paging limit ([#155](https://github.com/kahojyun/fricon/pull/155))
- dataset write status display ([#153](https://github.com/kahojyun/fricon/pull/153))
- filter by tags ([#138](https://github.com/kahojyun/fricon/pull/138))
- search by name ([#137](https://github.com/kahojyun/fricon/pull/137))
- edit and filter favorite dataset ([#136](https://github.com/kahojyun/fricon/pull/136))

### Fixed

- *(fricon-ui)* stabilize trace chart rendering and semantics ([#169](https://github.com/kahojyun/fricon/pull/169))
- update stale dataset status on startup ([#154](https://github.com/kahojyun/fricon/pull/154))

### Other

- cleanup dataset query and table data flow ([#162](https://github.com/kahojyun/fricon/pull/162))
- *(dataset)* clarify create flow and improve write-session safety ([#156](https://github.com/kahojyun/fricon/pull/156))
- remove writer status watcher channel ([#152](https://github.com/kahojyun/fricon/pull/152))
- poll dataset writing progress from frontend ([#149](https://github.com/kahojyun/fricon/pull/149))
- command.rs ([#146](https://github.com/kahojyun/fricon/pull/146))
- remove `downcast_array` util ([#141](https://github.com/kahojyun/fricon/pull/141))
- process chart data in the backend ([#139](https://github.com/kahojyun/fricon/pull/139))

## [0.1.0-alpha.1](https://github.com/kahojyun/fricon/compare/v0.1.0-alpha...v0.1.0-alpha.1) - 2025-12-16

### Added

- dataset viewer ([#108](https://github.com/kahojyun/fricon/pull/108))
- implement async chunked writes and zero-copy dataset reads ([#80](https://github.com/kahojyun/fricon/pull/80))
- add a draft GUI viewer ([#66](https://github.com/kahojyun/fricon/pull/66))

### Fixed

- rename uuid field to uid ([#99](https://github.com/kahojyun/fricon/pull/99))
- dev env setup on windows ([#93](https://github.com/kahojyun/fricon/pull/93))
- focused dataset MVP API ([#86](https://github.com/kahojyun/fricon/pull/86))
- Windows named pipe and application lifecycle issues ([#83](https://github.com/kahojyun/fricon/pull/83))

### Other

- bump dependencies ([#98](https://github.com/kahojyun/fricon/pull/98))
- dataset array management ([#97](https://github.com/kahojyun/fricon/pull/97))
- setup cargo with PYO3_PYTHON ([#94](https://github.com/kahojyun/fricon/pull/94))
- use arrow sub-crates ([#91](https://github.com/kahojyun/fricon/pull/91))
- more clippy restriction lint ([#85](https://github.com/kahojyun/fricon/pull/85))
- bump Rust toolchain and clean up unused dependencies ([#84](https://github.com/kahojyun/fricon/pull/84))
- Workspace initialization and integration tests ([#81](https://github.com/kahojyun/fricon/pull/81))
- switch to nightly rustfmt ([#78](https://github.com/kahojyun/fricon/pull/78))
- handle dataset creation events and improve dataset management ([#71](https://github.com/kahojyun/fricon/pull/71))

## [0.1.0-alpha](https://github.com/kahojyun/fricon/releases/tag/fricon-v0.1.0-alpha) - 2025-01-15

### Added

- basic dataset api (#3)

### Other

- prepare for first release (#8)
- bundle licenses before python packaging (#7)
- setup release-plz (#5)
- *(deps)* bump uuid from 1.11.1 to 1.12.0 (#4)
- remove license appendix
- init commit
