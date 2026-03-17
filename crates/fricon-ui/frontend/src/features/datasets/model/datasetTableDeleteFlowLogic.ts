import type { DatasetDeleteResult } from "../api/types";

export type DatasetDeleteOutcome = "success" | "failure" | "partial";

export interface DatasetDeleteSummary {
  outcome: DatasetDeleteOutcome;
  successIds: number[];
  failedIds: number[];
  failedResults: DatasetDeleteResult[];
}

export function buildSelectionFromIds(ids: number[]) {
  return ids.reduce<Record<string, boolean>>((selection, id) => {
    selection[id.toString()] = true;
    return selection;
  }, {});
}

export function summarizeDatasetDeleteResults(
  results: DatasetDeleteResult[],
): DatasetDeleteSummary {
  const successIds = results
    .filter((result) => result.success)
    .map((result) => result.id);
  const failedResults = results.filter((result) => !result.success);
  const failedIds = failedResults.map((result) => result.id);

  if (failedResults.length === 0) {
    return {
      outcome: "success",
      successIds,
      failedIds,
      failedResults,
    };
  }

  if (successIds.length === 0) {
    return {
      outcome: "failure",
      successIds,
      failedIds,
      failedResults,
    };
  }

  return {
    outcome: "partial",
    successIds,
    failedIds,
    failedResults,
  };
}
