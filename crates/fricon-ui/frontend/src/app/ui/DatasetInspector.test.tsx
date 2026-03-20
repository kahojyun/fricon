import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DatasetDetail } from "@/features/datasets";
import { useDatasetDetailQuery } from "@/features/datasets";
import { DatasetInspector } from "./DatasetInspector";

vi.mock("@/features/charts", () => ({
  ChartViewer: ({
    datasetId,
    datasetDetail,
  }: {
    datasetId: number;
    datasetDetail: DatasetDetail | null;
  }) => (
    <div data-testid="chart-viewer">
      {datasetId}:{datasetDetail?.name ?? "no-detail"}
    </div>
  ),
}));

vi.mock("@/features/datasets", () => ({
  DatasetPropertiesPanel: ({
    detail,
    isLoading,
    loadErrorMessage,
  }: {
    detail: DatasetDetail | null;
    isLoading: boolean;
    loadErrorMessage: string | null;
  }) => (
    <div data-testid="properties-panel">
      {isLoading ? "loading" : (detail?.name ?? loadErrorMessage ?? "empty")}
    </div>
  ),
  useDatasetDetailQuery: vi.fn(),
}));

const useDatasetDetailQueryMock = vi.mocked(useDatasetDetailQuery);

function makeDetail(overrides: Partial<DatasetDetail> = {}): DatasetDetail {
  return {
    id: 7,
    name: "Dataset 7",
    description: "Details",
    favorite: false,
    tags: [],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    trashedAt: null,
    columns: [],
    ...overrides,
  };
}

describe("DatasetInspector", () => {
  beforeEach(() => {
    useDatasetDetailQueryMock.mockReset();
  });

  it("renders the empty state when no dataset is selected", () => {
    render(<DatasetInspector />);

    expect(screen.getByText("No dataset selected")).toBeInTheDocument();
    expect(
      screen.queryByRole("tab", { name: "Charts" }),
    ).not.toBeInTheDocument();
  });

  it("renders the loading state on the properties tab", async () => {
    useDatasetDetailQueryMock.mockReturnValue({
      data: null,
      isLoading: true,
      error: null,
    } as unknown as ReturnType<typeof useDatasetDetailQuery>);

    const user = userEvent.setup();
    render(<DatasetInspector datasetId={7} />);

    expect(screen.getByRole("tab", { name: "Charts" })).toBeInTheDocument();
    expect(screen.getByTestId("chart-viewer")).toHaveTextContent("7:no-detail");

    await user.click(screen.getByRole("tab", { name: "Properties" }));
    expect(screen.getByTestId("properties-panel")).toHaveTextContent("loading");
  });

  it("renders chart and properties tabs for loaded detail", async () => {
    useDatasetDetailQueryMock.mockReturnValue({
      data: makeDetail(),
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useDatasetDetailQuery>);

    const user = userEvent.setup();
    render(<DatasetInspector datasetId={7} />);

    expect(screen.getByTestId("chart-viewer")).toHaveTextContent("7:Dataset 7");

    await user.click(screen.getByRole("tab", { name: "Properties" }));
    expect(screen.getByTestId("properties-panel")).toHaveTextContent(
      "Dataset 7",
    );
  });
});
