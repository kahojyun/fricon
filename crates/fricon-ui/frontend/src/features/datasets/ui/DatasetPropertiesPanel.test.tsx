import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { datasetKeys } from "../api/queryKeys";
import type { DatasetDetail } from "../api/types";
import { DatasetPropertiesPanel } from "./DatasetPropertiesPanel";

type UpdateDatasetInfoFn = (
  id: number,
  update: {
    name: string;
    description: string;
    favorite: boolean;
    tags: string[];
  },
) => Promise<void>;

const updateDatasetInfoMock = vi.fn<UpdateDatasetInfoFn>();

vi.mock("../api/client", () => ({
  updateDatasetInfo: (
    id: number,
    update: {
      name: string;
      description: string;
      favorite: boolean;
      tags: string[];
    },
  ) => updateDatasetInfoMock(id, update),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  return {
    queryClient,
    wrapper,
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
  beforeEach(() => {
    updateDatasetInfoMock.mockReset();
    updateDatasetInfoMock.mockResolvedValue(undefined);
  });

  it("disables save while there are no effective changes", async () => {
    const user = userEvent.setup();
    const { wrapper } = createWrapper();

    render(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail({ tags: ["alpha", "alpha"] })}
        isLoading={false}
        loadErrorMessage={null}
      />,
      { wrapper },
    );

    const saveButton = await screen.findByRole("button", { name: "Save" });
    expect(saveButton).toBeDisabled();

    const tagsInput = screen.getByPlaceholderText("Comma separated tags");
    await user.clear(tagsInput);
    await user.type(tagsInput, " alpha ");

    expect(saveButton).toBeDisabled();
  });

  it("saves normalized detail updates, invalidates the detail query, and shows success feedback", async () => {
    const user = userEvent.setup();
    const { queryClient, wrapper } = createWrapper();
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");

    render(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail({
          tags: ["beta", "alpha"],
        })}
        isLoading={false}
        loadErrorMessage={null}
      />,
      { wrapper },
    );

    await user.clear(await screen.findByLabelText("Name"));
    await user.type(screen.getByLabelText("Name"), "Dataset 1 (edited)");
    await user.clear(screen.getByLabelText("Description"));
    await user.type(
      screen.getByLabelText("Description"),
      "Updated description",
    );
    await user.clear(screen.getByPlaceholderText("Comma separated tags"));
    await user.type(
      screen.getByPlaceholderText("Comma separated tags"),
      " zeta, alpha, beta, alpha ",
    );
    await user.click(screen.getByRole("switch"));
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(updateDatasetInfoMock).toHaveBeenCalledWith(1, {
        name: "Dataset 1 (edited)",
        description: "Updated description",
        favorite: true,
        tags: ["alpha", "beta", "zeta"],
      });
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(1),
    });
    expect(await screen.findByText("Dataset updated.")).toBeInTheDocument();
  });

  it("shows an error message when saving dataset details fails", async () => {
    const user = userEvent.setup();
    const { wrapper } = createWrapper();
    updateDatasetInfoMock.mockRejectedValueOnce(new Error("Save exploded"));

    render(
      <DatasetPropertiesPanel
        datasetId={1}
        detail={makeDetail()}
        isLoading={false}
        loadErrorMessage={null}
      />,
      { wrapper },
    );

    await user.clear(await screen.findByLabelText("Name"));
    await user.type(screen.getByLabelText("Name"), "Dataset 1 (edited)");
    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("Save failed")).toBeInTheDocument();
    expect(screen.getByText("Save exploded")).toBeInTheDocument();
  });

  it("resyncs form fields when refreshed detail data changes", async () => {
    const user = userEvent.setup();
    const { wrapper } = createWrapper();
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
    const { wrapper } = createWrapper();
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
