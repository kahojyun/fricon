import {
  commands,
  type DatasetDetail as WireDatasetDetail,
  type DatasetFavoriteUpdate,
  type DatasetInfo as WireDatasetInfo,
  type DatasetListOptions as WireDatasetListOptions,
  type DatasetWriteStatus,
} from "@/shared/lib/bindings";
import { invoke } from "@/shared/lib/tauri";
import {
  normalizeDataset,
  normalizeDatasetDetail,
  type DatasetDetail,
  type DatasetInfo,
  type DatasetInfoUpdate,
  type ListDatasetsOptions,
} from "./types";

export async function listDatasets(
  options: ListDatasetsOptions = {},
): Promise<DatasetInfo[]> {
  const {
    search,
    tags,
    favoriteOnly,
    statuses,
    sortBy,
    sortDir,
    limit,
    offset,
  } = options;
  const wireOptions: WireDatasetListOptions = {
    search: search?.trim(),
    tags: tags && tags.length > 0 ? tags : undefined,
    favoriteOnly: favoriteOnly ? true : undefined,
    statuses: statuses && statuses.length > 0 ? statuses : undefined,
    sortBy,
    sortDir,
    limit,
    offset,
  };
  const datasets: WireDatasetInfo[] = await invoke(
    commands.listDatasets(wireOptions),
  );
  return datasets.map(normalizeDataset);
}

export async function listDatasetTags(): Promise<string[]> {
  return invoke(commands.listDatasetTags());
}

export async function updateDatasetFavorite(
  id: number,
  favorite: boolean,
): Promise<void> {
  const update: DatasetFavoriteUpdate = { favorite };
  await invoke(commands.updateDatasetFavorite(id, update));
}

export async function updateDatasetInfo(
  id: number,
  update: DatasetInfoUpdate,
): Promise<void> {
  await invoke(commands.updateDatasetInfo(id, update));
}

export async function getDatasetDetail(id: number): Promise<DatasetDetail> {
  const rawDetail: WireDatasetDetail = await invoke(commands.datasetDetail(id));
  return normalizeDatasetDetail(rawDetail);
}

export async function getDatasetWriteStatus(
  id: number,
): Promise<DatasetWriteStatus> {
  return invoke(commands.getDatasetWriteStatus(id));
}
