# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
