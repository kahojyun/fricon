export const datasetKeys = {
  all: () => ["datasets"] as const,
  list: (queryParams: unknown, visibleCount: number) =>
    ["datasets", "list", queryParams, visibleCount] as const,
  tags: () => ["datasets", "tags"] as const,
  detail: (datasetId: number) => ["datasets", "detail", datasetId] as const,
};
