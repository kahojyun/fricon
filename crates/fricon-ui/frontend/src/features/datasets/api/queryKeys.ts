import { datasetDetailQueryKey } from "@/shared/lib/queryKeys";

export const datasetKeys = {
  all: () => ["datasets"] as const,
  list: (queryParams: unknown, visibleCount: number) =>
    ["datasets", "list", queryParams, visibleCount] as const,
  tags: () => ["datasets", "tags"] as const,
  detail: datasetDetailQueryKey,
};
