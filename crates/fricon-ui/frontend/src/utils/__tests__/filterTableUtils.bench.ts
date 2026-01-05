import { bench, describe } from "vitest";
import { tableFromArrays } from "apache-arrow";
import {
  buildFilterTableData,
  getColumnUniqueValues,
} from "../filterTableUtils";

describe("filterTableUtils benchmark", () => {
  const size = 10000;
  const colA = new Int32Array(size);
  const colB = new Array(size);
  const x = new Float32Array(size);

  for (let i = 0; i < size; i++) {
    colA[i] = i % 100; // 100 unique values
    colB[i] = `value_${i % 50}`; // 50 unique values
    x[i] = Math.random();
  }

  const table = tableFromArrays({ colA, colB, x });

  bench("buildFilterTableData (10k rows)", () => {
    buildFilterTableData(table, "x");
  });

  const data = buildFilterTableData(table, "x");

  bench("getColumnUniqueValues (10k rows)", () => {
    getColumnUniqueValues(data);
  });
});
