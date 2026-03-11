import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { DatasetDetail } from "@/shared/lib/backend";
import { DatasetPropertiesPanel } from "./DatasetPropertiesPanel";

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

describe("DatasetPropertiesPanel", () => {
  it("resyncs form fields when refreshed detail data changes", async () => {
    const user = userEvent.setup();
    const wrapper = createWrapper();
    const { rerender } = render(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail()}
        isLoading={false}
        loadErrorMessage={null}
      />,
      { wrapper },
    );

    const nameInput = await screen.findByLabelText("Name");
    await user.clear(nameInput);
    await user.type(nameInput, "Local draft");
    expect(nameInput).toHaveValue("Local draft");

    rerender(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail({
          name: "Dataset 1 (server)",
          description: "Server description",
          favorite: true,
          tags: ["beta", "gamma"],
        })}
        isLoading={false}
        loadErrorMessage={null}
      />,
    );

    expect(await screen.findByLabelText("Name")).toHaveValue(
      "Dataset 1 (server)",
    );
    expect(screen.getByLabelText("Description")).toHaveValue(
      "Server description",
    );
    expect(screen.getByLabelText("Tags")).toHaveValue("beta, gamma");
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "true");
  });

  it("resets the form when detail values would collide under delimiter joining", async () => {
    const wrapper = createWrapper();
    const { rerender } = render(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail({
          name: "a",
          description: "b::c",
          tags: [],
        })}
        isLoading={false}
        loadErrorMessage={null}
      />,
      { wrapper },
    );

    expect(await screen.findByLabelText("Name")).toHaveValue("a");
    expect(screen.getByLabelText("Description")).toHaveValue("b::c");

    rerender(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail({
          name: "a::b",
          description: "c",
          tags: [],
        })}
        isLoading={false}
        loadErrorMessage={null}
      />,
    );

    expect(await screen.findByLabelText("Name")).toHaveValue("a::b");
    expect(screen.getByLabelText("Description")).toHaveValue("c");
  });
});
