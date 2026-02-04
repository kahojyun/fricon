# Frontend Vue -> React Migration Plan

This document maps the current Vue implementation (`crates/fricon-ui/frontend-vue`) to the React rewrite (`crates/fricon-ui/frontend`) and records the agreed UI component strategy: **Shadcn-first**.

## Scope Overview

- **Target UI**: React 19 + TanStack Router + TanStack Query + Shadcn components
- **Source UI**: Vue 3 + PrimeVue + ECharts + Tauri API
- **Primary goal**: Feature parity with the existing Vue UI

## Route Mapping

| Vue Route       | React Route     | Component                           |
| --------------- | --------------- | ----------------------------------- |
| `/`             | `/`             | `DataViewer` (table + detail split) |
| `/datasets/:id` | `/datasets/$id` | `DataViewer` with dataset selected  |
| `/credits`      | `/credits`      | `Credits`                           |

## Component Mapping

| Vue Component                 | React Component                    | Notes                                               |
| ----------------------------- | ---------------------------------- | --------------------------------------------------- |
| `App.vue`                     | `RootLayout` (`routes/__root.tsx`) | Sidebar + main outlet + status bar                  |
| `DataViewer.vue`              | `DataViewer`                       | Split layout: Dataset table + detail panel          |
| `DatasetTable.vue`            | `DatasetTable`                     | Search, tags, favorites, virtualized table, polling |
| `DatasetDetailPage.vue`       | `DatasetDetailPage`                | Tabs, form editing, metadata, columns               |
| `ChartViewer.vue`             | `ChartViewer`                      | Chart type logic + scatter variants + polling       |
| `components/ChartWrapper.vue` | `ChartWrapper`                     | ECharts rendering + resize                          |
| `components/FilterTable.vue`  | `FilterTable`                      | Combined vs split filter modes                      |
| `components/AppLink.vue`      | `SidebarLink`                      | Icon-only nav links                                 |
| `AppCredits.vue`              | `Credits`                          | External attribution link                           |

## Data/API Mapping

Source: `crates/fricon-ui/frontend-vue/src/backend.ts` and `types/chart.ts`

Target: `crates/fricon-ui/frontend/src/lib/backend.ts` and `lib/chartTypes.ts`

APIs to port:

- `getWorkspaceInfo`
- `listDatasets`
- `updateDatasetFavorite`
- `updateDatasetInfo`
- `fetchChartData`
- `getDatasetDetail`
- `onDatasetCreated` / `onDatasetUpdated`
- `getDatasetWriteStatus`
- `getFilterTableData`

## Shadcn Component Strategy

Primary: Shadcn components (and Base UI under the hood).

### Planned Shadcn Usage

- `Button` (already present)
- `Tabs`
- `Select`
- `Input`
- `Textarea`
- `Switch`
- `Badge` or `Tag` (status + tags)
- `Table` (with TanStack Table + virtualized rows)

### Gaps / Custom Implementations

- Splitter: build a simple resizable split view or use a minimal third-party splitter.
- Virtualized tables: `@tanstack/react-virtual` in table body.
- ECharts wrapper: custom `ChartWrapper` component.

## Migration Sequence (High-Level)

1. Shell layout + routes (React scaffolding).
2. API layer port.
3. Dataset table.
4. Dataset detail form.
5. Chart viewer + ECharts wrapper.
6. Filter table.
7. UI parity polish and cleanup.
