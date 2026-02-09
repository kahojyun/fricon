import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTableData } from "@/components/use-dataset-table-data";
import type { DatasetInfo } from "@/lib/backend";

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

describe("useDatasetTableData", () => {
  beforeEach(() => {
    listDatasetsMock.mockReset();
    updateDatasetFavoriteMock.mockReset();
    onDatasetCreatedMock.mockReset();
    onDatasetUpdatedMock.mockReset();

    listDatasetsMock.mockResolvedValue([]);
    updateDatasetFavoriteMock.mockResolvedValue(undefined);
    onDatasetCreatedMock.mockResolvedValue(() => undefined);
    onDatasetUpdatedMock.mockResolvedValue(() => undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads datasets with default query params", async () => {
    listDatasetsMock.mockResolvedValueOnce([makeDataset()]);

    renderHook(() => useDatasetTableData());

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledWith("", [], 3, 0);
    });
  });

  it("debounces search and refetches with updated query", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([makeDataset()])
      .mockResolvedValueOnce([makeDataset({ name: "Alpha dataset" })]);

    const { result } = renderHook(() => useDatasetTableData());

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(1);
    });

    vi.useFakeTimers();
    act(() => {
      result.current.setSearchQuery("Alpha");
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300);
    });

    expect(listDatasetsMock).toHaveBeenLastCalledWith("Alpha", [], 3, 0);
  });

  it("appends next page with offset based on current dataset count", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 4 })]);

    const { result } = renderHook(() => useDatasetTableData());

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(1, "", [], 3, 0);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    expect(listDatasetsMock).toHaveBeenNthCalledWith(2, "", [], 3, 3);
  });

  it("rolls back optimistic favorite update when backend update fails", async () => {
    listDatasetsMock.mockResolvedValueOnce([
      makeDataset({ id: 11, name: "Pinned", favorite: true }),
    ]);
    updateDatasetFavoriteMock.mockRejectedValueOnce(new Error("boom"));

    const { result } = renderHook(() => useDatasetTableData());

    await waitFor(() => {
      expect(result.current.datasets).toHaveLength(1);
    });

    const current = result.current.datasets[0];
    expect(current.favorite).toBe(true);

    await act(async () => {
      await result.current.toggleFavorite(current);
    });

    expect(updateDatasetFavoriteMock).toHaveBeenCalledWith(11, false);
    expect(result.current.datasets[0]?.favorite).toBe(true);
  });

  it("cleans up late-resolving listeners after unmount", async () => {
    let resolveCreated: ((unlisten: () => void) => void) | undefined;
    let resolveUpdated: ((unlisten: () => void) => void) | undefined;

    const unlistenCreated = vi.fn();
    const unlistenUpdated = vi.fn();

    onDatasetCreatedMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveCreated = resolve;
        }),
    );
    onDatasetUpdatedMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveUpdated = resolve;
        }),
    );

    const { unmount } = renderHook(() => useDatasetTableData());
    unmount();

    resolveCreated?.(unlistenCreated);
    resolveUpdated?.(unlistenUpdated);

    await waitFor(() => {
      expect(unlistenCreated).toHaveBeenCalledTimes(1);
      expect(unlistenUpdated).toHaveBeenCalledTimes(1);
    });
  });
});
