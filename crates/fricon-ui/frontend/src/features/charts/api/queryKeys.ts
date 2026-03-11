export const chartKeys = {
  chartData: (datasetId: number) =>
    ["charts", "chartData", datasetId] as const,
  filterTableData: (datasetId: number) =>
    ["charts", "filterTableData", datasetId] as const,
  writeStatus: (datasetId: number) =>
    ["charts", "writeStatus", datasetId] as const,
};
