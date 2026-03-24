import { describe, expect, it } from "vitest";
import {
  buildSelectionFromIds,
  summarizeDatasetDeleteResults,
} from "./datasetTableDeleteFlowLogic";

describe("datasetTableDeleteFlowLogic", () => {
  it("builds row selection state from numeric ids", () => {
    expect(buildSelectionFromIds([11, 12])).toEqual({
      "11": true,
      "12": true,
    });
  });

  it("classifies all-success delete results", () => {
    expect(
      summarizeDatasetDeleteResults([
        { id: 11, success: true, error: null },
        { id: 12, success: true, error: null },
      ]),
    ).toEqual({
      outcome: "success",
      successIds: [11, 12],
      failedIds: [],
      failedResults: [],
    });
  });

  it("classifies all-failure delete results", () => {
    expect(
      summarizeDatasetDeleteResults([
        {
          id: 11,
          success: false,
          error: { code: "internal", message: "locked" },
        },
      ]),
    ).toEqual({
      outcome: "failure",
      successIds: [],
      failedIds: [11],
      failedResults: [
        {
          id: 11,
          success: false,
          error: { code: "internal", message: "locked" },
        },
      ],
    });
  });

  it("classifies mixed delete results as partial", () => {
    expect(
      summarizeDatasetDeleteResults([
        { id: 11, success: true, error: null },
        {
          id: 12,
          success: false,
          error: { code: "internal", message: "locked" },
        },
      ]),
    ).toEqual({
      outcome: "partial",
      successIds: [11],
      failedIds: [12],
      failedResults: [
        {
          id: 12,
          success: false,
          error: { code: "internal", message: "locked" },
        },
      ],
    });
  });
});
