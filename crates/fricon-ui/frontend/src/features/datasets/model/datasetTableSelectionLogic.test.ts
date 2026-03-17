import { describe, expect, it } from "vitest";
import {
  applyToggleSelectionRange,
  buildRangeSelection,
  findRowIndexById,
  getSelectionRange,
} from "./datasetTableSelectionLogic";

const rows = [{ id: "1" }, { id: "2" }, { id: "3" }, { id: "4" }];

describe("datasetTableSelectionLogic", () => {
  it("finds row indices and computes an inclusive selection range", () => {
    expect(findRowIndexById(rows, "3")).toBe(2);
    expect(findRowIndexById(rows, null)).toBe(-1);
    expect(getSelectionRange(3, 1)).toEqual({ start: 1, end: 3 });
  });

  it("builds a replace selection across a row range", () => {
    expect(buildRangeSelection(rows, 1, 3)).toEqual({
      "2": true,
      "3": true,
      "4": true,
    });
  });

  it("applies toggle drag updates on top of the initial selection", () => {
    expect(applyToggleSelectionRange(rows, 1, 2, { "1": true }, true)).toEqual({
      "1": true,
      "2": true,
      "3": true,
    });

    expect(
      applyToggleSelectionRange(
        rows,
        1,
        2,
        { "1": true, "2": true, "3": true, "4": true },
        false,
      ),
    ).toEqual({
      "1": true,
      "4": true,
    });
  });
});
