import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { FilterTable } from "@/features/chart-viewer/ui/FilterTable";
import type { FilterTableData } from "@/shared/lib/backend";

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 32,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 32,
        end: (index + 1) * 32,
      })),
    measureElement: () => undefined,
  }),
}));

function makeData(): FilterTableData {
  return {
    fields: ["A", "B"],
    rows: [
      { index: 1, displayValues: ["A1", "B1"], valueIndices: [1, 1] },
      { index: 2, displayValues: ["A2", "B2"], valueIndices: [2, 2] },
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

describe("FilterTable", () => {
  it("keeps split columns as independent scroll containers", () => {
    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={null}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={() => undefined}
        />
      </div>,
    );

    expect(screen.getByTestId("filter-column-A")).toHaveClass(
      "flex",
      "flex-col",
      "min-h-0",
      "overflow-hidden",
    );
    expect(screen.getByTestId("filter-column-B")).toHaveClass(
      "flex",
      "flex-col",
      "min-h-0",
      "overflow-hidden",
    );
    expect(
      document.querySelectorAll(".overflow-auto").length,
    ).toBeGreaterThanOrEqual(2);
  });

  it("supports keyboard selection in split mode", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={null}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const option = screen.getByText("A2").closest("tr");
    if (!(option instanceof HTMLElement)) {
      throw new Error("Split mode option row not found");
    }

    option.focus();
    await user.keyboard("{Enter}");
    await user.keyboard(" ");

    expect(onSelectFieldValue).toHaveBeenNthCalledWith(1, 0, 2);
    expect(onSelectFieldValue).toHaveBeenNthCalledWith(2, 0, 2);
  });

  it("prevents scroll behavior when using Space to select", () => {
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={null}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const option = screen.getByText("A2").closest("tr");
    if (!(option instanceof HTMLElement)) {
      throw new Error("Split mode option row not found");
    }

    const event = new KeyboardEvent("keydown", {
      bubbles: true,
      cancelable: true,
      key: " ",
    });
    const prevented = !option.dispatchEvent(event);

    expect(prevented || event.defaultPrevented).toBe(true);
    expect(onSelectFieldValue).toHaveBeenCalledWith(0, 2);
  });
});
