import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTableData } from "@/components/use-dataset-table-data";
import type { DatasetInfo, ListDatasetsOptions } from "@/lib/backend";

type ListDatasetsFn = (options?: ListDatasetsOptions) => Promise<DatasetInfo[]>;
type ListDatasetTagsFn = () => Promise<string[]>;
type UpdateDatasetFavoriteFn = (id: number, favorite: boolean) => Promise<void>;
type DatasetEventListenerFn = (
  callback: (event: DatasetInfo) => void,
) => Promise<() => void>;

const listDatasetsMock = vi.fn<ListDatasetsFn>();
const listDatasetTagsMock = vi.fn<ListDatasetTagsFn>();
const updateDatasetFavoriteMock = vi.fn<UpdateDatasetFavoriteFn>();
const onDatasetCreatedMock = vi.fn<DatasetEventListenerFn>();
const onDatasetUpdatedMock = vi.fn<DatasetEventListenerFn>();

vi.mock("@/lib/backend", () => ({
  DATASET_PAGE_SIZE: 3,
  listDatasets: (options?: ListDatasetsOptions) => listDatasetsMock(options),
  listDatasetTags: () => listDatasetTagsMock(),
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

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(
      QueryClientProvider,
      { client: queryClient },
      children,
    );
  };
}

describe("useDatasetTableData", () => {
  beforeEach(() => {
    listDatasetsMock.mockReset();
    listDatasetTagsMock.mockReset();
    updateDatasetFavoriteMock.mockReset();
    onDatasetCreatedMock.mockReset();
    onDatasetUpdatedMock.mockReset();

    listDatasetsMock.mockResolvedValue([]);
    listDatasetTagsMock.mockResolvedValue([]);
    updateDatasetFavoriteMock.mockResolvedValue(undefined);
    onDatasetCreatedMock.mockResolvedValue(() => undefined);
    onDatasetUpdatedMock.mockResolvedValue(() => undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads datasets with default query params", async () => {
    listDatasetsMock.mockResolvedValueOnce([makeDataset()]);

    renderHook(() => useDatasetTableData(), { wrapper: createWrapper() });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledWith({
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 3,
        offset: 0,
      });
    });
    expect(listDatasetTagsMock).toHaveBeenCalledTimes(1);
  });

  it("debounces search and refetches with updated query", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([makeDataset()])
      .mockResolvedValueOnce([makeDataset({ name: "Alpha dataset" })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

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

    expect(listDatasetsMock).toHaveBeenLastCalledWith({
      search: "Alpha",
      tags: [],
      favoriteOnly: false,
      statuses: [],
      sortBy: "id",
      sortDir: "desc",
      limit: 3,
      offset: 0,
    });
  });

  it("appends next page with offset based on current dataset count", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 4 })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(1, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 3,
        offset: 0,
      });
    });
    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
      search: "",
      tags: [],
      favoriteOnly: false,
      statuses: [],
      sortBy: "id",
      sortDir: "desc",
      limit: 3,
      offset: 3,
    });
  });

  it("refreshes datasets on create event instead of prepending locally", async () => {
    let createdCallback: ((event: DatasetInfo) => void) | undefined;
    onDatasetCreatedMock.mockImplementation((callback) => {
      createdCallback = callback;
      return Promise.resolve(() => undefined);
    });
    listDatasetsMock
      .mockResolvedValueOnce([makeDataset({ id: 1 })])
      .mockResolvedValueOnce([makeDataset({ id: 2 })]);

    renderHook(() => useDatasetTableData(), { wrapper: createWrapper() });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(1);
    });

    act(() => {
      createdCallback?.(makeDataset({ id: 99 }));
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(2);
    });
  });

  it("rolls back optimistic favorite update when backend update fails", async () => {
    listDatasetsMock.mockResolvedValueOnce([
      makeDataset({ id: 11, name: "Pinned", favorite: true }),
    ]);
    updateDatasetFavoriteMock.mockRejectedValueOnce(new Error("boom"));

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

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

    const { unmount } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });
    unmount();

    resolveCreated?.(unlistenCreated);
    resolveUpdated?.(unlistenUpdated);

    await waitFor(() => {
      expect(unlistenCreated).toHaveBeenCalledTimes(1);
      expect(unlistenUpdated).toHaveBeenCalledTimes(1);
    });
  });
});
