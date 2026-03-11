export function datasetDetailQueryKey(datasetId: number) {
  return ["datasets", "detail", datasetId] as const;
}
