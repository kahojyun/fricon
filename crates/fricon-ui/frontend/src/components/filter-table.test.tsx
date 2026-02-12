import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FilterTable } from "@/components/filter-table";
import type { FilterTableData } from "@/lib/backend";

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
      document.querySelectorAll('[data-slot="scroll-area-viewport"]').length,
    ).toBeGreaterThanOrEqual(2);
  });
});
