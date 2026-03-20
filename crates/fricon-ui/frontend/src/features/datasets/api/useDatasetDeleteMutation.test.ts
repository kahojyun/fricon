import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  useDatasetDeleteMutation,
  useDatasetRestoreMutation,
  useDatasetTrashMutation,
} from "./useDatasetDeleteMutation";
import { datasetKeys } from "./queryKeys";
import type { DatasetDeleteResult } from "./types";

type DatasetMutationFn = (ids: number[]) => Promise<DatasetDeleteResult[]>;

const deleteDatasetsMock = vi.fn<DatasetMutationFn>();
const trashDatasetsMock = vi.fn<DatasetMutationFn>();
const restoreDatasetsMock = vi.fn<DatasetMutationFn>();

vi.mock("./client", () => ({
  deleteDatasets: (ids: number[]) => deleteDatasetsMock(ids),
  trashDatasets: (ids: number[]) => trashDatasetsMock(ids),
  restoreDatasets: (ids: number[]) => restoreDatasetsMock(ids),
  emptyTrash: vi.fn(),
}));

function createWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(
      QueryClientProvider,
      { client: queryClient },
      children,
    );
  };
}

describe("useDatasetDeleteMutation", () => {
  let consoleErrorSpy: { mockRestore: () => void };

  beforeEach(() => {
    deleteDatasetsMock.mockReset();
    trashDatasetsMock.mockReset();
    restoreDatasetsMock.mockReset();
    consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {
      return undefined;
    });
  });

  afterEach(() => {
    consoleErrorSpy.mockRestore();
  });

  it("returns per-id results and refreshes affected dataset queries", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    const results: DatasetDeleteResult[] = [
      { id: 1, success: true, error: null },
      { id: 2, success: false, error: "missing" },
    ];

    deleteDatasetsMock.mockResolvedValue(results);

    const { result } = renderHook(
      () => useDatasetDeleteMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    let actualResults: DatasetDeleteResult[] | undefined;
    await act(async () => {
      actualResults = await result.current.deleteDatasets([1, 2]);
    });

    expect(actualResults).toEqual(results);
    expect(deleteDatasetsMock).toHaveBeenCalledWith([1, 2]);
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(1),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(2),
    });
  });

  it("rethrows API failures and clears deleting state", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    deleteDatasetsMock.mockRejectedValueOnce(new Error("delete failed"));

    const { result } = renderHook(
      () => useDatasetDeleteMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    let deletePromise!: Promise<DatasetDeleteResult[]>;
    act(() => {
      deletePromise = result.current.deleteDatasets([1]);
    });
    void deletePromise.catch(() => undefined);

    await waitFor(() => {
      expect(result.current.isDeleting).toBe(true);
    });
    await expect(deletePromise).rejects.toThrow("delete failed");
    await waitFor(() => {
      expect(result.current.isDeleting).toBe(false);
    });

    expect(refreshDatasets).not.toHaveBeenCalled();
  });

  it("rethrows refresh failures and clears deleting state", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const refreshDatasets = vi
      .fn()
      .mockRejectedValue(new Error("refresh failed"));
    deleteDatasetsMock.mockResolvedValue([
      { id: 1, success: true, error: null },
    ]);

    const { result } = renderHook(
      () => useDatasetDeleteMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    let deletePromise!: Promise<DatasetDeleteResult[]>;
    act(() => {
      deletePromise = result.current.deleteDatasets([1]);
    });
    void deletePromise.catch(() => undefined);

    await waitFor(() => {
      expect(result.current.isDeleting).toBe(true);
    });
    await expect(deletePromise).rejects.toThrow("refresh failed");
    await waitFor(() => {
      expect(result.current.isDeleting).toBe(false);
    });
  });

  it("keeps isDeleting true while refresh is still pending after a successful delete", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    let resolveRefresh: (() => void) | undefined;
    const refreshDatasets = vi.fn().mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveRefresh = resolve;
        }),
    );
    deleteDatasetsMock.mockResolvedValue([
      { id: 1, success: true, error: null },
    ]);

    const { result } = renderHook(
      () => useDatasetDeleteMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    let deletePromise!: Promise<DatasetDeleteResult[]>;
    act(() => {
      deletePromise = result.current.deleteDatasets([1]);
    });

    await waitFor(() => {
      expect(refreshDatasets).toHaveBeenCalledTimes(1);
      expect(result.current.isDeleting).toBe(true);
    });

    resolveRefresh?.();
    await expect(deletePromise).resolves.toEqual([
      { id: 1, success: true, error: null },
    ]);
    await waitFor(() => {
      expect(result.current.isDeleting).toBe(false);
    });
  });

  it("invalidates tags and detail queries after trashing datasets", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    const results: DatasetDeleteResult[] = [
      { id: 3, success: true, error: null },
      { id: 4, success: true, error: null },
    ];

    trashDatasetsMock.mockResolvedValue(results);

    const { result } = renderHook(
      () => useDatasetTrashMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    await act(async () => {
      await result.current.trashDatasets([3, 4]);
    });

    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(3),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(4),
    });
  });

  it("invalidates tags and detail queries after restoring datasets", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    const results: DatasetDeleteResult[] = [
      { id: 5, success: true, error: null },
    ];

    restoreDatasetsMock.mockResolvedValue(results);

    const { result } = renderHook(
      () => useDatasetRestoreMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    await act(async () => {
      await result.current.restoreDatasets([5]);
    });

    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(5),
    });
  });
});
