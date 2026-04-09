import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it, vi } from "vitest";
import type { DatasetDetail } from "../api/types";
import { ChartViewer } from "./ChartViewer";

vi.mock("./ChartWrapper", () => ({
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

function makeDetail(overrides: Partial<DatasetDetail> = {}): DatasetDetail {
  return {
    status: "Completed",
    payloadAvailable: true,
    columns: [],
    ...overrides,
  };
}

async function getSelectTrigger(label: string) {
  const fieldLabel = await screen.findByText(label);
  const container = fieldLabel.parentElement;
  if (!(container instanceof HTMLElement)) {
    throw new Error(`Container not found for select label: ${label}`);
  }
  return within(container).getByRole("combobox");
}

describe("ChartViewer", () => {
  it("renders a neutral loading state while dataset detail is loading", () => {
    mockIPC(() => null);

    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer datasetId={1} datasetDetail={null} />
      </QueryClientProvider>,
    );

    expect(screen.getByText("Loading dataset...")).toBeInTheDocument();
    expect(screen.queryByTestId("chart")).not.toBeInTheDocument();
    expect(screen.queryByText("Chart load failed")).not.toBeInTheDocument();
    expect(
      screen.queryByText("Dataset Payload Deleted"),
    ).not.toBeInTheDocument();

    clearMocks();
  });

  it("renders tombstone alert when payload is not available", () => {
    mockIPC(() => null);

    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer
          datasetId={1}
          datasetDetail={makeDetail({ payloadAvailable: false })}
        />
      </QueryClientProvider>,
    );

    expect(screen.getByText("Dataset Payload Deleted")).toBeInTheDocument();
    expect(screen.queryByTestId("chart")).not.toBeInTheDocument();
    expect(screen.queryByText("Chart load failed")).not.toBeInTheDocument();

    clearMocks();
  });

  it("renders chart error alert on query failure", async () => {
    mockIPC((cmd) => {
      if (cmd === "get_filter_table_data") {
        return { fields: [], rows: [], columnUniqueValues: {} };
      }
      if (cmd === "dataset_chart_data") {
        throw new Error("Internal Server Error");
      }
      if (cmd === "get_dataset_write_status") {
        return { rowCount: 0 };
      }
      return null;
    });

    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer
          datasetId={1}
          datasetDetail={makeDetail({
            columns: [
              { name: "t", isComplex: false, isTrace: false, isIndex: true },
              { name: "v", isComplex: false, isTrace: false, isIndex: false },
            ],
          })}
        />
      </QueryClientProvider>,
    );

    await screen.findByText("Chart load failed");
    expect(
      screen.queryByText("Dataset Payload Deleted"),
    ).not.toBeInTheDocument();

    clearMocks();
  });

  it("uses trailing Y default for trace heatmap without X axis", async () => {
    const chartPayloads: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
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
          type: "heatmap",
          xName: "trace index",
          yName: "idxB",
          xCategories: [0, 1],
          yCategories: [10],
          series: [
            {
              id: "trace_signal",
              label: "trace_signal",
              pointCount: 2,
              values: [0, 0, 1, 1, 0, 2],
            },
          ],
        };
      }
      if (cmd === "get_dataset_write_status") {
        return { rowCount: 0 };
      }
      return null;
    });

    const user = userEvent.setup();
    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer
          datasetId={1}
          datasetDetail={makeDetail({
            columns: [
              { name: "idxA", isComplex: false, isTrace: false, isIndex: true },
              { name: "idxB", isComplex: false, isTrace: false, isIndex: true },
              {
                name: "trace_signal",
                isComplex: false,
                isTrace: true,
                isIndex: false,
              },
            ],
          })}
        />
      </QueryClientProvider>,
    );

    await screen.findByTestId("chart");

    const viewTrigger = await getSelectTrigger("View");
    expect(viewTrigger).toHaveTextContent("XY");
    await user.click(viewTrigger);
    await user.click(await screen.findByRole("option", { name: "Heatmap" }));

    await waitFor(() => {
      const lastPayload = chartPayloads.at(-1);
      const options = lastPayload?.options as {
        view: string;
        xColumn?: string | null;
        yColumn?: string;
      };
      expect(options.view).toBe("heatmap");
      expect(options.xColumn).toBeNull();
      expect(options.yColumn).toBe("idxB");
    });

    clearMocks();
  });

  it("fetches chart data once per meaningful change", async () => {
    let chartCallCount = 0;
    let filterTableCallCount = 0;
    mockIPC((cmd) => {
      if (cmd === "get_filter_table_data") {
        filterTableCallCount += 1;
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
              id: "signal",
              label: "signal",
              pointCount: 2,
              values: [0, 1, 1, 2],
            },
          ],
        };
      }
      if (cmd === "get_dataset_write_status") {
        return { rowCount: 0 };
      }
      return null;
    });

    const user = userEvent.setup();

    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer
          datasetId={1}
          datasetDetail={makeDetail({
            columns: [
              { name: "t", isComplex: false, isTrace: false, isIndex: true },
              {
                name: "signal",
                isComplex: false,
                isTrace: false,
                isIndex: false,
              },
            ],
          })}
        />
      </QueryClientProvider>,
    );

    await screen.findByTestId("chart");

    await waitFor(() => {
      expect(chartCallCount).toBe(1);
      expect(filterTableCallCount).toBe(1);
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

  it("does not send legacy scatter exclusion columns", async () => {
    const chartPayloads: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
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
          type: "xy",
          projection: "complex_xy",
          drawStyle: "points",
          xName: "c (real)",
          yName: "c (imag)",
          series: [{ id: "c", label: "c", pointCount: 1, values: [1, 2] }],
        };
      }
      if (cmd === "get_dataset_write_status") {
        return { rowCount: 0 };
      }
      return null;
    });

    const user = userEvent.setup();
    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer
          datasetId={1}
          datasetDetail={makeDetail({
            columns: [
              { name: "idxA", isComplex: false, isTrace: false, isIndex: true },
              { name: "idxB", isComplex: false, isTrace: false, isIndex: true },
              { name: "c", isComplex: true, isTrace: false, isIndex: false },
            ],
          })}
        />
      </QueryClientProvider>,
    );

    await screen.findByTestId("chart");

    const projectionTrigger = await getSelectTrigger("Projection");
    await user.click(projectionTrigger);
    await user.click(
      await screen.findByRole("option", { name: "Complex Plane" }),
    );

    await waitFor(() => {
      const lastPayload = chartPayloads.at(-1);
      const options = lastPayload?.options as {
        view: string;
        projection: string;
        excludeColumns?: string[];
      };
      expect(options.view).toBe("xy");
      expect(options.projection).toBe("complex_xy");
      expect(options.excludeColumns).toEqual([]);
    });

    clearMocks();
  });

  it("sends the selected complex views to live line queries", async () => {
    const livePayloads: Record<string, unknown>[] = [];
    mockIPC((cmd, payload) => {
      if (cmd === "dataset_live_chart_data") {
        if (payload && typeof payload === "object") {
          livePayloads.push(payload as Record<string, unknown>);
        }
        return {
          mode: "reset",
          row_count: 1,
          snapshot: {
            type: "xy",
            projection: "trend",
            drawStyle: "line",
            xName: "t",
            yName: null,
            series: [
              {
                id: "sig:real",
                label: "sig (real)",
                pointCount: 1,
                values: [0, 1],
              },
            ],
          },
        };
      }
      if (cmd === "get_dataset_write_status") {
        return { rowCount: 1 };
      }
      return null;
    });

    const user = userEvent.setup();
    render(
      <QueryClientProvider client={createQueryClient()}>
        <ChartViewer
          datasetId={1}
          datasetDetail={makeDetail({
            status: "Writing",
            columns: [
              { name: "t", isComplex: false, isTrace: false, isIndex: true },
              { name: "sig", isComplex: true, isTrace: false, isIndex: false },
            ],
          })}
        />
      </QueryClientProvider>,
    );

    await screen.findByTestId("chart");

    await waitFor(() => {
      const lastPayload = livePayloads.at(-1);
      const options = lastPayload?.options as {
        view: string;
        projection: string;
        drawStyle?: string;
        complex_views?: string[] | null;
      };
      expect(options.view).toBe("xy");
      expect(options.projection).toBe("trend");
      expect(options.drawStyle).toBe("line");
      expect(options.complex_views).toEqual(["real", "imag"]);
    });

    const magLabel = await screen.findByText("Magnitude");
    const magToggle =
      magLabel.parentElement?.querySelector('[role="checkbox"]');
    if (!(magToggle instanceof HTMLElement)) {
      throw new Error("Complex view checkbox not found");
    }
    await user.click(magToggle);

    await waitFor(() => {
      const lastPayload = livePayloads.at(-1);
      const options = lastPayload?.options as {
        complex_views?: string[] | null;
      };
      expect(options.complex_views).toEqual(["real", "imag", "mag"]);
    });

    clearMocks();
  });
});
