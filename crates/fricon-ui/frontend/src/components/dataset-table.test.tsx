import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
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
        end: (index + 1) * 56,
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

function renderDatasetTable(overrides: Record<string, unknown> = {}) {
  const hook = mockHookReturn(overrides);
  const onDatasetSelected = vi.fn();
  render(<DatasetTable onDatasetSelected={onDatasetSelected} />);
  return { hook, onDatasetSelected };
}

async function openColumnsMenu(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: /View/i }));
  const menus = await screen.findAllByRole("menu");
  return menus.at(-1)!;
}

async function toggleColumn(
  user: ReturnType<typeof userEvent.setup>,
  label: string,
) {
  const menu = await openColumnsMenu(user);
  fireEvent.click(within(menu).getByRole("menuitemcheckbox", { name: label }));
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
    const { onDatasetSelected } = renderDatasetTable();
    const user = userEvent.setup();

    await user.click(screen.getByText("Dataset 1"));

    expect(onDatasetSelected).toHaveBeenCalledWith(1);
  });

  it("updates search query from input", async () => {
    const { hook } = renderDatasetTable();
    const user = userEvent.setup();

    await user.type(screen.getByPlaceholderText("Filter datasets..."), "Alpha");

    await waitFor(() => {
      expect(hook.setSearchQuery).toHaveBeenCalled();
    });
  });

  it("toggles favorite via row action", async () => {
    const dataset = makeDataset({ id: 11, name: "Pinned", favorite: true });
    const { hook } = renderDatasetTable({
      datasets: [dataset],
      favoriteOnly: true,
    });
    const user = userEvent.setup();

    await user.click(screen.getByLabelText("Remove from favorites"));

    expect(hook.toggleFavorite).toHaveBeenCalledWith(dataset);
  });

  it("exposes full dataset name on hover while using truncated cell text", () => {
    mockHookReturn({
      datasets: [
        makeDataset({
          id: 21,
          name: "A very long dataset name for hover preview validation",
        }),
      ],
    });

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    const nameCell = screen
      .getByText("A very long dataset name for hover preview validation")
      .closest("div");
    expect(nameCell).toHaveAttribute(
      "title",
      "A very long dataset name for hover preview validation",
    );
  });

  it("uses clear filters action from hook", async () => {
    const { hook } = renderDatasetTable({
      hasActiveFilters: true,
      selectedTags: ["vision"],
      searchQuery: "Alpha",
    });
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: "Reset" }));

    expect(hook.clearFilters).toHaveBeenCalledTimes(1);
  });

  it("triggers backend sorting state when clicking sortable header", async () => {
    const { hook } = renderDatasetTable();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /^ID/ }));

    expect(hook.setSorting).toHaveBeenCalled();
  });

  it("uses compact column visibility defaults on first render", () => {
    renderDatasetTable();

    expect(
      screen.getByRole("columnheader", { name: /^ID/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: /^Name/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: /^Status/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: /^Tags/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: /^Created At/ }),
    ).not.toBeInTheDocument();
  });

  it("allows toggling column visibility and keeps name required", async () => {
    renderDatasetTable();
    const user = userEvent.setup();

    let menu = await openColumnsMenu(user);

    const nameCheckbox = within(menu).getByRole("menuitemcheckbox", {
      name: "Name",
    });
    expect(nameCheckbox).toHaveAttribute("aria-disabled", "true");
    expect(
      screen.getByRole("columnheader", { name: /^Name/ }),
    ).toBeInTheDocument();

    fireEvent.click(
      within(menu).getByRole("menuitemcheckbox", { name: "Tags" }),
    );
    expect(
      screen.getByRole("columnheader", { name: /^Tags/ }),
    ).toBeInTheDocument();

    menu = await openColumnsMenu(user);
    fireEvent.click(
      within(menu).getByRole("menuitemcheckbox", { name: "Status" }),
    );
    await waitFor(() => {
      expect(
        screen.queryByRole("columnheader", { name: /^Status/ }),
      ).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("columnheader", { name: /^Tags/ }),
    ).toBeInTheDocument();
  });

  it("supports show all and reset default column actions", async () => {
    renderDatasetTable();
    const user = userEvent.setup();

    let menu = await openColumnsMenu(user);
    await user.click(within(menu).getByRole("menuitem", { name: /Show all/i }));

    expect(
      screen.getByRole("columnheader", { name: /^Tags/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: /^Created At/ }),
    ).toBeInTheDocument();

    menu = await openColumnsMenu(user);
    await user.click(
      within(menu).getByRole("menuitem", { name: /Reset default/i }),
    );

    expect(
      screen.queryByRole("columnheader", { name: /^Tags/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: /^Created At/ }),
    ).not.toBeInTheDocument();
  });

  it("loads saved column visibility from localStorage", () => {
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
    renderDatasetTable();

    expect(
      screen.getByRole("columnheader", { name: /^Name/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: /^Status/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: /^Tags/ }),
    ).toBeInTheDocument();
  });

  it("persists column visibility changes to localStorage", async () => {
    renderDatasetTable();
    const user = userEvent.setup();

    await toggleColumn(user, "Status");

    await waitFor(() => {
      const stored = window.localStorage.getItem(COLUMN_VISIBILITY_STORAGE_KEY);
      expect(stored).not.toBeNull();
      const parsed = stored
        ? (JSON.parse(stored) as Record<string, boolean>)
        : {};
      expect(parsed.status).toBe(false);
      expect(parsed.name).toBe(true);
    });
  });

  it("falls back to defaults when localStorage data is invalid", () => {
    window.localStorage.setItem(COLUMN_VISIBILITY_STORAGE_KEY, "not-json");
    renderDatasetTable();

    expect(
      screen.getByRole("columnheader", { name: /^ID/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: /^Status/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: /^Tags/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: /^Created At/ }),
    ).not.toBeInTheDocument();
  });

  it("toggles status filter via popover action", async () => {
    const { hook } = renderDatasetTable();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Status/i }));
    await user.click(screen.getByRole("button", { name: /Completed/i }));

    expect(hook.handleStatusToggle).toHaveBeenCalledWith("Completed");
  });
});
