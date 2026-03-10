import type { FilterTableData, FilterTableRow } from "@/lib/backend";

export type CascadeMode = "row" | "split";

export interface CascadeState {
  mode: CascadeMode;
  selectedRowIndex: number | null;
}

export type CascadeAction =
  | { type: "mode/set"; mode: CascadeMode }
  | { type: "row/select"; rowIndex: number }
  | {
      type: "field/select";
      fieldIndex: number;
      valueIndex: number;
      data: FilterTableData;
      baselineRowIndex: number | null;
    };

export const initialCascadeState: CascadeState = {
  mode: "row",
  selectedRowIndex: null,
};

export function resolveRow(
  data: FilterTableData | null | undefined,
  selectedRowIndex: number | null,
): FilterTableRow | undefined {
  if (!data || data.rows.length === 0) return undefined;
  if (selectedRowIndex == null) return data.rows[0];
  return (
    data.rows.find((row) => row.index === selectedRowIndex) ?? data.rows[0]
  );
}

function selectClosestRow(
  data: FilterTableData,
  fieldIndex: number,
  valueIndex: number,
  baselineRowIndex: number | null,
): FilterTableRow {
  const baseline =
    data.rows.find((row) => row.index === baselineRowIndex) ?? data.rows[0];
  const candidates = data.rows.filter(
    (row) => row.valueIndices[fieldIndex] === valueIndex,
  );
  if (candidates.length === 0) return baseline;

  let best = candidates[0];
  let bestScore = -1;

  for (const candidate of candidates) {
    let score = 0;
    for (let idx = 0; idx < candidate.valueIndices.length; idx += 1) {
      if (idx === fieldIndex) continue;
      if (candidate.valueIndices[idx] === baseline.valueIndices[idx]) {
        score += 1;
      }
    }
    if (score > bestScore) {
      best = candidate;
      bestScore = score;
    }
  }

  return best;
}

export function cascadeReducer(
  state: CascadeState,
  action: CascadeAction,
): CascadeState {
  switch (action.type) {
    case "mode/set":
      return { ...state, mode: action.mode };
    case "row/select":
      return { ...state, selectedRowIndex: action.rowIndex };
    case "field/select": {
      const chosen = selectClosestRow(
        action.data,
        action.fieldIndex,
        action.valueIndex,
        action.baselineRowIndex,
      );
      return { ...state, selectedRowIndex: chosen.index };
    }
    default:
      return state;
  }
}
