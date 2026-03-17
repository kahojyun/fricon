import { act, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTableData } from "../api/useDatasetTableData";
import {
  COLUMN_VISIBILITY_STORAGE_KEY,
  createMemoryStorage,
  getRowByText,
  makeDataset,
  openColumnsMenu,
  openRowContextMenu,
  renderDatasetTable,
  toggleColumn,
} from "./test-utils";

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

const useDatasetTableDataMock = vi.mocked(useDatasetTableData);

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
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("renders rows and selects dataset on row click", async () => {
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock);
    const user = userEvent.setup();

    await user.click(screen.getByText("Dataset 1"));

    expect(onDatasetSelected).toHaveBeenCalledWith(1);
  });

  it("moves selection down with ArrowDown and focuses the next row", async () => {
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock, {
      datasets: [
        makeDataset({ id: 1, name: "Dataset 1" }),
        makeDataset({ id: 2, name: "Dataset 2" }),
      ],
    });
    const user = userEvent.setup();

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
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock, {
      datasets: [
        makeDataset({ id: 1, name: "Dataset 1" }),
        makeDataset({ id: 2, name: "Dataset 2" }),
      ],
    });
    const user = userEvent.setup();

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
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock, {
      datasets: [makeDataset({ id: 7, name: "Dataset 7" })],
    });
    const user = userEvent.setup();

    const row = getRowByText("Dataset 7");
    row.focus();

    await user.keyboard("{Enter}");
    await user.keyboard(" ");

    expect(onDatasetSelected).toHaveBeenNthCalledWith(1, 7);
    expect(onDatasetSelected).toHaveBeenNthCalledWith(2, 7);
    expect(row).toHaveFocus();
    expect(screen.getByLabelText("Select row")).not.toBeChecked();
  });

  it("keeps existing multi-row selection when activating a row from the keyboard", async () => {
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock, {
      datasets: [
        makeDataset({ id: 1, name: "Dataset 1" }),
        makeDataset({ id: 2, name: "Dataset 2" }),
      ],
    });
    const user = userEvent.setup();

    const rowCheckboxes = screen.getAllByLabelText("Select row");
    await user.click(rowCheckboxes[0]);
    await user.click(rowCheckboxes[1]);

    const secondRow = getRowByText("Dataset 2");
    secondRow.focus();
    await user.keyboard("{Enter}");

    expect(onDatasetSelected).toHaveBeenLastCalledWith(2);
    expect(rowCheckboxes[0]).toBeChecked();
    expect(rowCheckboxes[1]).toBeChecked();
  });

  it("keeps keyboard activation working for interactive controls inside a row", async () => {
    const dataset = makeDataset({ id: 11, name: "Dataset 11" });
    const { hook, onDatasetSelected } = renderDatasetTable(
      useDatasetTableDataMock,
      {
        datasets: [dataset],
      },
    );
    const user = userEvent.setup();

    const checkbox = screen.getByLabelText("Select row");
    act(() => {
      checkbox.focus();
    });
    await user.keyboard(" ");

    expect(checkbox).toBeChecked();
    expect(onDatasetSelected).not.toHaveBeenCalled();

    const favoriteButton = screen.getByLabelText("Add to favorites");
    act(() => {
      favoriteButton.focus();
    });
    await user.keyboard("{Enter}");

    expect(hook.toggleFavorite).toHaveBeenCalledWith(dataset);
    expect(onDatasetSelected).not.toHaveBeenCalled();
  });

  it("wires the toolbar search input to the dataset table hook", async () => {
    const { hook } = renderDatasetTable(useDatasetTableDataMock);
    const user = userEvent.setup();

    await user.type(screen.getByPlaceholderText("Filter datasets..."), "Alpha");

    await waitFor(() => {
      expect(hook.setSearchQuery).toHaveBeenCalled();
    });
  });

  it("toggles favorite via row action", async () => {
    const dataset = makeDataset({ id: 11, name: "Pinned", favorite: true });
    const { hook } = renderDatasetTable(useDatasetTableDataMock, {
      datasets: [dataset],
      favoriteOnly: true,
    });
    const user = userEvent.setup();

    await user.click(screen.getByLabelText("Remove from favorites"));

    expect(hook.toggleFavorite).toHaveBeenCalledWith(dataset);
  });

  it("exposes full dataset name on hover while using truncated cell text", () => {
    renderDatasetTable(useDatasetTableDataMock, {
      datasets: [
        makeDataset({
          id: 21,
          name: "A very long dataset name for hover preview validation",
        }),
      ],
    });

    const nameCell = screen
      .getByText("A very long dataset name for hover preview validation")
      .closest("div");
    expect(nameCell).toHaveAttribute(
      "title",
      "A very long dataset name for hover preview validation",
    );
  });

  it("triggers backend sorting state when clicking sortable header", async () => {
    const { hook } = renderDatasetTable(useDatasetTableDataMock);
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /^ID/ }));

    expect(hook.setSorting).toHaveBeenCalled();
  });

  it("uses compact column visibility defaults on first render", () => {
    renderDatasetTable(useDatasetTableDataMock);

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

    renderDatasetTable(useDatasetTableDataMock);

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
    renderDatasetTable(useDatasetTableDataMock);
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

    renderDatasetTable(useDatasetTableDataMock);

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

  it("deletes a dataset from the context menu and clears selection on success", async () => {
    const dataset = makeDataset({ id: 11, name: "Delete me" });
    const deleteDatasets = vi
      .fn()
      .mockResolvedValue([{ id: 11, success: true, error: null }]);
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock, {
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
    renderDatasetTable(useDatasetTableDataMock, {
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

  it("shows Add Tags sub-menu with available tags from the context menu", async () => {
    const dataset = makeDataset({ id: 5, name: "Tagged Dataset", tags: [] });
    renderDatasetTable(useDatasetTableDataMock, {
      datasets: [dataset],
      allTags: ["vision", "audio"],
    });

    const menu = await openRowContextMenu("Tagged Dataset");
    expect(within(menu).getByText(/Add Tags/i)).toBeInTheDocument();
  });

  it("shows Remove Tags sub-menu only when target datasets have tags", async () => {
    const dataset = makeDataset({ id: 7, name: "Has Tags", tags: ["vision"] });
    renderDatasetTable(useDatasetTableDataMock, {
      datasets: [dataset],
      allTags: ["vision"],
    });

    const menu = await openRowContextMenu("Has Tags");
    expect(within(menu).getByText(/Remove Tags/i)).toBeInTheDocument();
  });

  it("does not show Remove Tags sub-menu when target datasets have no tags", async () => {
    const dataset = makeDataset({ id: 8, name: "No Tags", tags: [] });
    renderDatasetTable(useDatasetTableDataMock, {
      datasets: [dataset],
      allTags: ["vision"],
    });

    const menu = await openRowContextMenu("No Tags");
    expect(within(menu).queryByText(/Remove Tags/i)).not.toBeInTheDocument();
  });

  it("targets all selected rows for tag operations when right-clicking a selected row", async () => {
    const datasets = [
      makeDataset({ id: 10, name: "Dataset A", tags: [] }),
      makeDataset({ id: 11, name: "Dataset B", tags: [] }),
    ];
    renderDatasetTable(useDatasetTableDataMock, {
      datasets,
      allTags: ["vision"],
    });
    const user = userEvent.setup();

    const checkboxes = screen.getAllByLabelText("Select row");
    await user.click(checkboxes[0]);
    await user.click(checkboxes[1]);

    const menu = await openRowContextMenu("Dataset B");
    expect(within(menu).getByText(/Add Tags \(2\)/i)).toBeInTheDocument();
    expect(within(menu).queryByText(/Remove Tags/i)).not.toBeInTheDocument();
  });

  it("opens the view menu from the table toolbar", async () => {
    renderDatasetTable(useDatasetTableDataMock);
    const user = userEvent.setup();

    const menu = await openColumnsMenu(user);

    expect(
      within(menu).getByRole("menuitemcheckbox", { name: "Status" }),
    ).toBeInTheDocument();
  });
});
