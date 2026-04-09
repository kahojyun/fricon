import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { useLiveChartDataQuery } from "./useLiveChartDataQuery";
import { chartKeys } from "./queryKeys";

const { fetchLiveChartDataMock } = vi.hoisted(() => ({
  fetchLiveChartDataMock: vi.fn(),
}));

vi.mock("./client", () => ({
  fetchLiveChartData: fetchLiveChartDataMock,
}));

function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
      },
    },
  });
}

function createWrapper(client: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
  };
}

describe("useLiveChartDataQuery", () => {
  beforeEach(() => {
    fetchLiveChartDataMock.mockReset();
  });

  it("drops the incremental baseline after live query invalidation", async () => {
    const queryClient = createQueryClient();
    const wrapper = createWrapper(queryClient);
    const options = {
      view: "xy" as const,
      projection: "trend" as const,
      drawStyle: "line" as const,
      series: "signal",
      tailCount: 5,
    };

    fetchLiveChartDataMock
      .mockResolvedValueOnce({
        mode: "reset",
        rowCount: 3,
        snapshot: {
          type: "xy",
          projection: "trend",
          drawStyle: "line",
          xName: "t",
          yName: null,
          series: [],
        },
      })
      .mockResolvedValueOnce({
        mode: "reset",
        rowCount: 3,
        snapshot: {
          type: "xy",
          projection: "trend",
          drawStyle: "line",
          xName: "t",
          yName: null,
          series: [],
        },
      });

    renderHook(() => useLiveChartDataQuery(1, options), { wrapper });

    await waitFor(() => {
      expect(fetchLiveChartDataMock).toHaveBeenCalledTimes(1);
    });
    expect(fetchLiveChartDataMock).toHaveBeenNthCalledWith(1, 1, options, null);

    await act(async () => {
      await queryClient.invalidateQueries({
        queryKey: chartKeys.liveChartData(1),
      });
    });

    await waitFor(() => {
      expect(fetchLiveChartDataMock).toHaveBeenCalledTimes(2);
    });
    expect(fetchLiveChartDataMock).toHaveBeenNthCalledWith(2, 1, options, null);
  });
});
