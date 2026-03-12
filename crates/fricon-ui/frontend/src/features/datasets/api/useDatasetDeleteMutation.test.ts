import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetDeleteMutation } from "./useDatasetDeleteMutation";
import { datasetKeys } from "./queryKeys";
import type { DatasetDeleteResult } from "./types";

type DeleteDatasetsFn = (ids: number[]) => Promise<DatasetDeleteResult[]>;

const deleteDatasetsMock = vi.fn<DeleteDatasetsFn>();

vi.mock("./client", () => ({
  deleteDatasets: (ids: number[]) => deleteDatasetsMock(ids),
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
  beforeEach(() => {
    deleteDatasetsMock.mockReset();
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
});
