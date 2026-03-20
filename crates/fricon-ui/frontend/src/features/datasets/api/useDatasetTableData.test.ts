import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTableData } from "./useDatasetTableData";
import type { DatasetInfo, ListDatasetsOptions } from "./types";

type ListDatasetsFn = (options?: ListDatasetsOptions) => Promise<DatasetInfo[]>;
type ListDatasetTagsFn = () => Promise<string[]>;
type UpdateDatasetFavoriteFn = (id: number, favorite: boolean) => Promise<void>;
type DeleteTagFn = (tag: string) => Promise<void>;
type RenameTagFn = (oldName: string, newName: string) => Promise<void>;
type MergeTagFn = (source: string, target: string) => Promise<void>;
type BatchUpdateDatasetTagsFn = (
  ids: number[],
  addTags: string[],
  removeTags: string[],
) => Promise<unknown[]>;
type DatasetEventListenerFn = (
  callback: (event: DatasetInfo) => void,
) => Promise<() => void>;

const listDatasetsMock = vi.fn<ListDatasetsFn>();
const listDatasetTagsMock = vi.fn<ListDatasetTagsFn>();
const updateDatasetFavoriteMock = vi.fn<UpdateDatasetFavoriteFn>();
const deleteTagMock = vi.fn<DeleteTagFn>();
const renameTagMock = vi.fn<RenameTagFn>();
const mergeTagMock = vi.fn<MergeTagFn>();
const batchUpdateDatasetTagsMock = vi.fn<BatchUpdateDatasetTagsFn>();
const onDatasetCreatedMock = vi.fn<DatasetEventListenerFn>();
const onDatasetUpdatedMock = vi.fn<DatasetEventListenerFn>();

vi.mock("./client", () => ({
  listDatasets: (options?: ListDatasetsOptions) => listDatasetsMock(options),
  listDatasetTags: () => listDatasetTagsMock(),
  updateDatasetFavorite: (id: number, favorite: boolean) =>
    updateDatasetFavoriteMock(id, favorite),
  deleteDatasets: vi.fn(),
  trashDatasets: vi.fn(),
  restoreDatasets: vi.fn(),
  emptyTrash: vi.fn(),
  deleteTag: (tag: string) => deleteTagMock(tag),
  renameTag: (oldName: string, newName: string) =>
    renameTagMock(oldName, newName),
  mergeTag: (source: string, target: string) => mergeTagMock(source, target),
  batchUpdateDatasetTags: (
    ids: number[],
    addTags: string[],
    removeTags: string[],
  ) => batchUpdateDatasetTagsMock(ids, addTags, removeTags),
}));

vi.mock("./events", () => ({
  onDatasetCreated: (callback: (event: DatasetInfo) => void) =>
    onDatasetCreatedMock(callback),
  onDatasetUpdated: (callback: (event: DatasetInfo) => void) =>
    onDatasetUpdatedMock(callback),
}));

vi.mock("./types", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./types")>();
  return {
    ...actual,
    DATASET_PAGE_SIZE: 3,
  };
});

function makeDataset(overrides: Partial<DatasetInfo> = {}): DatasetInfo {
  return {
    id: 1,
    name: "Dataset 1",
    description: "desc",
    favorite: false,
    tags: [],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    trashedAt: null,
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
    deleteTagMock.mockReset();
    renameTagMock.mockReset();
    mergeTagMock.mockReset();
    batchUpdateDatasetTagsMock.mockReset();
    onDatasetCreatedMock.mockReset();
    onDatasetUpdatedMock.mockReset();

    listDatasetsMock.mockResolvedValue([]);
    listDatasetTagsMock.mockResolvedValue([]);
    updateDatasetFavoriteMock.mockResolvedValue(undefined);
    deleteTagMock.mockResolvedValue(undefined);
    renameTagMock.mockResolvedValue(undefined);
    mergeTagMock.mockResolvedValue(undefined);
    batchUpdateDatasetTagsMock.mockResolvedValue([]);
    onDatasetCreatedMock.mockResolvedValue(() => undefined);
    onDatasetUpdatedMock.mockResolvedValue(() => undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("loads datasets with default query params", async () => {
    listDatasetsMock.mockResolvedValueOnce([makeDataset()]);

    renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

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
      result.current.setSearchInput("Alpha");
    });

    expect(result.current.searchInput).toBe("Alpha");
    expect(listDatasetsMock).toHaveBeenCalledTimes(1);

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

  it("expands query limit when loading the next page", async () => {
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
      limit: 6,
      offset: 0,
    });
  });

  it("keeps hasMore true while the next page is loading with placeholder data", async () => {
    let resolveNextPage: ((value: DatasetInfo[]) => void) | undefined;
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockImplementationOnce(
        () =>
          new Promise<DatasetInfo[]>((resolve) => {
            resolveNextPage = resolve;
          }),
      );

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.datasets).toHaveLength(3);
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(2);
      expect(result.current.datasets).toHaveLength(3);
      expect(result.current.hasMore).toBe(true);
    });

    act(() => {
      resolveNextPage?.([makeDataset({ id: 4 })]);
    });
  });

  it("does not reset visibleCount on the initial debounce window", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    await act(async () => {
      await new Promise((resolve) => window.setTimeout(resolve, 350));
    });

    expect(listDatasetsMock).toHaveBeenCalledTimes(2);
  });

  it("resets the query limit when the debounced search changes", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 9, name: "Alpha dataset" })]);

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

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    vi.useFakeTimers();
    act(() => {
      result.current.setSearchInput("Alpha");
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300);
    });
    vi.useRealTimers();

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
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
  });

  it("does not reuse a stale pre-search limit for later searches", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 9, name: "A dataset" })])
      .mockResolvedValueOnce([makeDataset({ id: 10, name: "AB dataset" })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    vi.useFakeTimers();

    act(() => {
      result.current.setSearchInput("A");
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300);
    });

    expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
      search: "A",
      tags: [],
      favoriteOnly: false,
      statuses: [],
      sortBy: "id",
      sortDir: "desc",
      limit: 3,
      offset: 0,
    });

    act(() => {
      result.current.setSearchInput("AB");
    });

    expect(listDatasetsMock).toHaveBeenCalledTimes(3);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300);
    });
    vi.useRealTimers();

    expect(listDatasetsMock).toHaveBeenNthCalledWith(4, {
      search: "AB",
      tags: [],
      favoriteOnly: false,
      statuses: [],
      sortBy: "id",
      sortDir: "desc",
      limit: 3,
      offset: 0,
    });
  });

  it("ignores loadNextPage while a search debounce is pending", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 9, name: "Alpha dataset" })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(1);
      expect(result.current.hasMore).toBe(true);
    });

    vi.useFakeTimers();
    act(() => {
      result.current.setSearchInput("Alpha");
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    expect(listDatasetsMock).toHaveBeenCalledTimes(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300);
    });
    vi.useRealTimers();

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
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
  });

  it("applies tag filters immediately and resets the query limit", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 9, tags: ["vision"] })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    act(() => {
      result.current.handleTagToggle("vision");
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
        search: "",
        tags: ["vision"],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 3,
        offset: 0,
      });
    });
  });

  it("applies status filters immediately and resets the query limit", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 10, status: "Writing" })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    act(() => {
      result.current.handleStatusToggle("Writing");
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: ["Writing"],
        sortBy: "id",
        sortDir: "desc",
        limit: 3,
        offset: 0,
      });
    });
  });

  it("applies favorite-only filters immediately and resets the query limit", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 11, favorite: true })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    act(() => {
      result.current.setShowFavoritesOnly(true);
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
        search: "",
        tags: [],
        favoriteOnly: true,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 3,
        offset: 0,
      });
    });
  });

  it("applies sorting changes immediately and resets the query limit", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
        makeDataset({ id: 4 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 12, name: "Alpha dataset" })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.hasMore).toBe(true);
    });

    await act(async () => {
      await result.current.loadNextPage();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "id",
        sortDir: "desc",
        limit: 6,
        offset: 0,
      });
    });

    act(() => {
      result.current.setSorting([{ id: "name", desc: false }]);
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "name",
        sortDir: "asc",
        limit: 3,
        offset: 0,
      });
    });
  });

  it("clears filters while preserving the current sorting", async () => {
    listDatasetsMock
      .mockResolvedValueOnce([
        makeDataset({ id: 1 }),
        makeDataset({ id: 2 }),
        makeDataset({ id: 3 }),
      ])
      .mockResolvedValueOnce([makeDataset({ id: 12, name: "Alpha dataset" })])
      .mockResolvedValueOnce([makeDataset({ id: 13, name: "Vision dataset" })])
      .mockResolvedValueOnce([makeDataset({ id: 14, name: "Sorted dataset" })]);

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

    act(() => {
      result.current.setSorting([{ id: "name", desc: false }]);
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(2, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "name",
        sortDir: "asc",
        limit: 3,
        offset: 0,
      });
    });

    act(() => {
      result.current.handleTagToggle("vision");
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(3, {
        search: "",
        tags: ["vision"],
        favoriteOnly: false,
        statuses: [],
        sortBy: "name",
        sortDir: "asc",
        limit: 3,
        offset: 0,
      });
    });

    act(() => {
      result.current.clearFilters();
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenNthCalledWith(4, {
        search: "",
        tags: [],
        favoriteOnly: false,
        statuses: [],
        sortBy: "name",
        sortDir: "asc",
        limit: 3,
        offset: 0,
      });
    });

    expect(result.current.searchInput).toBe("");
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

    renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

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

  it("does not enqueue an extra refetch when invalidating an in-flight request", async () => {
    let createdCallback: ((event: DatasetInfo) => void) | undefined;
    onDatasetCreatedMock.mockImplementation((callback) => {
      createdCallback = callback;
      return Promise.resolve(() => undefined);
    });

    let resolveInitial: ((value: DatasetInfo[]) => void) | undefined;
    listDatasetsMock
      .mockImplementationOnce(
        () =>
          new Promise<DatasetInfo[]>((resolve) => {
            resolveInitial = resolve;
          }),
      )
      .mockResolvedValueOnce([makeDataset({ id: 2 })]);

    const { result } = renderHook(() => useDatasetTableData(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(listDatasetsMock).toHaveBeenCalledTimes(1);
    });

    act(() => {
      createdCallback?.(makeDataset({ id: 99 }));
    });

    expect(listDatasetsMock).toHaveBeenCalledTimes(1);

    act(() => {
      resolveInitial?.([makeDataset({ id: 1 })]);
    });

    await waitFor(() => {
      expect(result.current.datasets).toHaveLength(1);
    });

    expect(listDatasetsMock).toHaveBeenCalledTimes(1);
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
