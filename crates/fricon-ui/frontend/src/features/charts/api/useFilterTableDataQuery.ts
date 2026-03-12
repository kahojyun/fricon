import { useQuery } from "@tanstack/react-query";
import { getFilterTableData } from "./client";
import { chartKeys } from "./queryKeys";

function buildExcludeColumnsKey(excludeColumns: string[]) {
  return excludeColumns.slice().sort().join("|");
}

export function useFilterTableDataQuery(
  datasetId: number,
  excludeColumns: string[],
  enabled = true,
) {
  const excludeColumnsKey = buildExcludeColumnsKey(excludeColumns);
  return useQuery({
    queryKey: [...chartKeys.filterTableData(datasetId), excludeColumnsKey],
    queryFn: () => getFilterTableData(datasetId, { excludeColumns }),
    enabled,
  });
}
