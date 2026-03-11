import {
  commands,
  type ChartDataResponse as WireChartResponse,
  type DatasetWriteStatus,
  type TableData as WireFilterTableData,
} from "@/shared/lib/bindings";
import { invoke } from "@/shared/lib/tauri";
import {
  normalizeChartOptions,
  normalizeFilterTableData,
  toWireChartOptions,
  type ChartDataOptions,
  type FilterTableData,
  type FilterTableOptions,
} from "./types";

export async function fetchChartData(
  id: number,
  options: ChartDataOptions,
) {
  const result: WireChartResponse = await invoke(
    commands.datasetChartData(id, toWireChartOptions(options)),
  );
  return normalizeChartOptions(result);
}

export async function getFilterTableData(
  id: number,
  options: FilterTableOptions,
): Promise<FilterTableData> {
  const result: WireFilterTableData = await invoke(
    commands.getFilterTableData(id, options),
  );
  return normalizeFilterTableData(result);
}

export async function getDatasetWriteStatus(
  id: number,
): Promise<DatasetWriteStatus> {
  return invoke(commands.getDatasetWriteStatus(id));
}
