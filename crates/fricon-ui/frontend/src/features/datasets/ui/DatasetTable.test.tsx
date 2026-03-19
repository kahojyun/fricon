import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTableData } from "../api/useDatasetTableData";
import {
  COLUMN_VISIBILITY_STORAGE_KEY,
  buildDatasetTableDataValue,
  getRowByText,
  openColumnsMenu,
  makeDataset,
  openRowContextMenu,
  renderDatasetTable,
} from "./test-utils";
import { DatasetTable } from "./DatasetTable";

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
    window.localStorage.clear();
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("selects the dataset when a row is clicked", async () => {
    const { onDatasetSelected } = renderDatasetTable(useDatasetTableDataMock);
    const user = userEvent.setup();

    await user.click(screen.getByText("Dataset 1"));

    expect(onDatasetSelected).toHaveBeenCalledWith(1);
  });

  it("moves row focus with ArrowUp and ArrowDown within table bounds", async () => {
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

    onDatasetSelected.mockClear();
    firstRow.focus();
    await user.keyboard("{ArrowUp}");

    expect(onDatasetSelected).not.toHaveBeenCalled();
    expect(firstRow).toHaveFocus();
  });

  it("keeps existing multi-row selection when a focused row is activated from the keyboard", async () => {
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

  it("does not activate the row when keyboard interaction targets row controls", async () => {
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

  it("toggles favorite via the row action", async () => {
    const dataset = makeDataset({ id: 11, name: "Pinned", favorite: true });
    const { hook } = renderDatasetTable(useDatasetTableDataMock, {
      datasets: [dataset],
      showFavoritesOnly: true,
    });
    const user = userEvent.setup();

    await user.click(screen.getByLabelText("Remove from favorites"));

    expect(hook.toggleFavorite).toHaveBeenCalledWith(dataset);
  });

  it("applies saved column visibility from localStorage", () => {
    window.localStorage.setItem(
      COLUMN_VISIBILITY_STORAGE_KEY,
      JSON.stringify({
        favorite: true,
        id: true,
        name: false,
        tags: true,
        createdAt: false,
      }),
    );

    renderDatasetTable(useDatasetTableDataMock);

    expect(
      screen.getByRole("columnheader", { name: /^Name/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: /^Tags/ }),
    ).toBeInTheDocument();
  });

  it("updates sortable header state when sorting changes on rerender", () => {
    let currentHook = buildDatasetTableDataValue({
      sorting: [{ id: "id", desc: true }],
    });
    useDatasetTableDataMock.mockImplementation(() => currentHook);

    const onDatasetSelected = vi.fn();
    const { rerender } = render(
      <DatasetTable onDatasetSelected={onDatasetSelected} />,
    );

    expect(screen.getByRole("columnheader", { name: /^ID/ })).toHaveAttribute(
      "aria-sort",
      "descending",
    );

    currentHook = buildDatasetTableDataValue({
      sorting: [{ id: "id", desc: false }],
    });
    rerender(<DatasetTable onDatasetSelected={onDatasetSelected} />);

    expect(screen.getByRole("columnheader", { name: /^ID/ })).toHaveAttribute(
      "aria-sort",
      "ascending",
    );
  });

  it("keeps column visibility controls in sync with the current table state", async () => {
    window.localStorage.setItem(
      COLUMN_VISIBILITY_STORAGE_KEY,
      JSON.stringify({
        favorite: true,
        id: true,
        name: true,
        status: true,
        tags: true,
        createdAt: false,
      }),
    );

    renderDatasetTable(useDatasetTableDataMock);
    const user = userEvent.setup();

    const menu = await openColumnsMenu(user);

    expect(
      within(menu).getByRole("menuitemcheckbox", { name: "Tags" }),
    ).toHaveAttribute("aria-checked", "true");
    expect(
      within(menu).getByRole("menuitemcheckbox", { name: "Created At" }),
    ).toHaveAttribute("aria-checked", "false");
  });

  it("loads the next page when the virtualized rows reach the fetch threshold", async () => {
    const datasets = Array.from({ length: 20 }, (_, index) =>
      makeDataset({ id: index + 1, name: `Dataset ${index + 1}` }),
    );
    const loadNextPage = vi.fn().mockResolvedValue(undefined);

    renderDatasetTable(useDatasetTableDataMock, {
      datasets,
      hasMore: true,
      loadNextPage,
    });

    await waitFor(() => {
      expect(loadNextPage).toHaveBeenCalledTimes(1);
    });
  });

  it("does not fetch another page when there are no more backend pages", async () => {
    const datasets = Array.from({ length: 20 }, (_, index) =>
      makeDataset({ id: index + 1, name: `Dataset ${index + 1}` }),
    );
    const loadNextPage = vi.fn().mockResolvedValue(undefined);

    renderDatasetTable(useDatasetTableDataMock, {
      datasets,
      hasMore: false,
      loadNextPage,
    });

    await waitFor(() => {
      expect(screen.getByText("Dataset 20")).toBeInTheDocument();
    });
    expect(loadNextPage).not.toHaveBeenCalled();
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
    const [firstCheckbox, secondCheckbox] = rowCheckboxes;
    await user.click(firstCheckbox);
    await user.click(secondCheckbox);
    await waitFor(() => {
      expect(firstCheckbox).toBeChecked();
      expect(secondCheckbox).toBeChecked();
    });

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
    await waitFor(() => {
      expect(checkboxes[0]).toBeChecked();
      expect(checkboxes[1]).toBeChecked();
    });

    const menu = await openRowContextMenu("Dataset B");
    expect(within(menu).getByText(/Add Tags \(2\)/i)).toBeInTheDocument();
    expect(within(menu).queryByText(/Remove Tags/i)).not.toBeInTheDocument();
  });
});
