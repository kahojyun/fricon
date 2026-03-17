import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Table } from "@tanstack/react-table";
import { vi } from "vitest";
import type { UseDatasetTableDataResult } from "../api/datasetTableShared";
import type { DatasetInfo } from "../api/types";
import type { DatasetColumnMeta } from "../model/datasetColumnMeta";
import { DatasetTable } from "./DatasetTable";

export { COLUMN_VISIBILITY_STORAGE_KEY } from "../model/useDatasetColumnVisibility";

export function makeDataset(overrides: Partial<DatasetInfo> = {}): DatasetInfo {
  return {
    id: 1,
    name: "Dataset 1",
    description: "desc",
    favorite: false,
    tags: ["vision"],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    ...overrides,
  };
}

export function createMemoryStorage(): Storage {
  const store = new Map<string, string>();
  return {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.get(key) ?? null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    },
  };
}

export function buildDatasetTableDataValue(
  overrides: Partial<UseDatasetTableDataResult> = {},
): UseDatasetTableDataResult {
  return {
    datasets: [makeDataset()],
    searchQuery: "",
    setSearchQuery: vi.fn(),
    selectedTags: [],
    selectedStatuses: [],
    sorting: [{ id: "id", desc: true }],
    setSorting: vi.fn(),
    allTags: ["vision"],
    favoriteOnly: false,
    setFavoriteOnly: vi.fn(),
    hasMore: false,
    hasActiveFilters: false,
    toggleFavorite: vi.fn().mockResolvedValue(undefined),
    deleteDatasets: vi.fn().mockResolvedValue([]),
    isDeleting: false,
    batchAddTags: vi.fn().mockResolvedValue([]),
    batchRemoveTags: vi.fn().mockResolvedValue([]),
    deleteTag: vi.fn().mockResolvedValue(undefined),
    renameTag: vi.fn().mockResolvedValue(undefined),
    mergeTag: vi.fn().mockResolvedValue(undefined),
    isUpdatingTags: false,
    handleTagToggle: vi.fn(),
    handleStatusToggle: vi.fn(),
    clearFilters: vi.fn(),
    loadNextPage: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
}

export function renderDatasetTable(
  useDatasetTableDataMock: {
    mockReturnValue: (value: UseDatasetTableDataResult) => unknown;
  },
  overrides: Partial<UseDatasetTableDataResult> = {},
) {
  const hook = buildDatasetTableDataValue(overrides);
  const onDatasetSelected = vi.fn();

  useDatasetTableDataMock.mockReturnValue(hook);
  render(<DatasetTable onDatasetSelected={onDatasetSelected} />);

  return { hook, onDatasetSelected };
}

export function getRowByText(text: string) {
  const row = screen.getByText(text).closest("tr");
  if (!(row instanceof HTMLElement)) {
    throw new Error(`Row not found for text: ${text}`);
  }
  return row;
}

export async function openRowContextMenu(name: string) {
  const row = screen.getByText(name).closest("tr");
  if (!(row instanceof HTMLElement)) {
    throw new Error(`Row not found for text: ${name}`);
  }

  fireEvent.contextMenu(row);
  const menus = await screen.findAllByRole("menu");
  return menus.at(-1)!;
}

export async function openColumnsMenu(
  user: ReturnType<typeof userEvent.setup>,
) {
  await user.click(screen.getByRole("button", { name: /View/i }));
  const menus = await screen.findAllByRole("menu");
  return menus.at(-1)!;
}

export async function toggleColumn(
  user: ReturnType<typeof userEvent.setup>,
  label: string,
) {
  const menu = await openColumnsMenu(user);
  fireEvent.click(within(menu).getByRole("menuitemcheckbox", { name: label }));
}

interface MockTableColumnOptions {
  id: string;
  label?: string;
  hideable?: boolean;
  visible?: boolean;
}

export function createMockTable(
  columns: MockTableColumnOptions[] = [
    { id: "id", label: "ID", visible: true },
    { id: "name", label: "Name", hideable: false, visible: true },
    { id: "status", label: "Status", visible: true },
    { id: "tags", label: "Tags", visible: false },
    { id: "createdAt", label: "Created At", visible: false },
  ],
): Table<DatasetInfo> {
  return {
    getAllLeafColumns: () =>
      columns.map((column) => ({
        id: column.id,
        columnDef: {
          meta: {
            label: column.label ?? column.id,
            hideable: column.hideable ?? true,
          } as DatasetColumnMeta,
        },
        getIsVisible: () => column.visible ?? true,
      })),
  } as unknown as Table<DatasetInfo>;
}
