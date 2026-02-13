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
  it("prefers trailing index columns for default line/heatmap axes", async () => {
    const chartPayloads: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
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
            { name: "idxA", isComplex: false, isTrace: false, isIndex: true },
            { name: "idxB", isComplex: false, isTrace: false, isIndex: true },
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
          fields: ["idxA", "idxB"],
          rows: [
            { index: 1, displayValues: ["1", "10"], valueIndices: [1, 1] },
          ],
          columnUniqueValues: {
            idxA: [{ index: 1, displayValue: "1" }],
            idxB: [{ index: 1, displayValue: "10" }],
          },
        };
      }
      if (cmd === "dataset_chart_data") {
        if (payload && typeof payload === "object") {
          chartPayloads.push(payload as Record<string, unknown>);
        }
        const options = (payload as { options?: { chartType?: string } })
          ?.options;
        if (options?.chartType === "heatmap") {
          return {
            type: "heatmap",
            xName: "idxB",
            yName: "idxA",
            xCategories: [10],
            yCategories: [1],
            series: [{ name: "signal", data: [[0, 0, 1]] }],
          };
        }
        return {
          type: "line",
          xName: "idxB",
          series: [{ name: "signal", data: [[10, 1]] }],
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
      const firstPayload = chartPayloads[0];
      const options = firstPayload?.options as {
        chartType: string;
        xColumn?: string;
      };
      expect(options.chartType).toBe("line");
      expect(options.xColumn).toBe("idxB");
    });

    const chartTypeLabel = await screen.findByText("Chart Type");
    const chartTypeTrigger = chartTypeLabel
      .closest("div")
      ?.querySelector('[data-slot="select-trigger"]');
    if (!(chartTypeTrigger instanceof HTMLElement)) {
      throw new Error("Chart type trigger not found");
    }
    await user.click(chartTypeTrigger);
    await user.click(await screen.findByText("heatmap"));

    await waitFor(() => {
      const lastPayload = chartPayloads.at(-1);
      const options = lastPayload?.options as {
        chartType: string;
        xColumn?: string;
        yColumn?: string;
      };
      expect(options.chartType).toBe("heatmap");
      expect(options.xColumn).toBe("idxB");
      expect(options.yColumn).toBe("idxA");
    });

    clearMocks();
  });

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

  it("allows index exclusion for scalar complex scatter mode", async () => {
    const chartPayloads: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
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
            { name: "idxA", isComplex: false, isTrace: false, isIndex: true },
            { name: "idxB", isComplex: false, isTrace: false, isIndex: true },
            { name: "c", isComplex: true, isTrace: false, isIndex: false },
          ],
        };
      }
      if (cmd === "get_filter_table_data") {
        return {
          fields: ["idxA", "idxB"],
          rows: [
            { index: 1, displayValues: ["1", "10"], valueIndices: [1, 1] },
          ],
          columnUniqueValues: {
            idxA: [{ index: 1, displayValue: "1" }],
            idxB: [{ index: 1, displayValue: "10" }],
          },
        };
      }
      if (cmd === "dataset_chart_data") {
        if (payload && typeof payload === "object") {
          chartPayloads.push(payload as Record<string, unknown>);
        }
        return {
          type: "scatter",
          xName: "c (real)",
          yName: "c (imag)",
          series: [{ name: "c", data: [[1, 2]] }],
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

    const chartTypeLabel = await screen.findByText("Chart Type");
    const chartTypeTrigger = chartTypeLabel
      .closest("div")
      ?.querySelector('[data-slot="select-trigger"]');
    if (!(chartTypeTrigger instanceof HTMLElement)) {
      throw new Error("Chart type trigger not found");
    }
    await user.click(chartTypeTrigger);
    await user.click(await screen.findByText("scatter"));

    await waitFor(() => {
      const lastPayload = chartPayloads.at(-1);
      const options = lastPayload?.options as {
        chartType: string;
        scatter: { mode: string };
        excludeColumns?: string[];
      };
      expect(options.chartType).toBe("scatter");
      expect(options.scatter.mode).toBe("complex");
      expect(options.excludeColumns).toEqual(["idxB"]);
    });

    const excludeLabel = await screen.findByText("Index Column (excluded)");
    const excludeTrigger = excludeLabel
      .closest("div")
      ?.querySelector('[data-slot="select-trigger"]');
    if (!(excludeTrigger instanceof HTMLElement)) {
      throw new Error("Excluded index trigger not found");
    }
    await user.click(excludeTrigger);
    const idxBChoices = await screen.findAllByText("idxB");
    await user.click(idxBChoices[idxBChoices.length - 1]);

    await waitFor(() => {
      const lastPayload = chartPayloads.at(-1);
      const options = lastPayload?.options as { excludeColumns?: string[] };
      expect(options.excludeColumns).toEqual(["idxB"]);
    });

    clearMocks();
  });
});
