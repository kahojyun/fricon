import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DatasetDetailPage } from "@/components/dataset-detail-page";
import type { DatasetDetail } from "@/lib/backend";
import { useDatasetDetailQuery } from "@/hooks/useDatasetDetailQuery";

vi.mock("@/components/chart-viewer", () => ({
  ChartViewer: () => <div data-testid="chart-viewer" />,
}));

vi.mock("@/hooks/useDatasetDetailQuery", () => ({
  useDatasetDetailQuery: vi.fn(),
}));

const useDatasetDetailQueryMock = vi.mocked(useDatasetDetailQuery);

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

function makeDetail(overrides: Partial<DatasetDetail> = {}): DatasetDetail {
  return {
    id: 1,
    name: "Dataset 1",
    description: "First description",
    favorite: false,
    tags: ["alpha"],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    columns: [],
    ...overrides,
  };
}

describe("DatasetDetailPage", () => {
  beforeEach(() => {
    useDatasetDetailQueryMock.mockReset();
  });

  it("resyncs form fields when refreshed detail data changes", async () => {
    let currentDetail = makeDetail();
    useDatasetDetailQueryMock.mockImplementation(
      () =>
        ({
          data: currentDetail,
          isLoading: false,
          error: null,
        }) as ReturnType<typeof useDatasetDetailQuery>,
    );

    const user = userEvent.setup();
    const wrapper = createWrapper();
    const { rerender } = render(<DatasetDetailPage datasetId={1} />, {
      wrapper,
    });

    await user.click(screen.getByRole("tab", { name: "Properties" }));

    const nameInput = await screen.findByLabelText("Name");
    await user.clear(nameInput);
    await user.type(nameInput, "Local draft");
    expect(nameInput).toHaveValue("Local draft");

    currentDetail = makeDetail({
      name: "Dataset 1 (server)",
      description: "Server description",
      favorite: true,
      tags: ["beta", "gamma"],
    });
    rerender(<DatasetDetailPage datasetId={1} />);

    expect(await screen.findByLabelText("Name")).toHaveValue(
      "Dataset 1 (server)",
    );
    expect(screen.getByLabelText("Description")).toHaveValue(
      "Server description",
    );
    expect(screen.getByLabelText("Tags")).toHaveValue("beta, gamma");
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "true");
  });
});
