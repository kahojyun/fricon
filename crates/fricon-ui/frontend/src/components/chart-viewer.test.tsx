import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { ChartViewer } from "@/components/chart-viewer";

vi.mock("@/components/chart-wrapper", () => ({
  ChartWrapper: ({ data }: { data?: unknown }) => (
    <div data-testid="chart">{data ? "data" : "empty"}</div>
  ),
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 32,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 32,
      })),
    measureElement: () => undefined,
  }),
}));

vi.mock("react-resizable-panels", () => ({
  Group: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  Panel: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  Separator: () => <div />,
}));

describe("ChartViewer", () => {
  it("fetches chart data once per meaningful change", async () => {
    let chartCallCount = 0;
    mockIPC((cmd) => {
      if (cmd === "dataset_detail") {
        return {
          id: 1,
          name: "Dataset 1",
          description: "Test dataset",
          favorite: false,
          tags: [],
          status: "Completed",
          createdAt: new Date().toISOString(),
          columns: [
            { name: "t", isComplex: false, isTrace: false, isIndex: true },
            {
              name: "signal",
              isComplex: false,
              isTrace: false,
              isIndex: false,
            },
          ],
        };
      }
      if (cmd === "get_filter_table_data") {
        return {
          fields: ["A", "B"],
          rows: [
            { index: 1, displayValues: ["A1", "B1"], valueIndices: [1, 1] },
            { index: 2, displayValues: ["A2", "B1"], valueIndices: [2, 1] },
            { index: 3, displayValues: ["A2", "B2"], valueIndices: [2, 2] },
          ],
          columnUniqueValues: {
            A: [
              { index: 1, displayValue: "A1" },
              { index: 2, displayValue: "A2" },
            ],
            B: [
              { index: 1, displayValue: "B1" },
              { index: 2, displayValue: "B2" },
            ],
          },
        };
      }
      if (cmd === "dataset_chart_data") {
        chartCallCount += 1;
        return {
          type: "line",
          xName: "t",
          series: [
            {
              name: "signal",
              data: [
                [0, 1],
                [1, 2],
              ],
            },
          ],
        };
      }
      if (cmd === "get_dataset_write_status") {
        return { rowCount: 0, isComplete: true };
      }
      return null;
    });

    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          refetchOnWindowFocus: false,
        },
      },
    });

    const user = userEvent.setup();

    render(
      <QueryClientProvider client={queryClient}>
        <ChartViewer datasetId={1} />
      </QueryClientProvider>,
    );

    await screen.findByTestId("chart");

    await waitFor(() => {
      expect(chartCallCount).toBe(1);
    });

    const switchToggle = await screen.findByRole("switch");
    await user.click(switchToggle);

    await waitFor(() => {
      expect(chartCallCount).toBe(1);
    });

    const target = await screen.findByText("A2");
    await user.click(target);

    await waitFor(() => {
      expect(chartCallCount).toBe(2);
    });

    clearMocks();
  });
});
