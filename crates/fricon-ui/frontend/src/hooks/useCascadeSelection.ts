import { useReducer } from "react";
import type { FilterTableData } from "@/lib/backend";
import {
  cascadeReducer,
  initialCascadeState,
  resolveRow,
  type CascadeAction,
  type CascadeMode,
} from "@/hooks/cascadeReducer";

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
  ) =>
    dispatch({
      type: "field/select",
      fieldIndex,
      valueIndex,
      data: data!,
      baselineRowIndex,
    } satisfies CascadeAction);

  return {
    state,
    resolvedRow,
    selectedValueIndices,
    setMode,
    selectRow,
    selectFieldValue,
  };
}
