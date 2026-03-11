import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DatasetExplorerScreen } from "./DatasetExplorerScreen";

vi.mock("@/features/dataset-table", () => ({
  DatasetTable: ({
    selectedDatasetId,
    onDatasetSelected,
  }: {
    selectedDatasetId?: number;
    onDatasetSelected: (id: number) => void;
  }) => (
    <div>
      <div data-testid="table-selection">{selectedDatasetId ?? "none"}</div>
      <button type="button" onClick={() => onDatasetSelected(42)}>
        Select dataset 42
      </button>
    </div>
  ),
}));

vi.mock("./DatasetInspector", () => ({
  DatasetInspector: ({ datasetId }: { datasetId?: number }) => (
    <div data-testid="inspector-selection">{datasetId ?? "none"}</div>
  ),
}));

describe("DatasetExplorerScreen", () => {
  it("keeps selection state in the app composition layer", async () => {
    const user = userEvent.setup();
    const { rerender } = render(<DatasetExplorerScreen />);

    expect(screen.getByTestId("table-selection")).toHaveTextContent("none");
    expect(screen.getByTestId("inspector-selection")).toHaveTextContent("none");

    await user.click(screen.getByRole("button", { name: "Select dataset 42" }));

    expect(screen.getByTestId("table-selection")).toHaveTextContent("42");
    expect(screen.getByTestId("inspector-selection")).toHaveTextContent("42");

    rerender(<DatasetExplorerScreen datasetId="17" />);

    expect(screen.getByTestId("table-selection")).toHaveTextContent("17");
    expect(screen.getByTestId("inspector-selection")).toHaveTextContent("17");
  });
});
