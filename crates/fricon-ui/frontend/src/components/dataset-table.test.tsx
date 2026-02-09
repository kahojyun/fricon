import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DatasetTable } from "@/components/dataset-table";
import { useDatasetTableData } from "@/components/use-dataset-table-data";
import type { DatasetInfo } from "@/lib/backend";

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

  it("toggles status filter via popover action", async () => {
    const hook = mockHookReturn();
    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Filter status" }));
    await user.click(screen.getByRole("button", { name: "Completed" }));

    expect(hook.handleStatusToggle).toHaveBeenCalledWith("Completed");
  });
});
