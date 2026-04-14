import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useChartEventSync } from "./useChartEventSync";
import { chartKeys } from "./queryKeys";
import type { DatasetChanged } from "@/shared/lib/bindings";

const { datasetChangedListenMock } = vi.hoisted(() => ({
  datasetChangedListenMock: vi.fn(),
}));

vi.mock("@/shared/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/shared/lib/bindings")>();
  return {
    ...actual,
    events: {
      ...actual.events,
      datasetChanged: {
        ...actual.events.datasetChanged,
        listen: datasetChangedListenMock,
      },
    },
  };
});

function createWrapper(client: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
  };
}

function createDatasetEvent(kind: DatasetChanged["kind"], id = 7) {
  if (kind === "globalTagsChanged") {
    return { kind } as DatasetChanged;
  }

  if (kind === "writeProgress") {
    return {
      kind,
      progress: {
        id,
        rowCount: 1,
      },
    } as DatasetChanged;
  }

  return {
    kind,
    info: {
      id,
    },
  } as unknown as DatasetChanged;
}

describe("useChartEventSync", () => {
  let listener: ((event: { payload: DatasetChanged }) => void) | undefined;

  beforeEach(() => {
    listener = undefined;
    datasetChangedListenMock.mockReset();
    datasetChangedListenMock.mockImplementation(
      (callback: (event: { payload: DatasetChanged }) => void) => {
        listener = callback;
        return Promise.resolve(() => undefined);
      },
    );
  });

  it("refetches live queries without resetting baseline on write progress", () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const invalidateSpy = vi
      .spyOn(queryClient, "invalidateQueries")
      .mockResolvedValue(undefined);
    const refetchSpy = vi
      .spyOn(queryClient, "refetchQueries")
      .mockResolvedValue(undefined);

    renderHook(() => useChartEventSync(), {
      wrapper: createWrapper(queryClient),
    });

    act(() => {
      listener?.({ payload: createDatasetEvent("writeProgress", 11) });
    });

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: chartKeys.chartData(11),
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: chartKeys.filterTableData(11),
    });
    expect(refetchSpy).toHaveBeenCalledWith({
      queryKey: chartKeys.liveChartData(11),
      type: "active",
    });
    expect(invalidateSpy).not.toHaveBeenCalledWith({
      queryKey: chartKeys.liveChartData(11),
    });
  });

  it("invalidates all chart queries when a dataset is finalized", () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const invalidateSpy = vi
      .spyOn(queryClient, "invalidateQueries")
      .mockResolvedValue(undefined);
    const refetchSpy = vi
      .spyOn(queryClient, "refetchQueries")
      .mockResolvedValue(undefined);

    renderHook(() => useChartEventSync(), {
      wrapper: createWrapper(queryClient),
    });

    act(() => {
      listener?.({ payload: createDatasetEvent("statusChanged", 5) });
    });

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: chartKeys.chartData(5),
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: chartKeys.liveChartData(5),
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: chartKeys.filterTableData(5),
    });
    expect(refetchSpy).not.toHaveBeenCalled();
  });
});
