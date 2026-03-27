import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetWriteStatusQuery } from "./useDatasetWriteStatusQuery";
import { chartKeys } from "./queryKeys";

type GetWriteStatusFn = (
  id: number,
) => Promise<{ rowCount: number; isComplete: boolean }>;

const getDatasetWriteStatusMock = vi.fn<GetWriteStatusFn>();

vi.mock("./client", () => ({
  getDatasetWriteStatus: (id: number) => getDatasetWriteStatusMock(id),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(
      QueryClientProvider,
      { client: queryClient },
      children,
    );
  };
}

describe("useDatasetWriteStatusQuery", () => {
  beforeEach(() => {
    getDatasetWriteStatusMock.mockReset();
  });

  it("fetches write status when enabled", async () => {
    const status = { rowCount: 5, isComplete: false };
    getDatasetWriteStatusMock.mockResolvedValue(status);

    const { result } = renderHook(() => useDatasetWriteStatusQuery(1, true), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.data).toEqual(status);
    });
    expect(getDatasetWriteStatusMock).toHaveBeenCalledWith(1);
  });

  it("does not fetch when disabled", () => {
    const { result } = renderHook(() => useDatasetWriteStatusQuery(1, false), {
      wrapper: createWrapper(),
    });

    expect(result.current.data).toBeUndefined();
    expect(getDatasetWriteStatusMock).not.toHaveBeenCalled();
  });

  it("uses per-dataset write status query key", () => {
    expect(chartKeys.writeStatus(42)).toEqual(["charts", "writeStatus", 42]);
  });
});
