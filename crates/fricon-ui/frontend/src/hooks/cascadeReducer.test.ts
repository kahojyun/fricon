import { describe, expect, it } from "vitest";
import type { FilterTableData } from "@/lib/backend";
import {
  cascadeReducer,
  initialCascadeState,
  resolveRow,
} from "@/hooks/cascadeReducer";

const data: FilterTableData = {
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

describe("cascadeReducer", () => {
  it("selects closest row by matching baseline fields", () => {
    const state = { ...initialCascadeState, selectedRowIndex: 1 };
    const next = cascadeReducer(state, {
      type: "field/select",
      fieldIndex: 0,
      valueIndex: 2,
      data,
      baselineRowIndex: state.selectedRowIndex,
    });
    expect(next.selectedRowIndex).toBe(2);
  });

  it("resolves to first row when selection is missing", () => {
    const resolved = resolveRow(data, 999);
    expect(resolved?.index).toBe(1);
  });

  it("updates mode and row selection", () => {
    const modeState = cascadeReducer(initialCascadeState, {
      type: "mode/set",
      mode: "split",
    });
    expect(modeState.mode).toBe("split");
    const rowState = cascadeReducer(modeState, {
      type: "row/select",
      rowIndex: 3,
    });
    expect(rowState.selectedRowIndex).toBe(3);
  });
});
