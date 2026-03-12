import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DatasetInfo } from "../api/types";
import { useDatasetTableData } from "../api/useDatasetTableData";
import { DatasetTable } from "./DatasetTable";

const COLUMN_VISIBILITY_STORAGE_KEY = "fricon.datasetTable.columnVisibility.v1";
const { toastSuccess, toastError, toastWarning } = vi.hoisted(() => ({
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
  toastWarning: vi.fn(),
}));

vi.mock("../api/useDatasetTableData", () => ({
  useDatasetTableData: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: toastSuccess,
    error: toastError,
    warning: toastWarning,
  },
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
    scrollToIndex: () => undefined,
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
  const deleteDatasets = vi.fn().mockResolvedValue(undefined);

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
    deleteDatasets,
    isDeleting: false,
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

function getRowByText(text: string) {
  const row = screen.getByText(text).closest("tr");
  if (!(row instanceof HTMLElement)) {
    throw new Error(`Row not found for text: ${text}`);
  }
  return row;
}

async function openRowContextMenu(name: string) {
  const row = screen.getByText(name).closest("tr");
  expect(row).not.toBeNull();
  fireEvent.contextMenu(row!);
  const menus = await screen.findAllByRole("menu");
  return menus.at(-1)!;
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
    toastSuccess.mockReset();
    toastError.mockReset();
    toastWarning.mockReset();
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

  it("moves selection down with ArrowDown and focuses the next row", async () => {
    const user = userEvent.setup();
    const { onDatasetSelected } = renderDatasetTable({
      datasets: [
        makeDataset({ id: 1, name: "Dataset 1" }),
        makeDataset({ id: 2, name: "Dataset 2" }),
      ],
    });

    const firstRow = getRowByText("Dataset 1");
    const secondRow = getRowByText("Dataset 2");

    firstRow.focus();
    await user.keyboard("{ArrowDown}");

    expect(onDatasetSelected).toHaveBeenCalledWith(2);
    expect(secondRow).toHaveFocus();

    const rowCheckboxes = screen.getAllByLabelText("Select row");
    expect(rowCheckboxes[0]).not.toBeChecked();
    expect(rowCheckboxes[1]).toBeChecked();
  });

  it("moves selection up with ArrowUp and stops at table boundaries", async () => {
    const user = userEvent.setup();
    const { onDatasetSelected } = renderDatasetTable({
      datasets: [
        makeDataset({ id: 1, name: "Dataset 1" }),
        makeDataset({ id: 2, name: "Dataset 2" }),
      ],
    });

    const firstRow = getRowByText("Dataset 1");
    const secondRow = getRowByText("Dataset 2");

    secondRow.focus();
    await user.keyboard("{ArrowUp}");

    expect(onDatasetSelected).toHaveBeenCalledWith(1);
    expect(firstRow).toHaveFocus();

    onDatasetSelected.mockClear();
    firstRow.focus();
    await user.keyboard("{ArrowUp}");

    expect(onDatasetSelected).not.toHaveBeenCalled();
    expect(firstRow).toHaveFocus();

    secondRow.focus();
    await user.keyboard("{ArrowDown}");

    expect(onDatasetSelected).not.toHaveBeenCalled();
    expect(secondRow).toHaveFocus();
  });

  it("keeps Enter and Space row activation working from the keyboard", async () => {
    const user = userEvent.setup();
    const { onDatasetSelected } = renderDatasetTable({
      datasets: [makeDataset({ id: 7, name: "Dataset 7" })],
    });

    const row = getRowByText("Dataset 7");
    row.focus();

    await user.keyboard("{Enter}");
    await user.keyboard(" ");

    expect(onDatasetSelected).toHaveBeenNthCalledWith(1, 7);
    expect(onDatasetSelected).toHaveBeenNthCalledWith(2, 7);
    expect(screen.getByLabelText("Select row")).toBeChecked();
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

    const menu = await openColumnsMenu(user);

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

  it("deletes a dataset from the context menu and clears selection on success", async () => {
    const dataset = makeDataset({ id: 11, name: "Delete me" });
    const deleteDatasets = vi
      .fn()
      .mockResolvedValue([{ id: 11, success: true, error: null }]);
    const { onDatasetSelected } = renderDatasetTable({
      datasets: [dataset],
      deleteDatasets,
    });
    const user = userEvent.setup();

    await user.click(screen.getByLabelText("Select row"));

    const menu = await openRowContextMenu("Delete me");
    await user.click(within(menu).getByRole("menuitem", { name: "Delete" }));

    const dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText(/delete 1 dataset/i)).toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(deleteDatasets).toHaveBeenCalledWith([11]);
    });
    expect(onDatasetSelected).not.toHaveBeenCalledWith(undefined);
    expect(toastSuccess).toHaveBeenCalledWith(
      "Successfully deleted 1 dataset(s)",
    );
    expect(screen.getByLabelText("Select row")).not.toBeChecked();
  });

  it("keeps failed rows selected after partial delete failure", async () => {
    const datasets = [
      makeDataset({ id: 11, name: "Delete ok" }),
      makeDataset({ id: 12, name: "Delete fails" }),
    ];
    const deleteDatasets = vi.fn().mockResolvedValue([
      { id: 11, success: true, error: null },
      { id: 12, success: false, error: "locked" },
    ]);
    renderDatasetTable({
      datasets,
      deleteDatasets,
    });
    const user = userEvent.setup();

    const rowCheckboxes = screen.getAllByLabelText("Select row");
    expect(rowCheckboxes).toHaveLength(2);
    const [firstCheckbox, secondCheckbox] = rowCheckboxes;
    await user.click(firstCheckbox);
    await user.click(secondCheckbox);

    const menu = await openRowContextMenu("Delete fails");
    await user.click(
      within(menu).getByRole("menuitem", { name: "Delete Selected (2)" }),
    );

    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(deleteDatasets).toHaveBeenCalledWith([11, 12]);
    });
    expect(toastWarning).toHaveBeenCalled();
    expect(screen.getAllByLabelText("Select row")[0]).not.toBeChecked();
    expect(screen.getAllByLabelText("Select row")[1]).toBeChecked();
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(
      within(screen.getByRole("alertdialog")).getByText(/delete 1 dataset/i),
    ).toBeInTheDocument();
  });
});
