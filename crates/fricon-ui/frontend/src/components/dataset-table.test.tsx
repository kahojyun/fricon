import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DatasetTable } from "@/components/dataset-table";
import { useDatasetTableData } from "@/components/use-dataset-table-data";
import type { DatasetInfo } from "@/lib/backend";

const COLUMN_VISIBILITY_STORAGE_KEY = "fricon.datasetTable.columnVisibility.v1";

vi.mock("@/components/use-dataset-table-data", () => ({
  useDatasetTableData: vi.fn(),
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 56,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 56,
      })),
    measureElement: () => undefined,
  }),
}));

function makeDataset(overrides: Partial<DatasetInfo> = {}): DatasetInfo {
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

const useDatasetTableDataMock = vi.mocked(useDatasetTableData);

function createMemoryStorage(): Storage {
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

function mockHookReturn(overrides: Record<string, unknown> = {}) {
  const setSearchQuery = vi.fn();
  const setTagFilterQuery = vi.fn();
  const setSorting = vi.fn();
  const setFavoriteOnly = vi.fn();
  const toggleFavorite = vi.fn().mockResolvedValue(undefined);
  const handleTagToggle = vi.fn();
  const handleStatusToggle = vi.fn();
  const clearFilters = vi.fn();
  const loadNextPage = vi.fn().mockResolvedValue(undefined);

  const value = {
    datasets: [makeDataset()],
    searchQuery: "",
    setSearchQuery,
    selectedTags: [],
    selectedStatuses: [],
    tagFilterQuery: "",
    setTagFilterQuery,
    sorting: [{ id: "id", desc: true }],
    setSorting,
    filteredTagOptions: ["vision"],
    favoriteOnly: false,
    setFavoriteOnly,
    hasMore: false,
    hasActiveFilters: false,
    toggleFavorite,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
    ...overrides,
  };

  useDatasetTableDataMock.mockReturnValue(
    value as ReturnType<typeof useDatasetTableData>,
  );
  return value;
}

describe("DatasetTable", () => {
  beforeEach(() => {
    useDatasetTableDataMock.mockReset();
    Object.defineProperty(window, "localStorage", {
      value: createMemoryStorage(),
      configurable: true,
    });
  });

  it("renders rows and selects dataset on row click", async () => {
    mockHookReturn();
    const onDatasetSelected = vi.fn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={onDatasetSelected} />);

    await user.click(screen.getByText("Dataset 1"));

    expect(onDatasetSelected).toHaveBeenCalledWith(1);
  });

  it("updates search query from input", async () => {
    const hook = mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.type(screen.getByLabelText("Search datasets"), "Alpha");

    await waitFor(() => {
      expect(hook.setSearchQuery).toHaveBeenCalled();
    });
  });

  it("toggles favorite via row action", async () => {
    const dataset = makeDataset({ id: 11, name: "Pinned", favorite: true });
    const hook = mockHookReturn({ datasets: [dataset], favoriteOnly: true });
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByLabelText("Remove from favorites"));

    expect(hook.toggleFavorite).toHaveBeenCalledWith(dataset);
  });

  it("uses clear filters action from hook", async () => {
    const hook = mockHookReturn({
      hasActiveFilters: true,
      selectedTags: ["vision"],
      searchQuery: "Alpha",
    });
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Clear filters" }));

    expect(hook.clearFilters).toHaveBeenCalledTimes(1);
  });

  it("triggers backend sorting state when clicking sortable header", async () => {
    const hook = mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /^ID/ }));

    expect(hook.setSorting).toHaveBeenCalled();
  });

  it("uses compact column visibility defaults on first render", () => {
    mockHookReturn();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    expect(screen.getByRole("button", { name: /^ID/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^Name/ })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^Status/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Tags/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Created At/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Filter tags" }),
    ).not.toBeInTheDocument();
  });

  it("allows toggling column visibility and keeps name required", async () => {
    mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Columns" }));

    const nameCheckbox = screen.getByLabelText("Toggle Name column");
    expect(nameCheckbox).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByRole("button", { name: /^Name/ })).toBeInTheDocument();

    await user.click(screen.getByLabelText("Toggle Tags column"));
    expect(screen.getByRole("button", { name: /^Tags/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Filter tags" })).toBeInTheDocument();

    await user.click(screen.getByLabelText("Toggle Status column"));
    expect(
      screen.queryByRole("button", { name: /^Status/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Filter status" }),
    ).not.toBeInTheDocument();
  });

  it("supports show all and reset default column actions", async () => {
    mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Columns" }));
    await user.click(screen.getByRole("button", { name: "Show all" }));

    expect(screen.getByRole("button", { name: /^Tags/ })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^Created At/ }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Filter tags" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Reset default" }));

    expect(
      screen.queryByRole("button", { name: /^Tags/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Created At/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Filter tags" }),
    ).not.toBeInTheDocument();
  });

  it("loads saved column visibility from localStorage", () => {
    mockHookReturn();
    window.localStorage.setItem(
      COLUMN_VISIBILITY_STORAGE_KEY,
      JSON.stringify({
        favorite: true,
        id: true,
        name: false,
        status: false,
        tags: true,
        createdAt: false,
      }),
    );

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    expect(screen.getByRole("button", { name: /^Name/ })).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Status/ }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^Tags/ })).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Filter status" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Filter tags" })).toBeInTheDocument();
  });

  it("persists column visibility changes to localStorage", async () => {
    mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Columns" }));
    await user.click(screen.getByLabelText("Toggle Status column"));

    await waitFor(() => {
      const stored = window.localStorage.getItem(COLUMN_VISIBILITY_STORAGE_KEY);
      expect(stored).not.toBeNull();
      const parsed = stored ? (JSON.parse(stored) as Record<string, boolean>) : {};
      expect(parsed.status).toBe(false);
      expect(parsed.name).toBe(true);
    });
  });

  it("falls back to defaults when localStorage data is invalid", () => {
    mockHookReturn();
    window.localStorage.setItem(COLUMN_VISIBILITY_STORAGE_KEY, "not-json");

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    expect(screen.getByRole("button", { name: /^ID/ })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^Status/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Tags/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^Created At/ }),
    ).not.toBeInTheDocument();
  });

  it("toggles status filter via popover action", async () => {
    const hook = mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Filter status" }));
    await user.click(screen.getByRole("button", { name: "Completed" }));

    expect(hook.handleStatusToggle).toHaveBeenCalledWith("Completed");
  });
});
