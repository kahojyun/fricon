import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DatasetTable } from "@/components/dataset-table";
import type { DatasetInfo } from "@/lib/backend";

let virtualNearEnd = false;

type ListDatasetsFn = (
  search?: string,
  tags?: string[],
  limit?: number,
  offset?: number,
) => Promise<DatasetInfo[]>;
type UpdateDatasetFavoriteFn = (id: number, favorite: boolean) => Promise<void>;
type DatasetEventListenerFn = (
  callback: (event: DatasetInfo) => void,
) => Promise<() => void>;

const listDatasetsMock = vi.fn<ListDatasetsFn>();
const updateDatasetFavoriteMock = vi.fn<UpdateDatasetFavoriteFn>();
const onDatasetCreatedMock = vi.fn<DatasetEventListenerFn>();
const onDatasetUpdatedMock = vi.fn<DatasetEventListenerFn>();

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 56,
    getVirtualItems: () => {
      if (count === 0) return [];
      if (virtualNearEnd) {
        return [{ index: count - 1, start: (count - 1) * 56 }];
      }
      return Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 56,
      }));
    },
    measureElement: () => undefined,
  }),
}));

vi.mock("@/lib/backend", () => ({
  DATASET_PAGE_SIZE: 3,
  listDatasets: (
    search?: string,
    tags?: string[],
    limit?: number,
    offset?: number,
  ) => listDatasetsMock(search, tags, limit, offset),
  updateDatasetFavorite: (id: number, favorite: boolean) =>
    updateDatasetFavoriteMock(id, favorite),
  onDatasetCreated: (callback: (event: DatasetInfo) => void) =>
    onDatasetCreatedMock(callback),
  onDatasetUpdated: (callback: (event: DatasetInfo) => void) =>
    onDatasetUpdatedMock(callback),
}));

function makeDataset(overrides: Partial<DatasetInfo> = {}): DatasetInfo {
  return {
    id: 1,
    name: "Dataset 1",
    description: "desc",
    favorite: false,
    tags: [],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    ...overrides,
  };
}

describe("DatasetTable", () => {
  beforeEach(() => {
    virtualNearEnd = false;
    listDatasetsMock.mockReset();
    updateDatasetFavoriteMock.mockReset();
    onDatasetCreatedMock.mockReset();
    onDatasetUpdatedMock.mockReset();

    onDatasetCreatedMock.mockResolvedValue(() => undefined);
    onDatasetUpdatedMock.mockResolvedValue(() => undefined);
    updateDatasetFavoriteMock.mockResolvedValue(undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads datasets with default query params", async () => {
    listDatasetsMock.mockResolvedValueOnce([makeDataset()]);

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledWith("", [], 3, 0);
    });
    expect(screen.getByText("Dataset 1")).toBeInTheDocument();
  });

  it("debounces name search and refetches with search param", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([makeDataset()])
      .mockResolvedValueOnce([makeDataset({ name: "Alpha dataset" })]);

    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(1);
    });

    await user.type(screen.getByLabelText("Search datasets"), "Alpha");
    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenLastCalledWith("Alpha", [], 3, 0);
    });
  });

  it("refetches with selected tag filters", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([makeDataset({ tags: ["vision"] })])
      .mockResolvedValueOnce([makeDataset({ tags: ["vision"] })]);

    const user = userEvent.setup();

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await screen.findByRole("button", { name: "vision" });

    await user.click(screen.getByRole("button", { name: "vision" }));
    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenLastCalledWith("", ["vision"], 3, 0);
    });
  });

  it("applies favorite-only filter locally without refetch", async () => {
    listDatasetsMock.mockResolvedValueOnce([
      makeDataset({ id: 1, name: "Not Favorite", favorite: false }),
      makeDataset({ id: 2, name: "Favorite Dataset", favorite: true }),
    ]);

    const user = userEvent.setup();
    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await screen.findByText("Not Favorite");
    await user.click(screen.getByLabelText("Favorites only"));

    expect(listDatasetsMock).toHaveBeenCalledTimes(1);
    expect(screen.queryByText("Not Favorite")).not.toBeInTheDocument();
    expect(screen.getByText("Favorite Dataset")).toBeInTheDocument();
  });

  it("rolls back optimistic favorite toggle when update fails", async () => {
    listDatasetsMock.mockResolvedValueOnce([
      makeDataset({ id: 11, name: "Pinned", favorite: true }),
    ]);
    updateDatasetFavoriteMock.mockRejectedValueOnce(new Error("boom"));

    const user = userEvent.setup();
    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await screen.findByText("Pinned");
    await user.click(screen.getByLabelText("Favorites only"));

    const row = screen.getByText("Pinned").closest('[role="button"]');
    expect(row).not.toBeNull();

    await user.click(within(row as HTMLElement).getByRole("button"));

    await waitFor(() => {
      expect(updateDatasetFavoriteMock).toHaveBeenCalledWith(11, false);
    });
    await waitFor(() => {
      expect(screen.getByText("Pinned")).toBeInTheDocument();
    });
  });

  it("loads next page when virtualized row reaches the end", async () => {
    virtualNearEnd = true;
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 4 })]);

    render(<DatasetTable onDatasetSelected={vi.fn()} />);

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(1, "", [], 3, 0);
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, "", [], 3, 3);
    });
  });
});
