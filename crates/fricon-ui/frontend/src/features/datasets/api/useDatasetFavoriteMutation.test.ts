import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { datasetKeys } from "./queryKeys";
import { useDatasetFavoriteMutation } from "./useDatasetFavoriteMutation";
import type { DatasetInfo } from "./types";

type UpdateDatasetFavoriteFn = (id: number, favorite: boolean) => Promise<void>;

const updateDatasetFavoriteMock = vi.fn<UpdateDatasetFavoriteFn>();

vi.mock("./client", () => ({
  updateDatasetFavorite: (id: number, favorite: boolean) =>
    updateDatasetFavoriteMock(id, favorite),
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

describe("useDatasetFavoriteMutation", () => {
  const datasetQueryKey = datasetKeys.list(
    {
      search: "",
      tags: [],
      favoriteOnly: false,
      statuses: [],
      sorting: [{ id: "id", desc: true }],
    },
    3,
  );

  beforeEach(() => {
    updateDatasetFavoriteMock.mockReset();
  });

  it("optimistically updates the cached favorite state and keeps it after refresh", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    const first = makeDataset({ id: 1, favorite: false });
    const second = makeDataset({ id: 2, favorite: true, name: "Pinned" });
    queryClient.setQueryData(datasetQueryKey, [first, second]);
    updateDatasetFavoriteMock.mockResolvedValue(undefined);

    const { result } = renderHook(
      () => useDatasetFavoriteMutation(datasetQueryKey, refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    await act(async () => {
      await result.current.toggleFavorite(first);
    });

    expect(updateDatasetFavoriteMock).toHaveBeenCalledWith(1, true);
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
    expect(queryClient.getQueryData(datasetQueryKey)).toEqual([
      expect.objectContaining({ id: 1, favorite: true }),
      expect.objectContaining({ id: 2, favorite: true }),
    ]);
  });

  it("rolls back the optimistic cache update when the backend write fails", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    const dataset = makeDataset({ id: 1, favorite: false });
    queryClient.setQueryData(datasetQueryKey, [dataset]);
    updateDatasetFavoriteMock.mockRejectedValue(new Error("write failed"));

    const { result } = renderHook(
      () => useDatasetFavoriteMutation(datasetQueryKey, refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    await act(async () => {
      await result.current.toggleFavorite(dataset);
    });

    expect(refreshDatasets).not.toHaveBeenCalled();
    expect(queryClient.getQueryData(datasetQueryKey)).toEqual([
      expect.objectContaining({ id: 1, favorite: false }),
    ]);
  });

  it("keeps the optimistic cache state when refresh fails after a successful write", async () => {
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
    const dataset = makeDataset({ id: 1, favorite: false });
    queryClient.setQueryData(datasetQueryKey, [dataset]);
    updateDatasetFavoriteMock.mockResolvedValue(undefined);

    const { result } = renderHook(
      () => useDatasetFavoriteMutation(datasetQueryKey, refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    await act(async () => {
      await result.current.toggleFavorite(dataset);
    });

    expect(updateDatasetFavoriteMock).toHaveBeenCalledWith(1, true);
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
    expect(queryClient.getQueryData(datasetQueryKey)).toEqual([
      expect.objectContaining({ id: 1, favorite: true }),
    ]);
  });
});
