export type DatasetRowSelection = Record<string, boolean>;

interface DatasetSelectableRow {
  id: string;
}

export function findRowIndexById(
  rows: DatasetSelectableRow[],
  rowId: string | null,
): number {
  if (!rowId) {
    return -1;
  }

  return rows.findIndex((row) => row.id === rowId);
}

export function getSelectionRange(anchorIndex: number, rowIndex: number) {
  return {
    start: Math.min(anchorIndex, rowIndex),
    end: Math.max(anchorIndex, rowIndex),
  };
}

export function buildRangeSelection(
  rows: DatasetSelectableRow[],
  start: number,
  end: number,
): DatasetRowSelection {
  const selection: DatasetRowSelection = {};

  for (let index = start; index <= end; index += 1) {
    const rowId = rows[index]?.id;
    if (rowId) {
      selection[rowId] = true;
    }
  }

  return selection;
}

export function applyToggleSelectionRange(
  rows: DatasetSelectableRow[],
  start: number,
  end: number,
  initialSelection: DatasetRowSelection,
  targetValue: boolean,
): DatasetRowSelection {
  const nextSelection = { ...initialSelection };

  for (let index = start; index <= end; index += 1) {
    const rowId = rows[index]?.id;
    if (!rowId) {
      continue;
    }

    if (targetValue) {
      nextSelection[rowId] = true;
    } else {
      delete nextSelection[rowId];
    }
  }

  return nextSelection;
}
