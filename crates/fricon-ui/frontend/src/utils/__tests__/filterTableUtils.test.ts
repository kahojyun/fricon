import { describe, it, expect } from "vitest";
import { tableFromArrays } from "apache-arrow";
import {
  buildFilterTableData,
  getColumnUniqueValues,
  findMatchingRowFromSelections,
} from "../filterTableUtils";

describe("filterTableUtils", () => {
  const createTestTable = () => {
    return tableFromArrays({
      colA: [1, 2, 1, 3],
      colB: ["a", "b", "a", "c"],
      x: [10, 20, 30, 40],
    });
  };

  it("buildFilterTableData filters x column and deduplicates rows", () => {
    const table = createTestTable();
    const result = buildFilterTableData(table, "x");

    expect(result).toBeDefined();
    if (!result) return;

    expect(result.fields.length).toBe(2);
    expect(result.fields.map((f) => f.name)).toEqual(["colA", "colB"]);

    // Rows should be deduplicated.
    // Row 0: {colA: 1, colB: "a"}
    // Row 1: {colA: 2, colB: "b"}
    // Row 2: {colA: 1, colB: "a"} -> Duplicate of Row 0
    // Row 3: {colA: 3, colB: "c"}
    // So we expect 3 unique rows.
    expect(result.rows.length).toBe(3);

    // Check values
    expect(result!.rows[0]!.row.colA).toBe(1);
    expect(result!.rows[1]!.row.colA).toBe(2);
    expect(result!.rows[2]!.row.colA).toBe(3);
  });

  it("getColumnUniqueValues extracts unique values correctly", () => {
    const table = createTestTable();
    const data = buildFilterTableData(table, "x");
    const uniqueValues = getColumnUniqueValues(data);

    expect(uniqueValues.colA).toBeDefined();
    expect(uniqueValues.colB).toBeDefined();

    // colA unique values: 1, 2, 3
    const colAValues = uniqueValues.colA!.map((v) => v.value).sort();
    expect(colAValues).toEqual([1, 2, 3]);

    // colB unique values: "a", "b", "c"
    const colBValues = uniqueValues.colB!.map((v) => v.value).sort();
    expect(colBValues).toEqual(["a", "b", "c"]);
  });

  it("findMatchingRowFromSelections returns correct row", () => {
    const table = createTestTable();
    const data = buildFilterTableData(table, "x");

    // Select colA = 2
    const result = findMatchingRowFromSelections(data, {
      colA: [2],
      colB: [], // Empty selection means "all"/ignore for this column? No, code says:
      // if (selectedForColumn!.length === 0) return true;
    });

    expect(result).toBeDefined();
    expect(result?.row.colA).toBe(2);
    expect(result?.row.colB).toBe("b");
  });

  it("findMatchingRowFromSelections handles multiple criteria", () => {
    const table = createTestTable();
    const data = buildFilterTableData(table, "x");

    // Select colA = 1 and colB = "a"
    const result = findMatchingRowFromSelections(data, {
      colA: [1],
      colB: ["a"],
    });

    expect(result).toBeDefined();
    expect(result?.row.colA).toBe(1);
    expect(result?.row.colB).toBe("a");
  });

  it("findMatchingRowFromSelections returns null if no match", () => {
    const table = createTestTable();
    const data = buildFilterTableData(table, "x");

    // Select colA = 1 but colB = "b" (no such row exists: 1 is always with "a")
    const result = findMatchingRowFromSelections(data, {
      colA: [1],
      colB: ["b"],
    });

    expect(result).toBeNull();
  });
});
