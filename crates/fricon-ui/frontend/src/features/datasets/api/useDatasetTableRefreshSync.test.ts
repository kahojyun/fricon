import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { datasetKeys } from "./queryKeys";
import { useDatasetTableRefreshSync } from "./useDatasetTableRefreshSync";
import type { DatasetInfo } from "./types";

type DatasetEventListenerFn = (
  callback: (event: DatasetInfo) => void,
) => Promise<() => void>;

const onDatasetCreatedMock = vi.fn<DatasetEventListenerFn>();
const onDatasetUpdatedMock = vi.fn<DatasetEventListenerFn>();

vi.mock("./events", () => ({
  onDatasetCreated: (callback: (event: DatasetInfo) => void) =>
    onDatasetCreatedMock(callback),
  onDatasetUpdated: (callback: (event: DatasetInfo) => void) =>
    onDatasetUpdatedMock(callback),
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

describe("useDatasetTableRefreshSync", () => {
  beforeEach(() => {
    onDatasetCreatedMock.mockReset();
    onDatasetUpdatedMock.mockReset();
    onDatasetCreatedMock.mockResolvedValue(() => undefined);
    onDatasetUpdatedMock.mockResolvedValue(() => undefined);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("refreshes datasets and invalidates tags when an update event arrives", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    let updatedCallback: ((event: DatasetInfo) => void) | undefined;

    onDatasetUpdatedMock.mockImplementation((callback) => {
      updatedCallback = callback;
      return Promise.resolve(() => undefined);
    });

    renderHook(() => useDatasetTableRefreshSync([], refreshDatasets), {
      wrapper: createWrapper(queryClient),
    });

    act(() => {
      updatedCallback?.(makeDataset({ id: 7, status: "Writing" }));
    });

    await waitFor(() => {
      expect(refreshDatasets).toHaveBeenCalledTimes(1);
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
  });

  it("polls while datasets are writing and stops polling when writing clears or unmounts", async () => {
    vi.useFakeTimers();
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);

    const { rerender, unmount } = renderHook(
      ({ datasets }) => useDatasetTableRefreshSync(datasets, refreshDatasets),
      {
        initialProps: {
          datasets: [makeDataset({ id: 1, status: "Writing" })],
        },
        wrapper: createWrapper(queryClient),
      },
    );

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4000);
    });

    expect(refreshDatasets).toHaveBeenCalledTimes(2);
    expect(invalidateQueriesSpy).not.toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });

    rerender({
      datasets: [makeDataset({ id: 1, status: "Completed" })],
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4000);
    });

    expect(refreshDatasets).toHaveBeenCalledTimes(2);

    rerender({
      datasets: [makeDataset({ id: 1, status: "Writing" })],
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000);
    });

    expect(refreshDatasets).toHaveBeenCalledTimes(3);

    unmount();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4000);
    });

    expect(refreshDatasets).toHaveBeenCalledTimes(3);
  });
});
