import type {
  ColumnInfo,
  DatasetDetail as WireDatasetDetail,
  DatasetInfo as WireDatasetInfo,
  DatasetInfoUpdate,
  DatasetWriteStatus,
  UiDatasetSortBy as DatasetListSortBy,
  UiDatasetStatus as DatasetStatus,
  UiSortDirection as DatasetListSortDir,
} from "@/shared/lib/bindings";
import { normalizeCreatedAtDate } from "@/shared/lib/tauri";

export type DatasetInfo = Omit<WireDatasetInfo, "createdAt"> & {
  createdAt: Date;
};

export type DatasetDetail = Omit<WireDatasetDetail, "createdAt"> & {
  createdAt: Date;
};

export const DATASET_PAGE_SIZE = 200;

export type {
  ColumnInfo,
  DatasetInfoUpdate,
  DatasetListSortBy,
  DatasetListSortDir,
  DatasetStatus,
  DatasetWriteStatus,
};

export interface ListDatasetsOptions {
  search?: string;
  tags?: string[];
  favoriteOnly?: boolean;
  statuses?: DatasetStatus[];
  sortBy?: DatasetListSortBy;
  sortDir?: DatasetListSortDir;
  limit?: number;
  offset?: number;
}

export function normalizeDataset(value: WireDatasetInfo): DatasetInfo {
  return normalizeCreatedAtDate(value);
}

export function normalizeDatasetDetail(value: WireDatasetDetail): DatasetDetail {
  return normalizeCreatedAtDate(value);
}
