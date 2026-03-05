import { useEffect, useState } from "react";
import type { ColumnDef, VisibilityState } from "@tanstack/react-table";
import type { DatasetInfo } from "@/lib/backend";
import type { DatasetColumnMeta } from "@/components/dataset-table-columns";

const COLUMN_VISIBILITY_STORAGE_KEY = "fricon.datasetTable.columnVisibility.v1";
const REQUIRED_DATASET_COLUMN_ID = "name";

function getDefaultColumnVisibility(
  columns: ColumnDef<DatasetInfo>[],
): VisibilityState {
  const visibility: VisibilityState = {};
  for (const column of columns) {
    if (!column.id) continue;
    const meta = column.meta as DatasetColumnMeta | undefined;
    visibility[column.id] = meta?.defaultVisible ?? true;
  }
  visibility[REQUIRED_DATASET_COLUMN_ID] = true;
  return visibility;
}

function sanitizeColumnVisibility(
  value: unknown,
  columns: ColumnDef<DatasetInfo>[],
  defaults: VisibilityState,
): VisibilityState {
  const objectValue =
    value && typeof value === "object"
      ? (value as Record<string, unknown>)
      : {};
  const visibility: VisibilityState = {};
  for (const column of columns) {
    if (!column.id) continue;
    const fallback = defaults[column.id] ?? true;
    const candidate = objectValue[column.id];
    visibility[column.id] =
      typeof candidate === "boolean" ? candidate : fallback;
  }
  visibility[REQUIRED_DATASET_COLUMN_ID] = true;
  return visibility;
}

function loadStoredColumnVisibility(): unknown {
  if (typeof window === "undefined") {
    return null;
  }
  try {
    const raw = window.localStorage.getItem(COLUMN_VISIBILITY_STORAGE_KEY);
    if (!raw) return null;
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function useDatasetColumnVisibility(columns: ColumnDef<DatasetInfo>[]) {
  const defaultColumnVisibility = getDefaultColumnVisibility(columns);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>(
    () =>
      sanitizeColumnVisibility(
        loadStoredColumnVisibility(),
        columns,
        defaultColumnVisibility,
      ),
  );

  useEffect(() => {
    try {
      window.localStorage.setItem(
        COLUMN_VISIBILITY_STORAGE_KEY,
        JSON.stringify(columnVisibility),
      );
    } catch {
      // Ignore storage failures and keep in-memory state.
    }
  }, [columnVisibility]);

  const resetColumnVisibilityToDefault = () => {
    setColumnVisibility({
      ...defaultColumnVisibility,
      [REQUIRED_DATASET_COLUMN_ID]: true,
    });
  };

  const showAllColumns = () => {
    const next: VisibilityState = {};
    for (const column of columns) {
      if (!column.id) continue;
      next[column.id] = true;
    }
    next[REQUIRED_DATASET_COLUMN_ID] = true;
    setColumnVisibility(next);
  };

  const handleColumnVisibilityChange = (columnId: string, visible: boolean) => {
    const columnExists = columns.some((column) => column.id === columnId);
    if (!columnExists) return;
    setColumnVisibility((previous) => ({
      ...previous,
      [columnId]: visible,
      [REQUIRED_DATASET_COLUMN_ID]: true,
    }));
  };

  return {
    columnVisibility,
    resetColumnVisibilityToDefault,
    showAllColumns,
    handleColumnVisibilityChange,
  };
}
