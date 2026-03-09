import { useQuery } from "@tanstack/react-query";
import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetWriteStatusQuery } from "@/hooks/useDatasetWriteStatusQuery";

const invalidateQueriesMock = vi.fn();

vi.mock("@tanstack/react-query", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tanstack/react-query")>();
  return {
    ...actual,
    useQuery: vi.fn(),
    useQueryClient: vi.fn(() => ({
      invalidateQueries: invalidateQueriesMock,
    })),
  };
});

function makeQueryResult(rowCount: number, isComplete: boolean) {
  return {
    data: { rowCount, isComplete },
  } as ReturnType<typeof useQuery>;
}

describe("useDatasetWriteStatusQuery", () => {
  beforeEach(() => {
    invalidateQueriesMock.mockReset();
    vi.mocked(useQuery).mockReset();
  });

  it("invalidates dependent queries only when write status changes", () => {
    vi.mocked(useQuery)
      .mockReturnValueOnce(makeQueryResult(5, false))
      .mockReturnValueOnce(makeQueryResult(5, false))
      .mockReturnValueOnce(makeQueryResult(6, false))
      .mockReturnValueOnce(makeQueryResult(6, true));

    const { rerender } = renderHook(() => useDatasetWriteStatusQuery(1, true));

    expect(invalidateQueriesMock).toHaveBeenCalledTimes(2);
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(1, {
      queryKey: ["filterTableData", 1],
    });
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(2, {
      queryKey: ["chartData", 1],
    });

    rerender();
    expect(invalidateQueriesMock).toHaveBeenCalledTimes(2);

    rerender();
    expect(invalidateQueriesMock).toHaveBeenCalledTimes(4);

    rerender();
    expect(invalidateQueriesMock).toHaveBeenCalledTimes(7);
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(5, {
      queryKey: ["datasetDetail", 1],
    });
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(6, {
      queryKey: ["filterTableData", 1],
    });
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(7, {
      queryKey: ["chartData", 1],
    });
  });

  it("resets the cached snapshot when the dataset changes", () => {
    vi.mocked(useQuery)
      .mockReturnValueOnce(makeQueryResult(5, false))
      .mockReturnValueOnce(makeQueryResult(5, false));

    const { rerender } = renderHook(
      ({ datasetId }) => useDatasetWriteStatusQuery(datasetId, true),
      {
        initialProps: { datasetId: 1 },
      },
    );

    expect(invalidateQueriesMock).toHaveBeenCalledTimes(2);
    invalidateQueriesMock.mockClear();

    rerender({ datasetId: 2 });

    expect(invalidateQueriesMock).toHaveBeenCalledTimes(2);
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(1, {
      queryKey: ["filterTableData", 2],
    });
    expect(invalidateQueriesMock).toHaveBeenNthCalledWith(2, {
      queryKey: ["chartData", 2],
    });
  });
});
