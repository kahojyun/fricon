import { useReducer } from "react";
import type { FilterTableData } from "../api/types";
import {
  cascadeReducer,
  initialCascadeState,
  resolveRow,
  type CascadeAction,
  type CascadeMode,
} from "../model/cascadeReducer";

export function useCascadeSelection(data?: FilterTableData | null) {
  const [state, dispatch] = useReducer(cascadeReducer, initialCascadeState);
  const resolvedRow = resolveRow(data ?? undefined, state.selectedRowIndex);
  const selectedValueIndices = resolvedRow?.valueIndices;

  const setMode = (mode: CascadeMode) => dispatch({ type: "mode/set", mode });
  const selectRow = (rowIndex: number) =>
    dispatch({ type: "row/select", rowIndex });
  const selectFieldValue = (
    fieldIndex: number,
    valueIndex: number,
    baselineRowIndex: number | null,
  ) => {
    if (!data) return;
    dispatch({
      type: "field/select",
      fieldIndex,
      valueIndex,
      data,
      baselineRowIndex,
    } satisfies CascadeAction);
  };

  return {
    state,
    resolvedRow,
    selectedValueIndices,
    setMode,
    selectRow,
    selectFieldValue,
  };
}
