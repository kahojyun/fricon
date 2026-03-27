# Changelog

All notable changes to this project will be documented in this file.

The project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.1 (2026-03-27)

### Fixes

- make uv sdist builds self-contained (#299)

#### Targeted dataset event system for UI cache invalidation

Replace the single `Updated` backend event with fine-grained dataset lifecycle events
(Created, StatusChanged, MetadataUpdated, TagsChanged, Trashed, Restored, Deleted,
Imported, GlobalTagsChanged). The frontend now performs targeted React Query cache
invalidation per-event instead of blanket refetches, and chart data refreshes
correctly during active write sessions.

## 0.1.0 (2026-03-26)

### Features

- basic dataset api (#3)
- add a draft GUI viewer (#66)
- implement async chunked writes and zero-copy dataset reads (#80)
- dataset viewer (#108)
- filter table with individual column (#113)
- heatmap (#133)
- scatter plot (#135)
- edit and filter favorite dataset (#136)
- search by name (#137)
- filter by tags (#138)
- dataset write status display (#153)
- dataset paging limit (#155)
- dataset detail page (#158)
- migrate frontend to React and refresh tooling/docs (#159)
- integrate tauri-specta bindings and typed events (#164)
- add dataset test case generators and realtime scan case (#168)
- prefer trailing index columns for chart defaults (#171)
- improve workspace launch flow for cli and dialog modes (#173)
- add structured logging with tracing instrumentation (#180)
- refactor dataset creation streaming with robust termination handling (#189)
- improve dataset table column visibility UX (#201)
- align UI with shadcn patterns (#210)
- add multi-select delete workflow (#227)
- add keyboard navigation for selectable tables (#230)
- add dataset tag management (#231)
- bring up running GUI for opened workspace (#236)
- implement dataset trashing and restoration functionality (#266)
- add tombstone graveyard deletion (#268)
- add dataset portability import and export flows (#271)
- add dataset import and export flows (#272)
- improve dataset import archive version handling (#288)
- enhance workspace migration error handling and validation logic (#289)

### Fixes

- Windows named pipe and application lifecycle issues (#83)
- focused dataset MVP API (#86)
- dev env setup on windows (#93)
- cargo config.toml generation on windows (#96)
- rename uuid field to uid (#99)
- add license-files field to pyproject.toml (#112)
- minor improvement of chart viewer UI (#121)
- correct gitignore (#122)
- page overflow behavior (#126)
- keep filter selection after toggling split mode (#130)
- meta-key occupied when focusing tables (#131)
- heatmap realtime update (#134)
- update stale dataset status on startup (#154)
- stabilize trace chart rendering and semantics (#169)
- release GIL when doing blocking rust operations (#178)
- improve logging registration (#179)
- store and use `tokio::runtime::Handle` in `DatasetWriter` drop (#192)
- clean up dataset detail and viewer flows (#213)
- enhance ChartViewer with loading and tombstone states (#275)

## 0.1.0-alpha.1 (2025-12-16)

### Features

- dataset viewer ([#108](https://github.com/kahojyun/fricon/pull/108))
- implement async chunked writes and zero-copy dataset reads ([#80](https://github.com/kahojyun/fricon/pull/80))
- add a draft GUI viewer ([#66](https://github.com/kahojyun/fricon/pull/66))

### Fixes

- rename uuid field to uid ([#99](https://github.com/kahojyun/fricon/pull/99))
- dev env setup on windows ([#93](https://github.com/kahojyun/fricon/pull/93))
- focused dataset MVP API ([#86](https://github.com/kahojyun/fricon/pull/86))
- Windows named pipe and application lifecycle issues ([#83](https://github.com/kahojyun/fricon/pull/83))

### Notes

- bump dependencies ([#98](https://github.com/kahojyun/fricon/pull/98))
- dataset array management ([#97](https://github.com/kahojyun/fricon/pull/97))
- setup cargo with PYO3_PYTHON ([#94](https://github.com/kahojyun/fricon/pull/94))
- use arrow sub-crates ([#91](https://github.com/kahojyun/fricon/pull/91))
- more clippy restriction lint ([#85](https://github.com/kahojyun/fricon/pull/85))
- bump Rust toolchain and clean up unused dependencies ([#84](https://github.com/kahojyun/fricon/pull/84))
- Workspace initialization and integration tests ([#81](https://github.com/kahojyun/fricon/pull/81))
- switch to nightly rustfmt ([#78](https://github.com/kahojyun/fricon/pull/78))
- handle dataset creation events and improve dataset management ([#71](https://github.com/kahojyun/fricon/pull/71))

## 0.1.0-alpha (2025-01-15)

### Features

- basic dataset api (#3)

### Notes

- prepare for first release (#8)
- bundle licenses before python packaging (#7)
- setup release-plz (#5)
- *(deps)* bump uuid from 1.11.1 to 1.12.0 (#4)
- remove license appendix
- init commit
