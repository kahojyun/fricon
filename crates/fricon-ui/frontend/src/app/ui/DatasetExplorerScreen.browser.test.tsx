import type { ReactNode } from "react";
import { createElement } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  columnId,
  makeSemanticCapabilities,
  makeSemanticDescriptor,
} from "@/features/charts/test-utils";
import { encodeChartSnapshotBufferForTest } from "@/shared/test/chartWire";
import { DatasetExplorerScreen } from "./DatasetExplorerScreen";

const { datasetCreatedListenMock, datasetUpdatedListenMock } = vi.hoisted(
  () => ({
    datasetCreatedListenMock: vi.fn(),
    datasetUpdatedListenMock: vi.fn(),
  }),
);

vi.mock("@/shared/lib/bindings", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/shared/lib/bindings")>();
  return {
    ...actual,
    events: {
      datasetCreated: {
        listen: datasetCreatedListenMock,
      },
      datasetUpdated: {
        listen: datasetUpdatedListenMock,
      },
    },
  };
});

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 36,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 36,
        end: (index + 1) * 36,
      })),
    measureElement: () => undefined,
    scrollToIndex: () => undefined,
  }),
}));

const numericSemantic = makeSemanticDescriptor();
const numericAxisSemantic = makeSemanticDescriptor({ role: "logical_index" });
const numericCapabilities = makeSemanticCapabilities();
const numericAxisCapabilities = makeSemanticCapabilities({ plottableValue: false });

vi.mock("react-resizable-panels", () => ({
  Group: ({
    children,
    ...props
  }: {
    children: ReactNode;
    ["aria-orientation"]?: "horizontal" | "vertical";
  }) => <div {...props}>{children}</div>,
  Panel: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  Separator: (props: { ["aria-orientation"]?: "horizontal" | "vertical" }) => (
    <div {...props} />
  ),
}));

// next-themes stub for ChartWrapper
vi.mock("next-themes", () => ({
  useTheme: () => ({ resolvedTheme: "light" }),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(
      QueryClientProvider,
      { client: queryClient },
      children,
    );
  };
}

describe("DatasetExplorerScreen integration", () => {
  const ipcCalls: { cmd: string; payload: unknown }[] = [];

  beforeEach(() => {
    clearMocks();
    ipcCalls.length = 0;
    datasetCreatedListenMock.mockReset();
    datasetUpdatedListenMock.mockReset();
    datasetCreatedListenMock.mockResolvedValue(() => undefined);
    datasetUpdatedListenMock.mockResolvedValue(() => undefined);
    window.localStorage.clear();
    Element.prototype.scrollIntoView = vi.fn();

    mockIPC((cmd, payload) => {
      ipcCalls.push({ cmd, payload });

      switch (cmd) {
        case "list_datasets":
          return [
            {
              id: 1,
              name: "Dataset Alpha",
              description: "Alpha dataset",
              favorite: false,
              tags: ["vision"],
              status: "Completed",
              createdAt: "2026-01-01T00:00:00Z",
              trashedAt: null,
              deletedAt: null,
            },
          ];
        case "list_dataset_tags":
          return ["vision", "audio"];
        case "dataset_detail": {
          const tAxis = {
            id: columnId("t"),
            name: "t",
            label: null,
            kind: "column" as const,
            semantic: numericAxisSemantic,
            capabilities: numericAxisCapabilities,
            isInferredAxis: true,
          };
          return {
            id: 1,
            name: "Dataset Alpha",
            description: "Alpha dataset",
            favorite: false,
            tags: ["vision"],
            status: "Completed",
            createdAt: "2026-01-01T00:00:00Z",
            trashedAt: null,
            deletedAt: null,
            payloadAvailable: true,
            columns: [
              {
                name: "t",
                label: null,
                unit: null,
                semantic: numericAxisSemantic,
                capabilities: numericAxisCapabilities,
                isInferredAxis: true,
                hiddenByDefault: false,
                isChartAxisCandidate: false,
              },
              {
                name: "signal",
                label: null,
                unit: null,
                semantic: numericSemantic,
                capabilities: numericCapabilities,
                isInferredAxis: false,
                hiddenByDefault: false,
                isChartAxisCandidate: false,
              },
            ],
            chartSemantics: {
              duplicatePolicy: "row_order_placeholder",
              indexRealization: "none",
              axes: [tAxis],
              valueColumns: [
                {
                  id: columnId("signal"),
                  name: "signal",
                  label: null,
                  semantic: numericSemantic,
                  capabilities: numericCapabilities,
                  hiddenByDefault: false,
                },
              ],
              chartAxisCandidates: [],
            },
          };
        }
        case "get_filter_table_data":
          return {
            fields: [],
            rows: [],
            columnUniqueValues: {},
          };
        case "dataset_chart_data":
          return encodeChartSnapshotBufferForTest({
            type: "xy",
            plotMode: "quantity_vs_sweep",
            drawStyle: "line",
            xName: "t",
            yName: null,
            series: [
              {
                id: "signal",
                label: "signal",
                pointCount: 2,
                values: new Float64Array([0, 1, 1, 2]),
              },
            ],
          });
        case "get_dataset_write_status":
          return { rowCount: 0 };
        default:
          return null;
      }
    });
  });

  afterEach(() => {
    clearMocks();
  });

  it("loads datasets, selects one, and renders the real inspector query flow", async () => {
    const user = userEvent.setup();

    const { container } = render(<DatasetExplorerScreen />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText("No dataset selected")).toBeInTheDocument();
    await screen.findByText("Dataset Alpha");
    const datasetRow = await waitFor(() => {
      const row = document.querySelector(
        'tr[data-slot="context-menu-trigger"]',
      );
      expect(row).not.toBeNull();
      return row;
    });
    expect(datasetRow).not.toBeNull();
    fireEvent.pointerDown(datasetRow!, { button: 0 });

    expect(
      await screen.findByRole("tab", { name: "Charts" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("No dataset selected")).not.toBeInTheDocument();

    await waitFor(() => {
      expect(
        ipcCalls.some(
          ({ cmd, payload }) =>
            cmd === "dataset_detail" &&
            (payload as { id?: number } | null)?.id === 1,
        ),
      ).toBe(true);
    });

    await waitFor(() => {
      expect(
        ipcCalls.some(
          ({ cmd, payload }) =>
            cmd === "get_filter_table_data" &&
            (
              payload as {
                id?: number;
                options?: { excludeColumns?: string[] };
              } | null
            )?.id === 1 &&
            (
              payload as {
                options?: { excludeColumns?: string[] };
              } | null
            )?.options?.excludeColumns?.[0] === columnId("t"),
        ),
      ).toBe(true);
    });

    await waitFor(() => {
      expect(
        ipcCalls.some(
          ({ cmd, payload }) =>
            cmd === "dataset_chart_data" &&
            (
              payload as {
                id?: number;
                options?: {
                  view?: string;
                  plotMode?: string;
                  quantity?: string;
                  sweepIndexColumn?: string;
                };
              } | null
            )?.id === 1 &&
            (
              payload as {
                options?: {
                  view?: string;
                  plotMode?: string;
                  quantity?: string;
                  sweepIndexColumn?: string;
                };
              } | null
            )?.options?.view === "xy" &&
            (
              payload as {
                options?: {
                  view?: string;
                  plotMode?: string;
                  quantity?: string;
                  sweepIndexColumn?: string;
                };
              } | null
            )?.options?.plotMode === "quantity_vs_sweep" &&
            (
              payload as {
                options?: {
                  view?: string;
                  plotMode?: string;
                  quantity?: string;
                  sweepIndexColumn?: string;
                };
              } | null
            )?.options?.quantity === columnId("signal") &&
            (
              payload as {
                options?: {
                  view?: string;
                  plotMode?: string;
                  quantity?: string;
                  sweepIndexColumn?: string;
                };
              } | null
            )?.options?.sweepIndexColumn === columnId("t"),
        ),
      ).toBe(true);
    });

    // Chart viewer renders a canvas for WebGL2 rendering
    expect(container.querySelector("canvas")).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Properties" }));

    expect(await screen.findByLabelText("Name")).toHaveValue("Dataset Alpha");
    expect(screen.getByLabelText("Description")).toHaveValue("Alpha dataset");
    expect(screen.getByText("signal")).toBeInTheDocument();
  });
});
